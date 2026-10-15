//! Nebula skybox backdrop - stands in for the three.js original's flat
//! `COLORS.spaceBg` scene background colour, matching the soft
//! drifting cloud-blob look of the reference game screenshot the tactical UI
//! itself is modeled on rather than the JS port's flat placeholder.
//!
//! The nebula is a 5-octave fbm evaluated per pixel (`shaders/background.frag`),
//! too expensive to run every frame for what is, in the end, a static
//! backdrop the camera orbits but never approaches. `Background::new` bakes
//! that formula once into a cube map (one draw per face, camera at the
//! origin - see `bake_cubemap`; with no per-frame evaluation the clouds no
//! longer drift, so the formula has no time term) and every subsequent
//! frame just samples it (`shaders/skybox.frag`), one texture fetch instead
//! of the whole noise stack. Drawn first each frame with depth test/write
//! both off, so it always sits behind every other draw call regardless of
//! camera orbit.

use minwebgl as gl;
use gl::GL;
use gl::web_sys::wasm_bindgen::JsCast;

// Smooth, low-frequency nebula content - no fine detail to preserve, so a
// modest face resolution is plenty and keeps the one-time bake cheap.
const BAKE_RESOLUTION : i32 = 512;

/// Per-face view-projection for baking: slot `i` is the camera for
/// `TEXTURE_CUBE_MAP_POSITIVE_X + i`, looking down that face's axis with the
/// up vector the GL cube-map lookup implies (-Y for the four side faces,
/// +Z / -Z for +Y / -Y). `tests` checks every slot against the lookup
/// table, so a swapped or mirrored face (which once showed up here as a seam
/// between the X faces and their neighbours) fails a test instead of only
/// being visible when orbiting the backdrop.
fn cube_face_view_proj() -> [ gl::F32x4x4; 6 ]
{
  let px = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::X, gl::F32x3::NEG_Y );
  let nx = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::NEG_X, gl::F32x3::NEG_Y );
  let py = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::Y, gl::F32x3::Z );
  let ny = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::NEG_Y, gl::F32x3::NEG_Z );
  let pz = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::Z, gl::F32x3::NEG_Y );
  let nz = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::NEG_Z, gl::F32x3::NEG_Y );

  let proj = gl::math::mat3x3h::perspective_rh_gl( 90.0f32.to_radians(), 1.0, 0.1, 10.0 );
  [ proj * px, proj * nx, proj * py, proj * ny, proj * pz, proj * nz ]
}

/// Renders `shaders/background.frag`'s nebula formula once per cube face
/// into a freshly created cube texture. Reuses `background.vert`'s
/// attributeless "big triangle" trick and `background.frag`'s own
/// `u_inv_view_proj`/`u_camera_position` ray reconstruction unmodified - with
/// the camera pinned to the origin, `far.xyz - u_camera_position` reduces to
/// exactly the direction each face's view-projection was built to cover, so
/// sampling the result later with that same direction reproduces the formula
/// faithfully.
///
/// Leaves the GL state as it found it: the viewport is restored, and the
/// one-shot program, VAO and framebuffer are deleted whether or not every
/// face rendered.
///
/// # Errors
///
/// A shader that fails to compile or link, a GL object that can't be
/// created, or a face attachment that leaves the framebuffer incomplete.
fn bake_cubemap( gl : &GL ) -> Result< gl::web_sys::WebGlTexture, gl::WebglError >
{
  let program = bake_program_create( gl )?;
  let texture = bake_texture_create( gl )?;
  let viewport = viewport_get( gl );

  // No attributes, same reasoning as `Background`'s own runtime vao below.
  let vao = gl::vao::create( gl );
  let framebuffer = gl.create_framebuffer();
  let rendered = match ( vao.as_ref(), framebuffer.as_ref() )
  {
    ( Ok( vao ), Some( framebuffer ) ) => bake_faces_render( gl, &program, vao, framebuffer, &texture ),
    ( Err( _ ), _ ) => Err( gl::WebglError::Other( "failed to create the cube map bake VAO" ) ),
    ( _, None ) => Err( gl::WebglError::Other( "failed to create the cube map bake framebuffer" ) ),
  };

  gl.viewport( viewport[ 0 ], viewport[ 1 ], viewport[ 2 ], viewport[ 3 ] );
  bake_teardown( gl, &program, vao.as_ref().ok(), framebuffer.as_ref() );

  match rendered
  {
    Ok( () ) => Ok( texture ),
    Err( e ) =>
    {
      gl.delete_texture( Some( &texture ) );
      Err( e )
    }
  }
}

/// Compiles `background.frag`'s nebula formula with the attributeless
/// full-screen `background.vert`.
fn bake_program_create( gl : &GL ) -> Result< gl::WebGlProgram, gl::WebglError >
{
  let vertex_shader = include_str!( "shaders/background.vert" );
  let fragment_shader = include_str!( "shaders/background.frag" );
  Ok( gl::ProgramFromSources::new( vertex_shader, fragment_shader ).compile_and_link( gl )? )
}

/// An empty `BAKE_RESOLUTION` RGBA8 cube map, linearly filtered and clamped,
/// left bound to `TEXTURE_CUBE_MAP`.
fn bake_texture_create( gl : &GL ) -> Result< gl::web_sys::WebGlTexture, gl::WebglError >
{
  let texture = gl.create_texture().ok_or( gl::WebglError::Other( "failed to create the cube map texture" ) )?;
  gl.bind_texture( GL::TEXTURE_CUBE_MAP, Some( &texture ) );
  for i in 0 .. 6
  {
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
    (
      GL::TEXTURE_CUBE_MAP_POSITIVE_X + i, 0, GL::RGBA as i32,
      BAKE_RESOLUTION, BAKE_RESOLUTION, 0, GL::RGBA, GL::UNSIGNED_BYTE, None,
    )
    .map_err( | _ | gl::WebglError::Other( "failed to allocate a cube map face" ) )?;
  }
  gl::texture::cube::filter_linear( gl );
  gl::texture::cube::wrap_clamp( gl );
  Ok( texture )
}

/// The current viewport as `[ x, y, width, height ]`.
fn viewport_get( gl : &GL ) -> [ i32; 4 ]
{
  let mut viewport = [ 0; 4 ];
  let value = gl.get_parameter( GL::VIEWPORT ).ok()
  .and_then( | v | v.dyn_into::< gl::web_sys::js_sys::Int32Array >().ok() );
  if let Some( value ) = value
  {
    value.copy_to( &mut viewport );
  }
  viewport
}

/// Draws the nebula into each face of `texture` through `framebuffer`,
/// checking the framebuffer is complete before each draw.
fn bake_faces_render
(
  gl : &GL,
  program : &gl::WebGlProgram,
  vao : &gl::WebGlVertexArrayObject,
  framebuffer : &gl::web_sys::WebGlFramebuffer,
  texture : &gl::web_sys::WebGlTexture,
) -> Result< (), gl::WebglError >
{
  let inv_view_proj_loc = gl.get_uniform_location( program, "u_inv_view_proj" );
  let camera_position_loc = gl.get_uniform_location( program, "u_camera_position" );

  gl.bind_framebuffer( GL::FRAMEBUFFER, Some( framebuffer ) );
  gl.viewport( 0, 0, BAKE_RESOLUTION, BAKE_RESOLUTION );
  gl.use_program( Some( program ) );
  gl.bind_vertex_array( Some( vao ) );
  gl::uniform::upload( gl, camera_position_loc, gl::F32x3::ZERO.to_array().as_slice() )?;

  for ( i, view_proj ) in cube_face_view_proj().iter().enumerate()
  {
    let inv_view_proj = view_proj.inverse()
    .ok_or( gl::WebglError::Other( "a cube face view-projection is not invertible" ) )?;
    gl::uniform::matrix_upload( gl, inv_view_proj_loc.clone(), inv_view_proj.to_array().as_slice(), true )?;
    gl.framebuffer_texture_2d
    (
      GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0,
      GL::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32, Some( texture ), 0,
    );
    if gl.check_framebuffer_status( GL::FRAMEBUFFER ) != GL::FRAMEBUFFER_COMPLETE
    {
      return Err( gl::WebglError::Other( "the cube map bake framebuffer is incomplete" ) );
    }
    gl.draw_arrays( GL::TRIANGLES, 0, 3 );
  }
  Ok( () )
}

/// Unbinds and deletes everything the bake created except the texture:
/// nothing draws through the bake program, its VAO or its framebuffer
/// again. Unbinding first makes the deletes take effect now instead of
/// waiting for the objects to stop being current.
fn bake_teardown
(
  gl : &GL,
  program : &gl::WebGlProgram,
  vao : Option< &gl::WebGlVertexArrayObject >,
  framebuffer : Option< &gl::web_sys::WebGlFramebuffer >,
)
{
  gl.bind_framebuffer( GL::FRAMEBUFFER, None );
  gl.bind_vertex_array( None );
  gl.use_program( None );
  gl.delete_framebuffer( framebuffer );
  gl.delete_vertex_array( vao );
  // `compile_and_link` leaves its two shader objects attached to the program;
  // deleting the program alone would leave them allocated.
  if let Some( shaders ) = gl.get_attached_shaders( program )
  {
    for shader in shaders.iter().filter_map( | s | s.dyn_into::< gl::web_sys::WebGlShader >().ok() )
    {
      gl.delete_shader( Some( &shader ) );
    }
  }
  gl.delete_program( Some( program ) );
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
  /// Bakes the nebula cube map and builds the program that samples it.
  ///
  /// # Errors
  ///
  /// Whatever `bake_cubemap` reports, or a skybox program that fails to
  /// compile or link.
  pub fn new( gl : &GL ) -> Result< Self, gl::WebglError >
  {
    let cubemap = bake_cubemap( gl )?;

    // No attributes - `background.vert` draws its triangle purely off
    // `gl_VertexID`, but WebGL2 still requires *a* VAO bound to draw at all.
    let vao = gl::vao::create( gl )?;

    let vertex_shader = include_str!( "shaders/background.vert" );
    let fragment_shader = include_str!( "shaders/skybox.frag" );
    let program = gl::ProgramFromSources::new( vertex_shader, fragment_shader ).compile_and_link( gl )?;

    let uniforms = SkyboxUniforms
    {
      inv_view_proj : gl.get_uniform_location( &program, "u_inv_view_proj" ),
      camera_position : gl.get_uniform_location( &program, "u_camera_position" ),
      skybox : gl.get_uniform_location( &program, "u_skybox" ),
    };

    Ok( Self { vao, program, uniforms, cubemap } )
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

#[ cfg( test ) ]
mod tests
{
  use super::cube_face_view_proj;
  use minwebgl as gl;

  /// Where the GL cube-map lookup (OpenGL ES 3.0 §3.8.10, table 3.21) puts
  /// direction `d` on `face` (0..6 = +X, -X, +Y, -Y, +Z, -Z), as NDC on that
  /// face's render target: `( sc / |ma|, tc / |ma| )`. Face row 0 is the
  /// render target's bottom row, so `t` maps to NDC y without a flip.
  fn expected_ndc( face : usize, d : [ f32; 3 ] ) -> [ f32; 2 ]
  {
    let [ rx, ry, rz ] = d;
    let ( sc, tc, ma ) = match face
    {
      0 => ( -rz, -ry, rx ),
      1 => ( rz, -ry, -rx ),
      2 => ( rx, rz, ry ),
      3 => ( rx, -rz, -ry ),
      4 => ( rx, -ry, rz ),
      5 => ( -rx, -ry, -rz ),
      _ => unreachable!(),
    };
    [ sc / ma, tc / ma ]
  }

  #[ test ]
  fn each_slot_renders_the_face_the_sampler_reads()
  {
    let face_axes : [ [ f32; 3 ]; 6 ] =
    [
      [ 1.0, 0.0, 0.0 ], [ -1.0, 0.0, 0.0 ],
      [ 0.0, 1.0, 0.0 ], [ 0.0, -1.0, 0.0 ],
      [ 0.0, 0.0, 1.0 ], [ 0.0, 0.0, -1.0 ],
    ];
    // Off-centre by different amounts on the two minor axes, so a swapped
    // face, a mirrored axis or a transposed pair all land somewhere else.
    let offsets = [ [ 0.0, 0.0 ], [ 0.3, -0.2 ], [ -0.45, 0.1 ] ];

    for ( slot, view_proj ) in cube_face_view_proj().iter().enumerate()
    {
      let axis = face_axes[ slot ];
      let major = axis.iter().position( | c | *c != 0.0 ).unwrap();
      let minors : Vec< usize > = ( 0 .. 3 ).filter( | i | *i != major ).collect();
      for [ a, b ] in offsets
      {
        let mut d = axis;
        d[ minors[ 0 ] ] = a;
        d[ minors[ 1 ] ] = b;

        let clip = *view_proj * gl::math::F32x4::new( d[ 0 ] * 5.0, d[ 1 ] * 5.0, d[ 2 ] * 5.0, 1.0 );
        assert!( clip.w() > 0.0, "slot {slot}: direction {d:?} is behind the bake camera" );
        let ndc = [ clip.x() / clip.w(), clip.y() / clip.w() ];
        let want = expected_ndc( slot, d );
        assert!
        (
          ( ndc[ 0 ] - want[ 0 ] ).abs() < 1e-4 && ( ndc[ 1 ] - want[ 1 ] ).abs() < 1e-4,
          "slot {slot}: direction {d:?} lands at {ndc:?}, the sampler reads it at {want:?}"
        );
      }
    }
  }
}
