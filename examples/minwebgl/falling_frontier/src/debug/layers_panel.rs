//! Standalone "Render Layers" dev panel - every renderer-aspect visibility
//! switch (grid, background, starfield, per-object-type hull rendering,
//! view-zone ribbon, selection gizmo, lighting, shadows, CRT scanlines)
//! united in one place, separate from `grid_tuning_panel`'s slider-heavy
//! shader tuning controls and from `hud`'s in-game "real UI". Lets any
//! combination of scene layers be isolated (e.g. "only the grid") or hidden
//! (e.g. "everything but asteroids") from a single menu instead of hunting
//! each switch down across `grid_tuning_panel`'s subsections or `hud`'s
//! toolbar.
//!
//! Trajectories and ship animation were cut from this panel deliberately -
//! both were still unfinished/experimental, so they're left off by default
//! (`GridTuning::default`) with no UI surface to turn them back on. The
//! underlying `GridTuning::show_trajectories`/`animate_ships` fields and the
//! rendering code that reads them (`trajectories.rs`, `main.rs`'s fleet-
//! motion step) are untouched - only this panel's rows are gone. Sensor
//! rings were cut further still: the feature itself is gone from
//! `trajectories.rs`, not just hidden.
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
  Document, Element, HtmlInputElement, MouseEvent,
};

use super::grid_tuning::GridTuning;

fn checkbox_row_html( id : &str, label : &str, checked : bool ) -> String
{
  let checked = if checked { "checked" } else { "" };
  format!
  (
    r#"<div id="{id}-row" title="Right click: solo this layer&#10;Shift+Right click: hide this layer, show the rest" style="display:flex;justify-content:space-between;align-items:center;font-size:10px;color:#7dd3fc;margin-bottom:6px;cursor:pointer">
      <span>{label}</span>
      <input type="checkbox" id="{id}" {checked} style="accent-color:#22d3ee">
    </div>"#
  )
}

fn input_by_id( document : &Document, id : &str ) -> HtmlInputElement
{
  document.get_element_by_id( id ).unwrap().dyn_into::< HtmlInputElement >().unwrap()
}

/// One row's id + label + the `GridTuning` bool field it reads/writes.
/// `field` is a projection (`|t| &mut t.some_bool`), same pattern
/// `hud::bind_tuning_toggle` uses, so every row (and the solo gesture, which
/// needs to reach every *other* row's field too) shares one path instead of
/// each repeating its own borrow/read/write/rebuild sequence.
struct LayerToggle
{
  id : &'static str,
  label : &'static str,
  field : fn( &mut GridTuning ) -> &mut bool,
}

const LAYER_TOGGLES : &[ LayerToggle ] =
&[
  LayerToggle { id : "layers-show-grid", label : "Tactical Grid", field : | t | &mut t.show_grid },
  LayerToggle { id : "layers-show-view-ribbon", label : "View-Zone Ribbon", field : | t | &mut t.show_view_ribbon },
  LayerToggle { id : "layers-show-background", label : "Background", field : | t | &mut t.show_background },
  LayerToggle { id : "layers-show-starfield", label : "Starfield", field : | t | &mut t.show_starfield },
  LayerToggle { id : "layers-show-asteroids", label : "Asteroids", field : | t | &mut t.show_asteroids },
  LayerToggle { id : "layers-show-ships", label : "Ships", field : | t | &mut t.show_ships },
  LayerToggle { id : "layers-show-station", label : "Station", field : | t | &mut t.show_station },
  LayerToggle { id : "layers-show-gizmo", label : "Selection Gizmo", field : | t | &mut t.show_gizmo },
  LayerToggle { id : "layers-lighting-enabled", label : "Lighting", field : | t | &mut t.lighting_enabled },
  LayerToggle { id : "layers-shadows-enabled", label : "Shadows", field : | t | &mut t.shadows_enabled },
  LayerToggle { id : "layers-show-scanlines", label : "CRT Scanlines", field : | t | &mut t.show_scanlines },
];

/// Makes every row's checkbox (and the `hud`-owned CRT scanlines overlay)
/// match `t` - the one place both the plain left-click path and the solo
/// right-click path funnel through, so neither has to remember the other's
/// side effects.
fn sync_dom( document : &Document, t : &GridTuning )
{
  for toggle in LAYER_TOGGLES
  {
    let mut snapshot = *t;
    let checked = *( toggle.field )( &mut snapshot );
    if let Some( input ) = document.get_element_by_id( toggle.id ).and_then( | el | el.dyn_into::< HtmlInputElement >().ok() )
    {
      input.set_checked( checked );
    }
  }

  // `ff-scanlines` is created by `hud::setup_hud`, not this module - by the
  // time either event handler below can fire, `main.rs` has already called
  // both setup functions, so the element is guaranteed to exist.
  if let Some( overlay ) = document.get_element_by_id( "ff-scanlines" )
  {
    overlay.set_class_name( if t.show_scanlines { "ff-scanlines visible" } else { "ff-scanlines" } );
  }
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
        *( toggle.field )( &mut t ) = input.checked();
      }
      sync_dom( &document, &tuning.borrow() );
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
        for other in LAYER_TOGGLES { *( other.field )( &mut t ) = isolate_out; }
        *( toggle.field )( &mut t ) = !isolate_out;
      }
      sync_dom( &document, &tuning.borrow() );
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
  let t = *tuning.borrow();

  let rows_html : String = LAYER_TOGGLES.iter()
  .map( | toggle |
  {
    let mut snapshot = t;
    checkbox_row_html( toggle.id, toggle.label, *( toggle.field )( &mut snapshot ) )
  } )
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
