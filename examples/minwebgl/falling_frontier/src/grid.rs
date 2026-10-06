//! The tactical grid: one large ground quad whose fragment shader
//! (`shaders/grid.frag`) draws the anti-aliased cell lines, the camera
//! distance fade and, around a selected ship, the view-zone ribbon with its
//! inside brightening and asteroid glow. Ported from the three.js original's
//! `tacticalGrid.js`.

use minwebgl as gl;
use gl::GL;

use crate::debug::GridTuning;

// Matches tacticalGrid.js's PLANE_SIZE - structural, not exposed in the
// tuning panel.
const PLANE_SIZE : f32 = 3000.0;

// Must match the fixed-size uniform arrays declared in grid.frag
// (`u_asteroid_pos[16]`/`u_asteroid_radius[16]`).
pub const MAX_ASTEROID_GLOW : usize = 16;

/// The grid's view-zone ribbon, derived fresh every frame from whatever is
/// currently selected — ported from `main.js`'s `animate()`, which points
/// the ribbon at `gizmo.object` whenever it defines a `viewRadius` (only
/// ships do in `fleet.js`) and leaves it off otherwise. `active` mirrors the
/// JS shader's `uFocusActive`.
#[ derive( Clone, Copy ) ]
pub struct FocusState
{
  /// Whether a ship is currently selected (and so the ribbon should show).
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

pub struct TacticalGrid
{
  vao : gl::WebGlVertexArrayObject,
  vertex_count : i32,
  program : gl::WebGlProgram,
  uniforms : GridUniforms,
}

impl TacticalGrid
{
  pub fn new( gl : &GL ) -> Self
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
  pub fn draw
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
