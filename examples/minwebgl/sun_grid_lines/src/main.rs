//! Procedural sci-fi HUD diagram: animated star, orbit ring, and a Cartesian
//! grid, rendered entirely by a single fullscreen fragment shader.

use minwebgl as gl;
use gl::GL;

fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );

  let canvas = gl::canvas::make()?;
  canvas.set_width( 800 );
  canvas.set_height( 800 ); // square canvas, matching the reference composition

  let gl = gl::context::from_canvas( &canvas )?;

  let vertex_shader_src = include_str!( "../shaders/scene.vert" );
  let fragment_shader_src = include_str!( "../shaders/scene.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  let u_time_loc = gl.get_uniform_location( &program, "u_time" );

  let update_and_draw = move | t : f64 |
  {
    let time = ( t / 1000.0 ) as f32;
    gl::uniform::upload( &gl, u_time_loc.clone(), &time ).unwrap();
    gl.draw_arrays( GL::TRIANGLES, 0, 3 );
    true
  };

  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

fn main()
{
  run().unwrap();
}
