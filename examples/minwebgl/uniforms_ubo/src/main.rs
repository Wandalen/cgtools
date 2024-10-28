//! # Uniforms And Animation Example with UBOs
//!
//! This program demonstrates how to render a triangle in the middle of the screen using WebGL in Rust. It utilizes shaders with Uniform Block Objects (UBOs) to manage uniforms efficiently.

use minwebgl as gl;
use gl::{ GL, WebGl2RenderingContext, DebugLog, AsBytes };
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

  // Prepare to change color every frame
  let color = Rc::new( RefCell::new( vec![ 0.5, 0.0, 0.0, 1.0, 0.75 ] ) );

  // Prepare to change transformation every frame
  // std140 alignment require to allocate 4 words for the first row and 4 for the second row.
  let trans = vec!
  [
    1.0_f32, 0.0_f32, 0.0_f32, 0.0_f32,
    0.0_f32, 1.0_f32, 0.0_f32, 0.0_f32,
  ];
  let trans = Rc::new( RefCell::new( trans ) );

  // Create and bind uniform buffers. Bind uniform blocks to binding points.

  // Color
  let color_buffer = gl::buffer::create( &gl )?;
  let color_block_index = gl.get_uniform_block_index( &program, "ColorBlock" );
  let color_block_point = 0;
  gl.uniform_block_binding( &program, color_block_index, color_block_point );
  gl.buffer_data_with_i32( WebGl2RenderingContext::UNIFORM_BUFFER, color.borrow().byte_size() as _, GL::DYNAMIC_DRAW );
  // qqq : does it give any benefit?

  // Transformation
  let trans_buffer = gl::buffer::create( &gl )?;
  let trans_block_index : u32 = gl.get_uniform_block_index( &program, "TransformBlock" );
  let trans_block_point = 1;
  gl.uniform_block_binding( &program, trans_block_index, trans_block_point );
  gl.buffer_data_with_i32( GL::UNIFORM_BUFFER, trans.borrow().byte_size() as _, GL::DYNAMIC_DRAW );
  // qqq : does it give any benefit?

  // Retrieve UBO information for diagnostic purposes only; these lines should be removed in production builds.
  gl::ubo::diagnostic_info( &gl, &program, color_block_index ).debug_info();
  gl::ubo::diagnostic_info( &gl, &program, trans_block_index ).debug_info();

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
      color[ 0 ] = t2 % 1.0;
      color[ 1 ] = t2 % 1.0;
      gl::ubo::upload( &gl, &color_buffer, color_block_point, &color[..], GL::DYNAMIC_DRAW );

      // Update transformation
      let mut trans = trans.borrow_mut();
      let t2 = t / 1000.0;
      let ct = t2.cos();
      let st = t2.sin();
      // std140 alignment require to allocate 4 words for the first row and 4 for the second row.
      trans[ 0 ] = ct as _;
      trans[ 1 ] = st as _;
      trans[ 4 ] = -st as _;
      trans[ 5 ] = ct as _;
      gl::ubo::upload( &gl, &trans_buffer, trans_block_point, &trans[..], GL::DYNAMIC_DRAW );

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
