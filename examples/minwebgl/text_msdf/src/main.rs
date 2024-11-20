//! # Uniforms And Animation Example with UBOs
//!
//! This program demonstrates how to render a triangle in the middle of the screen using WebGL in Rust. It utilizes shaders with Uniform Block Objects (UBOs) to manage uniforms efficiently.

use minwebgl as gl;
use gl::{ GL };

mod text;

fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make()?;

  // Vertex and fragment shader source code
  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  let width = gl.drawing_buffer_width() as f32;
  let height = gl.drawing_buffer_height() as f32;

  let resolution_loc = gl.get_uniform_location( &program, "u_resolution" );
  let metallic_loc = gl.get_uniform_location( &program, "u_metallic" );
  let roughness_loc = gl.get_uniform_location( &program, "u_roughness" );
  let reflactance_loc = gl.get_uniform_location( &program, "u_reflectance" );
  let base_color_loc = gl.get_uniform_location( &program, "u_base_color" );
  let time_loc = gl.get_uniform_location( &program, "u_time" );

  gl.uniform1f( metallic_loc.as_ref(), 0.0 );
  gl.uniform1f( roughness_loc.as_ref(), 0.5 ); // 0.027 - minimum value;
  gl.uniform1f( reflactance_loc.as_ref(), 2.0 );
  // gl.uniform3f( base_color_loc.as_ref(), 0.562, 0.565, 0.578 ); // iron
  // gl.uniform3f( base_color_loc.as_ref(), 1.022, 0.782, 0.344 ); // gold
  gl.uniform3f( base_color_loc.as_ref(), 0.673, 0.637, 0.585 ); // platinum


  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      gl.uniform1f( time_loc.as_ref(), t as f32 );
      gl.uniform2f( resolution_loc.as_ref(), width, height );
      // Draw points
      gl.draw_arrays( GL::TRIANGLE_STRIP, 0, 4 );
      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );
  Ok( () )
}

fn main()
{
  run().unwrap()
}
