//! Just draw a large point in the middle of the screen.

use minwebgl as gl;
use gl::{ GL };

fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make()?;

  // Vertex and fragment shaders
  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  // Bind VAO and draw
  gl.draw_arrays( GL::POINTS, 0, 1 );
  gl.bind_vertex_array( None );

  Ok(())
}

fn main()
{
  run().unwrap()
}
