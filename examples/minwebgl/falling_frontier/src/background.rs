//! Full-viewport nebula backdrop - stands in for `scene/world.js`'s flat
//! `scene.background = new THREE.Color(COLORS.spaceBg)`, matching the
//! soft drifting cloud-blob look of the reference game screenshot the
//! tactical UI itself is modeled on rather than the JS port's flat
//! placeholder. Drawn first each frame with depth test/write both off, so
//! it always sits behind every other draw call regardless of camera orbit.

use minwebgl as gl;
use gl::GL;

struct BackgroundUniforms
{
  inv_view_proj : Option< gl::WebGlUniformLocation >,
  camera_position : Option< gl::WebGlUniformLocation >,
  time : Option< gl::WebGlUniformLocation >,
}

pub struct Background
{
  vao : gl::WebGlVertexArrayObject,
  program : gl::WebGlProgram,
  uniforms : BackgroundUniforms,
}

impl Background
{
  pub fn new( gl : &GL ) -> Self
  {
    // No attributes - `background.vert` draws its triangle purely off
    // `gl_VertexID`, but WebGL2 still requires *a* VAO bound to draw at all.
    let vao = gl::vao::create( gl ).unwrap();

    let vertex_shader = include_str!( "shaders/background.vert" );
    let fragment_shader = include_str!( "shaders/background.frag" );
    let program = gl::ProgramFromSources::new( vertex_shader, fragment_shader )
    .compile_and_link( gl )
    .unwrap();

    let uniforms = BackgroundUniforms
    {
      inv_view_proj : gl.get_uniform_location( &program, "u_inv_view_proj" ),
      camera_position : gl.get_uniform_location( &program, "u_camera_position" ),
      time : gl.get_uniform_location( &program, "u_time" ),
    };

    Self { vao, program, uniforms }
  }

  pub fn draw( &self, gl : &GL, view_proj : gl::F32x4x4, camera_position : gl::F32x3, time : f32 )
  {
    // A non-invertible view_proj can't happen with this scene's fixed
    // perspective projection, but skip the draw rather than upload garbage
    // if it ever did.
    let Some( inv_view_proj ) = view_proj.inverse() else { return };

    gl.use_program( Some( &self.program ) );
    let u = &self.uniforms;
    gl::uniform::matrix_upload( gl, u.inv_view_proj.clone(), inv_view_proj.to_array().as_slice(), true ).unwrap();
    gl::uniform::upload( gl, u.camera_position.clone(), camera_position.to_array().as_slice() ).unwrap();
    gl::uniform::upload( gl, u.time.clone(), &time ).unwrap();

    gl.disable( GL::DEPTH_TEST );
    gl.depth_mask( false );

    gl.bind_vertex_array( Some( &self.vao ) );
    gl.draw_arrays( GL::TRIANGLES, 0, 3 );

    gl.depth_mask( true );
    gl.enable( GL::DEPTH_TEST );
  }
}
