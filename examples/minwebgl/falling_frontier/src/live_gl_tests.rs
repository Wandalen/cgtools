//! Live-GL-context tests for the layer filtering that the pure tests in
//! `main.rs` can't reach: `visible_parts` and `pick_at_client` take real
//! `Asteroids` / `Ships` / `Station`, whose constructors upload meshes, so
//! these run in a browser (wasm32 only, like `gpu_picking`'s
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
  let ( canvas, gl ) = canvas_and_gl();
  let asteroids = Asteroids::new( &gl, ASTEROID_ID_BASE );

  // Look straight at asteroid 0's centre from close by, so the canvas
  // centre lands on the rock whatever its random jitter.
  let center4 = asteroids.object_transform( 0 ) * gl::math::F32x4::new( 0.0, 0.0, 0.0, 1.0 );
  let center = gl::F32x3::new( center4.x(), center4.y(), center4.z() );
  let eye = center + gl::F32x3::new( 0.0, 30.0, 30.0 );
  let camera = Camera::new( eye, gl::F32x3::new( 0.0, 1.0, 0.0 ), center, 1.0, 55.0f32.to_radians(), 0.1, 2000.0 ).unwrap();

  let tuning = Rc::new( RefCell::new( GridTuning::default() ) );
  tuning.borrow_mut().layers = all_shown();
  let ctx = InteractionCtx
  {
    gl : gl.clone(),
    canvas : canvas.clone(),
    document : gl::web_sys::window().unwrap().document().unwrap(),
    id_program : IdProgram::new( &gl ),
    pick_buffer : RefCell::new( PickBuffer::new( &gl, CANVAS_SIZE as i32, CANVAS_SIZE as i32 ) ),
    gizmo : Gizmo::new( &gl ),
    latest_view_proj : Cell::new( camera.projection_matrix_get() * camera.view_matrix_get() ),
    selected_id : Cell::new( None ),
    gizmo_mode : Cell::new( GizmoMode::Translate ),
    drag_state : Cell::new( None ),
    camera_controls : camera.controls_get(),
    asteroids : RefCell::new( asteroids ),
    ships : RefCell::new( Ships::new( &gl, SHIP_ID_BASE ) ),
    station : RefCell::new( Station::new( &gl, STATION_ID ) ),
    tuning : tuning.clone(),
  };

  let rect = canvas.get_bounding_client_rect();
  let ( x, y ) = ( rect.left() + rect.width() / 2.0, rect.top() + rect.height() / 2.0 );

  assert_eq!( pick_at_client( &ctx, x, y ), Some( ASTEROID_ID_BASE ), "the click must hit asteroid 0 while Asteroids is shown" );

  tuning.borrow_mut().layers.show_asteroids = false;
  let picked = pick_at_client( &ctx, x, y );
  assert!
  (
    !matches!( picked.and_then( classify_pick ), Some( PickedKind::Asteroid( _ ) ) ),
    "a hidden asteroid must not be picked, got {picked:?}",
  );
}
