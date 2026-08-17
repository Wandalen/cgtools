//! Floating dev panel with live sliders for the tactical grid, ported from
//! `examples/threejs/falling_frontier/src/debug/gridTuningPanel.js`. Built
//! via raw DOM calls (web-sys) rather than a GUI crate — no egui/winit
//! integration exists anywhere in this workspace (see the audit's Dev
//! Tooling section), and a DOM panel matches how every other browser-facing
//! piece of this port already talks to the page.
//!
//! Only meant for tuning defaults during development — not part of any
//! in-game UI.

use minwebgl as gl;
use std::{ cell::RefCell, fmt::Write, rc::Rc };
use gl::web_sys::
{
  wasm_bindgen::{ prelude::Closure, JsCast },
  Document, Element, HtmlButtonElement, HtmlInputElement, HtmlSelectElement,
};

use super::grid_tuning::{ GridTuning, FADE_CURVES };
use crate::FocusState;

fn slider_row_html( id : &str, label : &str, min : f32, max : f32, step : f32, value : f32, decimals : usize ) -> String
{
  format!
  (
    r#"<div style="margin-bottom:6px">
      <div style="display:flex;justify-content:space-between;font-size:10px;color:#7dd3fc">
        <span>{label}</span>
        <span id="{id}-value" style="font-family:monospace;color:#22d3ee">{value:.decimals$}</span>
      </div>
      <input type="range" id="{id}" min="{min}" max="{max}" step="{step}" value="{value}" style="width:100%;accent-color:#22d3ee">
    </div>"#
  )
}

fn select_row_html( id : &str, label : &str, value : f32 ) -> String
{
  let options = FADE_CURVES.iter()
  .fold( String::new(), | mut acc, ( index, name ) |
  {
    let selected = if ( *index - value ).abs() < 0.01 { "selected" } else { "" };
    let _ = write!( acc, r#"<option value="{index}" {selected}>{name}</option>"# );
    acc
  } );

  format!
  (
    r#"<div style="margin-bottom:6px">
      <div style="font-size:10px;color:#7dd3fc;margin-bottom:2px">{label}</div>
      <select id="{id}" style="width:100%;background:#0f172a;border:1px solid #164e63;border-radius:4px;color:#7dd3fc;font-size:10px;padding:2px">{options}</select>
    </div>"#
  )
}

fn color_row_html( id : &str, label : &str, value_hex : &str ) -> String
{
  format!
  (
    r#"<div style="display:flex;justify-content:space-between;align-items:center;font-size:10px;color:#7dd3fc;margin-bottom:6px">
      <span>{label}</span>
      <input type="color" id="{id}" value="{value_hex}" style="width:40px;height:20px;background:transparent;border:1px solid #164e63;border-radius:4px">
    </div>"#
  )
}

fn subheading_html( text : &str ) -> String
{
  format!( r#"<div style="font-size:10px;color:#0e7490;text-transform:uppercase;letter-spacing:0.05em;padding-top:6px;margin-bottom:4px">{text}</div>"# )
}

fn rgb_to_hex( c : [ f32; 3 ] ) -> String
{
  let to_u8 = | v : f32 | ( v.clamp( 0.0, 1.0 ) * 255.0 ).round() as u8;
  format!( "#{:02x}{:02x}{:02x}", to_u8( c[ 0 ] ), to_u8( c[ 1 ] ), to_u8( c[ 2 ] ) )
}

fn hex_to_rgb( hex : &str ) -> Option< [ f32; 3 ] >
{
  let hex = hex.trim_start_matches( '#' );
  if hex.len() != 6 { return None; }
  let r = u8::from_str_radix( &hex[ 0 .. 2 ], 16 ).ok()?;
  let g = u8::from_str_radix( &hex[ 2 .. 4 ], 16 ).ok()?;
  let b = u8::from_str_radix( &hex[ 4 .. 6 ], 16 ).ok()?;
  Some( [ f32::from( r ) / 255.0, f32::from( g ) / 255.0, f32::from( b ) / 255.0 ] )
}

fn curve_label( mode : f32 ) -> &'static str
{
  FADE_CURVES.iter().find( | ( index, _ ) | ( *index - mode ).abs() < 0.01 ).map_or( "?", | ( _, name ) | name )
}

fn build_tuning_summary( t : &GridTuning ) -> String
{
  format!
  (
    "===== Grid tuning config =====\n\
    (paste this whole block back to update GridTuning::default())\n\
    \n\
    line color: {}\n\
    line width (px): {:.2}\n\
    cell size: {:.2}\n\
    outside opacity: {:.2}\n\
    inside (focused) opacity: {:.2}\n\
    \n\
    camera fade start: {}\n\
    camera fade end: {}\n\
    camera fade curve: {}\n\
    camera fade gamma: {:.2}\n\
    \n\
    ribbon core color: {}\n\
    ribbon edge color: {}\n\
    ribbon width outer: {:.2}\n\
    ribbon width inner: {:.2}\n\
    ribbon gap: {:.2}\n\
    ribbon opacity: {:.2}\n\
    inside fade width: {:.2}\n\
    inside fade curve: {}\n\
    inside fade gamma: {:.2}\n\
    \n\
    asteroid glow opacity: {:.2}\n\
    asteroid glow width: {:.2}\n\
    asteroid glow curve: {}\n\
    asteroid glow gamma: {:.2}\n\
    \n\
    view radius (focus stand-in): {:.0}",
    rgb_to_hex( t.line_color ), t.line_width_px, t.cell_size, t.dim_alpha, t.bright_alpha,
    t.camera_fade_start, t.camera_fade_end, curve_label( t.camera_fade_mode ), t.camera_fade_gamma,
    rgb_to_hex( t.ribbon_color_core ), rgb_to_hex( t.ribbon_color_edge ),
    t.ribbon_width_outer, t.ribbon_width_inner, t.ribbon_gap, t.ribbon_opacity,
    t.inside_fade_width, curve_label( t.inside_fade_mode ), t.inside_fade_gamma,
    t.asteroid_glow_alpha, t.asteroid_glow_width, curve_label( t.asteroid_glow_mode ), t.asteroid_glow_gamma,
    t.view_radius
  )
}

fn focus_status_text( focus : &FocusState ) -> String
{
  if focus.active
  {
    format!( "focus: ({:.0}, {:.0})", focus.point[ 0 ], focus.point[ 1 ] )
  }
  else
  {
    "focus: none (click the ground)".to_string()
  }
}

/// Updates the panel's focus-status line after `main.rs`'s ground-click
/// handler changes `focus` — the panel itself doesn't own that state (it's
/// app state, not tuning), so this is the seam between the two.
pub fn refresh_focus_status( document : &Document, focus : &FocusState )
{
  if let Some( el ) = document.get_element_by_id( "grid-focus-status" )
  {
    el.set_text_content( Some( &focus_status_text( focus ) ) );
  }
}

fn input_by_id( document : &Document, id : &str ) -> HtmlInputElement
{
  document.get_element_by_id( id ).unwrap().dyn_into::< HtmlInputElement >().unwrap()
}

fn select_by_id( document : &Document, id : &str ) -> HtmlSelectElement
{
  document.get_element_by_id( id ).unwrap().dyn_into::< HtmlSelectElement >().unwrap()
}

fn set_value_label( document : &Document, id : &str, text : &str )
{
  if let Some( el ) = document.get_element_by_id( &format!( "{id}-value" ) )
  {
    el.set_text_content( Some( text ) );
  }
}

/// Binds a slider's `input` event to a setter closure over the shared tuning
/// state, and keeps its adjacent value label in sync.
fn bind_slider< F >( document : &Document, id : &'static str, decimals : usize, mut on_change : F )
where F : FnMut( f32 ) + 'static
{
  let element = input_by_id( document, id );
  let document = document.clone();
  let closure = Closure::< dyn FnMut() >::new
  (
    move ||
    {
      let input = input_by_id( &document, id );
      let value : f32 = input.value().parse().unwrap_or( 0.0 );
      set_value_label( &document, id, &format!( "{value:.decimals$}" ) );
      on_change( value );
    }
  );
  element.add_event_listener_with_callback( "input", closure.as_ref().unchecked_ref() ).unwrap();
  closure.forget();
}

fn bind_select< F >( document : &Document, id : &'static str, mut on_change : F )
where F : FnMut( f32 ) + 'static
{
  let element = select_by_id( document, id );
  let document = document.clone();
  let closure = Closure::< dyn FnMut() >::new
  (
    move ||
    {
      let select = select_by_id( &document, id );
      let value : f32 = select.value().parse().unwrap_or( 0.0 );
      on_change( value );
    }
  );
  element.add_event_listener_with_callback( "change", closure.as_ref().unchecked_ref() ).unwrap();
  closure.forget();
}

fn bind_color< F >( document : &Document, id : &'static str, mut on_change : F )
where F : FnMut( [ f32; 3 ] ) + 'static
{
  let element = input_by_id( document, id );
  let document = document.clone();
  let closure = Closure::< dyn FnMut() >::new
  (
    move ||
    {
      let input = input_by_id( &document, id );
      if let Some( rgb ) = hex_to_rgb( &input.value() )
      {
        on_change( rgb );
      }
    }
  );
  element.add_event_listener_with_callback( "input", closure.as_ref().unchecked_ref() ).unwrap();
  closure.forget();
}

#[ expect( clippy::too_many_lines, reason = "one flat panel-assembly sequence (build every row's HTML, then bind every row's listener); splitting it would scatter each control's HTML and its own binding across helper functions for no real grouping" ) ]
pub fn setup_grid_tuning_panel( document : &Document, tuning : &Rc< RefCell< GridTuning > >, focus : &Rc< RefCell< FocusState > > )
{
  let t = *tuning.borrow();
  let focus_status_line = focus_status_text( &focus.borrow() );

  let body_html = format!
  (
    r"{copy_button}
    {line_color}
    {line_width}
    {cell_size}
    {dim_alpha}
    {fade_heading}
    {fade_start}
    {fade_end}
    {fade_curve}
    {fade_gamma}
    {focus_heading}
    {focus_status}
    {view_radius}
    {clear_focus}
    {ribbon_heading}
    {bright_alpha}
    {ring_core}
    {ring_edge}
    {ribbon_width_outer}
    {ribbon_width_inner}
    {ribbon_gap}
    {ribbon_opacity}
    {inside_fade_width}
    {inside_fade_curve}
    {inside_fade_gamma}
    {glow_heading}
    {glow_alpha}
    {glow_width}
    {glow_curve}
    {glow_gamma}",
    copy_button = r#"<button type="button" id="grid-copy" style="width:100%;padding:6px;border-radius:4px;background:#0e7490;border:1px solid #22d3ee;color:#e0f2fe;font-weight:bold;font-size:11px;margin-bottom:10px;cursor:pointer">Copy Settings</button>"#,
    line_color = color_row_html( "grid-line-color", "Line Color", &rgb_to_hex( t.line_color ) ),
    line_width = slider_row_html( "grid-line-width", "Line Width (px)", 0.5, 6.0, 0.1, t.line_width_px, 1 ),
    cell_size = slider_row_html( "grid-cell-size", "Cell Size", 1.0, 50.0, 1.0, t.cell_size, 0 ),
    dim_alpha = slider_row_html( "grid-dim-alpha", "Outside Opacity", 0.0, 1.0, 0.01, t.dim_alpha, 2 ),
    fade_heading = subheading_html( "Camera Distance Fade" ),
    fade_start = slider_row_html( "grid-fade-start", "Fade Start", 0.0, 1000.0, 10.0, t.camera_fade_start, 0 ),
    fade_end = slider_row_html( "grid-fade-end", "Fade End", 0.0, 2000.0, 10.0, t.camera_fade_end, 0 ),
    fade_curve = select_row_html( "grid-fade-curve", "Fade Curve", t.camera_fade_mode ),
    fade_gamma = slider_row_html( "grid-fade-gamma", "Fade Gamma", 0.2, 3.0, 0.05, t.camera_fade_gamma, 2 ),
    focus_heading = subheading_html( "Focus (M3 stand-in — click ground)" ),
    focus_status = format_args!( r#"<div id="grid-focus-status" style="font-size:10px;color:#7dd3fc;margin-bottom:6px">{focus_status_line}</div>"# ),
    view_radius = slider_row_html( "grid-view-radius", "View Radius", 20.0, 400.0, 5.0, t.view_radius, 0 ),
    clear_focus = r#"<button type="button" id="grid-clear-focus" style="width:100%;padding:5px;border-radius:4px;background:#164e63;border:1px solid #22d3ee;color:#e0f2fe;font-size:10px;margin-bottom:10px;cursor:pointer">Clear Focus</button>"#,
    ribbon_heading = subheading_html( "View Zone Ribbon" ),
    bright_alpha = slider_row_html( "grid-bright-alpha", "Inside Opacity", 0.0, 1.0, 0.01, t.bright_alpha, 2 ),
    ring_core = color_row_html( "grid-ring-core", "Ribbon Core Color", &rgb_to_hex( t.ribbon_color_core ) ),
    ring_edge = color_row_html( "grid-ring-edge", "Ribbon Edge Color", &rgb_to_hex( t.ribbon_color_edge ) ),
    ribbon_width_outer = slider_row_html( "grid-ribbon-width-outer", "Ribbon Width Outer", 0.1, 4.0, 0.05, t.ribbon_width_outer, 2 ),
    ribbon_width_inner = slider_row_html( "grid-ribbon-width-inner", "Ribbon Width Inner", 0.1, 4.0, 0.05, t.ribbon_width_inner, 2 ),
    ribbon_gap = slider_row_html( "grid-ribbon-gap", "Ribbon Gap", 0.0, 3.0, 0.05, t.ribbon_gap, 2 ),
    ribbon_opacity = slider_row_html( "grid-ribbon-opacity", "Ribbon Opacity", 0.0, 1.0, 0.01, t.ribbon_opacity, 2 ),
    inside_fade_width = slider_row_html( "grid-inside-fade-width", "Inside Fade Width", 0.0, 5.0, 0.1, t.inside_fade_width, 2 ),
    inside_fade_curve = select_row_html( "grid-inside-fade-curve", "Inside Fade Curve", t.inside_fade_mode ),
    inside_fade_gamma = slider_row_html( "grid-inside-fade-gamma", "Inside Fade Gamma", 0.2, 3.0, 0.05, t.inside_fade_gamma, 2 ),
    glow_heading = subheading_html( "Asteroid Glow" ),
    glow_alpha = slider_row_html( "grid-glow-alpha", "Glow Opacity", 0.0, 1.0, 0.01, t.asteroid_glow_alpha, 2 ),
    glow_width = slider_row_html( "grid-glow-width", "Glow Width (cells)", 0.0, 10.0, 0.1, t.asteroid_glow_width, 2 ),
    glow_curve = select_row_html( "grid-glow-curve", "Glow Curve", t.asteroid_glow_mode ),
    glow_gamma = slider_row_html( "grid-glow-gamma", "Glow Gamma", 0.2, 3.0, 0.05, t.asteroid_glow_gamma, 2 ),
  );

  let panel_html = format!
  (
    r#"<div style="position:fixed;bottom:12px;right:12px;z-index:30;width:220px;max-height:85vh;overflow-y:auto;
        background:rgba(8,17,26,0.9);border:1px solid #164e63;border-radius:8px;padding:10px;
        font-family:monospace;font-size:11px;color:#e0f2fe">
      <div style="font-weight:bold;text-transform:uppercase;border-bottom:1px solid #164e63;padding-bottom:4px;margin-bottom:8px">Grid Tuning (dev)</div>
      {body_html}
    </div>"#
  );

  let panel : Element = document.create_element( "div" ).unwrap();
  panel.set_inner_html( &panel_html );
  document.body().unwrap().append_child( &panel ).unwrap();

  bind_color( document, "grid-line-color", { let tuning = tuning.clone(); move | rgb | tuning.borrow_mut().line_color = rgb } );
  bind_slider( document, "grid-line-width", 1, { let tuning = tuning.clone(); move | v | tuning.borrow_mut().line_width_px = v } );
  bind_slider( document, "grid-cell-size", 0, { let tuning = tuning.clone(); move | v | tuning.borrow_mut().cell_size = v } );
  bind_slider( document, "grid-dim-alpha", 2, { let tuning = tuning.clone(); move | v | tuning.borrow_mut().dim_alpha = v } );
  bind_slider( document, "grid-fade-start", 0, { let tuning = tuning.clone(); move | v | tuning.borrow_mut().camera_fade_start = v } );
  bind_slider( document, "grid-fade-end", 0, { let tuning = tuning.clone(); move | v | tuning.borrow_mut().camera_fade_end = v } );
  bind_select( document, "grid-fade-curve", { let tuning = tuning.clone(); move | v | tuning.borrow_mut().camera_fade_mode = v } );
  bind_slider( document, "grid-fade-gamma", 2, { let tuning = tuning.clone(); move | v | tuning.borrow_mut().camera_fade_gamma = v } );

  bind_slider( document, "grid-view-radius", 0, { let tuning = tuning.clone(); move | v | tuning.borrow_mut().view_radius = v } );
  bind_slider( document, "grid-bright-alpha", 2, { let tuning = tuning.clone(); move | v | tuning.borrow_mut().bright_alpha = v } );
  bind_color( document, "grid-ring-core", { let tuning = tuning.clone(); move | rgb | tuning.borrow_mut().ribbon_color_core = rgb } );
  bind_color( document, "grid-ring-edge", { let tuning = tuning.clone(); move | rgb | tuning.borrow_mut().ribbon_color_edge = rgb } );
  bind_slider( document, "grid-ribbon-width-outer", 2, { let tuning = tuning.clone(); move | v | tuning.borrow_mut().ribbon_width_outer = v } );
  bind_slider( document, "grid-ribbon-width-inner", 2, { let tuning = tuning.clone(); move | v | tuning.borrow_mut().ribbon_width_inner = v } );
  bind_slider( document, "grid-ribbon-gap", 2, { let tuning = tuning.clone(); move | v | tuning.borrow_mut().ribbon_gap = v } );
  bind_slider( document, "grid-ribbon-opacity", 2, { let tuning = tuning.clone(); move | v | tuning.borrow_mut().ribbon_opacity = v } );
  bind_slider( document, "grid-inside-fade-width", 2, { let tuning = tuning.clone(); move | v | tuning.borrow_mut().inside_fade_width = v } );
  bind_select( document, "grid-inside-fade-curve", { let tuning = tuning.clone(); move | v | tuning.borrow_mut().inside_fade_mode = v } );
  bind_slider( document, "grid-inside-fade-gamma", 2, { let tuning = tuning.clone(); move | v | tuning.borrow_mut().inside_fade_gamma = v } );
  bind_slider( document, "grid-glow-alpha", 2, { let tuning = tuning.clone(); move | v | tuning.borrow_mut().asteroid_glow_alpha = v } );
  bind_slider( document, "grid-glow-width", 2, { let tuning = tuning.clone(); move | v | tuning.borrow_mut().asteroid_glow_width = v } );
  bind_select( document, "grid-glow-curve", { let tuning = tuning.clone(); move | v | tuning.borrow_mut().asteroid_glow_mode = v } );
  bind_slider( document, "grid-glow-gamma", 2, { let tuning = tuning.clone(); move | v | tuning.borrow_mut().asteroid_glow_gamma = v } );

  let clear_focus_button = document.get_element_by_id( "grid-clear-focus" ).unwrap().dyn_into::< HtmlButtonElement >().unwrap();
  let document_for_clear = document.clone();
  let focus_for_clear = focus.clone();
  let clear_closure = Closure::< dyn FnMut() >::new
  (
    move ||
    {
      focus_for_clear.borrow_mut().active = false;
      refresh_focus_status( &document_for_clear, &focus_for_clear.borrow() );
    }
  );
  clear_focus_button.add_event_listener_with_callback( "click", clear_closure.as_ref().unchecked_ref() ).unwrap();
  clear_closure.forget();

  let copy_button = document.get_element_by_id( "grid-copy" ).unwrap().dyn_into::< HtmlButtonElement >().unwrap();
  let document_for_copy = document.clone();
  let tuning_for_copy = tuning.clone();
  let closure = Closure::< dyn FnMut() >::new
  (
    move ||
    {
      let text = build_tuning_summary( &tuning_for_copy.borrow() );
      let button = document_for_copy.get_element_by_id( "grid-copy" ).unwrap().dyn_into::< HtmlButtonElement >().unwrap();
      let promise = gl::web_sys::window().unwrap().navigator().clipboard().write_text( &text );
      let button_for_result = button.clone();
      gl::spawn_local
      (
        async move
        {
          match gl::JsFuture::from( promise ).await
          {
            Ok( _ ) => button_for_result.set_text_content( Some( "Copied!" ) ),
            Err( _ ) => button_for_result.set_text_content( Some( "Copy failed — see console" ) ),
          }
        }
      );
    }
  );
  copy_button.add_event_listener_with_callback( "click", closure.as_ref().unchecked_ref() ).unwrap();
  closure.forget();
}
