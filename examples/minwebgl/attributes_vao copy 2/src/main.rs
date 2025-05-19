//! Just draw a large point in the middle of the screen.

use cube_texture::load_to_mip;
use minwebgl as gl;

mod cube_texture;

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let canvas = gl::canvas::retrieve_or_make()?;
  let gl = gl::context::from_canvas( &canvas )?;

  let vert = include_str!( "../shaders/big_triangle.vert" );
  let frag = include_str!( "../shaders/main.frag" );

  let program = gl::ProgramFromSources::new( &vert, &frag ).compile_and_link( &gl )?;

  let mut width = canvas. width();
  let mut height = canvas.height();

  let some_texture = gl.create_texture();

  load_to_mip( &gl, some_texture.as_ref(), 0, 512, 512, [ 255, 0, 0 ] );
  load_to_mip( &gl, some_texture.as_ref(), 1, 256, 256, [ 0, 255, 0 ] );
  load_to_mip( &gl, some_texture.as_ref(), 2, 128, 128, [ 0, 0, 255 ] );
  load_to_mip( &gl, some_texture.as_ref(), 3, 64, 64, [ 255, 0, 255 ] );
  load_to_mip( &gl, some_texture.as_ref(), 4, 32, 32, [ 255, 255, 255 ] );
  load_to_mip( &gl, some_texture.as_ref(), 5, 16, 16, [ 255, 255, 0 ] );
  load_to_mip( &gl, some_texture.as_ref(), 6, 8, 8, [ 128, 255, 0 ] );
  load_to_mip( &gl, some_texture.as_ref(), 7, 4, 4, [ 128, 255, 50 ] );
  load_to_mip( &gl, some_texture.as_ref(), 8, 2, 2, [ 77, 77, 50 ] );
  //load_to_mip( &gl, some_texture.as_ref(), 9, 1, 1, [ 255, 77, 255 ] );


  gl.use_program( Some( &program ) );

  gl.uniform1i( gl.get_uniform_location( &program, "some_texture" ).as_ref(), 0 );
  gl.bind_texture( gl::TEXTURE_2D, some_texture.as_ref() );
  //gl.generate_mipmap( gl::TEXTURE_2D );
  gl::texture::d2::wrap_clamp( &gl );
  gl::texture::d2::filter_linear( &gl );
  gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER,  gl::LINEAR_MIPMAP_NEAREST as i32 );
  let update_and_draw =
  {
    move | t : f64 |
    {
      let _time = t as f32 / 1000.0;


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
