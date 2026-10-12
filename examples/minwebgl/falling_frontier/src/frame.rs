//! What `app_run`'s frame closure runs each frame: following a canvas resize
//! (`viewport_resize`), then `SceneRenderers::frame_render`, which draws the
//! passes in order. Its steps that need more than a line or two are separate
//! functions: ship motion, the view-zone ribbon's inputs, the sun light and
//! its shadow pass, and the lazily built trajectory ribbons.

use minwebgl as gl;
use gl::GL;
use std::cell::RefCell;
use renderer::webgl::Camera;
use renderer::webgl::shadow::{ ShadowMap, Light };

use crate::
{
  asteroids::Asteroids,
  background::Background,
  boundary::{ build_boundary_polyline, MAX_BOUNDARY_PTS },
  debug::{ GridTuning, layers_panel_sync },
  grid::{ FocusState, TacticalGrid, MAX_ASTEROID_GLOW },
  hull::{ HullPart, HullProgram },
  ribbon_ship,
  ships::{ self, Ships },
  starfield::Starfield,
  trajectories::Trajectories,
  classify_pick,
  gizmo_visible,
  selected_transform,
  selection_drop_if_hidden,
  visible_parts,
  InteractionCtx,
  PickedKind,
  GIZMO_ID,
  SHIP_ID_BASE,
};

// Shadow map sizing - deliberately covers just the ships/station/asteroid
// cluster (see asteroids.rs's ASTEROID_SPECS, all within roughly ±175 on
// X/Z), not the whole `grid::PLANE_SIZE` plane, so the ortho frustum stays tight
// enough for reasonable shadow resolution at SHADOW_MAP_RESOLUTION.
pub const SHADOW_MAP_RESOLUTION : u32 = 1024;
const SHADOW_HALF_EXTENT : f32 = 260.0;
const SHADOW_LIGHT_DISTANCE : f32 = 400.0;

/// Converts the dev panel's azimuth/elevation (degrees) into a normalized
/// world-space direction pointing *toward* the light - the convention
/// `hull.frag`'s `n_dot_l = dot(normal, light_dir)` and `ShadowMap`'s light
/// placement (see `sun_light`) both expect.
fn light_direction( azimuth_deg : f32, elevation_deg : f32 ) -> gl::F32x3
{
  let az = azimuth_deg.to_radians();
  let el = elevation_deg.to_radians();
  gl::F32x3::new( el.cos() * az.cos(), el.sin(), el.cos() * az.sin() )
}

/// M7: advances every ship along its patrol path, except whichever one is
/// currently selected - matches `main.js`'s `updateFleetMotion` skipping
/// `excludeMesh` (the gizmo-attached ship) so a drag isn't fought by the
/// path animation.
pub fn ships_advance( ships : &mut Ships, tuning : &GridTuning, selected : Option< i32 > )
{
  if !tuning.animate_ships { return; }
  for i in 0 .. ships::SHIP_COUNT
  {
    if Some( SHIP_ID_BASE + i as i32 ) == selected { continue; }
    let speed = ships.speed( i ) * tuning.speed_multiplier;
    ships.advance( i, speed );
  }
}

/// What the grid pass needs for the view-zone ribbon this frame: the focus
/// point, the boundary polyline around blocking asteroids and the asteroids
/// that glow inside the zone.
pub struct RibbonInputs
{
  /// Where the ribbon is centred: the `ribbon_ship`'s position, or inactive
  /// when no ship drives the ribbon this frame.
  pub focus : FocusState,
  boundary_buf : [ [ f32; 2 ]; MAX_BOUNDARY_PTS ],
  boundary_count : usize,
  /// The asteroids that glow on the grid, as XZ position and block radius:
  /// those within the view radius of an active focus, capped at
  /// `MAX_ASTEROID_GLOW`. Empty while the focus is inactive or asteroids are
  /// hidden.
  pub glow : Vec< ( [ f32; 2 ], f32 ) >,
}

impl RibbonInputs
{
  /// Only a selected ship drives the ribbon (`ribbon_ship`) - matches
  /// `main.js`'s `animate()`, which points the grid's focus at
  /// `gizmo.object` only when it defines a `viewRadius` (only ships do in
  /// `fleet.js`; the station and asteroids have none, so selecting them
  /// highlights the object but leaves the ribbon off). Hidden asteroids
  /// neither notch the ribbon nor glow on the grid.
  pub fn new( selected : Option< PickedKind >, tuning : &GridTuning, asteroids : &Asteroids, ships : &Ships ) -> Self
  {
    let focus = match ribbon_ship( selected, &tuning.layers )
    {
      Some( i ) => FocusState { active : true, point : ships.position( i ) },
      None => FocusState::default(),
    };
    let mut inputs = Self { focus, boundary_buf : [ [ 0.0; 2 ]; MAX_BOUNDARY_PTS ], boundary_count : 0, glow : Vec::new() };
    if focus.active && tuning.layers.show_asteroids
    {
      let blockers = asteroids.blockers();
      inputs.boundary_count = build_boundary_polyline
      (
        focus.point[ 0 ], focus.point[ 1 ],
        tuning.view_radius, &blockers, &mut inputs.boundary_buf
      );
      inputs.glow = asteroids.glow_candidates( focus.point, tuning.view_radius );
      inputs.glow.truncate( MAX_ASTEROID_GLOW );
    }
    inputs
  }

  /// The view-zone ribbon's outline on the ground plane, as XZ points of the
  /// closed polyline `build_boundary_polyline` traces around the focus,
  /// pulled in where asteroids block the view radius. Empty while the ribbon
  /// is off or asteroids are hidden; `Grid::draw` uploads it as
  /// `u_boundary_pts`.
  pub fn boundary( &self ) -> &[ [ f32; 2 ] ]
  {
    &self.boundary_buf[ .. self.boundary_count ]
  }
}

/// The directional light for the hull material, as its direction (toward
/// the light) and the shadow map's view-projection. The ortho frustum is
/// centered on the ship/station/asteroid cluster (not the whole grid) and
/// placed `SHADOW_LIGHT_DISTANCE` back along the light direction, matching
/// the `-light_dir` "camera looks back down at the scene" convention
/// `Light::view_projection` expects.
pub fn sun_light( tuning : &GridTuning ) -> ( gl::F32x3, gl::F32x4x4 )
{
  let light_dir = light_direction( tuning.light_azimuth, tuning.light_elevation );
  let shadow_scene_center = gl::F32x3::new( 0.0, 12.0, 0.0 );
  let shadow_projection = gl::math::mat3x3h::orthographic_rh_gl
  (
    -SHADOW_HALF_EXTENT, SHADOW_HALF_EXTENT, -SHADOW_HALF_EXTENT, SHADOW_HALF_EXTENT,
    1.0, SHADOW_LIGHT_DISTANCE + SHADOW_HALF_EXTENT,
  );
  let mut light = Light::new( shadow_scene_center + light_dir * SHADOW_LIGHT_DISTANCE, -light_dir, shadow_projection, tuning.light_size );
  ( light_dir, light.view_projection() )
}

/// Renders `parts` into the shadow map from the light. The caller passes
/// `visible_parts`, so a hidden object doesn't still cast a shadow onto the
/// rest of the scene.
pub fn shadow_pass< 'a >( gl : &GL, shadow_map : &ShadowMap, light_view_proj : gl::F32x4x4, parts : impl Iterator< Item = &'a HullPart > )
{
  shadow_map.bind();
  shadow_map.clear();
  for part in parts
  {
    shadow_map.mvp_upload( light_view_proj * part.model );
    gl.bind_vertex_array( Some( &part.vao ) );
    gl.draw_elements_with_i32( GL::TRIANGLES, part.index_count, GL::UNSIGNED_INT, 0 );
  }
}

/// Draws the trajectory ribbons, building them into `slot` the first time
/// they are shown. A failed build switches the layer and its Render Layers
/// row back off, so it isn't retried (and warned about) every frame.
pub fn trajectories_draw
(
  gl : &GL,
  slot : &mut Option< Trajectories >,
  tuning : &RefCell< GridTuning >,
  ships : &Ships,
  camera : &Camera,
  resolution : [ f32; 2 ],
)
{
  if slot.is_none()
  {
    match Trajectories::new( gl, ships, camera.projection_matrix_get(), resolution )
    {
      Ok( built ) => *slot = Some( built ),
      Err( e ) =>
      {
        web_sys::console::warn_1( &format!( "Falling Frontier: trajectory ribbons unavailable: {e}" ).into() );
        let mut tuning = tuning.borrow_mut();
        tuning.layers.show_trajectories = false;
        layers_panel_sync( &tuning.layers );
      }
    }
  }
  if let Some( trajectories ) = slot
  {
    trajectories.draw( gl, camera.view_matrix_get(), camera.projection_matrix_get(), resolution );
  }
}

/// The camera's projection parameters, kept to rebuild its projection when
/// the canvas is resized.
#[ derive( Clone, Copy ) ]
pub struct Projection
{
  pub fov : f32,
  pub near : f32,
  pub far : f32,
}

/// Follows a canvas resize to `size` (device pixels): the canvas's backing
/// store, the viewport, the pick buffer, and the camera's projection and
/// window size.
pub fn viewport_resize( gl : &GL, ctx : &InteractionCtx, camera : &mut Camera, projection : Projection, size : ( u32, u32 ) )
{
  let ( w, h ) = size;
  ctx.canvas.set_width( w );
  ctx.canvas.set_height( h );
  gl.viewport( 0, 0, w as i32, h as i32 );
  ctx.pick_buffer.borrow_mut().resize( gl, w as i32, h as i32 );

  let proj = gl::math::mat3x3h::perspective_rh_gl( projection.fov, w as f32 / h as f32, projection.near, projection.far );
  if let Err( e ) = camera.projection_matrix_set( proj )
  {
    web_sys::console::warn_1( &format!( "Falling Frontier: skipping invalid projection matrix on resize: {e}" ).into() );
  }
  camera.window_size_set( [ w as f32, h as f32 ].into() );
}

/// The scene's draw passes, and the trajectory ribbons they build on first
/// use: what the frame draws with, apart from the camera and the pickable
/// objects, which `InteractionCtx` holds.
pub struct SceneRenderers
{
  grid : TacticalGrid,
  hull_program : HullProgram,
  starfield : Starfield,
  background : Background,
  shadow_map : ShadowMap,
  /// Built on first use rather than at startup: the Trajectories row starts
  /// off, so a session that never shows them never builds one ribbon mesh
  /// per ship.
  trajectories : Option< Trajectories >,
}

impl SceneRenderers
{
  /// Creates every pass's program and bakes the background cube map.
  ///
  /// # Errors
  ///
  /// Returns the background bake's or the shadow map's `WebglError`.
  pub fn new( gl : &GL ) -> Result< Self, gl::WebglError >
  {
    Ok( Self
    {
      grid : TacticalGrid::new( gl ),
      hull_program : HullProgram::new( gl ),
      starfield : Starfield::new( gl ),
      background : Background::new( gl )?,
      shadow_map : ShadowMap::new( gl, SHADOW_MAP_RESOLUTION )?,
      trajectories : None,
    } )
  }

  /// Renders one frame at `size` (device pixels). It records the frame's
  /// view-projection for the pointer listeners and drops a selection whose
  /// layer was hidden, then draws each pass its Render Layers row allows:
  /// background, the shadow pass, hulls, starfield, trajectories, grid, and
  /// last the gizmo handle on top. Ships advance along their paths between
  /// the background and the shadow pass.
  pub fn frame_render( &mut self, gl : &GL, ctx : &InteractionCtx, camera : &Camera, tuning : &RefCell< GridTuning >, size : ( u32, u32 ) )
  {
    let ( w, h ) = size;
    gl.clear( GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT );

    let view_proj = camera.projection_matrix_get() * camera.view_matrix_get();
    ctx.latest_view_proj.set( view_proj );

    let tuning_snapshot = *tuning.borrow();
    let layers = &tuning_snapshot.layers;
    selection_drop_if_hidden( ctx, layers );
    let selected = ctx.selected_id.get();
    let selected_kind = selected.and_then( classify_pick );

    if layers.show_background
    {
      self.background.draw( gl, view_proj, camera.eye_get() );
    }

    ships_advance( &mut ctx.ships.borrow_mut(), &tuning_snapshot, selected );

    let asteroids = ctx.asteroids.borrow();
    let ships = ctx.ships.borrow();
    let station = ctx.station.borrow();

    let ribbon = RibbonInputs::new( selected_kind, &tuning_snapshot, &asteroids, &ships );

    let ( light_dir, light_view_proj ) = sun_light( &tuning_snapshot );
    if layers.shadows_drawn()
    {
      shadow_pass( gl, &self.shadow_map, light_view_proj, visible_parts( layers, &asteroids, &ships, &station ) );
    }
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );
    gl.viewport( 0, 0, w as i32, h as i32 );
    gl.disable( GL::CULL_FACE );

    self.hull_program.begin_frame
    (
      gl, view_proj, camera.eye_get(), light_dir, tuning_snapshot.light_color, tuning_snapshot.light_intensity, tuning_snapshot.light_size,
      light_view_proj, self.shadow_map.depth_buffer(), layers,
    );
    for part in visible_parts( layers, &asteroids, &ships, &station )
    {
      self.hull_program.draw_part( gl, part, Some( part.pick_id ) == selected );
    }

    if layers.show_starfield
    {
      self.starfield.draw( gl, view_proj );
    }

    if layers.show_trajectories
    {
      trajectories_draw( gl, &mut self.trajectories, tuning, &ships, camera, [ w as f32, h as f32 ] );
    }

    if layers.show_grid
    {
      self.grid.draw
      (
        gl, view_proj, camera.eye_get(), &tuning_snapshot, &ribbon.focus,
        ribbon.boundary(), &ribbon.glow
      );
    }

    // M6: the gizmo handle, drawn on top of everything at whatever is
    // currently selected (translate cross or rotate ring, per
    // `ctx.gizmo_mode`), while `gizmo_visible` - the same condition the
    // pick pass renders it under, so it can be grabbed exactly when seen.
    if let Some( kind ) = selected_kind && gizmo_visible( kind, layers )
    {
      let object_transform = selected_transform( kind, &asteroids, &ships, &station );
      let gizmo_part = ctx.gizmo.part( ctx.gizmo_mode.get(), object_transform, GIZMO_ID );
      ctx.gizmo.draw( gl, view_proj, &gizmo_part );
    }
  }
}
