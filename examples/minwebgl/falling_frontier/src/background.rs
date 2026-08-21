//! Nebula skybox backdrop - stands in for `scene/world.js`'s flat
//! `scene.background = new THREE.Color(COLORS.spaceBg)`, matching the soft
//! drifting cloud-blob look of the reference game screenshot the tactical UI
//! itself is modeled on rather than the JS port's flat placeholder.
//!
//! The nebula is a 5-octave fbm evaluated per pixel (`shaders/background.frag`),
//! too expensive to run every frame for what is, in the end, a static
//! backdrop the camera orbits but never approaches. `Background::new` bakes
//! that formula once into a cube map (one draw per face, camera at the
//! origin, `u_time` frozen at 0 - see `bake_cubemap`) and every subsequent
//! frame just samples it (`shaders/skybox.frag`), one texture fetch instead
//! of the whole noise stack. Drawn first each frame with depth test/write
//! both off, so it always sits behind every other draw call regardless of
//! camera orbit.

use minwebgl as gl;
use gl::GL;

// Smooth, low-frequency nebula content - no fine detail to preserve, so a
// modest face resolution is plenty and keeps the one-time bake cheap.
const BAKE_RESOLUTION : i32 = 512;

/// Per-face view-projection for baking, sharing the six-direction convention
/// (and `TEXTURE_CUBE_MAP_POSITIVE_X`-relative face order) `examples/minwebgl/
/// make_cube_map`'s own `cube_camera_make` uses, so face `i` here always
/// lines up with `TEXTURE_CUBE_MAP_POSITIVE_X + i`.
///
/// `px`/`nx` are deliberately *not* in POSITIVE_X/NEGATIVE_X slot order in
/// the returned array - `px` (as named/defined below) is actually the
/// standard camera for the NEGATIVE_X slot, and `nx` for POSITIVE_X, per the
/// usual OpenGL cubemap-face camera table (e.g. LearnOpenGL's point-shadow
/// cubemap setup), which `py`/`ny`/`pz`/`nz` already follow directly. Mixing
/// up that one pair showed up as a visible seam between the X faces and
/// their neighbors; swapping the two slots (not their camera directions)
/// fixes it, confirmed seam-free by orbiting a world-direction-colored test
/// bake (RGB = ray_dir) across the whole sphere.
fn cube_face_view_proj() -> [ gl::F32x4x4; 6 ]
{
  let px = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::NEG_X, gl::F32x3::NEG_Y );
  let nx = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::X, gl::F32x3::NEG_Y );
  let py = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::Y, gl::F32x3::Z );
  let ny = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::NEG_Y, gl::F32x3::NEG_Z );
  let pz = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::Z, gl::F32x3::NEG_Y );
  let nz = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::NEG_Z, gl::F32x3::NEG_Y );

  let proj = gl::math::mat3x3h::perspective_rh_gl( 90.0f32.to_radians(), 1.0, 0.1, 10.0 );
  [ proj * nx, proj * px, proj * py, proj * ny, proj * pz, proj * nz ]
}

/// Renders `shaders/background.frag`'s nebula formula once per cube face
/// into a freshly created cube texture. Reuses `background.vert`'s
/// attributeless "big triangle" trick and `background.frag`'s own
/// `u_inv_view_proj`/`u_camera_position` ray reconstruction unmodified - with
/// the camera pinned to the origin, `far.xyz - u_camera_position` reduces to
/// exactly the direction each face's view-projection was built to cover, so
/// sampling the result later with that same direction reproduces the formula
/// faithfully.
fn bake_cubemap( gl : &GL ) -> gl::web_sys::WebGlTexture
{
  let vertex_shader = include_str!( "shaders/background.vert" );
  let fragment_shader = include_str!( "shaders/background.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader, fragment_shader )
  .compile_and_link( gl )
  .unwrap();

  let inv_view_proj_loc = gl.get_uniform_location( &program, "u_inv_view_proj" );
  let camera_position_loc = gl.get_uniform_location( &program, "u_camera_position" );
  let time_loc = gl.get_uniform_location( &program, "u_time" );

  // No attributes, same reasoning as `Background`'s own runtime vao below.
  let vao = gl::vao::create( gl ).unwrap();

  let texture = gl.create_texture().unwrap();
  gl.bind_texture( GL::TEXTURE_CUBE_MAP, Some( &texture ) );
  for i in 0 .. 6
  {
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
    (
      GL::TEXTURE_CUBE_MAP_POSITIVE_X + i, 0, GL::RGBA as i32,
      BAKE_RESOLUTION, BAKE_RESOLUTION, 0, GL::RGBA, GL::UNSIGNED_BYTE, None,
    ).unwrap();
  }
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32 );
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32 );
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( GL::TEXTURE_CUBE_MAP, GL::TEXTURE_WRAP_R, GL::CLAMP_TO_EDGE as i32 );

  let framebuffer = gl.create_framebuffer();
  gl.bind_framebuffer( GL::FRAMEBUFFER, framebuffer.as_ref() );
  gl.viewport( 0, 0, BAKE_RESOLUTION, BAKE_RESOLUTION );
  gl.use_program( Some( &program ) );
  gl.bind_vertex_array( Some( &vao ) );
  gl::uniform::upload( gl, camera_position_loc, gl::F32x3::ZERO.to_array().as_slice() ).unwrap();
  gl::uniform::upload( gl, time_loc, &0.0f32 ).unwrap();

  for ( i, view_proj ) in cube_face_view_proj().iter().enumerate()
  {
    // The bake camera's perspective/look-at is never degenerate, so this
    // inverse always exists.
    let inv_view_proj = view_proj.inverse().unwrap();
    gl::uniform::matrix_upload( gl, inv_view_proj_loc.clone(), inv_view_proj.to_array().as_slice(), true ).unwrap();
    gl.framebuffer_texture_2d
    (
      GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0,
      GL::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32, Some( &texture ), 0,
    );
    gl.draw_arrays( GL::TRIANGLES, 0, 3 );
  }

  gl.bind_framebuffer( GL::FRAMEBUFFER, None );
  gl.delete_framebuffer( framebuffer.as_ref() );

  texture
}

struct SkyboxUniforms
{
  inv_view_proj : Option< gl::WebGlUniformLocation >,
  camera_position : Option< gl::WebGlUniformLocation >,
  skybox : Option< gl::WebGlUniformLocation >,
}

pub struct Background
{
  vao : gl::WebGlVertexArrayObject,
  program : gl::WebGlProgram,
  uniforms : SkyboxUniforms,
  cubemap : gl::web_sys::WebGlTexture,
}

impl Background
{
  pub fn new( gl : &GL ) -> Self
  {
    let cubemap = bake_cubemap( gl );

    // No attributes - `background.vert` draws its triangle purely off
    // `gl_VertexID`, but WebGL2 still requires *a* VAO bound to draw at all.
    let vao = gl::vao::create( gl ).unwrap();

    let vertex_shader = include_str!( "shaders/background.vert" );
    let fragment_shader = include_str!( "shaders/skybox.frag" );
    let program = gl::ProgramFromSources::new( vertex_shader, fragment_shader )
    .compile_and_link( gl )
    .unwrap();

    let uniforms = SkyboxUniforms
    {
      inv_view_proj : gl.get_uniform_location( &program, "u_inv_view_proj" ),
      camera_position : gl.get_uniform_location( &program, "u_camera_position" ),
      skybox : gl.get_uniform_location( &program, "u_skybox" ),
    };

    Self { vao, program, uniforms, cubemap }
  }

  pub fn draw( &self, gl : &GL, view_proj : gl::F32x4x4, camera_position : gl::F32x3 )
  {
    // A non-invertible view_proj can't happen with this scene's fixed
    // perspective projection, but skip the draw rather than upload garbage
    // if it ever did.
    let Some( inv_view_proj ) = view_proj.inverse() else { return };

    gl.use_program( Some( &self.program ) );
    let u = &self.uniforms;
    gl::uniform::matrix_upload( gl, u.inv_view_proj.clone(), inv_view_proj.to_array().as_slice(), true ).unwrap();
    gl::uniform::upload( gl, u.camera_position.clone(), camera_position.to_array().as_slice() ).unwrap();

    gl.active_texture( GL::TEXTURE0 );
    gl.bind_texture( GL::TEXTURE_CUBE_MAP, Some( &self.cubemap ) );
    gl::uniform::upload( gl, u.skybox.clone(), &0i32 ).unwrap();

    gl.disable( GL::DEPTH_TEST );
    gl.depth_mask( false );

    gl.bind_vertex_array( Some( &self.vao ) );
    gl.draw_arrays( GL::TRIANGLES, 0, 3 );

    gl.depth_mask( true );
    gl.enable( GL::DEPTH_TEST );
  }
}
