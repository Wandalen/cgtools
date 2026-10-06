//! Falling Frontier — tactical space-scene demo: a fleet of selectable ships
//! and a station orbit an asteroid field, tracked by a shader-driven
//! tactical grid whose selection-driven view-zone ribbon wraps around
//! blocking asteroids as a faceted boundary polyline. Object picking uses
//! an off-screen id buffer (`gpu_picking`), selected units get a
//! movable/rotatable transform gizmo, fleets move along Catmull-Rom
//! patrol paths, a HUD surfaces unit info and playback controls, and a
//! Render Layers dev panel shows or hides each scene layer. See
//! `PORT_PLAN.md` in this crate for the milestone history and porting notes.

mod debug;
mod hud;
mod boundary;
mod hull;
mod gizmo;
mod trajectories;
mod asteroids;
mod ships;
mod station;
mod starfield;
mod background;
mod grid;

use minwebgl as gl;
use gl::GL;
use renderer::webgl::Camera;
use renderer::webgl::shadow::{ ShadowMap, Light };
use std::{ cell::{ Cell, RefCell }, rc::Rc };
use debug::{ GridTuning, RenderLayers, setup_grid_tuning_panel, setup_layers_panel, refresh_selection_status };
use hud::{ setup_hud, refresh_unit_panel, bind_reset_camera };
use boundary::{ build_boundary_polyline, MAX_BOUNDARY_PTS };
use hull::{ HullPart, HullProgram };
use gpu_picking::{ IdProgram, PickBuffer };
use gizmo::{ Gizmo, GizmoMode };
use asteroids::Asteroids;
use ships::Ships;
use station::Station;
use trajectories::Trajectories;
use starfield::Starfield;
use background::Background;
use grid::{ FocusState, TacticalGrid, MAX_ASTEROID_GLOW };

// Shadow map sizing - deliberately covers just the ships/station/asteroid
// cluster (see asteroids.rs's ASTEROID_SPECS, all within roughly ±175 on
// X/Z), not the whole `grid::PLANE_SIZE` plane, so the ortho frustum stays tight
// enough for reasonable shadow resolution at SHADOW_MAP_RESOLUTION.
const SHADOW_MAP_RESOLUTION : u32 = 1024;
const SHADOW_HALF_EXTENT : f32 = 260.0;
const SHADOW_LIGHT_DISTANCE : f32 = 400.0;

// Pick-id ranges handed out to each pickable group (see `picking.rs`) -
// asteroids first, then ships, then the station gets the one id left over.
// `asteroids::ASTEROID_COUNT`/`ships::SHIP_COUNT` come from those modules'
// own spec arrays, so these stay in sync automatically if a roster grows.
const ASTEROID_ID_BASE : i32 = 0;
const SHIP_ID_BASE : i32 = ASTEROID_ID_BASE + asteroids::ASTEROID_COUNT as i32;
const STATION_ID : i32 = SHIP_ID_BASE + ships::SHIP_COUNT as i32;
// The M6 gizmo handle's own pick id - one past every real scene object, so
// it never collides with `classify_pick`'s ranges above.
const GIZMO_ID : i32 = STATION_ID + 1;

/// What a raw pick id (see `picking.rs`) refers to - the mapping only
/// `main.rs` knows, since it's the one that handed out the id ranges above.
#[ derive( Clone, Copy ) ]
enum PickedKind
{
  Asteroid( usize ),
  Ship( usize ),
  Station,
}

fn classify_pick( id : i32 ) -> Option< PickedKind >
{
  if id < ASTEROID_ID_BASE { return None; }
  let offset = id - ASTEROID_ID_BASE;
  if ( offset as usize ) < asteroids::ASTEROID_COUNT { return Some( PickedKind::Asteroid( offset as usize ) ); }
  let offset = offset - asteroids::ASTEROID_COUNT as i32;
  if ( offset as usize ) < ships::SHIP_COUNT { return Some( PickedKind::Ship( offset as usize ) ); }
  if id == STATION_ID { return Some( PickedKind::Station ); }
  None
}

fn selection_status_text( kind : Option< PickedKind > ) -> String
{
  match kind
  {
    Some( PickedKind::Ship( i ) ) => format!( "selected: ship {i}" ),
    Some( PickedKind::Station ) => "selected: station".to_string(),
    Some( PickedKind::Asteroid( i ) ) => format!( "selected: asteroid {i}" ),
    None => "selected: none (click a ship, station, or asteroid)".to_string(),
  }
}

/// M8: what the HUD's unit-info card shows for `kind`, or `None` to hide the
/// card entirely (nothing selected). Asteroids have no `fleet.js`/
/// `spaceStation.js`-style name/commander in the JS reference either (JS's
/// own `selectUnit` is only ever called for objects with a `userData.name`,
/// which asteroids never get) - shown here as a generic "ASTEROID n"
/// contact instead of hiding the card, since this port's own selection
/// (M5/M6) already treats all three kinds uniformly and hiding the card
/// only for asteroids would be a pointless inconsistency.
fn unit_info_for( kind : PickedKind, ships : &Ships, station : &Station ) -> hud::UnitInfo
{
  match kind
  {
    PickedKind::Ship( i ) => hud::UnitInfo
    {
      name : ships.name( i ).to_string(),
      commander : ships.commander( i ).to_string(),
      class_label : ships.class_label( i ).to_string(),
    },
    PickedKind::Station => hud::UnitInfo
    {
      name : station.name().to_string(),
      commander : station.commander().to_string(),
      class_label : "STATION".to_string(),
    },
    PickedKind::Asteroid( i ) => hud::UnitInfo
    {
      name : format!( "ASTEROID {i}" ),
      commander : "UNASSIGNED".to_string(),
      class_label : "ASTEROID".to_string(),
    },
  }
}

/// Whether `kind`'s Render Layers switch is on - the only place an object
/// kind is mapped to its switch. Everything that decides whether an object
/// is present goes through it: the visible draw, the shadow pass and the
/// pick pass (via `visible_groups`), the gizmo (`gizmo_visible`), the
/// view-zone ribbon (`ribbon_ship`), and the frame loop's check that drops
/// a selection whose layer was hidden. So a hidden object can't be clicked,
/// cast a shadow, cover a visible one, notch the ribbon or keep its
/// selection, unit card and ribbon.
fn kind_visible( kind : PickedKind, t : &RenderLayers ) -> bool
{
  match kind
  {
    PickedKind::Asteroid( _ ) => t.show_asteroids,
    PickedKind::Ship( _ ) => t.show_ships,
    PickedKind::Station => t.show_station,
  }
}

/// The parts of every object group whose layer is visible, in pick-id order
/// (asteroids, ships, station). Generic over the part type so the filtering
/// can be tested without a GL context; `visible_parts` is the GL-side use.
fn visible_groups< 'a, T >( t : &RenderLayers, asteroids : &'a [ T ], ships : &'a [ T ], station : &'a [ T ] ) -> impl Iterator< Item = &'a T >
{
  // Visibility is per kind, so any index stands for the whole group.
  [ ( PickedKind::Asteroid( 0 ), asteroids ), ( PickedKind::Ship( 0 ), ships ), ( PickedKind::Station, station ) ]
  .into_iter()
  .filter( move | ( kind, _ ) | kind_visible( *kind, t ) )
  .flat_map( | ( _, parts ) | parts.iter() )
}

/// Every hull part whose Render Layers switch is on - the one list the
/// visible draw, the shadow pass and the pick pass all walk, so none of them
/// can drift from the others.
fn visible_parts< 'a >( t : &'a RenderLayers, asteroids : &'a Asteroids, ships : &'a Ships, station : &'a Station ) -> impl Iterator< Item = &'a HullPart >
{
  visible_groups( t, asteroids.parts(), ships.parts(), station.parts() )
}

/// The ship whose view-zone ribbon the grid draws, if any: a selected ship
/// (the only kind with a view radius) whose own layer is visible, with the
/// grid and its ribbon row on.
fn ribbon_ship( selected : Option< PickedKind >, t : &RenderLayers ) -> Option< usize >
{
  match selected
  {
    Some( kind @ PickedKind::Ship( i ) ) if t.show_grid && t.show_view_ribbon && kind_visible( kind, t ) => Some( i ),
    _ => None,
  }
}

/// Whether the gizmo handle exists for a selected `kind`: its own switch is
/// on and the object it's attached to is visible. The frame loop draws the
/// handle and the pick pass renders it under this one condition, so a
/// handle is grabbable exactly when it can be seen.
fn gizmo_visible( kind : PickedKind, t : &RenderLayers ) -> bool
{
  t.show_gizmo && kind_visible( kind, t )
}

/// The selected object's current world transform - what the M6 gizmo draws
/// its handle at.
fn selected_transform( kind : PickedKind, asteroids : &Asteroids, ships : &Ships, station : &Station ) -> gl::F32x4x4
{
  match kind
  {
    PickedKind::Asteroid( i ) => asteroids.object_transform( i ),
    PickedKind::Ship( i ) => ships.object_transform( i ),
    PickedKind::Station => station.object_transform(),
  }
}

/// The selected object's current XZ position and Y rotation - what a gizmo
/// drag needs at grab time to compute a grab offset (translate) or angle
/// offset (rotate), so the object doesn't snap to the cursor the instant
/// it's grabbed.
fn selected_position_rotation( kind : PickedKind, asteroids : &Asteroids, ships : &Ships, station : &Station ) -> ( [ f32; 2 ], f32 )
{
  match kind
  {
    PickedKind::Asteroid( i ) => ( asteroids.position( i ), asteroids.rotation_y( i ) ),
    PickedKind::Ship( i ) => ( ships.position( i ), ships.rotation_y( i ) ),
    PickedKind::Station => ( station.position(), station.rotation_y() ),
  }
}

fn drag_selected( kind : PickedKind, position : [ f32; 2 ], asteroids : &mut Asteroids, ships : &mut Ships, station : &mut Station )
{
  match kind
  {
    PickedKind::Asteroid( i ) => asteroids.drag_to( i, position ),
    PickedKind::Ship( i ) => ships.drag_to( i, position ),
    PickedKind::Station => station.drag_to( position ),
  }
}

fn rotate_selected( kind : PickedKind, rotation_y : f32, asteroids : &mut Asteroids, ships : &mut Ships, station : &mut Station )
{
  match kind
  {
    PickedKind::Asteroid( i ) => asteroids.rotate_to( i, rotation_y ),
    PickedKind::Ship( i ) => ships.rotate_to( i, rotation_y ),
    PickedKind::Station => station.rotate_to( rotation_y ),
  }
}

/// Converts the dev panel's azimuth/elevation (degrees) into a normalized
/// world-space direction pointing *toward* the light - the convention
/// `hull.frag`'s `n_dot_l = dot(normal, light_dir)` and `ShadowMap`'s light
/// placement (see `app_run`) both expect.
fn light_direction( azimuth_deg : f32, elevation_deg : f32 ) -> gl::F32x3
{
  let az = azimuth_deg.to_radians();
  let el = elevation_deg.to_radians();
  gl::F32x3::new( el.cos() * az.cos(), el.sin(), el.cos() * az.sin() )
}

fn canvas_size( canvas : &gl::web_sys::HtmlCanvasElement ) -> ( u32, u32 )
{
  let window = gl::web_sys::window().unwrap();
  let dpr = window.device_pixel_ratio();
  let css_w = f64::from( canvas.client_width() );
  let css_h = f64::from( canvas.client_height() );
  let w = ( css_w * dpr ) as u32;
  let h = ( css_h * dpr ) as u32;
  ( w.max( 1 ), h.max( 1 ) )
}

// Fix(BUG-053) in object_picking: `client_x`/`client_y` return `i32` or
// `f64` depending on whether `web_sys_unstable_apis` is active. `.into()`
// resolves in both cases; `f64` identity triggers `useless_conversion` under
// the unstable-apis cfg, so it's allowed here rather than at every call site.
#[ allow( clippy::useless_conversion, reason = "cfg-dependent per Fix(BUG-053) — identity only under the web_sys_unstable_apis f64 signature" ) ]
fn pointer_client_pos( e : &gl::web_sys::PointerEvent ) -> ( f64, f64 )
{
  let x : f64 = e.client_x().into();
  let y : f64 = e.client_y().into();
  ( x, y )
}

/// Converts a pointer event's window-relative client coordinates into
/// canvas-local, bottom-up device-pixel coordinates - the convention
/// `PickBuffer::pick`'s `read_pixels` call uses. Ratio-based (not a raw
/// offset) so it's correct regardless of `devicePixelRatio` scaling between
/// the canvas's CSS size (`rect.width()/height()`) and its backing-store
/// size (`canvas.width()/height()`, what `PickBuffer` is sized to).
fn canvas_pixel_from_client( canvas : &gl::web_sys::HtmlCanvasElement, client_x : f64, client_y : f64 ) -> ( i32, i32 )
{
  let rect = canvas.get_bounding_client_rect();
  let x_ratio = ( client_x - rect.left() ) / rect.width();
  let y_ratio = ( client_y - rect.top() ) / rect.height();
  let px = ( x_ratio * f64::from( canvas.width() ) ) as i32;
  let py_top_down = ( y_ratio * f64::from( canvas.height() ) ) as i32;
  ( px, canvas.height() as i32 - py_top_down )
}

fn unproject( inv_view_proj : gl::F32x4x4, ndc_x : f32, ndc_y : f32, ndc_z : f32 ) -> gl::math::F32x3
{
  let clip = gl::math::F32x4::new( ndc_x, ndc_y, ndc_z, 1.0 );
  let world = inv_view_proj * clip;
  gl::math::F32x3::new( world.x() / world.w(), world.y() / world.w(), world.z() / world.w() )
}

/// Casts a ray from the camera through the given client-space pixel and
/// intersects it with the `y = 0` ground plane - re-derived for the M6
/// gizmo's drag math (constrains translate to this plane, and rotate reads
/// an angle off it) after M5 deleted the identical M3-era functions once
/// GPU id-buffer picking made them unnecessary for click-to-select itself.
/// `view_proj` is a snapshot from the most recent frame (see
/// `latest_view_proj`), not recomputed here.
fn ray_ground_hit( view_proj : gl::F32x4x4, canvas : &gl::web_sys::HtmlCanvasElement, client_x : f64, client_y : f64 ) -> Option< [ f32; 2 ] >
{
  let rect = canvas.get_bounding_client_rect();
  let x = ( client_x - rect.left() ) as f32;
  let y = ( client_y - rect.top() ) as f32;
  let w = rect.width() as f32;
  let h = rect.height() as f32;
  if w <= 0.0 || h <= 0.0 { return None; }

  let ndc_x = ( x / w ) * 2.0 - 1.0;
  let ndc_y = 1.0 - ( y / h ) * 2.0;

  let inv = view_proj.inverse()?;
  let near = unproject( inv, ndc_x, ndc_y, -1.0 );
  let far = unproject( inv, ndc_x, ndc_y, 1.0 );
  let dir = far - near;
  if dir.y().abs() < 1e-6 { return None; }
  let t = -near.y() / dir.y();
  if t < 0.0 { return None; }
  let hit = near + dir * t;
  Some( [ hit.x(), hit.z() ] )
}

/// A gizmo handle grabbed at `kind`'s current position/rotation - captured
/// at grab time so a translate/rotate drag can preserve the exact point/
/// angle grabbed instead of snapping the object to the cursor.
#[ derive( Clone, Copy ) ]
struct DragState
{
  kind : PickedKind,
  mode : GizmoMode,
  /// Translate only: `object.xz - hit.xz` at grab time.
  grab_offset : [ f32; 2 ],
  /// Rotate only: `object.rotation_y - angle_to_cursor` at grab time.
  rotation_offset : f32,
  /// The object's XZ position and Y rotation at grab time, which Escape
  /// restores when it cancels the drag.
  start_position : [ f32; 2 ],
  start_rotation : f32,
}

/// Everything a click, a gizmo drag, or a render frame needs to pick/select/
/// drag scene objects - bundled behind one `Rc` (mirroring
/// `examples/minwebgl/object_picking`'s own `RenderCtx`) so each pointer/
/// keyboard listener captures a single clone instead of a dozen separate
/// `Rc`s. Interior mutability (`Cell`/`RefCell`) lives directly on the
/// fields rather than wrapping each field in its own `Rc`, since nothing
/// outside this struct ever needs to share just one field independently.
struct InteractionCtx
{
  gl : GL,
  canvas : gl::web_sys::HtmlCanvasElement,
  document : gl::web_sys::Document,
  id_program : IdProgram,
  pick_buffer : RefCell< PickBuffer >,
  gizmo : Gizmo,
  /// Last frame's view_proj - pointer/keyboard listeners fire outside the
  /// render loop's closure (which owns `Camera`), and the id pass needs to
  /// draw parts at the same transforms the visible frame used.
  latest_view_proj : Cell< gl::F32x4x4 >,
  selected_id : Cell< Option< i32 > >,
  gizmo_mode : Cell< GizmoMode >,
  drag_state : Cell< Option< DragState > >,
  /// Shared with `Camera`'s own internal state (via `Camera::controls_get`).
  /// Toggling `rotation.enabled` here is exactly how a gizmo drag suppresses
  /// camera-orbit rotation while active, same mechanism the JS reference's
  /// `transform.js` uses (`world.controls.enabled = !event.value`).
  camera_controls : Rc< RefCell< mingl::CameraOrbitControls > >,
  asteroids : RefCell< Asteroids >,
  ships : RefCell< Ships >,
  station : RefCell< Station >,
  /// Read by the pick pass so it skips whatever the Render Layers panel hides.
  tuning : Rc< RefCell< GridTuning > >,
}

/// Re-renders the id pass (including the gizmo handle, if something's
/// selected) and reads back the pick id at `client_x`/`client_y` - shared by
/// the pointerdown gizmo-grab check and the pointerup click-to-select path,
/// since both need the exact same render+read+viewport-restore sequence.
/// Only what the frame loop actually draws goes into the id pass: layers
/// hidden in the Render Layers panel, and a gizmo handle that isn't shown,
/// are left out.
fn pick_at_client( ctx : &InteractionCtx, client_x : f64, client_y : f64 ) -> Option< i32 >
{
  let asteroids = ctx.asteroids.borrow();
  let ships = ctx.ships.borrow();
  let station = ctx.station.borrow();
  let t = ctx.tuning.borrow().layers;

  let gizmo_part = ctx.selected_id.get().and_then( classify_pick ).filter( | kind | gizmo_visible( *kind, &t ) ).map( | kind |
  {
    let transform = selected_transform( kind, &asteroids, &ships, &station );
    ctx.gizmo.part( ctx.gizmo_mode.get(), transform, GIZMO_ID )
  } );

  let parts = visible_parts( &t, &asteroids, &ships, &station );
  ctx.pick_buffer.borrow().render( &ctx.gl, &ctx.id_program, ctx.latest_view_proj.get(), parts, gizmo_part.as_ref() );

  let ( px, py ) = canvas_pixel_from_client( &ctx.canvas, client_x, client_y );
  let picked = ctx.pick_buffer.borrow().pick( &ctx.gl, px, py );

  // Restore the viewport `render()` changed to the pick buffer's size - the
  // main render loop only re-sets it on an actual canvas resize, not every
  // frame, so leaving it wrong here would stick.
  let ( w, h ) = canvas_size( &ctx.canvas );
  ctx.gl.viewport( 0, 0, w as i32, h as i32 );

  picked
}

#[ expect( clippy::too_many_lines, reason = "one flat setup-then-render-loop sequence (camera, scene, picking, panel, click handler, then the per-frame closure); splitting it up would scatter closely-related setup across helper functions for no real grouping" ) ]
fn app_run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );

  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas( &canvas )?;
  gl.enable( GL::DEPTH_TEST );
  // Matches background.frag's own "deep" color - only ever visible as a
  // one-frame flash before the background shader itself draws, since that
  // shader fills every pixel every frame.
  gl.clear_color( 0.08, 0.20, 0.30, 1.0 );

  let ( pixel_w, pixel_h ) = canvas_size( &canvas );
  canvas.set_width( pixel_w );
  canvas.set_height( pixel_h );

  let eye = gl::math::F32x3::from( [ 0.0, 220.0, 260.0 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = gl::math::F32x3::splat( 0.0 );

  let fov = 55.0f32.to_radians();
  let near = 0.1;
  let far = 2000.0;
  let aspect_ratio = pixel_w as f32 / pixel_h as f32;

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far )?;
  camera.window_size_set( [ pixel_w as f32, pixel_h as f32 ].into() );
  camera.controls_bind( &canvas );

  let grid = TacticalGrid::new( &gl );
  let hull_program = HullProgram::new( &gl );
  let starfield = Starfield::new( &gl );
  let background = Background::new( &gl )?;
  let shadow_map = ShadowMap::new( &gl, SHADOW_MAP_RESOLUTION )?;
  gl.viewport( 0, 0, pixel_w as i32, pixel_h as i32 );

  let tuning = Rc::new( RefCell::new( GridTuning::default() ) );
  let document = gl::web_sys::window().unwrap().document().unwrap();

  // Last frame's view_proj, kept for pointer/keyboard listeners below - they
  // fire outside this closure, which owns `camera`.
  let initial_view_proj = camera.projection_matrix_get() * camera.view_matrix_get();

  let ctx = Rc::new( InteractionCtx
  {
    gl : gl.clone(),
    canvas : canvas.clone(),
    document : document.clone(),
    id_program : IdProgram::new( &gl ),
    pick_buffer : RefCell::new( PickBuffer::new( &gl, pixel_w as i32, pixel_h as i32 ) ),
    gizmo : Gizmo::new( &gl ),
    latest_view_proj : Cell::new( initial_view_proj ),
    selected_id : Cell::new( None ),
    gizmo_mode : Cell::new( GizmoMode::Translate ),
    drag_state : Cell::new( None ),
    camera_controls : camera.controls_get(),
    asteroids : RefCell::new( Asteroids::new( &gl, ASTEROID_ID_BASE ) ),
    ships : RefCell::new( Ships::new( &gl, SHIP_ID_BASE ) ),
    station : RefCell::new( Station::new( &gl, STATION_ID ) ),
    tuning : tuning.clone(),
  } );

  {
    let ctx = ctx.clone();
    setup_grid_tuning_panel
    (
      &document, &tuning,
      move || deselect( &ctx )
    );
  }

  setup_layers_panel( &document, &tuning );

  setup_hud( &document, &tuning );
  {
    let ctx = ctx.clone();
    bind_reset_camera
    (
      &document,
      move ||
      {
        let mut controls = ctx.camera_controls.borrow_mut();
        controls.eye = eye;
        controls.up = up;
        controls.center = center;
      }
    );
  }

  setup_selection_and_gizmo( &ctx );

  let prev_size = Cell::new( ( pixel_w, pixel_h ) );
  let mut prev_time = 0.0f64;

  let update_and_draw =
  {
    let canvas = canvas.clone();
    let ctx = ctx.clone();
    // Built on first use rather than at startup: nothing in the UI turns
    // trajectories on yet, so a session that never shows them never builds
    // one ribbon mesh per ship.
    let mut trajectories : Option< Trajectories > = None;
    move | t : f64 |
    {
      let delta_time = if prev_time == 0.0 { 0.0 } else { ( t - prev_time ) / 1000.0 };
      prev_time = t;
      camera.update( delta_time );

      let ( w, h ) = canvas_size( &canvas );
      if ( w, h ) != prev_size.get()
      {
        canvas.set_width( w );
        canvas.set_height( h );
        gl.viewport( 0, 0, w as i32, h as i32 );
        ctx.pick_buffer.borrow_mut().resize( &gl, w as i32, h as i32 );

        let proj = gl::math::mat3x3h::perspective_rh_gl( fov, w as f32 / h as f32, near, far );
        if let Err( e ) = camera.projection_matrix_set( proj )
        {
          web_sys::console::warn_1( &format!( "Falling Frontier: skipping invalid projection matrix on resize: {e}" ).into() );
        }
        camera.window_size_set( [ w as f32, h as f32 ].into() );

        prev_size.set( ( w, h ) );
      }

      gl.clear( GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT );

      let view_proj = camera.projection_matrix_get() * camera.view_matrix_get();
      ctx.latest_view_proj.set( view_proj );

      let tuning_snapshot = *tuning.borrow();
      // A Render Layers row (or a solo gesture) hid the selected object's
      // layer: drop the selection, so its unit card, ribbon and gizmo go
      // with it instead of outliving the object on screen.
      if let Some( kind ) = ctx.selected_id.get().and_then( classify_pick ) && !kind_visible( kind, &tuning_snapshot.layers )
      {
        deselect( &ctx );
      }
      let selected = ctx.selected_id.get();
      let selected_kind = selected.and_then( classify_pick );

      if tuning_snapshot.layers.show_background
      {
        background.draw( &gl, view_proj, camera.eye_get() );
      }

      // M7: advance every ship along its patrol path, except whichever one
      // is currently selected - matches `main.js`'s `updateFleetMotion`
      // skipping `excludeMesh` (the gizmo-attached ship) so a drag isn't
      // fought by the path animation.
      if tuning_snapshot.animate_ships
      {
        let mut ships = ctx.ships.borrow_mut();
        for i in 0 .. ships::SHIP_COUNT
        {
          if Some( SHIP_ID_BASE + i as i32 ) == selected { continue; }
          let speed = ships.speed( i ) * tuning_snapshot.speed_multiplier;
          ships.advance( i, speed );
        }
      }

      let asteroids = ctx.asteroids.borrow();
      let ships = ctx.ships.borrow();
      let station = ctx.station.borrow();

      // Only a selected ship drives the ribbon - matches `main.js`'s
      // `animate()`, which points the grid's focus at `gizmo.object` only
      // when it defines a `viewRadius` (only ships do in `fleet.js`; the
      // station and asteroids have none, so selecting them highlights the
      // object but leaves the ribbon off).
      let focus_snapshot = match ribbon_ship( selected_kind, &tuning_snapshot.layers )
      {
        Some( i ) => FocusState { active : true, point : ships.position( i ) },
        None => FocusState::default(),
      };

      let mut boundary_buf = [ [ 0.0f32; 2 ]; MAX_BOUNDARY_PTS ];
      let mut boundary_count = 0;
      let mut glow : Vec< ( [ f32; 2 ], f32 ) > = Vec::new();
      // Hidden asteroids neither notch the ribbon nor glow on the grid.
      if focus_snapshot.active && tuning_snapshot.layers.show_asteroids
      {
        let blockers = asteroids.blockers();
        boundary_count = build_boundary_polyline
        (
          focus_snapshot.point[ 0 ], focus_snapshot.point[ 1 ],
          tuning_snapshot.view_radius, &blockers, &mut boundary_buf
        );
        glow = asteroids.glow_candidates( focus_snapshot.point, tuning_snapshot.view_radius );
        glow.truncate( MAX_ASTEROID_GLOW );
      }

      // Directional light + shadow map for the hull material - the ortho
      // frustum is centered on the ship/station/asteroid cluster (not the
      // whole grid) and placed SHADOW_LIGHT_DISTANCE back along the light
      // direction, matching the `-light_dir` "camera looks back down at the
      // scene" convention `Light::view_projection` expects.
      let light_dir = light_direction( tuning_snapshot.light_azimuth, tuning_snapshot.light_elevation );
      let shadow_scene_center = gl::F32x3::new( 0.0, 12.0, 0.0 );
      let shadow_projection = gl::math::mat3x3h::orthographic_rh_gl
      (
        -SHADOW_HALF_EXTENT, SHADOW_HALF_EXTENT, -SHADOW_HALF_EXTENT, SHADOW_HALF_EXTENT,
        1.0, SHADOW_LIGHT_DISTANCE + SHADOW_HALF_EXTENT,
      );
      let mut light = Light::new( shadow_scene_center + light_dir * SHADOW_LIGHT_DISTANCE, -light_dir, shadow_projection, tuning_snapshot.light_size );
      let light_view_proj = light.view_projection();

      if tuning_snapshot.layers.shadows_drawn()
      {
        shadow_map.bind();
        shadow_map.clear();
        // A hidden object (per the Render Layers toggles below) shouldn't
        // still be casting a shadow onto the rest of the scene, so the same
        // per-type visibility gates apply here, not just to the visible draw
        // further down.
        for part in visible_parts( &tuning_snapshot.layers, &asteroids, &ships, &station )
        {
          shadow_map.mvp_upload( light_view_proj * part.model );
          gl.bind_vertex_array( Some( &part.vao ) );
          gl.draw_elements_with_i32( GL::TRIANGLES, part.index_count, GL::UNSIGNED_INT, 0 );
        }
      }
      gl.bind_framebuffer( GL::FRAMEBUFFER, None );
      gl.viewport( 0, 0, w as i32, h as i32 );
      gl.disable( GL::CULL_FACE );

      hull_program.begin_frame
      (
        &gl, view_proj, camera.eye_get(), light_dir, tuning_snapshot.light_color, tuning_snapshot.light_intensity, tuning_snapshot.light_size,
        light_view_proj, shadow_map.depth_buffer(), &tuning_snapshot.layers,
      );
      for part in visible_parts( &tuning_snapshot.layers, &asteroids, &ships, &station )
      {
        hull_program.draw_part( &gl, part, Some( part.pick_id ) == selected );
      }

      if tuning_snapshot.layers.show_starfield
      {
        starfield.draw( &gl, view_proj );
      }

      if tuning_snapshot.layers.show_trajectories
      {
        if trajectories.is_none()
        {
          match Trajectories::new( &gl, &ships, camera.projection_matrix_get(), [ w as f32, h as f32 ] )
          {
            Ok( built ) => trajectories = Some( built ),
            Err( e ) =>
            {
              // Switch the layer back off so a failing build isn't retried
              // (and warned about) every frame.
              web_sys::console::warn_1( &format!( "Falling Frontier: trajectory ribbons unavailable: {e}" ).into() );
              tuning.borrow_mut().layers.show_trajectories = false;
            }
          }
        }
        if let Some( trajectories ) = &mut trajectories
        {
          trajectories.draw( &gl, camera.view_matrix_get(), camera.projection_matrix_get(), [ w as f32, h as f32 ] );
        }
      }

      if tuning_snapshot.layers.show_grid
      {
        grid.draw
        (
          &gl, view_proj, camera.eye_get(), &tuning_snapshot, &focus_snapshot,
          &boundary_buf[ .. boundary_count ], &glow
        );
      }

      // M6: the gizmo handle, drawn on top of everything at whatever is
      // currently selected (translate cross or rotate ring, per
      // `ctx.gizmo_mode`).
      if let Some( kind ) = selected_kind && gizmo_visible( kind, &tuning_snapshot.layers )
      {
        let object_transform = selected_transform( kind, &asteroids, &ships, &station );
        let gizmo_part = ctx.gizmo.part( ctx.gizmo_mode.get(), object_transform, GIZMO_ID );
        ctx.gizmo.draw( &gl, view_proj, &gizmo_part );
      }

      true
    }
  };

  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

/// Wires the M5 click-to-select handler and the M6 gizmo drag handler on the
/// same `pointerdown`/`pointerup` pair, since a gizmo grab and a normal
/// selection click both start as a `pointerdown` and are only told apart by
/// what's under the cursor. `pointerdown` stays canvas-scoped (a press must
/// start over the canvas); `pointermove`/`pointerup` are window-scoped so a
/// gizmo drag survives the cursor leaving canvas bounds mid-drag, same
/// rationale as `examples/minwebgl/object_picking`'s own mousemove/mouseup
/// binding. `keydown` (G/R/Escape) is window-scoped too, matching the JS
/// reference's own `window.addEventListener('keydown', ...)`.
#[ expect( clippy::too_many_lines, reason = "four closely-related event listeners (pointerdown/pointermove/pointerup/keydown) sharing the same down_pos/ctx setup - splitting them into separate functions would just move the same total line count behind more indirection" ) ]
fn setup_selection_and_gizmo( ctx : &Rc< InteractionCtx > )
{
  use gl::web_sys::wasm_bindgen::{ prelude::Closure, JsCast };

  let window = gl::web_sys::window().unwrap();
  let canvas = ctx.canvas.clone();
  let down_pos : Rc< Cell< Option< ( f64, f64 ) > > > = Rc::new( Cell::new( None ) );

  {
    let ctx = ctx.clone();
    let down_pos = down_pos.clone();
    let closure = Closure::< dyn FnMut( _ ) >::new
    (
      move | e : gl::web_sys::PointerEvent |
      {
        if e.button() != 0 { return; }
        let pos = pointer_client_pos( &e );
        down_pos.set( Some( pos ) );

        // A gizmo handle only exists while something's selected and the
        // handle is shown - nothing to grab otherwise, so skip the
        // speculative pick (an id pass plus a synchronous pixel read).
        let layers = ctx.tuning.borrow().layers;
        let Some( kind ) = ctx.selected_id.get().and_then( classify_pick ).filter( | kind | gizmo_visible( *kind, &layers ) ) else { return };
        if pick_at_client( &ctx, pos.0, pos.1 ) != Some( GIZMO_ID ) { return; }

        let Some( hit ) = ray_ground_hit( ctx.latest_view_proj.get(), &ctx.canvas, pos.0, pos.1 ) else { return };
        let ( position, rotation_y ) = selected_position_rotation( kind, &ctx.asteroids.borrow(), &ctx.ships.borrow(), &ctx.station.borrow() );

        let drag = match ctx.gizmo_mode.get()
        {
          GizmoMode::Translate => DragState
          {
            kind, mode : GizmoMode::Translate,
            grab_offset : [ position[ 0 ] - hit[ 0 ], position[ 1 ] - hit[ 1 ] ],
            rotation_offset : 0.0,
            start_position : position,
            start_rotation : rotation_y,
          },
          GizmoMode::Rotate => DragState
          {
            kind, mode : GizmoMode::Rotate,
            grab_offset : [ 0.0, 0.0 ],
            rotation_offset : rotation_y - ( hit[ 0 ] - position[ 0 ] ).atan2( hit[ 1 ] - position[ 1 ] ),
            start_position : position,
            start_rotation : rotation_y,
          },
        };
        ctx.drag_state.set( Some( drag ) );
        ctx.camera_controls.borrow_mut().rotation.enabled = false;
      }
    );
    canvas.add_event_listener_with_callback( "pointerdown", closure.as_ref().unchecked_ref() ).unwrap();
    closure.forget();
  }

  {
    let ctx = ctx.clone();
    let closure = Closure::< dyn FnMut( _ ) >::new
    (
      move | e : gl::web_sys::PointerEvent |
      {
        let Some( drag ) = ctx.drag_state.get() else { return };
        let ( x, y ) = pointer_client_pos( &e );
        let Some( hit ) = ray_ground_hit( ctx.latest_view_proj.get(), &ctx.canvas, x, y ) else { return };

        let mut asteroids = ctx.asteroids.borrow_mut();
        let mut ships = ctx.ships.borrow_mut();
        let mut station = ctx.station.borrow_mut();

        match drag.mode
        {
          GizmoMode::Translate =>
          {
            let position = [ hit[ 0 ] + drag.grab_offset[ 0 ], hit[ 1 ] + drag.grab_offset[ 1 ] ];
            drag_selected( drag.kind, position, &mut asteroids, &mut ships, &mut station );
          }
          GizmoMode::Rotate =>
          {
            let ( position, _ ) = selected_position_rotation( drag.kind, &asteroids, &ships, &station );
            let angle = ( hit[ 0 ] - position[ 0 ] ).atan2( hit[ 1 ] - position[ 1 ] );
            rotate_selected( drag.kind, angle + drag.rotation_offset, &mut asteroids, &mut ships, &mut station );
          }
        }
      }
    );
    window.add_event_listener_with_callback( "pointermove", closure.as_ref().unchecked_ref() ).unwrap();
    closure.forget();
  }

  {
    let ctx = ctx.clone();
    let down_pos = down_pos.clone();
    let closure = Closure::< dyn FnMut( _ ) >::new
    (
      move | e : gl::web_sys::PointerEvent |
      {
        if ctx.drag_state.take().is_some()
        {
          ctx.camera_controls.borrow_mut().rotation.enabled = true;
          return;
        }

        if e.button() != 0 { return; }
        let Some( ( dx0, dy0 ) ) = down_pos.get() else { return };
        let ( x, y ) = pointer_client_pos( &e );
        let ( dx, dy ) = ( x - dx0, y - dy0 );
        // A real drag (camera orbit), not a click - leave selection alone.
        if dx.hypot( dy ) > 6.0 { return; }

        let picked = pick_at_client( &ctx, x, y );
        ctx.selected_id.set( picked );
        let kind = picked.and_then( classify_pick );
        refresh_selection_status( &ctx.document, &selection_status_text( kind ) );
        let info = kind.map( | k | unit_info_for( k, &ctx.ships.borrow(), &ctx.station.borrow() ) );
        refresh_unit_panel( &ctx.document, info.as_ref() );
      }
    );
    window.add_event_listener_with_callback( "pointerup", closure.as_ref().unchecked_ref() ).unwrap();
    closure.forget();
  }

  {
    let ctx = ctx.clone();
    let down_pos = down_pos.clone();
    let closure = Closure::< dyn FnMut( _ ) >::new
    (
      move | e : gl::web_sys::KeyboardEvent |
      {
        // During a gizmo drag the keys act on the drag, not the selection:
        // Escape cancels it, putting the object back where it was grabbed,
        // and G/R wait for the drag to end, since the drag keeps the mode it
        // was grabbed in. Deselecting or switching the drawn handle mid-drag
        // would leave the selection and the drag disagreeing.
        if let Some( drag ) = ctx.drag_state.get()
        {
          if e.key() == "Escape"
          {
            drag_cancel( &ctx, drag );
            // The press that started the drag must not become a click on
            // release: over the handle it would select the gizmo's own id.
            down_pos.set( None );
          }
          return;
        }
        if ctx.selected_id.get().is_none() { return; }
        match e.key().as_str()
        {
          "g" | "G" => ctx.gizmo_mode.set( GizmoMode::Translate ),
          "r" | "R" => ctx.gizmo_mode.set( GizmoMode::Rotate ),
          "Escape" => deselect( &ctx ),
          _ => {}
        }
      }
    );
    window.add_event_listener_with_callback( "keydown", closure.as_ref().unchecked_ref() ).unwrap();
    closure.forget();
  }
}

/// Clears the selection and the HUD that shows it: the status line and the
/// unit card. Also ends a gizmo drag on the deselected object, where it
/// leaves the object (and gives camera orbit rotation back).
fn deselect( ctx : &InteractionCtx )
{
  if ctx.drag_state.take().is_some()
  {
    ctx.camera_controls.borrow_mut().rotation.enabled = true;
  }
  ctx.selected_id.set( None );
  refresh_selection_status( &ctx.document, &selection_status_text( None ) );
  refresh_unit_panel( &ctx.document, None );
}

/// Ends `drag` without applying it: the object goes back to its grab-time
/// position and rotation, and camera orbit rotation (switched off for the
/// drag) comes back on.
fn drag_cancel( ctx : &InteractionCtx, drag : DragState )
{
  ctx.drag_state.set( None );
  ctx.camera_controls.borrow_mut().rotation.enabled = true;
  let mut asteroids = ctx.asteroids.borrow_mut();
  let mut ships = ctx.ships.borrow_mut();
  let mut station = ctx.station.borrow_mut();
  drag_selected( drag.kind, drag.start_position, &mut asteroids, &mut ships, &mut station );
  rotate_selected( drag.kind, drag.start_rotation, &mut asteroids, &mut ships, &mut station );
}

fn main()
{
  app_run().unwrap();
}

#[ cfg( test ) ]
mod tests
{
  use super::{ gizmo_visible, kind_visible, ribbon_ship, visible_groups, PickedKind, RenderLayers };

  /// Every object layer and the gizmo shown explicitly, so a changed default
  /// can't make a test pass or fail for the wrong reason.
  fn all_shown() -> RenderLayers
  {
    RenderLayers
    {
      show_grid : true,
      show_view_ribbon : true,
      show_asteroids : true,
      show_ships : true,
      show_station : true,
      show_gizmo : true,
      ..RenderLayers::default()
    }
  }

  /// The pick, shadow and visible passes walk exactly the groups whose layer
  /// is on, in pick-id order.
  #[ test ]
  fn visible_groups_keeps_only_shown_groups_in_order()
  {
    let ( asteroids, ships, station ) = ( [ 1, 2 ], [ 3 ], [ 4 ] );
    let walk = | t : &RenderLayers | visible_groups( t, &asteroids, &ships, &station ).copied().collect::< Vec< i32 > >();
    assert_eq!( walk( &all_shown() ), vec![ 1, 2, 3, 4 ] );
    assert_eq!( walk( &RenderLayers { show_asteroids : false, ..all_shown() } ), vec![ 3, 4 ] );
    assert_eq!( walk( &RenderLayers { show_ships : false, ..all_shown() } ), vec![ 1, 2, 4 ] );
    assert_eq!( walk( &RenderLayers { show_station : false, ..all_shown() } ), vec![ 1, 2, 3 ] );
  }

  /// Only a selected, visible ship drives the ribbon, and only while the
  /// grid and its ribbon row are on.
  #[ test ]
  fn ribbon_follows_a_visible_selected_ship_only()
  {
    let ship = Some( PickedKind::Ship( 2 ) );
    assert_eq!( ribbon_ship( ship, &all_shown() ), Some( 2 ) );
    assert_eq!( ribbon_ship( ship, &RenderLayers { show_ships : false, ..all_shown() } ), None );
    assert_eq!( ribbon_ship( ship, &RenderLayers { show_view_ribbon : false, ..all_shown() } ), None );
    assert_eq!( ribbon_ship( ship, &RenderLayers { show_grid : false, ..all_shown() } ), None );
    assert_eq!( ribbon_ship( Some( PickedKind::Station ), &all_shown() ), None );
    assert_eq!( ribbon_ship( Some( PickedKind::Asteroid( 0 ) ), &all_shown() ), None );
    assert_eq!( ribbon_ship( None, &all_shown() ), None );
  }

  #[ test ]
  fn each_kind_follows_its_own_layer_switch()
  {
    let all_hidden = RenderLayers { show_asteroids : false, show_ships : false, show_station : false, ..RenderLayers::default() };
    for kind in [ PickedKind::Asteroid( 0 ), PickedKind::Ship( 0 ), PickedKind::Station ]
    {
      assert!( kind_visible( kind, &RenderLayers::default() ) );
      assert!( !kind_visible( kind, &all_hidden ) );
    }

    let ships_only = RenderLayers { show_asteroids : false, show_station : false, ..RenderLayers::default() };
    assert!( !kind_visible( PickedKind::Asteroid( 3 ), &ships_only ) );
    assert!( kind_visible( PickedKind::Ship( 3 ), &ships_only ) );
    assert!( !kind_visible( PickedKind::Station, &ships_only ) );
  }

  #[ test ]
  fn gizmo_needs_its_switch_and_a_visible_object()
  {
    let kind = PickedKind::Ship( 0 );
    assert!( gizmo_visible( kind, &RenderLayers::default() ) );
    assert!( !gizmo_visible( kind, &RenderLayers { show_gizmo : false, ..RenderLayers::default() } ) );
    assert!( !gizmo_visible( kind, &RenderLayers { show_ships : false, ..RenderLayers::default() } ) );
  }
}
