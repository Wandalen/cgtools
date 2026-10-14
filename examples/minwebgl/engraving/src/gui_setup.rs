//! The lil-gui panel of the engraving demo: text and font, the relief knobs on the engraved
//! `PbrMaterial`, and the node's text-sizing layout, each re-rendering the engraving as needed.

use std::{ cell::RefCell, rc::Rc };

use minwebgl as gl;
use gl::GL;
use gl::wasm_bindgen::prelude::*;
use renderer::webgl::{ Material, engraving::{ EngravingSession, EngravingTextLayout, SizingMode, TextAlign, font_load }, material::PbrMaterial };
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
    // `Relative` and any mode added later: the dropdown offers only the three above.
    _ => "RELATIVE",
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
    TextAlign::Right => "right",
    // `Center` and any alignment added later.
    _ => "center",
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
/// `in_flight`/`dirty` serialize re-renders: each one awaits a font load, and two overlapping
/// renders could finish out of order and leave the older text on the mask. A change that
/// arrives mid-render just flips `dirty` and lets the in-flight render loop once more with the
/// latest values.
struct EngravingState
{
  text : String,
  font : String,
  in_flight : bool,
  dirty : bool,
}

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

        // The font load is awaited without borrowing `session`, so GUI callbacks that borrow it
        // (the layout sliders) stay safe while a web font downloads; the render itself is
        // synchronous.
        let result = match font_load( &font, &text ).await
        {
          Ok( () ) => session.borrow_mut().text_render( &gl, &node_name, &text, &font ),
          Err( e ) => Err( e.into() ),
        };
        if let Err( e ) = result
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

/// What a control needs to re-render the engraving after changing its input: the context, the
/// session and node to render into, and the shared text/font [`EngravingState`].
#[ derive( Clone ) ]
struct Rerender
{
  gl : GL,
  session : Rc< RefCell< EngravingSession > >,
  node_name : String,
  state : Rc< RefCell< EngravingState > >,
}

impl Rerender
{
  /// Re-renders the current text and font through [`spawn_set_text`].
  fn spawn( &self )
  {
    spawn_set_text( self.gl.clone(), self.session.clone(), self.node_name.clone(), self.state.clone() );
  }

  /// Applies `set` to the node's text layout, then re-renders, so a sizing change takes effect
  /// without retyping.
  fn layout_set< V >( &self, set : fn( &mut EngravingTextLayout, V ), value : V )
  {
    if let Some( layout ) = self.session.borrow_mut().text_layout_mut( &self.node_name )
    {
      set( layout, value );
    }
    self.spawn();
  }
}

/// A slider on `field` of `object` that writes its value into the node's text layout with `set`
/// and re-renders.
fn layout_slider( folder : &JsValue, object : &JsValue, field : &str, ( min, max, step ) : ( f64, f64, f64 ), rerender : &Rerender, set : fn( &mut EngravingTextLayout, f32 ) )
{
  let prop = add_slider( folder, object, field, min, max, step );
  let rerender = rerender.clone();
  let callback = Closure::new( move | value : f32 | rerender.layout_set( set, value ) );
  on_change( &prop, &callback );
  callback.forget();
}

/// A dropdown on `field` of `object` offering `options` that writes the chosen one into the
/// node's text layout with `set` and re-renders.
fn layout_dropdown( folder : &JsValue, object : &JsValue, field : &str, options : &[ &str ], rerender : &Rerender, set : fn( &mut EngravingTextLayout, &str ) )
{
  let options = serde_wasm_bindgen::to_value( options ).unwrap();
  let prop = add_dropdown( folder, object, field, &options );
  let rerender = rerender.clone();
  let callback = Closure::new( move | value : String | rerender.layout_set( set, &value ) );
  on_change_str( &prop, &callback );
  callback.forget();
}

/// A slider on `field` of `object` that writes its value into the engraved `PbrMaterial` with
/// `set`. No re-render: these are plain uniforms, uploaded on the next frame.
fn material_slider( folder : &JsValue, object : &JsValue, field : &str, ( min, max, step ) : ( f64, f64, f64 ), material : &Rc< RefCell< Box< dyn Material > > >, set : fn( &mut PbrMaterial, f32 ) )
{
  let prop = add_slider( folder, object, field, min, max, step );
  let material = material.clone();
  let callback = Closure::new( move | value : f32 |
  {
    let mut mat = material.borrow_mut();
    if let Some( pbr ) = mat.as_any_mut().downcast_mut::< PbrMaterial >()
    {
      set( pbr, value );
    }
    mat.needs_update_set( true );
  } );
  on_change( &prop, &callback );
  callback.forget();
}

/// The panel's starting values: the material's relief knobs and the node's text layout, with
/// the sizing fields a layout may leave unset shown at their usual defaults.
fn settings_initial( default_font : &str, session : &RefCell< EngravingSession >, node_name : &str, material : &RefCell< Box< dyn Material > > ) -> Settings
{
  let material = material.borrow();
  let pbr = material.as_any().downcast_ref::< PbrMaterial >().expect( "engraved node's material is a PbrMaterial" );
  let session = session.borrow();
  let layout = session.text_layout( node_name ).expect( "session was built from a config containing this node" );
  Settings
  {
    text : String::new(),
    font : default_font.to_owned(),
    strength : pbr.engraving_strength,
    roughness : pbr.engraving_roughness,
    darkening : pbr.engraving_darkening,
    sizing_mode : sizing_mode_to_str( layout.resolved_sizing_mode() ).to_string(),
    strip_height_mm : layout.strip_height_mm.unwrap_or( 2.0 ),
    default_font_size_mm : layout.default_font_size_mm.unwrap_or( 1.2 ),
    min_font_size_mm : layout.min_font_size_mm.unwrap_or( 0.5 ),
    font_size_ratio : layout.font_size_ratio.unwrap_or( 0.6 ),
    align : align_to_str( layout.align ).to_string(),
  }
}

/// Wires up a lil-gui panel controlling the engraved text, its font (from the node's
/// `allowedFonts` whitelist), the bevel/roughness/darkening knobs on `material`'s
/// `USE_ENGRAVING` shader path, and the node's text-sizing layout.
pub fn setup
(
  gl : &GL,
  node_name : &str,
  default_font : &str,
  allowed_fonts : &[ Box< str > ],
  session : &Rc< RefCell< EngravingSession > >,
  material : &Rc< RefCell< Box< dyn Material > > >,
)
{
  let settings = settings_initial( default_font, session, node_name, material );
  let object = serde_wasm_bindgen::to_value( &settings ).unwrap();
  let gui = new_gui();

  let state = Rc::new( RefCell::new( EngravingState { text : settings.text, font : default_font.to_owned(), in_flight : false, dirty : false } ) );
  let rerender = Rerender { gl : gl.clone(), session : session.clone(), node_name : node_name.to_owned(), state };

  let text_folder = add_folder( &gui, "Text" );

  // Each `set_text` re-rasterizes, reads back and uploads every mip level of the mask, so it
  // only fires once the field loses focus / Enter is pressed, not on every keystroke.
  let prop = add_text( &text_folder, &object, "text" );
  let callback = Closure::new
  (
    {
      let rerender = rerender.clone();
      move | value : JsValue |
      {
        rerender.state.borrow_mut().text = value.as_string().unwrap_or_default();
        rerender.spawn();
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
      let rerender = rerender.clone();
      move | value : String |
      {
        rerender.state.borrow_mut().font = value;
        rerender.spawn();
      }
    }
  );
  on_change_str( &prop, &callback );
  callback.forget();

  let relief_folder = add_folder( &gui, "Relief" );
  material_slider( &relief_folder, &object, "strength", ( 0.0, 3.0, 0.05 ), material, | pbr, v | pbr.engraving_strength = v );
  // Drives both the base material's roughness (`roughness_factor`, the whole model) and the
  // groove-only `engraving_roughness` together, so the carved text doesn't visibly mismatch the
  // rest of the surface when this is tweaked away from the model's original roughness.
  material_slider( &relief_folder, &object, "roughness", ( 0.0, 1.0, 0.01 ), material, | pbr, v |
  {
    pbr.engraving_roughness = v;
    pbr.roughness_factor = v;
  } );
  material_slider( &relief_folder, &object, "darkening", ( 0.0, 1.0, 0.01 ), material, | pbr, v | pbr.engraving_darkening = v );

  // Live controls for `EngravingNodeConfig`'s text-sizing knobs (`SizingMode` and its
  // parameters), mutated via `EngravingSession::text_layout_mut` and re-rendered by re-running
  // whatever text is currently in `state` through the same `spawn_set_text` path the Text panel
  // uses — sizing takes effect immediately without retyping.
  let sizing_folder = add_folder( &gui, "Text Sizing" );
  layout_dropdown( &sizing_folder, &object, "sizingMode", &[ "RELATIVE", "HYBRID", "PHYSICAL" ], &rerender, | layout, v | layout.sizing_mode = Some( sizing_mode_from_str( v ) ) );
  layout_slider( &sizing_folder, &object, "stripHeightMm", ( 0.1, 5.0, 0.05 ), &rerender, | layout, v | layout.strip_height_mm = Some( v ) );
  layout_slider( &sizing_folder, &object, "defaultFontSizeMm", ( 0.05, 5.0, 0.05 ), &rerender, | layout, v | layout.default_font_size_mm = Some( v ) );
  layout_slider( &sizing_folder, &object, "minFontSizeMm", ( 0.05, 5.0, 0.05 ), &rerender, | layout, v | layout.min_font_size_mm = Some( v ) );
  layout_slider( &sizing_folder, &object, "fontSizeRatio", ( 0.05, 1.0, 0.01 ), &rerender, | layout, v | layout.font_size_ratio = Some( v ) );
  layout_dropdown( &sizing_folder, &object, "align", &[ "left", "center", "right" ], &rerender, | layout, v | layout.align = align_from_str( v ) );

  std::mem::forget( object );

  show( &gui );
}
