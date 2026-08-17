//! Falling Frontier — tactical space-scene demo, ported from the Three.js/Vite
//! prototype in `examples/threejs/falling_frontier/`.
//!
//! M3 slice: the view-zone ribbon (boundary polyline wrapped around blocking
//! asteroids), the inside/outside brightness fade, and the asteroid
//! proximity glow — plus just enough asteroid geometry/ground-click picking
//! to exercise it. See `PORT_PLAN.md` in this crate for the full milestone
//! list and porting notes.

mod debug;
mod boundary;
mod asteroids;

use minwebgl as gl;
use gl::GL;
use renderer::webgl::Camera;
use std::{ cell::{ Cell, RefCell }, rc::Rc };
use debug::{ GridTuning, setup_grid_tuning_panel, refresh_focus_status };
use boundary::{ build_boundary_polyline, MAX_BOUNDARY_PTS };
use asteroids::Asteroids;

// Matches tacticalGrid.js's PLANE_SIZE - structural, not exposed in the
// tuning panel.
const PLANE_SIZE : f32 = 3000.0;

// Must match the fixed-size uniform arrays declared in grid.frag
// (`u_asteroid_pos[16]`/`u_asteroid_radius[16]`).
const MAX_ASTEROID_GLOW : usize = 16;

/// Ground-click focus point - a throwaway stand-in for real unit selection
/// (that's M5). `active` mirrors the JS shader's `uFocusActive`.
#[ derive( Clone, Copy ) ]
pub struct FocusState
{
  /// Whether a ground click has set a focus point yet.
  pub active : bool,
  /// World-space XZ position of the focus point.
  pub point : [ f32; 2 ],
}

impl Default for FocusState
{
  fn default() -> Self
  {
    Self { active : false, point : [ 0.0, 0.0 ] }
  }
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

fn unproject( inv_view_proj : gl::F32x4x4, ndc_x : f32, ndc_y : f32, ndc_z : f32 ) -> gl::math::F32x3
{
  let clip = gl::math::F32x4::new( ndc_x, ndc_y, ndc_z, 1.0 );
  let world = inv_view_proj * clip;
  gl::math::F32x3::new( world.x() / world.w(), world.y() / world.w(), world.z() / world.w() )
}

/// Casts a ray from the camera through the clicked pixel and intersects it
/// with the `y = 0` ground plane — a stand-in for real object picking (M5).
/// `view_proj` is a snapshot from the most recent frame (see `RayContext`),
/// not recomputed here.
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

struct GridUniforms
{
  view_proj : Option< gl::WebGlUniformLocation >,
  camera_position : Option< gl::WebGlUniformLocation >,
  line_color : Option< gl::WebGlUniformLocation >,
  dim_alpha : Option< gl::WebGlUniformLocation >,
  cell_size : Option< gl::WebGlUniformLocation >,
  line_width_px : Option< gl::WebGlUniformLocation >,
  camera_fade_start : Option< gl::WebGlUniformLocation >,
  camera_fade_end : Option< gl::WebGlUniformLocation >,
  camera_fade_mode : Option< gl::WebGlUniformLocation >,
  camera_fade_gamma : Option< gl::WebGlUniformLocation >,

  ring_color_core : Option< gl::WebGlUniformLocation >,
  ring_color_edge : Option< gl::WebGlUniformLocation >,
  bright_alpha : Option< gl::WebGlUniformLocation >,
  focus_point : Option< gl::WebGlUniformLocation >,
  focus_active : Option< gl::WebGlUniformLocation >,
  view_radius : Option< gl::WebGlUniformLocation >,
  ribbon_width_outer : Option< gl::WebGlUniformLocation >,
  ribbon_width_inner : Option< gl::WebGlUniformLocation >,
  ribbon_gap : Option< gl::WebGlUniformLocation >,
  ribbon_opacity : Option< gl::WebGlUniformLocation >,
  inside_fade_width : Option< gl::WebGlUniformLocation >,
  inside_fade_mode : Option< gl::WebGlUniformLocation >,
  inside_fade_gamma : Option< gl::WebGlUniformLocation >,
  boundary_pts : Option< gl::WebGlUniformLocation >,
  boundary_count : Option< gl::WebGlUniformLocation >,
  asteroid_pos : Option< gl::WebGlUniformLocation >,
  asteroid_radius : Option< gl::WebGlUniformLocation >,
  asteroid_count : Option< gl::WebGlUniformLocation >,
  asteroid_glow_alpha : Option< gl::WebGlUniformLocation >,
  asteroid_glow_width : Option< gl::WebGlUniformLocation >,
  asteroid_glow_mode : Option< gl::WebGlUniformLocation >,
  asteroid_glow_gamma : Option< gl::WebGlUniformLocation >,
}

struct TacticalGrid
{
  vao : gl::WebGlVertexArrayObject,
  vertex_count : i32,
  program : gl::WebGlProgram,
  uniforms : GridUniforms,
}

impl TacticalGrid
{
  fn new( gl : &GL ) -> Self
  {
    let half = PLANE_SIZE * 0.5;
    let positions : [ [ f32; 3 ]; 6 ] =
    [
      [ -half, 0.0, -half ], [  half, 0.0, -half ], [  half, 0.0,  half ],
      [ -half, 0.0, -half ], [  half, 0.0,  half ], [ -half, 0.0,  half ],
    ];

    let vao = gl::vao::create( gl ).unwrap();
    gl.bind_vertex_array( Some( &vao ) );

    let position_buffer = gl::buffer::create( gl ).unwrap();
    gl::buffer::upload( gl, &position_buffer, positions.as_slice(), GL::STATIC_DRAW );
    gl::BufferDescriptor::new::< [ f32; 3 ] >()
    .stride( 0 )
    .offset( 0 )
    .attribute_pointer( gl, 0, &position_buffer )
    .unwrap();

    let vertex_shader = include_str!( "shaders/grid.vert" );
    let fragment_shader = include_str!( "shaders/grid.frag" );
    let program = gl::ProgramFromSources::new( vertex_shader, fragment_shader )
    .compile_and_link( gl )
    .unwrap();

    let uniforms = GridUniforms
    {
      view_proj : gl.get_uniform_location( &program, "u_view_proj" ),
      camera_position : gl.get_uniform_location( &program, "u_camera_position" ),
      line_color : gl.get_uniform_location( &program, "u_line_color" ),
      dim_alpha : gl.get_uniform_location( &program, "u_dim_alpha" ),
      cell_size : gl.get_uniform_location( &program, "u_cell_size" ),
      line_width_px : gl.get_uniform_location( &program, "u_line_width_px" ),
      camera_fade_start : gl.get_uniform_location( &program, "u_camera_fade_start" ),
      camera_fade_end : gl.get_uniform_location( &program, "u_camera_fade_end" ),
      camera_fade_mode : gl.get_uniform_location( &program, "u_camera_fade_mode" ),
      camera_fade_gamma : gl.get_uniform_location( &program, "u_camera_fade_gamma" ),

      ring_color_core : gl.get_uniform_location( &program, "u_ring_color_core" ),
      ring_color_edge : gl.get_uniform_location( &program, "u_ring_color_edge" ),
      bright_alpha : gl.get_uniform_location( &program, "u_bright_alpha" ),
      focus_point : gl.get_uniform_location( &program, "u_focus_point" ),
      focus_active : gl.get_uniform_location( &program, "u_focus_active" ),
      view_radius : gl.get_uniform_location( &program, "u_view_radius" ),
      ribbon_width_outer : gl.get_uniform_location( &program, "u_ribbon_width_outer" ),
      ribbon_width_inner : gl.get_uniform_location( &program, "u_ribbon_width_inner" ),
      ribbon_gap : gl.get_uniform_location( &program, "u_ribbon_gap" ),
      ribbon_opacity : gl.get_uniform_location( &program, "u_ribbon_opacity" ),
      inside_fade_width : gl.get_uniform_location( &program, "u_inside_fade_width" ),
      inside_fade_mode : gl.get_uniform_location( &program, "u_inside_fade_mode" ),
      inside_fade_gamma : gl.get_uniform_location( &program, "u_inside_fade_gamma" ),
      boundary_pts : gl.get_uniform_location( &program, "u_boundary_pts" ),
      boundary_count : gl.get_uniform_location( &program, "u_boundary_count" ),
      asteroid_pos : gl.get_uniform_location( &program, "u_asteroid_pos" ),
      asteroid_radius : gl.get_uniform_location( &program, "u_asteroid_radius" ),
      asteroid_count : gl.get_uniform_location( &program, "u_asteroid_count" ),
      asteroid_glow_alpha : gl.get_uniform_location( &program, "u_asteroid_glow_alpha" ),
      asteroid_glow_width : gl.get_uniform_location( &program, "u_asteroid_glow_width" ),
      asteroid_glow_mode : gl.get_uniform_location( &program, "u_asteroid_glow_mode" ),
      asteroid_glow_gamma : gl.get_uniform_location( &program, "u_asteroid_glow_gamma" ),
    };

    Self { vao, vertex_count : positions.len() as i32, program, uniforms }
  }

  #[ allow( clippy::too_many_arguments, reason = "mirrors the JS shader's own uniform surface — splitting it up would just move the same argument count into a struct with no real grouping" ) ]
  fn draw
  (
    &self,
    gl : &GL,
    view_proj : gl::F32x4x4,
    camera_position : gl::F32x3,
    tuning : &GridTuning,
    focus : &FocusState,
    boundary_pts : &[ [ f32; 2 ] ],
    glow : &[ ( [ f32; 2 ], f32 ) ],
  )
  {
    gl.use_program( Some( &self.program ) );
    let u = &self.uniforms;
    gl::uniform::matrix_upload( gl, u.view_proj.clone(), view_proj.to_array().as_slice(), true ).unwrap();
    gl::uniform::upload( gl, u.camera_position.clone(), camera_position.to_array().as_slice() ).unwrap();
    gl::uniform::upload( gl, u.line_color.clone(), tuning.line_color.as_slice() ).unwrap();
    gl::uniform::upload( gl, u.dim_alpha.clone(), &tuning.dim_alpha ).unwrap();
    gl::uniform::upload( gl, u.cell_size.clone(), &tuning.cell_size ).unwrap();
    gl::uniform::upload( gl, u.line_width_px.clone(), &tuning.line_width_px ).unwrap();
    gl::uniform::upload( gl, u.camera_fade_start.clone(), &tuning.camera_fade_start ).unwrap();
    gl::uniform::upload( gl, u.camera_fade_end.clone(), &tuning.camera_fade_end ).unwrap();
    gl::uniform::upload( gl, u.camera_fade_mode.clone(), &tuning.camera_fade_mode ).unwrap();
    gl::uniform::upload( gl, u.camera_fade_gamma.clone(), &tuning.camera_fade_gamma ).unwrap();

    gl::uniform::upload( gl, u.ring_color_core.clone(), tuning.ribbon_color_core.as_slice() ).unwrap();
    gl::uniform::upload( gl, u.ring_color_edge.clone(), tuning.ribbon_color_edge.as_slice() ).unwrap();
    gl::uniform::upload( gl, u.bright_alpha.clone(), &tuning.bright_alpha ).unwrap();
    gl::uniform::upload( gl, u.focus_point.clone(), focus.point.as_slice() ).unwrap();
    gl::uniform::upload( gl, u.focus_active.clone(), &if focus.active { 1.0f32 } else { 0.0f32 } ).unwrap();
    gl::uniform::upload( gl, u.view_radius.clone(), &tuning.view_radius ).unwrap();
    gl::uniform::upload( gl, u.ribbon_width_outer.clone(), &tuning.ribbon_width_outer ).unwrap();
    gl::uniform::upload( gl, u.ribbon_width_inner.clone(), &tuning.ribbon_width_inner ).unwrap();
    gl::uniform::upload( gl, u.ribbon_gap.clone(), &tuning.ribbon_gap ).unwrap();
    gl::uniform::upload( gl, u.ribbon_opacity.clone(), &tuning.ribbon_opacity ).unwrap();
    gl::uniform::upload( gl, u.inside_fade_width.clone(), &tuning.inside_fade_width ).unwrap();
    gl::uniform::upload( gl, u.inside_fade_mode.clone(), &tuning.inside_fade_mode ).unwrap();
    gl::uniform::upload( gl, u.inside_fade_gamma.clone(), &tuning.inside_fade_gamma ).unwrap();
    gl::uniform::upload( gl, u.boundary_count.clone(), &( boundary_pts.len() as i32 ) ).unwrap();
    if !boundary_pts.is_empty()
    {
      gl::uniform::upload( gl, u.boundary_pts.clone(), boundary_pts ).unwrap();
    }

    let asteroid_positions : Vec< [ f32; 2 ] > = glow.iter().map( | ( p, _ ) | *p ).collect();
    // Wrapped as [f32;1] rather than passed as a bare &[f32] - the latter
    // dispatches to the "single vecN uniform, length must be 1..=4" upload
    // path (see minwebgl's uniform::float32 impls), not the "array of N
    // scalar uniform elements" path an arbitrary-length float[] array needs.
    let asteroid_radii : Vec< [ f32; 1 ] > = glow.iter().map( | ( _, r ) | [ *r ] ).collect();
    gl::uniform::upload( gl, u.asteroid_count.clone(), &( glow.len() as i32 ) ).unwrap();
    if !glow.is_empty()
    {
      gl::uniform::upload( gl, u.asteroid_pos.clone(), asteroid_positions.as_slice() ).unwrap();
      gl::uniform::upload( gl, u.asteroid_radius.clone(), asteroid_radii.as_slice() ).unwrap();
    }
    gl::uniform::upload( gl, u.asteroid_glow_alpha.clone(), &tuning.asteroid_glow_alpha ).unwrap();
    gl::uniform::upload( gl, u.asteroid_glow_width.clone(), &tuning.asteroid_glow_width ).unwrap();
    gl::uniform::upload( gl, u.asteroid_glow_mode.clone(), &tuning.asteroid_glow_mode ).unwrap();
    gl::uniform::upload( gl, u.asteroid_glow_gamma.clone(), &tuning.asteroid_glow_gamma ).unwrap();

    gl.enable( GL::BLEND );
    gl.blend_func( GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA );
    gl.depth_mask( false );

    gl.bind_vertex_array( Some( &self.vao ) );
    gl.draw_arrays( GL::TRIANGLES, 0, self.vertex_count );

    gl.depth_mask( true );
    gl.disable( GL::BLEND );
  }
}

fn app_run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );

  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas( &canvas )?;
  gl.enable( GL::DEPTH_TEST );
  gl.clear_color( 0.0, 0.0, 0.0, 1.0 );

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
  let asteroids = Asteroids::new( &gl );
  gl.viewport( 0, 0, pixel_w as i32, pixel_h as i32 );

  let tuning = Rc::new( RefCell::new( GridTuning::default() ) );
  let focus = Rc::new( RefCell::new( FocusState::default() ) );
  let document = gl::web_sys::window().unwrap().document().unwrap();
  setup_grid_tuning_panel( &document, &tuning, &focus );

  // Last frame's view_proj, kept for the ground-click handler below — the
  // click fires outside the render loop's closure, which owns `camera`.
  let initial_view_proj = camera.projection_matrix_get() * camera.view_matrix_get();
  let ray_view_proj = Rc::new( Cell::new( initial_view_proj ) );

  setup_ground_click( &canvas, &document, &ray_view_proj, &focus );

  let prev_size = Cell::new( ( pixel_w, pixel_h ) );
  let mut prev_time = 0.0f64;

  let update_and_draw =
  {
    let canvas = canvas.clone();
    let ray_view_proj = ray_view_proj.clone();
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

        let proj = gl::math::mat3x3h::perspective_rh_gl( fov, w as f32 / h as f32, near, far );
        camera.projection_matrix_set( proj );
        camera.window_size_set( [ w as f32, h as f32 ].into() );

        prev_size.set( ( w, h ) );
      }

      gl.clear( GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT );

      let view_proj = camera.projection_matrix_get() * camera.view_matrix_get();
      ray_view_proj.set( view_proj );

      let focus_snapshot = *focus.borrow();
      let tuning_snapshot = *tuning.borrow();

      let mut boundary_buf = [ [ 0.0f32; 2 ]; MAX_BOUNDARY_PTS ];
      let mut boundary_count = 0;
      let mut glow : Vec< ( [ f32; 2 ], f32 ) > = Vec::new();
      if focus_snapshot.active
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

      asteroids.draw( &gl, view_proj );
      grid.draw
      (
        &gl, view_proj, camera.eye_get(), &tuning_snapshot, &focus_snapshot,
        &boundary_buf[ .. boundary_count ], &glow
      );

      true
    }
  };

  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

/// Wires a click-vs-drag-aware pointer handler on `canvas`: a press+release
/// within a few pixels of each other casts a ray through the clicked pixel
/// and intersects it with the `y = 0` ground plane, setting `focus` to the
/// hit point. A `pointerup` that followed a real drag (camera orbit) is
/// ignored. Real object picking replaces this in M5.
fn setup_ground_click
(
  canvas : &gl::web_sys::HtmlCanvasElement,
  document : &gl::web_sys::Document,
  ray_view_proj : &Rc< Cell< gl::F32x4x4 > >,
  focus : &Rc< RefCell< FocusState > >,
)
{
  use gl::web_sys::wasm_bindgen::{ prelude::Closure, JsCast };

  let down_pos : Rc< Cell< Option< ( f64, f64 ) > > > = Rc::new( Cell::new( None ) );

  {
    let down_pos = down_pos.clone();
    let closure = Closure::< dyn FnMut( _ ) >::new
    (
      move | e : gl::web_sys::PointerEvent | { down_pos.set( Some( pointer_client_pos( &e ) ) ); }
    );
    canvas.add_event_listener_with_callback( "pointerdown", closure.as_ref().unchecked_ref() ).unwrap();
    closure.forget();
  }

  {
    let down_pos = down_pos.clone();
    let ray_view_proj = ray_view_proj.clone();
    let focus = focus.clone();
    let canvas_for_hit = canvas.clone();
    let document = document.clone();
    let closure = Closure::< dyn FnMut( _ ) >::new
    (
      move | e : gl::web_sys::PointerEvent |
      {
        if e.button() != 0 { return; }
        let Some( ( dx0, dy0 ) ) = down_pos.get() else { return };
        let ( x, y ) = pointer_client_pos( &e );
        let ( dx, dy ) = ( x - dx0, y - dy0 );
        // A real drag (camera orbit), not a click - leave focus alone.
        if dx.hypot( dy ) > 6.0 { return; }

        if let Some( point ) = ray_ground_hit( ray_view_proj.get(), &canvas_for_hit, x, y )
        {
          let mut f = focus.borrow_mut();
          f.active = true;
          f.point = point;
          drop( f );
          refresh_focus_status( &document, &focus.borrow() );
        }
      }
    );
    canvas.add_event_listener_with_callback( "pointerup", closure.as_ref().unchecked_ref() ).unwrap();
    closure.forget();
  }
}

fn main()
{
  app_run().unwrap();
}
