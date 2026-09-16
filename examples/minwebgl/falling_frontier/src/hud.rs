//! Tactical HUD overlay - the "real" in-game UI, as opposed to `debug/`'s
//! dev-only tuning panel. Ported from
//! `examples/threejs/falling_frontier/index.html` + `src/style.css` +
//! `src/interaction/uiControls.js` + `src/ui/unitPanel.js`.
//!
//! Built via raw DOM calls (web-sys), same as `debug/grid_tuning_panel.rs` -
//! and for the same reason, plus one more: the JS reference's markup leans
//! on Tailwind's CDN build (`<script src="https://cdn.tailwindcss.com">`)
//! and a Google Fonts stylesheet, neither of which this port pulls in (self-
//! contained, no CDN dependency) - so the HUD's own `<style>` block below is
//! plain CSS reproducing just the classes (`.ui-panel`, `.btn-tactical`,
//! `.scanlines`, ...) the ported markup actually uses, with a system font
//! stack standing in for `Rajdhani`/`Share Tech Mono`.
//!
//! Deliberate scope cut vs. the JS reference: `index.html` also has a
//! mission-objectives checklist, a fake resources/credits readout, a unit
//! card's tactical-module buttons (uplink/jammer/targeting/torpedo) and
//! hull/shield/thrust status bars, a subsystem-matrix footer, and a bottom
//! toolbar of category icons - none of these are wired to any real state
//! even in the JS reference itself (no event listeners read or write them
//! anywhere in `uiControls.js`/`unitPanel.js`), so they're pure decoration
//! with zero behavior. Porting inert decoration for its own sake doesn't
//! serve this port's goal, so all of it is cut; everything below is either
//! functional (reads/writes real state) or cheap static flavor text ported
//! 1:1 (the date/location/resources readout in the top bar - equally static
//! in the JS reference, just kept for visual completeness of the "status
//! bar" this milestone asks for).
//!
//! Every scene-layer visibility switch (grid/trajectories/sensor
//! rings/animate ships/scanlines/etc.) used to also have its own toggle
//! button here, duplicating `debug::layers_panel`'s "Render Layers" dev
//! panel. That duplication is gone now - this module only keeps the Play/
//! Pause/Fast time controls and the Reset Camera button, which aren't
//! visibility switches and have no equivalent in the dev panel.

use minwebgl as gl;
use std::{ cell::RefCell, rc::Rc };
use gl::web_sys::
{
  wasm_bindgen::{ prelude::Closure, JsCast },
  Document, Element, HtmlButtonElement,
};

use crate::debug::GridTuning;

const CSS : &str = r"
  .ff-hud, .ff-hud * { box-sizing: border-box; }
  .ff-hud {
    position: fixed; inset: 0; z-index: 15; pointer-events: none;
    display: flex; flex-direction: column; justify-content: space-between;
    padding: 10px; font-family: 'Segoe UI', Verdana, sans-serif; color: #8be0ff;
    font-size: 11px; letter-spacing: 0.03em; user-select: none;
  }
  .ff-panel {
    background: rgba(8, 24, 38, 0.75); border: 1px solid rgba(0, 229, 255, 0.25);
    border-radius: 8px; box-shadow: 0 0 15px rgba(0, 229, 255, 0.08);
    pointer-events: auto;
  }
  .ff-top-bar { display: flex; justify-content: space-between; align-items: flex-start; width: 100%; }
  .ff-time-panel { display: flex; align-items: center; gap: 12px; padding: 8px 14px; border-left: 4px solid #22d3ee; }
  .ff-label { color: #7dd3fc; font-size: 9px; text-transform: uppercase; font-weight: 600; }
  .ff-value { color: #67e8f9; font-weight: bold; font-size: 13px; }
  .ff-value-big { color: #fff; font-size: 15px; }
  .ff-divider { width: 1px; height: 22px; background: #164e63; }
  .ff-location-panel { padding: 8px 20px; text-align: center; }
  .ff-location-title { font-weight: bold; color: #fff; text-transform: uppercase; letter-spacing: 0.1em; }
  .ff-location-stats { color: #7dd3fc; font-size: 10px; margin-top: 4px; }
  .ff-resources-panel { display: flex; align-items: center; gap: 16px; padding: 8px 14px; border-right: 4px solid #22d3ee; }
  .ff-resource-label { color: #7dd3fc; font-size: 9px; }
  .ff-resource-value { color: #fff; font-weight: bold; }
  .ff-btn {
    background: rgba(12, 38, 58, 0.6); border: 1px solid rgba(0, 229, 255, 0.3);
    color: #7dd3fc; border-radius: 4px; padding: 4px 8px; cursor: pointer; font-weight: bold;
  }
  .ff-btn:hover { background: rgba(0, 229, 255, 0.2); border-color: #22d3ee; color: #fff; }
  .ff-btn.active { background: rgba(0, 229, 255, 0.3); border-color: #22d3ee; color: #fff; }
  .ff-mid { display: flex; justify-content: flex-end; align-items: flex-start; flex: 1; padding: 12px 0; }
  .ff-unit-panel {
    position: fixed; top: 70px; right: 240px; width: 260px; padding: 12px;
    display: none; z-index: 17;
  }
  .ff-unit-panel.visible { display: block; }
  .ff-unit-header { display: flex; justify-content: space-between; align-items: center; font-size: 9px; color: #22d3ee; text-transform: uppercase; }
  .ff-unit-status { color: #34d399; font-weight: bold; }
  .ff-unit-name { font-size: 18px; font-weight: 900; color: #fff; margin: 2px 0 6px; }
  .ff-unit-class { display: inline-block; color: #7dd3fc; font-size: 10px; background: rgba(8, 51, 68, 0.8); padding: 3px 8px; border-radius: 3px; border: 1px solid rgba(34, 211, 238, 0.4); margin-bottom: 10px; }
  .ff-unit-commander-row { background: rgba(8, 51, 68, 0.5); padding: 8px; border-radius: 4px; border: 1px solid rgba(21, 94, 117, 0.4); }
  .ff-unit-commander-label { font-size: 9px; color: #7dd3fc; text-transform: uppercase; }
  .ff-unit-commander { font-weight: bold; color: #fff; font-size: 12px; }
  .ff-scanlines {
    position: fixed; inset: 0; z-index: 16; pointer-events: none; display: none;
    background:
      linear-gradient( rgba(18, 16, 16, 0) 50%, rgba(0, 0, 0, 0.25) 50% ),
      linear-gradient( 90deg, rgba(255, 0, 0, 0.03), rgba(0, 255, 0, 0.01), rgba(0, 0, 255, 0.03) );
    background-size: 100% 3px, 6px 100%;
  }
  .ff-scanlines.visible { display: block; }
";

/// Info about whatever is currently selected - `main.rs` resolves this from
/// `PickedKind` + `asteroids`/`ships`/`station` and passes it in; this
/// module doesn't know about pick ids at all.
pub struct UnitInfo
{
  pub name : String,
  pub commander : String,
  pub class_label : String,
}

/// Builds and appends the HUD overlay, wiring every functional control to
/// `tuning`. Returns nothing - the DOM is the only handle callers need
/// (`refresh_unit_panel` looks elements up by id).
pub fn setup_hud( document : &Document, tuning : &Rc< RefCell< GridTuning > > )
{
  let style : Element = document.create_element( "style" ).unwrap();
  style.set_text_content( Some( CSS ) );
  document.head().unwrap().append_child( &style ).unwrap();

  let t = *tuning.borrow();
  let ( pause_active, play_active, fast_active ) = time_control_button_classes( t.animate_ships, t.speed_multiplier );
  let body_html = format!
  (
    r#"<div class="ff-top-bar">
      <div class="ff-panel ff-time-panel">
        <div>
          <div class="ff-label">Date / Stardate</div>
          <div class="ff-value">SAT 2 JAN <span class="ff-value-big">2545</span></div>
        </div>
        <div class="ff-divider"></div>
        <button type="button" id="ff-btn-pause" class="ff-btn {pause_active}">||</button>
        <button type="button" id="ff-btn-play" class="ff-btn {play_active}">&gt;</button>
        <button type="button" id="ff-btn-fast" class="ff-btn {fast_active}">&gt;&gt;</button>
        <div class="ff-divider"></div>
        <button type="button" id="ff-btn-reset-cam" class="ff-btn">Reset Camera</button>
      </div>
      <div class="ff-panel ff-location-panel">
        <div class="ff-location-title">Titan Colony <span style="color:#0e7490">[SECTOR 04-B]</span></div>
        <div class="ff-location-stats">O2: 98% &nbsp;|&nbsp; GRAV: 0.14g &nbsp;|&nbsp; RAD: LOW</div>
      </div>
      <div class="ff-panel ff-resources-panel">
        <div><div class="ff-resource-label">METALS</div><div class="ff-resource-value">30,000</div></div>
        <div><div class="ff-resource-label">FUEL</div><div class="ff-resource-value">30,000</div></div>
        <div><div class="ff-resource-label">CREDITS</div><div class="ff-resource-value">75,222</div></div>
      </div>
    </div>
    <div class="ff-mid">
      <div id="ff-unit-panel" class="ff-panel ff-unit-panel">
        <div class="ff-unit-header">
          <span>Contact</span>
          <span class="ff-unit-status">&#9679; ACTIVE</span>
        </div>
        <div id="ff-unit-name" class="ff-unit-name"></div>
        <div id="ff-unit-class" class="ff-unit-class"></div>
        <div class="ff-unit-commander-row">
          <div class="ff-unit-commander-label">Commander</div>
          <div id="ff-unit-commander" class="ff-unit-commander"></div>
        </div>
      </div>
    </div>"#,
  );

  let root : Element = document.create_element( "div" ).unwrap();
  root.set_class_name( "ff-hud" );
  root.set_inner_html( &body_html );
  document.body().unwrap().append_child( &root ).unwrap();

  let scanlines : Element = document.create_element( "div" ).unwrap();
  scanlines.set_id( "ff-scanlines" );
  scanlines.set_class_name( "ff-scanlines" );
  document.body().unwrap().append_child( &scanlines ).unwrap();

  bind_time_controls( document, tuning );
}

/// Pause/Play/Fast - ported from `bindAnimateToggleAndTimeControls` in
/// `uiControls.js`. Play/Fast both enable `animate_ships`, differing only in
/// `speed_multiplier`; Pause clears `animate_ships` and leaves the
/// multiplier as-is (matching the JS, which never resets it on pause).
const TIME_CONTROL_IDS : [ &str; 3 ] = [ "ff-btn-pause", "ff-btn-play", "ff-btn-fast" ];

/// Decides which of the three time-control buttons should render as
/// `active` for `setup_hud`'s initial paint, from `tuning`'s already-current
/// state. Mirrors `set_time_control_active`'s own three-way split: paused
/// when `animate_ships` is clear, else Play/Fast is picked by
/// `speed_multiplier` (Play = 1.0, Fast = 2.5 - a `> 1.0` threshold avoids
/// relying on exact float equality) - so the initial render matches
/// whatever state `tuning` already holds, the same guarantee the other four
/// toggles have. Returns each button's own `active`-or-empty CSS class
/// fragment, in (pause, play, fast) order.
fn time_control_button_classes( animate_ships : bool, speed_multiplier : f32 ) -> ( &'static str, &'static str, &'static str )
{
  if !animate_ships
  {
    ( "active", "", "" )
  }
  else if speed_multiplier > 1.0
  {
    ( "", "", "active" )
  }
  else
  {
    ( "", "active", "" )
  }
}

fn set_time_control_active( document : &Document, active_id : &str )
{
  for id in TIME_CONTROL_IDS
  {
    if let Some( el ) = document.get_element_by_id( id )
    {
      el.set_class_name( if id == active_id { "ff-btn active" } else { "ff-btn" } );
    }
  }
}

// Fix(BUG-454): originally, a HUD "Animate Ships Motion" toggle button
// (`ff-toggle-animate`) and `bind_time_controls`'s Pause/Play/Fast buttons
// were two controls, in this same file, both driving
// `GridTuning.animate_ships`/`speed_multiplier` - not the HUD-vs-dev-panel
// gap this module's own doc comment calls out as intentional (that's a
// different pair of files). Clicking one surface left the other's `active`
// class/status text stale.
// Root cause: each handler only ever repainted its own button(s), never
// the other control's DOM.
// Pitfall: two controls over the same state need each handler to repaint
// *both* surfaces - repainting only the button that was clicked leaves the
// other one lying about the current state.
// The HUD-side "Animate Ships Motion" toggle itself no longer exists (see
// this module's doc comment - toggle buttons live in the dev panel only
// now), so `animate_toggle_repaint` below is a permanent no-op guarded by
// its own `else { return }`; kept rather than pulled since removing it
// would also mean re-deriving `bind_time_controls`'s Pause/Play/Fast
// handlers, out of scope for a merge-conflict resolution.

/// Repaints the "Animate Ships Motion" toggle to match `active`, if that
/// HUD button still exists (it doesn't anymore - see the comment above).
fn animate_toggle_repaint( document : &Document, active : bool )
{
  let Some( el ) = document.get_element_by_id( "ff-toggle-animate" ) else { return };
  el.set_class_name( if active { "ff-toggle active" } else { "ff-toggle" } );
  if let Some( status ) = el.query_selector( ".ff-toggle-status" ).unwrap()
  {
    status.set_text_content( Some( if active { "[ACTIVE]" } else { "[PAUSED]" } ) );
  }
}

fn bind_time_controls( document : &Document, tuning : &Rc< RefCell< GridTuning > > )
{
  {
    let tuning = tuning.clone();
    let document = document.clone();
    let button = document.get_element_by_id( "ff-btn-pause" ).unwrap().dyn_into::< HtmlButtonElement >().unwrap();
    let closure = Closure::< dyn FnMut() >::new
    (
      move ||
      {
        tuning.borrow_mut().animate_ships = false;
        set_time_control_active( &document, "ff-btn-pause" );
        // Fix(BUG-454): keep "Animate Ships Motion" in sync with Pause.
        animate_toggle_repaint( &document, false );
      }
    );
    button.add_event_listener_with_callback( "click", closure.as_ref().unchecked_ref() ).unwrap();
    closure.forget();
  }
  {
    let tuning = tuning.clone();
    let document = document.clone();
    let button = document.get_element_by_id( "ff-btn-play" ).unwrap().dyn_into::< HtmlButtonElement >().unwrap();
    let closure = Closure::< dyn FnMut() >::new
    (
      move ||
      {
        let mut t = tuning.borrow_mut();
        t.animate_ships = true;
        t.speed_multiplier = 1.0;
        drop( t );
        set_time_control_active( &document, "ff-btn-play" );
        // Fix(BUG-454): keep "Animate Ships Motion" in sync with Play.
        animate_toggle_repaint( &document, true );
      }
    );
    button.add_event_listener_with_callback( "click", closure.as_ref().unchecked_ref() ).unwrap();
    closure.forget();
  }
  {
    let tuning = tuning.clone();
    let document = document.clone();
    let button = document.get_element_by_id( "ff-btn-fast" ).unwrap().dyn_into::< HtmlButtonElement >().unwrap();
    let closure = Closure::< dyn FnMut() >::new
    (
      move ||
      {
        let mut t = tuning.borrow_mut();
        t.animate_ships = true;
        t.speed_multiplier = 2.5;
        drop( t );
        set_time_control_active( &document, "ff-btn-fast" );
        // Fix(BUG-454): keep "Animate Ships Motion" in sync with Fast.
        animate_toggle_repaint( &document, true );
      }
    );
    button.add_event_listener_with_callback( "click", closure.as_ref().unchecked_ref() ).unwrap();
    closure.forget();
  }
}

/// `reset` is called on click - `main.rs` owns the camera, so it supplies
/// the actual reset logic (set `eye`/`center` back to the initial values)
/// rather than this module reaching into camera state it doesn't own.
pub fn bind_reset_camera< F >( document : &Document, reset : F )
where F : Fn() + 'static
{
  let button = document.get_element_by_id( "ff-btn-reset-cam" ).unwrap().dyn_into::< HtmlButtonElement >().unwrap();
  let closure = Closure::< dyn FnMut() >::new( reset );
  button.add_event_listener_with_callback( "click", closure.as_ref().unchecked_ref() ).unwrap();
  closure.forget();
}

/// Shows/updates the unit-info card, or hides it when `info` is `None`
/// (nothing selected) - ported from `unitPanel.js`'s `selectUnit`, minus
/// its `ui-panel-glow` flash animation (cosmetic, not load-bearing for
/// reading the card).
pub fn refresh_unit_panel( document : &Document, info : Option< &UnitInfo > )
{
  let Some( panel ) = document.get_element_by_id( "ff-unit-panel" ) else { return };
  let Some( info ) = info else
  {
    panel.set_class_name( "ff-panel ff-unit-panel" );
    return;
  };
  panel.set_class_name( "ff-panel ff-unit-panel visible" );

  if let Some( el ) = document.get_element_by_id( "ff-unit-name" ) { el.set_text_content( Some( &info.name ) ); }
  if let Some( el ) = document.get_element_by_id( "ff-unit-commander" ) { el.set_text_content( Some( &info.commander ) ); }
  if let Some( el ) = document.get_element_by_id( "ff-unit-class" ) { el.set_text_content( Some( &format!( "{} CLASS", info.class_label ) ) ); }
}

// Every other function in this module takes a `&Document` and is DOM-bound,
// untestable natively (same Wasm Native-Check Blind Spot already
// established in this workspace). `time_control_button_classes` is the one
// piece of context-free decision logic pulled out for that reason.
#[ cfg( test ) ]
mod tests
{
  use super::time_control_button_classes;

  #[ test ]
  fn paused_highlights_only_pause_regardless_of_speed()
  {
    assert_eq!( time_control_button_classes( false, 1.0 ), ( "active", "", "" ) );
    assert_eq!( time_control_button_classes( false, 2.5 ), ( "active", "", "" ), "animate_ships=false always means paused, regardless of speed_multiplier" );
  }

  #[ test ]
  fn normal_speed_highlights_only_play()
  {
    assert_eq!( time_control_button_classes( true, 1.0 ), ( "", "active", "" ) );
  }

  #[ test ]
  fn fast_speed_highlights_only_fast()
  {
    assert_eq!( time_control_button_classes( true, 2.5 ), ( "", "", "active" ) );
    assert_eq!( time_control_button_classes( true, 1.0001 ), ( "", "", "active" ), "the Play/Fast split is a strict > 1.0 threshold" );
  }
}
