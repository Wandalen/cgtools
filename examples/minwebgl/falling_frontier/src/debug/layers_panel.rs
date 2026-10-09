//! Standalone "Render Layers" dev panel - every renderer-aspect visibility
//! switch united in one place, separate from `grid_tuning_panel`'s
//! slider-heavy shader tuning controls and from `hud`'s in-game "real UI";
//! the HUD carries no visibility toggles of its own.
//!
//! The rows come in two groups. **Scene layers** (`SCENE_TOGGLES`: grid,
//! background, starfield, asteroids, ships, station, trajectories) are each
//! a draw pass of their own, so any combination of them can be shown alone
//! (e.g. "only the grid") or hidden alone (e.g. "everything but asteroids").
//! **Overlays and lighting** (`OPTION_TOGGLES`: view-zone ribbon, selection
//! gizmo, lighting, shadows, CRT scanlines) change how the scene layers look
//! rather than drawing a layer by themselves - the ribbon is part of the grid pass,
//! the gizmo follows the selection, lighting and shadows shade the hulls -
//! so they are left alone by the solo gestures. A row that only means
//! something under another one is greyed out while that one is off:
//! View-Zone Ribbon under Tactical Grid, Shadows under Lighting.
//!
//! Trajectories start off, as in the three.js original; the frame loop
//! builds their ribbons the first time the row is on, and a failed build
//! turns the row back off (`layers_panel_sync`). Ship animation has no row
//! here: it is driven by the HUD's Pause/Play/Fast buttons
//! (`GridTuning::animate_ships` and `speed_multiplier`). Sensor rings were
//! cut entirely: the feature itself is gone from `trajectories.rs`, not
//! just hidden.
//!
//! Left click flips just the clicked row, same as any checkbox. Right click
//! on a scene row is an unconditional "solo" gesture: that layer turns on and every other scene
//! layer turns off, no matter what was on beforehand. Shift+right click is
//! the inverse "isolate out" gesture: that layer turns off and every other
//! scene layer turns on. Both are one-shot with no memory of prior state to
//! toggle back to (`scene_solo`); re-solo a different row, or plain-click
//! things back individually, to undo. The gestures are read on
//! `pointerdown` (right button), not `contextmenu`: Firefox shows its own
//! menu without dispatching `contextmenu` while Shift is held, so a
//! `contextmenu` listener could never see Shift+right click there. The
//! `contextmenu` listener only suppresses the browser menu where it can.
//! They need a mouse: touch input has no right button or Shift key. Both paths go through `sync_dom` so
//! the checkboxes (and the scanlines overlay, which lives in `hud`'s DOM,
//! not this panel's) always reflect whatever `tuning` ends up holding.
//!
//! Built via raw DOM calls (web-sys), same reasoning as `grid_tuning_panel`
//! and `hud` - no GUI crate integration exists anywhere in this workspace.

use minwebgl as gl;
use std::{ cell::RefCell, rc::Rc };
use gl::web_sys::
{
  wasm_bindgen::{ prelude::Closure, JsCast },
  Document, Element, MouseEvent, PointerEvent,
};

use super::{ grid_tuning::GridTuning, input_by_id, panel_shell_html, render_layers::RenderLayers, PanelSide };

const SOLO_HINT : &str = "Right click: show only this scene layer&#10;Shift+Right click: hide it, show the other scene layers";

/// A `label` wrapping both the text and the checkbox, so a left click
/// anywhere on the row toggles it, as its pointer cursor promises. Scene rows
/// carry the solo gestures' tooltip; a disabled row (see
/// `LayerToggle::enabled`) is greyed out by the panel's own stylesheet.
fn checkbox_row_html( toggle : &LayerToggle, t : &RenderLayers ) -> String
{
  let id = toggle.id;
  let label = toggle.label;
  let checked = if ( toggle.get )( t ) { "checked" } else { "" };
  let disabled = if ( toggle.enabled )( t ) { "" } else { "disabled" };
  let title = if toggle.scene { format!( r#" title="{SOLO_HINT}""# ) } else { String::new() };
  format!
  (
    r#"<label id="{id}-row"{title} style="display:flex;justify-content:space-between;align-items:center;font-size:10px;color:#7dd3fc;margin-bottom:6px;cursor:pointer">
      <span>{label}</span>
      <input type="checkbox" id="{id}" {checked} {disabled} style="accent-color:#22d3ee">
    </label>"#
  )
}

/// One row's id + label + the `RenderLayers` bool field it reads and writes,
/// so every row (and the solo gesture, which needs to reach every *other*
/// scene row's field too) shares one path instead of each repeating its own
/// borrow/read/write sequence. Built with `layer_toggle!`, which writes both
/// accessors from the one field name so they can't name different fields.
struct LayerToggle
{
  id : &'static str,
  label : &'static str,
  get : fn( &RenderLayers ) -> bool,
  set : fn( &mut RenderLayers, bool ),
  /// A scene layer the solo gestures sweep (see the module doc).
  scene : bool,
  /// Whether the row can be changed: `false` greys it out while the row it
  /// depends on is off.
  enabled : fn( &RenderLayers ) -> bool,
}

macro_rules! layer_toggle
{
  ( $id : literal, $label : literal, $field : ident, scene ) =>
  {
    LayerToggle
    {
      id : $id, label : $label, get : | t | t.$field, set : | t, on | t.$field = on,
      scene : true, enabled : | _ | true,
    }
  };
  ( $id : literal, $label : literal, $field : ident, option ) =>
  {
    layer_toggle!( $id, $label, $field, option, | _ | true )
  };
  ( $id : literal, $label : literal, $field : ident, option, $enabled : expr ) =>
  {
    LayerToggle
    {
      id : $id, label : $label, get : | t | t.$field, set : | t, on | t.$field = on,
      scene : false, enabled : $enabled,
    }
  };
}

/// Rows that are draw passes of their own - the ones the solo gestures sweep.
const SCENE_TOGGLES : &[ LayerToggle ] =
&[
  layer_toggle!( "layers-show-grid", "Tactical Grid", show_grid, scene ),
  layer_toggle!( "layers-show-background", "Background", show_background, scene ),
  layer_toggle!( "layers-show-starfield", "Starfield", show_starfield, scene ),
  layer_toggle!( "layers-show-asteroids", "Asteroids", show_asteroids, scene ),
  layer_toggle!( "layers-show-ships", "Ships", show_ships, scene ),
  layer_toggle!( "layers-show-station", "Station", show_station, scene ),
  layer_toggle!( "layers-show-trajectories", "Trajectories", show_trajectories, scene ),
];

/// Rows that change how the scene layers look; the solo gestures leave them
/// alone.
const OPTION_TOGGLES : &[ LayerToggle ] =
&[
  layer_toggle!( "layers-show-view-ribbon", "View-Zone Ribbon", show_view_ribbon, option, | t | t.show_grid ),
  layer_toggle!( "layers-show-gizmo", "Selection Gizmo", show_gizmo, option ),
  layer_toggle!( "layers-lighting-enabled", "Lighting", lighting_enabled, option ),
  layer_toggle!( "layers-shadows-enabled", "Shadows", shadows_enabled, option, | t | t.lighting_enabled ),
  layer_toggle!( "layers-show-scanlines", "CRT Scanlines", show_scanlines, option ),
];

/// Every row, scene layers first, in panel order.
fn all_toggles() -> impl Iterator< Item = &'static LayerToggle >
{
  SCENE_TOGGLES.iter().chain( OPTION_TOGGLES )
}

/// The solo gestures on scene row `row`: with `isolate_out` false (right
/// click) `row` turns on and every other scene layer off; with it true
/// (Shift+right click) `row` turns off and every other scene layer on. The
/// overlay and lighting rows keep whatever they held.
fn scene_solo( t : &mut RenderLayers, row : &LayerToggle, isolate_out : bool )
{
  for other in SCENE_TOGGLES { ( other.set )( t, isolate_out ); }
  ( row.set )( t, !isolate_out );
}

/// One `label: value` line per Render Layers row, for `grid_tuning_panel`'s
/// Copy Settings text. Built from the row tables themselves, so a row added
/// to the panel lands in the copied settings too instead of silently missing.
pub fn layers_summary( t : &RenderLayers ) -> String
{
  all_toggles()
  .map( | toggle | format!( "{}: {}", toggle.label.to_lowercase(), ( toggle.get )( t ) ) )
  .collect::< Vec< _ > >()
  .join( "\n" )
}

/// Makes every row's checkbox (its value and whether it is greyed out) and
/// the `hud`-owned CRT scanlines overlay match `t` - the one place both the
/// plain left-click path and the solo right-click path funnel through, so
/// neither has to remember the other's side effects. A missing row or
/// overlay is a bug in the panel's or the HUD's own markup, so it panics
/// like `input_by_id` instead of being skipped.
fn sync_dom( document : &Document, t : &RenderLayers )
{
  for toggle in all_toggles()
  {
    let input = input_by_id( document, toggle.id );
    input.set_checked( ( toggle.get )( t ) );
    input.set_disabled( !( toggle.enabled )( t ) );
  }

  // `ff-scanlines` is created by `hud::setup_hud`, not this module - by the
  // time either event handler below can fire, `main.rs` has already called
  // both setup functions, so the element exists.
  let overlay = document.get_element_by_id( "ff-scanlines" ).expect( "hud::setup_hud creates #ff-scanlines" );
  overlay.set_class_name( crate::hud::scanlines_class( t.show_scanlines ) );
}

/// Brings the panel back in line with `t` after the frame loop itself
/// changed a switch rather than a row - a failed trajectory build turning
/// `show_trajectories` back off would otherwise leave its row checked.
pub fn layers_panel_sync( t : &RenderLayers )
{
  let document = gl::web_sys::window().unwrap().document().unwrap();
  sync_dom( &document, t );
}

/// Left-click path: flip just `toggle`'s own field, then resync (for the
/// scanlines row's overlay and for the rows greyed out under this one).
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

/// Right-click "solo" path on scene row `toggle` (see `scene_solo`), read
/// on `pointerdown` so Shift is seen in every browser (see the module doc).
fn bind_solo( document : &Document, tuning : &Rc< RefCell< GridTuning > >, toggle : &'static LayerToggle )
{
  let row = document.get_element_by_id( &format!( "{}-row", toggle.id ) ).unwrap();
  {
    let tuning = tuning.clone();
    let document = document.clone();
    let closure = Closure::< dyn FnMut( _ ) >::new
    (
      move | e : PointerEvent |
      {
        if e.button() != 2 { return; }
        scene_solo( &mut tuning.borrow_mut().layers, toggle, e.shift_key() );
        sync_dom( &document, &tuning.borrow().layers );
      }
    );
    row.add_event_listener_with_callback( "pointerdown", closure.as_ref().unchecked_ref() ).unwrap();
    closure.forget();
  }
  let suppress_menu = Closure::< dyn FnMut( _ ) >::new( | e : MouseEvent | e.prevent_default() );
  row.add_event_listener_with_callback( "contextmenu", suppress_menu.as_ref().unchecked_ref() ).unwrap();
  suppress_menu.forget();
}

/// Builds and appends the Render Layers panel, wiring every row to `tuning`.
/// Bottom-left, opposite `grid_tuning_panel`'s bottom-right panel; see
/// `panel_shell_html` for what the shared layout does and doesn't
/// guarantee.
pub fn setup_layers_panel( document : &Document, tuning : &Rc< RefCell< GridTuning > > )
{
  let t = tuning.borrow().layers;
  let rows_html = | toggles : &[ LayerToggle ] | -> String
  {
    toggles.iter().map( | toggle | checkbox_row_html( toggle, &t ) ).collect()
  };
  let scene_rows = rows_html( SCENE_TOGGLES );
  let option_rows = rows_html( OPTION_TOGGLES );
  let group_heading = | text : &str | format!
  (
    r#"<div style="font-size:9px;color:#0e7490;text-transform:uppercase;letter-spacing:0.05em;padding-top:4px;margin-bottom:4px">{text}</div>"#
  );
  let scene_heading = group_heading( "Scene layers" );
  let option_heading = group_heading( "Overlays &amp; lighting" );

  let body_html = format!
  (
    r#"<style>#layers-panel label:has(input:disabled) {{ opacity:0.4; cursor:default; }}</style>
      <div style="color:#38708a;font-size:9px;line-height:1.4;margin-bottom:8px">Right click a scene layer: solo it<br>Shift+Right click: hide it, show the rest</div>
      {scene_heading}
      {scene_rows}
      {option_heading}
      {option_rows}"#
  );
  let panel_html = panel_shell_html( "layers-panel", PanelSide::Left, 190, "Render Layers (dev)", &body_html );

  let panel : Element = document.create_element( "div" ).unwrap();
  panel.set_inner_html( &panel_html );
  document.body().unwrap().append_child( &panel ).unwrap();

  for toggle in all_toggles()
  {
    bind_toggle( document, tuning, toggle );
  }
  for toggle in SCENE_TOGGLES
  {
    bind_solo( document, tuning, toggle );
  }
}

#[ cfg( test ) ]
mod tests
{
  use super::{ all_toggles, layers_summary, scene_solo, OPTION_TOGGLES, SCENE_TOGGLES };
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
  /// writes; with twelve rows over the twelve fields, every field has its
  /// row.
  #[ test ]
  fn every_row_owns_a_distinct_field()
  {
    let mut written : Vec< RenderLayers > = Vec::new();
    for toggle in all_toggles()
    {
      let mut t = all_off();
      ( toggle.set )( &mut t, true );
      assert!( ( toggle.get )( &t ), "row {:?} doesn't read back what it wrote", toggle.label );
      assert_ne!( t, all_off(), "row {:?} writes nothing", toggle.label );
      assert!( !written.contains( &t ), "row {:?} writes a field another row already writes", toggle.label );
      written.push( t );
    }
    assert_eq!( written.len(), 12 );
  }

  /// Right click on a scene row: only that scene layer stays on, and the
  /// overlay and lighting rows keep their values.
  #[ test ]
  fn solo_keeps_one_scene_layer_and_leaves_the_options_alone()
  {
    for ( index, row ) in SCENE_TOGGLES.iter().enumerate()
    {
      let before = RenderLayers { show_scanlines : true, shadows_enabled : false, ..RenderLayers::default() };
      let mut t = before;
      scene_solo( &mut t, row, false );
      for ( other_index, other ) in SCENE_TOGGLES.iter().enumerate()
      {
        assert_eq!( ( other.get )( &t ), other_index == index, "solo {:?}: scene row {:?}", row.label, other.label );
      }
      for option in OPTION_TOGGLES
      {
        assert_eq!( ( option.get )( &t ), ( option.get )( &before ), "solo {:?} changed option {:?}", row.label, option.label );
      }
    }
  }

  /// Shift+right click on a scene row: every other scene layer turns on, that
  /// one off, and the overlay and lighting rows keep their values - in
  /// particular the scanline overlay doesn't come on.
  #[ test ]
  fn isolate_out_hides_one_scene_layer_and_leaves_the_options_alone()
  {
    for ( index, row ) in SCENE_TOGGLES.iter().enumerate()
    {
      let before = all_off();
      let mut t = before;
      scene_solo( &mut t, row, true );
      for ( other_index, other ) in SCENE_TOGGLES.iter().enumerate()
      {
        assert_eq!( ( other.get )( &t ), other_index != index, "isolate {:?}: scene row {:?}", row.label, other.label );
      }
      for option in OPTION_TOGGLES
      {
        assert_eq!( ( option.get )( &t ), ( option.get )( &before ), "isolate {:?} changed option {:?}", row.label, option.label );
      }
    }
  }

  /// The ribbon row is greyed out exactly while the grid is off, the shadows
  /// row exactly while lighting is off, and nothing else ever is.
  #[ test ]
  fn dependent_rows_grey_out_under_their_parent()
  {
    let enabled = | label : &str, t : &RenderLayers | ( all_toggles().find( | r | r.label == label ).unwrap().enabled )( t );
    let on = RenderLayers::default();
    assert!( enabled( "View-Zone Ribbon", &on ) );
    assert!( !enabled( "View-Zone Ribbon", &RenderLayers { show_grid : false, ..on } ) );
    assert!( enabled( "Shadows", &on ) );
    assert!( !enabled( "Shadows", &RenderLayers { lighting_enabled : false, ..on } ) );
    for row in all_toggles().filter( | r | r.label != "View-Zone Ribbon" && r.label != "Shadows" )
    {
      assert!( ( row.enabled )( &all_off() ), "row {:?} greys out", row.label );
    }
  }

  /// Each row, flipped on alone from all-off, turns on the field this table
  /// (written independently of `SCENE_TOGGLES` / `OPTION_TOGGLES`) names for
  /// its label and puts exactly one `true` line in the summary - so two rows
  /// with swapped accessors, or a row wired to the wrong field, fail here.
  #[ test ]
  fn each_row_flips_the_field_its_label_names()
  {
    type FieldRead = fn( &RenderLayers ) -> bool;
    let expected : [ ( &str, FieldRead ); 12 ] =
    [
      ( "Tactical Grid", | t | t.show_grid ),
      ( "Background", | t | t.show_background ),
      ( "Starfield", | t | t.show_starfield ),
      ( "Asteroids", | t | t.show_asteroids ),
      ( "Ships", | t | t.show_ships ),
      ( "Station", | t | t.show_station ),
      ( "Trajectories", | t | t.show_trajectories ),
      ( "View-Zone Ribbon", | t | t.show_view_ribbon ),
      ( "Selection Gizmo", | t | t.show_gizmo ),
      ( "Lighting", | t | t.lighting_enabled ),
      ( "Shadows", | t | t.shadows_enabled ),
      ( "CRT Scanlines", | t | t.show_scanlines ),
    ];
    assert_eq!( all_toggles().count(), expected.len() );
    for ( label, field ) in expected
    {
      let row = all_toggles().find( | r | r.label == label ).unwrap_or_else( || panic!( "no row {label:?}" ) );
      let mut t = all_off();
      ( row.set )( &mut t, true );
      assert!( field( &t ), "row {label:?} doesn't write its own field" );
      let summary = layers_summary( &t );
      let on : Vec< &str > = summary.lines().filter( | line | line.ends_with( ": true" ) ).collect();
      assert_eq!( on, vec![ format!( "{}: true", label.to_lowercase() ).as_str() ], "{summary}" );
    }
  }

  #[ test ]
  fn summary_has_one_line_per_row()
  {
    let summary = layers_summary( &RenderLayers::default() );
    assert_eq!( summary.lines().count(), all_toggles().count() );
    for toggle in all_toggles()
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
