//! Live-GL-context tests for what the pure tests in `main.rs` can't reach,
//! because it takes real `Asteroids` / `Ships` / `Station`, whose
//! constructors upload meshes: the layer filtering of `visible_parts` and
//! `pick_at_client`, and the gizmo drag and selection handlers
//! (`gizmo_grab`, `gizmo_drag_move`, `pointer_release`, `key_press`,
//! `selection_drop_if_hidden`).
//! They run in a browser (wasm32 only, like `gpu_picking`'s
//! `live_gl_tests`). Run with `wasm-pack test --headless --chrome`.

use crate::*;
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

/// Canvas side in CSS and backing-store pixels alike, pinned so the pick
/// buffer and the client-to-pixel mapping don't depend on the test page.
const CANVAS_SIZE : u32 = 256;

fn canvas_and_gl() -> ( gl::web_sys::HtmlCanvasElement, GL )
{
  gl::browser::setup( gl::browser::Config::default() );
  let canvas = gl::canvas::make().unwrap();
  let style = canvas.style();
  style.set_property( "position", "fixed" ).unwrap();
  style.set_property( "left", "0" ).unwrap();
  style.set_property( "top", "0" ).unwrap();
  style.set_property( "width", &format!( "{CANVAS_SIZE}px" ) ).unwrap();
  style.set_property( "height", &format!( "{CANVAS_SIZE}px" ) ).unwrap();
  canvas.set_width( CANVAS_SIZE );
  canvas.set_height( CANVAS_SIZE );
  let gl = gl::context::from_canvas( &canvas ).unwrap();
  gl.enable( GL::DEPTH_TEST );
  ( canvas, gl )
}

fn all_shown() -> RenderLayers
{
  RenderLayers { show_asteroids : true, show_ships : true, show_station : true, show_gizmo : true, ..RenderLayers::default() }
}

fn pick_ids< 'a >( parts : impl Iterator< Item = &'a HullPart > ) -> Vec< i32 >
{
  parts.map( | part | part.pick_id ).collect()
}

/// An `InteractionCtx` on a fresh canvas whose camera looks straight at
/// asteroid 0's centre from close by, so the canvas centre lands on the rock
/// whatever its random jitter. Every layer is shown and nothing is selected.
fn ctx_facing_asteroid_0() -> InteractionCtx
{
  let ( canvas, gl ) = canvas_and_gl();
  // Asteroid positions are fixed, so a throwaway belt gives the one the
  // context builds.
  let center4 = Asteroids::new( &gl, ASTEROID_ID_BASE ).object_transform( 0 ) * gl::math::F32x4::new( 0.0, 0.0, 0.0, 1.0 );
  let center = gl::F32x3::new( center4.x(), center4.y(), center4.z() );
  let eye = center + gl::F32x3::new( 0.0, 30.0, 30.0 );
  let camera = Camera::new( eye, gl::F32x3::new( 0.0, 1.0, 0.0 ), center, 1.0, 55.0f32.to_radians(), 0.1, 2000.0 ).unwrap();

  let tuning = Rc::new( RefCell::new( GridTuning::default() ) );
  tuning.borrow_mut().layers = all_shown();
  InteractionCtx::new( &gl, &canvas, &camera, tuning )
}

/// The canvas centre, in client coordinates.
fn canvas_center( ctx : &InteractionCtx ) -> ( f64, f64 )
{
  let rect = ctx.canvas.get_bounding_client_rect();
  ( rect.left() + rect.width() / 2.0, rect.top() + rect.height() / 2.0 )
}

/// A client point on the gizmo handle, found with the real pick pass: the
/// first pixel right of the canvas centre that picks `GIZMO_ID`. The
/// translate cross covers the centre itself; the rotate ring is hollow, so
/// it is found further out.
fn gizmo_handle_point( ctx : &InteractionCtx ) -> ( f64, f64 )
{
  let ( x, y ) = canvas_center( ctx );
  ( 0 .. CANVAS_SIZE / 2 ).step_by( 2 ).map( | dx | ( x + f64::from( dx ), y ) )
  .find( | &( x, y ) | pick_at_client( ctx, x, y ) == Some( GIZMO_ID ) )
  .expect( "the gizmo handle must be on screen, right of the selected asteroid's centre" )
}

/// Asteroid 0's XZ position and Y rotation, as bits, so "put back" means
/// exactly the grab-time values.
fn asteroid_0_pose( ctx : &InteractionCtx ) -> ( [ u32; 2 ], u32 )
{
  let asteroids = ctx.asteroids.borrow();
  ( asteroids.position( 0 ).map( f32::to_bits ), asteroids.rotation_y( 0 ).to_bits() )
}

/// The HUD element with `id`, created on the test page if no test made it
/// yet: `pointer_release` writes the selection into the debug panel's status
/// line and the unit card, which only the app's own setup builds.
fn hud_element( document : &gl::web_sys::Document, id : &str ) -> gl::web_sys::Element
{
  document.get_element_by_id( id ).unwrap_or_else( ||
  {
    let el = document.create_element( "div" ).unwrap();
    el.set_id( id );
    document.body().unwrap().append_child( &el ).unwrap();
    el
  } )
}

/// What the HUD shows for the selection: the status line's text, whether
/// the unit card is visible, and the card's name.
fn hud_selection( ctx : &InteractionCtx ) -> ( String, bool, String )
{
  let status = hud_element( &ctx.document, "grid-focus-status" ).text_content().unwrap_or_default();
  let visible = hud_element( &ctx.document, "ff-unit-panel" ).class_list().contains( "visible" );
  let name = hud_element( &ctx.document, "ff-unit-name" ).text_content().unwrap_or_default();
  ( status, visible, name )
}

/// `visible_parts` hands each group's `parts()` to the slot of its own kind:
/// with any one group hidden, exactly the other two groups' parts come back,
/// in pick-id order. Passing ships where the station belongs (or the reverse)
/// would still compile, and `visible_groups`' pure test can't see it.
#[ wasm_bindgen_test ]
fn visible_parts_returns_exactly_the_shown_groups()
{
  let ( _canvas, gl ) = canvas_and_gl();
  let asteroids = Asteroids::new( &gl, ASTEROID_ID_BASE );
  let ships = Ships::new( &gl, SHIP_ID_BASE );
  let station = Station::new( &gl, STATION_ID );

  let asteroid_ids = pick_ids( asteroids.parts().iter() );
  let ship_ids = pick_ids( ships.parts().iter() );
  let station_ids = pick_ids( station.parts().iter() );
  assert!( asteroid_ids.iter().all( | &id | matches!( classify_pick( id ), Some( PickedKind::Asteroid( _ ) ) ) ) );
  assert!( ship_ids.iter().all( | &id | matches!( classify_pick( id ), Some( PickedKind::Ship( _ ) ) ) ) );
  assert!( station_ids.iter().all( | &id | matches!( classify_pick( id ), Some( PickedKind::Station ) ) ) );

  let cases =
  [
    ( "all shown", all_shown(), [ true, true, true ] ),
    ( "asteroids hidden", RenderLayers { show_asteroids : false, ..all_shown() }, [ false, true, true ] ),
    ( "ships hidden", RenderLayers { show_ships : false, ..all_shown() }, [ true, false, true ] ),
    ( "station hidden", RenderLayers { show_station : false, ..all_shown() }, [ true, true, false ] ),
  ];
  for ( name, layers, shown ) in cases
  {
    let expected : Vec< i32 > = [ &asteroid_ids, &ship_ids, &station_ids ]
    .into_iter()
    .zip( shown )
    .filter( | ( _, on ) | *on )
    .flat_map( | ( ids, _ ) | ids.iter().copied() )
    .collect();
    assert_eq!( pick_ids( visible_parts( &layers, &asteroids, &ships, &station ) ), expected, "{name}" );
  }
}

/// The pick pass leaves a hidden layer out of the id buffer: a click aimed
/// straight at an asteroid picks it while Asteroids is shown and can't pick
/// it once Asteroids is hidden. The first assertion proves the click is
/// aimed at the asteroid, so the second can't pass by missing it.
#[ wasm_bindgen_test ]
fn pick_at_client_skips_a_hidden_layer()
{
  let ctx = ctx_facing_asteroid_0();
  let ( x, y ) = canvas_center( &ctx );

  assert_eq!( pick_at_client( &ctx, x, y ), Some( ASTEROID_ID_BASE ), "the click must hit asteroid 0 while Asteroids is shown" );

  ctx.tuning.borrow_mut().layers.show_asteroids = false;
  let picked = pick_at_client( &ctx, x, y );
  assert!
  (
    !matches!( picked.and_then( classify_pick ), Some( PickedKind::Asteroid( _ ) ) ),
    "a hidden asteroid must not be picked, got {picked:?}",
  );
}

/// Escape mid-drag puts the dragged object back exactly where it was grabbed
/// and ends the drag, in both gizmo modes: the handle is grabbed through the
/// real pick pass, the cursor moves (which must move or turn the asteroid,
/// or the restore proves nothing), G / R are ignored while dragging, and
/// Escape restores the pose, gives camera orbit rotation back, forgets the
/// press so its release isn't a click, and keeps the selection.
#[ wasm_bindgen_test ]
fn escape_mid_drag_puts_the_object_back()
{
  for mode in [ GizmoMode::Translate, GizmoMode::Rotate ]
  {
    let ctx = ctx_facing_asteroid_0();
    ctx.selected_id.set( Some( ASTEROID_ID_BASE ) );
    ctx.gizmo_mode.set( mode );
    let grab_pose = asteroid_0_pose( &ctx );

    let ( x, y ) = gizmo_handle_point( &ctx );
    let press = Cell::new( Some( ( x, y ) ) );
    gizmo_grab( &ctx, x, y );
    assert!( ctx.drag_state.get().is_some(), "{mode:?}: a press on the handle starts a drag" );
    assert!( !ctx.camera_controls.borrow().rotation.enabled, "{mode:?}: the drag owns the pointer, not the camera" );

    gizmo_drag_move( &ctx, x + 40.0, y + 30.0 );
    assert_ne!( asteroid_0_pose( &ctx ), grab_pose, "{mode:?}: the drag moved the asteroid" );

    let other = match mode { GizmoMode::Translate => "r", GizmoMode::Rotate => "g" };
    key_press( &ctx, other, &press );
    assert_eq!( ctx.gizmo_mode.get(), mode, "{mode:?}: G / R wait for the drag to end" );

    key_press( &ctx, "Escape", &press );
    assert_eq!( asteroid_0_pose( &ctx ), grab_pose, "{mode:?}: Escape restores the grab-time pose" );
    assert!( ctx.drag_state.get().is_none(), "{mode:?}: Escape ends the drag" );
    assert!( ctx.camera_controls.borrow().rotation.enabled, "{mode:?}: camera orbit rotation is back" );
    assert_eq!( press.get(), None, "{mode:?}: the press must not become a click on release" );
    assert_eq!( ctx.selected_id.get(), Some( ASTEROID_ID_BASE ), "{mode:?}: Escape mid-drag keeps the selection" );
  }
}

/// Hiding the selected object's layer drops the selection, and with it a
/// drag on that object; hiding another layer leaves both alone. Without a
/// drag, Escape deselects.
#[ wasm_bindgen_test ]
fn hiding_the_selected_layer_drops_the_selection()
{
  let ctx = ctx_facing_asteroid_0();
  ctx.selected_id.set( Some( ASTEROID_ID_BASE ) );
  let ( x, y ) = gizmo_handle_point( &ctx );
  gizmo_grab( &ctx, x, y );
  assert!( ctx.drag_state.get().is_some(), "a press on the handle starts a drag" );

  let ships_hidden = RenderLayers { show_ships : false, ..all_shown() };
  selection_drop_if_hidden( &ctx, &ships_hidden );
  assert_eq!( ctx.selected_id.get(), Some( ASTEROID_ID_BASE ), "hiding ships leaves an asteroid selected" );
  assert!( ctx.drag_state.get().is_some(), "and its drag going" );

  let asteroids_hidden = RenderLayers { show_asteroids : false, ..all_shown() };
  selection_drop_if_hidden( &ctx, &asteroids_hidden );
  assert_eq!( ctx.selected_id.get(), None, "hiding asteroids drops the asteroid's selection" );
  assert!( ctx.drag_state.get().is_none(), "and ends its drag" );
  assert!( ctx.camera_controls.borrow().rotation.enabled, "giving camera orbit rotation back" );

  ctx.selected_id.set( Some( ASTEROID_ID_BASE ) );
  key_press( &ctx, "Escape", &Cell::new( None ) );
  assert_eq!( ctx.selected_id.get(), None, "Escape without a drag deselects" );
}

/// A left-button click selects what it hits and the HUD follows: a click on
/// asteroid 0 selects it and shows its card, a click on nothing deselects and
/// hides the card. What isn't a click leaves the selection alone: another
/// button, a press that moved more than 6 px (a camera orbit), a release with
/// no press, and the release that ends a gizmo drag, which also gives camera
/// orbit rotation back and must not select the gizmo's own id.
#[ wasm_bindgen_test ]
fn a_click_selects_what_it_hits()
{
  let ctx = ctx_facing_asteroid_0();
  for id in [ "grid-focus-status", "ff-unit-panel", "ff-unit-name" ]
  {
    hud_element( &ctx.document, id );
  }
  let ( x, y ) = canvas_center( &ctx );
  assert_eq!( pick_at_client( &ctx, x, y ), Some( ASTEROID_ID_BASE ), "the click must be aimed at asteroid 0" );

  pointer_release( &ctx, 2, Some( ( x, y ) ), x, y );
  assert_eq!( ctx.selected_id.get(), None, "a right-button click selects nothing" );
  pointer_release( &ctx, 0, Some( ( x - 7.0, y ) ), x, y );
  assert_eq!( ctx.selected_id.get(), None, "a press that moved 7 px is an orbit, not a click" );
  pointer_release( &ctx, 0, None, x, y );
  assert_eq!( ctx.selected_id.get(), None, "a release without a press selects nothing" );

  pointer_release( &ctx, 0, Some( ( x - 5.0, y ) ), x, y );
  assert_eq!( ctx.selected_id.get(), Some( ASTEROID_ID_BASE ), "a left click within 6 px selects the asteroid" );
  assert_eq!
  (
    hud_selection( &ctx ),
    ( "selected: asteroid 0".to_string(), true, "ASTEROID 0".to_string() ),
    "the HUD shows the selected asteroid",
  );

  let ( hx, hy ) = gizmo_handle_point( &ctx );
  gizmo_grab( &ctx, hx, hy );
  assert!( ctx.drag_state.get().is_some(), "a press on the handle starts a drag" );
  pointer_release( &ctx, 0, Some( ( hx, hy ) ), hx, hy );
  assert!( ctx.drag_state.get().is_none(), "the release ends the drag" );
  assert!( ctx.camera_controls.borrow().rotation.enabled, "giving camera orbit rotation back" );
  assert_eq!( ctx.selected_id.get(), Some( ASTEROID_ID_BASE ), "and keeps the selection, not the gizmo" );

  ctx.tuning.borrow_mut().layers = RenderLayers { show_asteroids : false, show_ships : false, show_station : false, show_gizmo : false, ..all_shown() };
  assert_eq!( pick_at_client( &ctx, x, y ), None, "with every pickable layer hidden the click hits nothing" );
  pointer_release( &ctx, 0, Some( ( x, y ) ), x, y );
  assert_eq!( ctx.selected_id.get(), None, "a click on nothing deselects" );
  let ( status, visible, _ ) = hud_selection( &ctx );
  assert_eq!( ( status.as_str(), visible ), ( "selected: none (click a ship, station, or asteroid)", false ), "and hides the card" );
}
