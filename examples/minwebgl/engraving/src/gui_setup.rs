#![ allow( clippy::needless_pass_by_value ) ]
#![ allow( clippy::field_reassign_with_default ) ]

use std::{ cell::RefCell, rc::Rc };

use minwebgl as gl;
use gl::GL;
use gl::wasm_bindgen::prelude::*;
use renderer::webgl::{ Material, engraving::{ EngravingSession, SizingMode, TextAlign }, material::PbrMaterial };
use serde::{ Deserialize, Serialize };

use crate::lil_gui::{ add_dropdown, add_folder, add_slider, add_text, new_gui, on_change, on_change_str, on_finish_change, show };

#[ derive( Default, Serialize, Deserialize ) ]
struct Settings
{
  text : String,
  font : String,
  strength : f32,
  roughness : f32,
  darkening : f32,
  #[ serde( rename = "sizingMode" ) ]
  sizing_mode : String,
  #[ serde( rename = "stripHeightMm" ) ]
  strip_height_mm : f32,
  #[ serde( rename = "defaultFontSizeMm" ) ]
  default_font_size_mm : f32,
  #[ serde( rename = "minFontSizeMm" ) ]
  min_font_size_mm : f32,
  #[ serde( rename = "fontSizeRatio" ) ]
  font_size_ratio : f32,
  align : String,
}

/// `SizingMode` <-> the uppercase strings the `sizingMode` dropdown option list uses (matching
/// `engraving_config.json`'s own `sizingMode` string values, see `engraving_config.schema.json`).
fn sizing_mode_to_str( mode : SizingMode ) -> &'static str
{
  match mode
  {
    SizingMode::Physical => "PHYSICAL",
    SizingMode::Hybrid => "HYBRID",
    SizingMode::Relative => "RELATIVE",
  }
}

fn sizing_mode_from_str( s : &str ) -> SizingMode
{
  match s
  {
    "PHYSICAL" => SizingMode::Physical,
    "HYBRID" => SizingMode::Hybrid,
    _ => SizingMode::Relative,
  }
}

fn align_to_str( align : TextAlign ) -> &'static str
{
  match align
  {
    TextAlign::Left => "left",
    TextAlign::Center => "center",
    TextAlign::Right => "right",
  }
}

fn align_from_str( s : &str ) -> TextAlign
{
  match s
  {
    "left" => TextAlign::Left,
    "right" => TextAlign::Right,
    _ => TextAlign::Center,
  }
}

/// Current text/font, shared between the two GUI fields that can trigger a re-render (typing
/// text and switching the font both need to reuse whichever value the *other* field last had).
///
/// `in_flight`/`dirty` serialize calls into [`EngravingSession::set_text`]: it holds `session`
/// mutably across an `await` (font loading), so a second call spawned while the first is still
/// suspended there would panic on the `RefCell` borrow. Rather than allow that race, a change
/// that arrives mid-render just flips `dirty` and lets the in-flight call loop once more with
/// the latest values, instead of racing it.
struct EngravingState
{
  text : String,
  font : String,
  in_flight : bool,
  dirty : bool,
}

#[ allow( clippy::await_holding_refcell_ref, reason = "single in-flight call is enforced by the in_flight/dirty guard below" ) ]
fn spawn_set_text( gl : GL, session : Rc< RefCell< EngravingSession > >, node_name : String, state : Rc< RefCell< EngravingState > > )
{
  if state.borrow().in_flight
  {
    state.borrow_mut().dirty = true;
    return;
  }
  state.borrow_mut().in_flight = true;

  gl::spawn_local
  (
    async move
    {
      loop
      {
        let ( text, font ) =
        {
          let state = state.borrow();
          ( state.text.clone(), state.font.clone() )
        };

        if let Err( e ) = session.borrow_mut().set_text( &gl, &node_name, &text, &font ).await
        {
          gl::info!( "failed to set engraving text: {e}" );
        }

        let mut state = state.borrow_mut();
        if std::mem::take( &mut state.dirty )
        {
          continue;
        }
        state.in_flight = false;
        break;
      }
    }
  );
}

/// Wires up a lil-gui panel controlling the engraved text, its font (from the node's
/// `allowedFonts` whitelist) and the bevel/roughness/darkening knobs on `material`'s
/// `USE_ENGRAVING` shader path.
pub fn setup
(
  gl : GL,
  node_name : String,
  default_font : String,
  allowed_fonts : Vec< Box< str > >,
  session : Rc< RefCell< EngravingSession > >,
  material : Rc< RefCell< Box< dyn Material > > >,
)
{
  let ( strength, roughness, darkening ) =
  {
    let mat = material.borrow();
    let pbr = mat.as_any().downcast_ref::< PbrMaterial >().expect( "engraved node's material is a PbrMaterial" );
    ( pbr.engraving_strength, pbr.engraving_roughness, pbr.engraving_darkening )
  };

  let ( sizing_mode, strip_height_mm, default_font_size_mm, min_font_size_mm, font_size_ratio, align ) =
  {
    let session_ref = session.borrow();
    let layout = session_ref.text_layout( &node_name ).expect( "session was built from a config containing this node" );
    (
      sizing_mode_to_str( layout.resolved_sizing_mode() ).to_string(),
      layout.strip_height_mm.unwrap_or( 2.0 ),
      layout.default_font_size_mm.unwrap_or( 1.2 ),
      layout.min_font_size_mm.unwrap_or( 0.5 ),
      layout.font_size_ratio.unwrap_or( 0.6 ),
      align_to_str( layout.align ).to_string(),
    )
  };

  let mut settings = Settings::default();
  settings.text = String::new();
  settings.font.clone_from( &default_font );
  settings.strength = strength;
  settings.roughness = roughness;
  settings.darkening = darkening;
  settings.sizing_mode = sizing_mode;
  settings.strip_height_mm = strip_height_mm;
  settings.default_font_size_mm = default_font_size_mm;
  settings.min_font_size_mm = min_font_size_mm;
  settings.font_size_ratio = font_size_ratio;
  settings.align = align;

  let object = serde_wasm_bindgen::to_value( &settings ).unwrap();
  let gui = new_gui();

  let state = Rc::new( RefCell::new( EngravingState { text : settings.text, font : default_font, in_flight : false, dirty : false } ) );

  let text_folder = add_folder( &gui, "Text" );

  // Rasterizing + uploading a new mask is comparatively expensive, so it only fires once the
  // field loses focus / Enter is pressed, not on every keystroke.
  let prop = add_text( &text_folder, &object, "text" );
  let callback = Closure::new
  (
    {
      let gl = gl.clone();
      let session = session.clone();
      let node_name = node_name.clone();
      let state = state.clone();
      move | value : JsValue |
      {
        state.borrow_mut().text = value.as_string().unwrap_or_default();
        spawn_set_text( gl.clone(), session.clone(), node_name.clone(), state.clone() );
      }
    }
  );
  on_finish_change( &prop, &callback );
  callback.forget();

  let options = serde_wasm_bindgen::to_value( &allowed_fonts.iter().map( ToString::to_string ).collect::< Vec< _ > >() ).unwrap();
  let prop = add_dropdown( &text_folder, &object, "font", &options );
  let callback = Closure::new
  (
    {
      let gl = gl.clone();
      let session = session.clone();
      let node_name = node_name.clone();
      let state = state.clone();
      move | value : String |
      {
        state.borrow_mut().font = value;
        spawn_set_text( gl.clone(), session.clone(), node_name.clone(), state.clone() );
      }
    }
  );
  on_change_str( &prop, &callback );
  callback.forget();

  let relief_folder = add_folder( &gui, "Relief" );

  let prop = add_slider( &relief_folder, &object, "strength", 0.0, 3.0, 0.05 );
  let callback = Closure::new
  (
    {
      let material = material.clone();
      move | value : f32 |
      {
        let mut mat = material.borrow_mut();
        if let Some( pbr ) = mat.as_any_mut().downcast_mut::< PbrMaterial >()
        {
          pbr.engraving_strength = value;
        }
        mat.set_needs_update( true );
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  // Drives both the base material's roughness (`roughness_factor`, the whole model) and the
  // groove-only `engraving_roughness` together, so the carved text doesn't visibly mismatch the
  // rest of the surface when this is tweaked away from the model's original roughness.
  let prop = add_slider( &relief_folder, &object, "roughness", 0.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let material = material.clone();
      move | value : f32 |
      {
        let mut mat = material.borrow_mut();
        if let Some( pbr ) = mat.as_any_mut().downcast_mut::< PbrMaterial >()
        {
          pbr.engraving_roughness = value;
          pbr.roughness_factor = value;
        }
        mat.set_needs_update( true );
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  let prop = add_slider( &relief_folder, &object, "darkening", 0.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let material = material.clone();
      move | value : f32 |
      {
        let mut mat = material.borrow_mut();
        if let Some( pbr ) = mat.as_any_mut().downcast_mut::< PbrMaterial >()
        {
          pbr.engraving_darkening = value;
        }
        mat.set_needs_update( true );
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  // Live controls for `EngravingNodeConfig`'s text-sizing knobs (`SizingMode` and its
  // parameters), mutated via `EngravingSession::text_layout_mut` and re-rendered by re-running
  // whatever text is currently in `state` through the same `spawn_set_text` path the Text panel
  // uses — sizing takes effect immediately without retyping.
  let sizing_folder = add_folder( &gui, "Text Sizing" );

  let sizing_mode_options = serde_wasm_bindgen::to_value( &[ "RELATIVE", "HYBRID", "PHYSICAL" ] ).unwrap();
  let prop = add_dropdown( &sizing_folder, &object, "sizingMode", &sizing_mode_options );
  let callback = Closure::new
  (
    {
      let gl = gl.clone();
      let session = session.clone();
      let node_name = node_name.clone();
      let state = state.clone();
      move | value : String |
      {
        {
          let mut session_mut = session.borrow_mut();
          if let Some( cfg ) = session_mut.text_layout_mut( &node_name )
          {
            cfg.sizing_mode = Some( sizing_mode_from_str( &value ) );
          }
        }
        spawn_set_text( gl.clone(), session.clone(), node_name.clone(), state.clone() );
      }
    }
  );
  on_change_str( &prop, &callback );
  callback.forget();

  let prop = add_slider( &sizing_folder, &object, "stripHeightMm", 0.1, 5.0, 0.05 );
  let callback = Closure::new
  (
    {
      let gl = gl.clone();
      let session = session.clone();
      let node_name = node_name.clone();
      let state = state.clone();
      move | value : f32 |
      {
        {
          let mut session_mut = session.borrow_mut();
          if let Some( cfg ) = session_mut.text_layout_mut( &node_name )
          {
            cfg.strip_height_mm = Some( value );
          }
        }
        spawn_set_text( gl.clone(), session.clone(), node_name.clone(), state.clone() );
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  let prop = add_slider( &sizing_folder, &object, "defaultFontSizeMm", 0.05, 5.0, 0.05 );
  let callback = Closure::new
  (
    {
      let gl = gl.clone();
      let session = session.clone();
      let node_name = node_name.clone();
      let state = state.clone();
      move | value : f32 |
      {
        {
          let mut session_mut = session.borrow_mut();
          if let Some( cfg ) = session_mut.text_layout_mut( &node_name )
          {
            cfg.default_font_size_mm = Some( value );
          }
        }
        spawn_set_text( gl.clone(), session.clone(), node_name.clone(), state.clone() );
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  let prop = add_slider( &sizing_folder, &object, "minFontSizeMm", 0.05, 5.0, 0.05 );
  let callback = Closure::new
  (
    {
      let gl = gl.clone();
      let session = session.clone();
      let node_name = node_name.clone();
      let state = state.clone();
      move | value : f32 |
      {
        {
          let mut session_mut = session.borrow_mut();
          if let Some( cfg ) = session_mut.text_layout_mut( &node_name )
          {
            cfg.min_font_size_mm = Some( value );
          }
        }
        spawn_set_text( gl.clone(), session.clone(), node_name.clone(), state.clone() );
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  let prop = add_slider( &sizing_folder, &object, "fontSizeRatio", 0.05, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let gl = gl.clone();
      let session = session.clone();
      let node_name = node_name.clone();
      let state = state.clone();
      move | value : f32 |
      {
        {
          let mut session_mut = session.borrow_mut();
          if let Some( cfg ) = session_mut.text_layout_mut( &node_name )
          {
            cfg.font_size_ratio = Some( value );
          }
        }
        spawn_set_text( gl.clone(), session.clone(), node_name.clone(), state.clone() );
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  let align_options = serde_wasm_bindgen::to_value( &[ "left", "center", "right" ] ).unwrap();
  let prop = add_dropdown( &sizing_folder, &object, "align", &align_options );
  let callback = Closure::new
  (
    {
      let gl = gl.clone();
      let session = session.clone();
      let node_name = node_name.clone();
      let state = state.clone();
      move | value : String |
      {
        {
          let mut session_mut = session.borrow_mut();
          if let Some( cfg ) = session_mut.text_layout_mut( &node_name )
          {
            cfg.align = align_from_str( &value );
          }
        }
        spawn_set_text( gl.clone(), session.clone(), node_name.clone(), state.clone() );
      }
    }
  );
  on_change_str( &prop, &callback );
  callback.forget();

  std::mem::forget( object );

  show( &gui );
}
