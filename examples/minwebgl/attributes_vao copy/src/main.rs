//! Just draw a large point in the middle of the screen.

use cube_texture2::CubeTextureRenderer2;
use minwebgl as gl;

mod cube_texture;
mod cube_texture2;

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let canvas = gl::canvas::retrieve_or_make()?;
  let gl = gl::context::from_canvas( &canvas )?;

  let lut_texture = gl.create_texture();
  cube_texture::load_to_mip_d2( &gl, lut_texture.as_ref(), 0, "specular_2.hdr" ).await;
  let cr = CubeTextureRenderer2::new( &gl ).await.unwrap();

  let vert = include_str!( "../shaders/big_triangle.vert" );
  let frag = include_str!( "../shaders/main.frag" );

  let program = gl::ProgramFromSources::new( &vert, &frag ).compile_and_link( &gl )?;

  let width = canvas. width() as f32;
  let height = canvas.height() as f32;

  let projectionMatrix = gl::math::mat3x3h::perspective_rh_gl( 60.0f32.to_radians(), width / height, 0.1, 1000.0 );
  let viewMatrix= gl::math::mat3x3h::look_to_rh( gl::F32x3::ZERO, gl::F32x3::X, gl::F32x3::Y );


  gl.use_program( Some( &program ) );
  gl::uniform::matrix_upload
  (
    &gl, 
    gl.get_uniform_location( &program, "invProjectionMatrix"), 
    projectionMatrix.inverse().unwrap().raw_slice(),
    true
  )?;

  gl::uniform::matrix_upload
  (
    &gl, 
    gl.get_uniform_location( &program, "viewMatrix"), 
    viewMatrix.raw_slice(),
    true
  )?;

  gl.uniform1i( gl.get_uniform_location( &program, "env_map" ).as_ref(), 0 );
  gl.uniform1i( gl.get_uniform_location( &program, "lut" ).as_ref(), 1 );

  cr.bind_texture( &gl );
  gl.active_texture( gl::TEXTURE1 );
  gl.bind_texture( gl::TEXTURE_2D, lut_texture.as_ref() );
  let update_and_draw =
  {
    move | t : f64 |
    {
      let _time = t as f32 / 1000.0;

      let dir = gl::F32x3::X;// + gl::F32x3::Y;
      let dir = gl::math::mat3x3::from_angle_y( _time / 2.0 ) * dir;
      let viewMatrix= gl::math::mat3x3h::look_to_rh( gl::F32x3::ZERO, dir, gl::F32x3::Y );
      gl::uniform::matrix_upload
      (
        &gl, 
        gl.get_uniform_location( &program, "viewMatrix"), 
        viewMatrix.raw_slice(),
        true
      ).unwrap();

      
      gl.draw_arrays( gl::TRIANGLES, 0, 3 );
      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );

  Ok(())
}

fn main()
{
  gl::spawn_local( async move { run().await.unwrap() } );
}
