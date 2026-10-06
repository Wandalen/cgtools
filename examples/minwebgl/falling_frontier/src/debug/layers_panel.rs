//! Standalone "Render Layers" dev panel - every renderer-aspect visibility
//! switch (grid, background, starfield, per-object-type hull rendering,
//! view-zone ribbon, selection gizmo, lighting, shadows, CRT scanlines)
//! united in one place, separate from `grid_tuning_panel`'s slider-heavy
//! shader tuning controls and from `hud`'s in-game "real UI". Lets any
//! combination of scene layers be isolated (e.g. "only the grid") or hidden
//! (e.g. "everything but asteroids") from a single menu; the HUD no longer
//! carries visibility toggles of its own.
//!
//! Trajectories and ship animation have no rows here. Ship animation is
//! driven by the HUD's Pause/Play/Fast buttons (`GridTuning::animate_ships`
//! and `speed_multiplier`). Trajectories are still unfinished, so
//! `RenderLayers::show_trajectories` stays off by default with no toggle
//! anywhere yet; the frame loop builds the ribbons the first time it is
//! set. Sensor rings were cut further still: the feature itself is gone
//! from `trajectories.rs`, not just hidden.
//!
//! Left click flips just the clicked row, same as any checkbox. Right click
//! (`contextmenu`, default browser menu suppressed) is an unconditional
//! "solo" gesture: the clicked row turns on and every other row turns off,
//! no matter what was on beforehand. Shift+right click is the inverse
//! "isolate out" gesture: the clicked row turns off and every other row
//! turns on. Both are one-shot with no memory of prior state to toggle back
//! to; re-solo a different row, or plain-click things back individually, to
//! undo. Both paths go through `sync_dom` so the checkboxes (and the
//! scanlines overlay, which lives in `hud`'s DOM, not this panel's) always
//! reflect whatever `tuning` ends up holding.
//!
//! Built via raw DOM calls (web-sys), same reasoning as `grid_tuning_panel`
//! and `hud` - no GUI crate integration exists anywhere in this workspace.

use minwebgl as gl;
use std::{ cell::RefCell, rc::Rc };
use gl::web_sys::
{
  wasm_bindgen::{ prelude::Closure, JsCast },
  Document, Element, MouseEvent,
};

use super::{ grid_tuning::GridTuning, input_by_id, render_layers::RenderLayers };

/// A `label` wrapping both the text and the checkbox, so a left click
/// anywhere on the row toggles it, as its pointer cursor promises.
fn checkbox_row_html( id : &str, label : &str, checked : bool ) -> String
{
  let checked = if checked { "checked" } else { "" };
  format!
  (
    r#"<label id="{id}-row" title="Right click: solo this layer&#10;Shift+Right click: hide this layer, show the rest" style="display:flex;justify-content:space-between;align-items:center;font-size:10px;color:#7dd3fc;margin-bottom:6px;cursor:pointer">
      <span>{label}</span>
      <input type="checkbox" id="{id}" {checked} style="accent-color:#22d3ee">
    </label>"#
  )
}

/// One row's id + label + the `RenderLayers` bool field it reads and writes,
/// so every row (and the solo gesture, which needs to reach every *other*
/// row's field too) shares one path instead of each repeating its own
/// borrow/read/write sequence. Built with `layer_toggle!`, which writes both
/// accessors from the one field name so they can't name different fields.
struct LayerToggle
{
  id : &'static str,
  label : &'static str,
  get : fn( &RenderLayers ) -> bool,
  set : fn( &mut RenderLayers, bool ),
}

macro_rules! layer_toggle
{
  ( $id : literal, $label : literal, $field : ident ) =>
  {
    LayerToggle { id : $id, label : $label, get : | t | t.$field, set : | t, on | t.$field = on }
  };
}

const LAYER_TOGGLES : &[ LayerToggle ] =
&[
  layer_toggle!( "layers-show-grid", "Tactical Grid", show_grid ),
  layer_toggle!( "layers-show-view-ribbon", "View-Zone Ribbon", show_view_ribbon ),
  layer_toggle!( "layers-show-background", "Background", show_background ),
  layer_toggle!( "layers-show-starfield", "Starfield", show_starfield ),
  layer_toggle!( "layers-show-asteroids", "Asteroids", show_asteroids ),
  layer_toggle!( "layers-show-ships", "Ships", show_ships ),
  layer_toggle!( "layers-show-station", "Station", show_station ),
  layer_toggle!( "layers-show-gizmo", "Selection Gizmo", show_gizmo ),
  layer_toggle!( "layers-lighting-enabled", "Lighting", lighting_enabled ),
  layer_toggle!( "layers-shadows-enabled", "Shadows", shadows_enabled ),
  layer_toggle!( "layers-show-scanlines", "CRT Scanlines", show_scanlines ),
];

/// One `label: value` line per Render Layers row, for `grid_tuning_panel`'s
/// Copy Settings text. Built from `LAYER_TOGGLES` itself, so a row added to
/// the panel lands in the copied settings too instead of silently missing.
pub fn layers_summary( t : &RenderLayers ) -> String
{
  LAYER_TOGGLES.iter()
  .map( | toggle | format!( "{}: {}", toggle.label.to_lowercase(), ( toggle.get )( t ) ) )
  .collect::< Vec< _ > >()
  .join( "\n" )
}

/// Makes every row's checkbox (and the `hud`-owned CRT scanlines overlay)
/// match `t` - the one place both the plain left-click path and the solo
/// right-click path funnel through, so neither has to remember the other's
/// side effects. A missing row or overlay is a bug in the panel's or the
/// HUD's own markup, so it panics like `input_by_id` instead of being
/// skipped.
fn sync_dom( document : &Document, t : &RenderLayers )
{
  for toggle in LAYER_TOGGLES
  {
    input_by_id( document, toggle.id ).set_checked( ( toggle.get )( t ) );
  }

  // `ff-scanlines` is created by `hud::setup_hud`, not this module - by the
  // time either event handler below can fire, `main.rs` has already called
  // both setup functions, so the element exists.
  let overlay = document.get_element_by_id( "ff-scanlines" ).expect( "hud::setup_hud creates #ff-scanlines" );
  overlay.set_class_name( if t.show_scanlines { "ff-scanlines visible" } else { "ff-scanlines" } );
}

/// Left-click path: flip just `toggle`'s own field, then resync (only
/// matters for the scanlines row's overlay side effect, but running it
/// unconditionally is simpler than special-casing that one row).
fn bind_toggle( document : &Document, tuning : &Rc< RefCell< GridTuning > >, toggle : &'static LayerToggle )
{
  let element = input_by_id( document, toggle.id );
  let tuning = tuning.clone();
  let document = document.clone();
  let closure = Closure::< dyn FnMut() >::new
  (
    move ||
    {
      let input = input_by_id( &document, toggle.id );
      {
        let mut t = tuning.borrow_mut();
        ( toggle.set )( &mut t.layers, input.checked() );
      }
      sync_dom( &document, &tuning.borrow().layers );
    }
  );
  element.add_event_listener_with_callback( "change", closure.as_ref().unchecked_ref() ).unwrap();
  closure.forget();
}

/// Right-click "solo" path on `toggle`'s row: unconditionally turns `toggle`
/// on and every other row off, regardless of whatever was on beforehand.
/// Shift+right-click is the inverse "isolate out" gesture: `toggle` turns
/// off and every other row turns on. Neither remembers prior state to
/// restore - each click is a fresh, one-shot "set it all to this" command.
fn bind_solo( document : &Document, tuning : &Rc< RefCell< GridTuning > >, toggle : &'static LayerToggle )
{
  let row = document.get_element_by_id( &format!( "{}-row", toggle.id ) ).unwrap();
  let tuning = tuning.clone();
  let document = document.clone();
  let closure = Closure::< dyn FnMut( _ ) >::new
  (
    move | e : MouseEvent |
    {
      e.prevent_default();
      let isolate_out = e.shift_key();
      {
        let mut t = tuning.borrow_mut();
        for other in LAYER_TOGGLES { ( other.set )( &mut t.layers, isolate_out ); }
        ( toggle.set )( &mut t.layers, !isolate_out );
      }
      sync_dom( &document, &tuning.borrow().layers );
    }
  );
  row.add_event_listener_with_callback( "contextmenu", closure.as_ref().unchecked_ref() ).unwrap();
  closure.forget();
}

/// Builds and appends the Render Layers panel, wiring every row to `tuning`.
/// Positioned bottom-left, deliberately clear of the HUD's own top-bar
/// layout and `grid_tuning_panel`'s bottom-right panel.
pub fn setup_layers_panel( document : &Document, tuning : &Rc< RefCell< GridTuning > > )
{
  let t = tuning.borrow().layers;

  let rows_html : String = LAYER_TOGGLES.iter()
  .map( | toggle | checkbox_row_html( toggle.id, toggle.label, ( toggle.get )( &t ) ) )
  .collect();

  let panel_html = format!
  (
    r#"<div style="position:fixed;bottom:12px;left:12px;z-index:30;width:190px;max-height:85vh;overflow-y:auto;
        background:rgba(8,17,26,0.9);border:1px solid #164e63;border-radius:8px;padding:10px;
        font-family:monospace;font-size:11px;color:#e0f2fe">
      <div style="font-weight:bold;text-transform:uppercase;border-bottom:1px solid #164e63;padding-bottom:4px;margin-bottom:4px">Render Layers (dev)</div>
      <div style="color:#38708a;font-size:9px;line-height:1.4;margin-bottom:8px">Right click: solo layer<br>Shift+Right click: hide layer, show rest</div>
      {rows_html}
    </div>"#
  );

  let panel : Element = document.create_element( "div" ).unwrap();
  panel.set_inner_html( &panel_html );
  document.body().unwrap().append_child( &panel ).unwrap();

  for toggle in LAYER_TOGGLES
  {
    bind_toggle( document, tuning, toggle );
    bind_solo( document, tuning, toggle );
  }
}

#[ cfg( test ) ]
mod tests
{
  use super::{ layers_summary, LAYER_TOGGLES };
  use crate::debug::RenderLayers;

  /// Every switch off, so turning one row on shows exactly which field it
  /// writes.
  fn all_off() -> RenderLayers
  {
    RenderLayers
    {
      show_grid : false,
      show_view_ribbon : false,
      show_background : false,
      show_starfield : false,
      show_asteroids : false,
      show_ships : false,
      show_station : false,
      show_gizmo : false,
      lighting_enabled : false,
      shadows_enabled : false,
      show_scanlines : false,
      show_trajectories : false,
    }
  }

  /// Each row reads back what it writes and writes a field no other row
  /// writes; with eleven rows over the twelve fields, only
  /// `show_trajectories` (which has no row) is left unwritten.
  #[ test ]
  fn every_row_owns_a_distinct_field()
  {
    let mut written : Vec< RenderLayers > = Vec::new();
    for toggle in LAYER_TOGGLES
    {
      let mut t = all_off();
      ( toggle.set )( &mut t, true );
      assert!( ( toggle.get )( &t ), "row {:?} doesn't read back what it wrote", toggle.label );
      assert_ne!( t, all_off(), "row {:?} writes nothing", toggle.label );
      assert!( !written.contains( &t ), "row {:?} writes a field another row already writes", toggle.label );
      assert!( !t.show_trajectories, "row {:?} writes show_trajectories", toggle.label );
      written.push( t );
    }
    assert_eq!( written.len(), 11 );
  }

  #[ test ]
  fn summary_has_one_line_per_row()
  {
    let summary = layers_summary( &RenderLayers::default() );
    assert_eq!( summary.lines().count(), LAYER_TOGGLES.len() );
    for toggle in LAYER_TOGGLES
    {
      let prefix = format!( "{}: ", toggle.label.to_lowercase() );
      assert!( summary.lines().any( | line | line.starts_with( &prefix ) ), "missing row {:?} in {summary}", toggle.label );
    }
  }

  #[ test ]
  fn summary_reports_each_row_value()
  {
    let t = RenderLayers { show_grid : false, show_scanlines : true, ..RenderLayers::default() };
    let summary = layers_summary( &t );
    assert!( summary.lines().any( | line | line == "tactical grid: false" ), "{summary}" );
    assert!( summary.lines().any( | line | line == "crt scanlines: true" ), "{summary}" );
  }
}
