//! # Uniforms And Animation Example
//!
//! This program demonstrates how to render a triangle in the middle of the screen using WebGL in Rust. It utilizes shaders to define the triangle's vertices and colors, and employs a rendering loop to dynamically update the triangle's color and transformation over time.
//!
//! ## Key Features
//!
//! - **Shader-Based Rendering**: Uses vertex and fragment shaders to define the triangle's appearance and behavior.
//! - **Dynamic Color Update**: The triangle's color changes continuously over time, creating a dynamic visual effect.
//! - **Transformation Animation**: The triangle undergoes a continuous rotation transformation, demonstrating the use of transformation matrices in WebGL.
//! - **Uniform Management**: Utilizes WebGL uniforms to pass dynamic data (color and transformation) to the shaders.
//!
//! ## Implementation Details
//!
//! - **Vertex and Fragment Shaders**: The shaders are loaded from external files and compiled into a shader program.
//! - **Uniform Variables**: `u_color` and `u_trans` are used to pass color and transformation data to the shaders.
//! - **Rendering Loop**: A continuous rendering loop updates the color and transformation matrices, and redraws the triangle each frame.
//!
//! ## Usage
//!
//! The program is designed to run in a web environment with WebGL support. It sets up the WebGL context, compiles the shaders, and enters a rendering loop that updates and draws the triangle continuously.

use minwebgl as gl;
use gl::GL;
use std::
{
  cell::RefCell,
  rc::Rc,
};

fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make()?;

  // Vertex and fragment shader source code
  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  let u_blue_offset_loc = gl.get_uniform_location( &program, "u_blue_offset" );
  gl::uniform::upload( &gl, u_blue_offset_loc, &0.75 ).unwrap();

  // Prepare to change color every frame
  let u_color_loc = gl.get_uniform_location( &program, "u_color" );
  let color = Rc::new( RefCell::new( vec!
  [
    0.5, 0.0, 0.0, 1.0,
  ]));

  // Prepare to change transformation every frame
  let u_trans_loc = gl.get_uniform_location( &program, "u_trans" );
  let trans = Rc::new( RefCell::new( vec!
  [
    1.0_f32, 0.0_f32,
    0.0_f32, 1.0_f32,
  ]));

  // Define the update and draw logic
  let update_and_draw =
  {
    let gl = gl.clone();
    let color = color.clone();
    let trans = trans.clone();
    move | t : f64 |
    {
      // Update color
      let mut color = color.borrow_mut();
      let t2 : f32 = ( t as f32 ) / 10000.0;
      color[ 0 ] = ( t2 ) % 1.0;
      color[ 1 ] = ( t2 ) % 1.0;
      gl::uniform::upload( &gl, u_color_loc.clone(), &color[ .. ] ).unwrap();

      // Update transformaption
      let mut trans = trans.borrow_mut();
      let t2 = t / 1000.0;
      let ct = t2.cos();
      let st = t2.sin();
      trans[ 0 ] = ct as _;
      trans[ 1 ] = st as _;
      trans[ 2 ] = -st as _;
      trans[ 3 ] = ct as _;
      gl::uniform::matrix_upload( &gl, u_trans_loc.clone(), &trans[ .. ], true ).unwrap();

      // gl::log::info!( "{t:?} {trans:?}" );

      // Draw points
      gl.draw_arrays( GL::TRIANGLES, 0, 3 );
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
