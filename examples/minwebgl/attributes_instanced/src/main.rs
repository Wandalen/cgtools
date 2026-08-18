//! Draws 6 static triangles via `draw_arrays_instanced`, with 5 instances offset along Y
//! by a per-instance attribute (divisor 1) — demonstrates instanced rendering with
//! per-instance vertex attributes in WebGL2.

// Fix(BUG-ZZZ): module doc comment above was a stale copy-paste leftover — it used to
// claim this crate drew one large, screen-centered point, but this crate actually
// instanced-draws 6 triangles with per-instance Y offsets (5 instances), and its own
// readme.md already correctly describes it as an instanced-rendering demo.
// Root cause: doc comment never updated as the demo grew past an early single-point sketch.
// Pitfall: `attributes_vao`'s sibling `main.rs` carried the exact same stale sentence —
// check other `attributes_*`/early-stage demo crates for the same leftover before assuming
// this was a one-off.

use minwebgl as gl;
use gl::{ GL };

fn app_run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let gl = gl::context::retrieve_or_make()?;

  // Vertex and fragment shader source code
  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  let position_data :  [ f32 ; 36 ] =
  [
    // 12x3x2 position
     -0.6, -0.4, -0.6, -0.5, -0.65, -0.35, // Triangle 5
     -0.4,  0.3, -0.35, 0.4, -0.3,  0.25,  // Triangle 3
     -0.1, -0.1,  0.0,  0.2,  0.0, -0.15,  // Triangle 1
      0.1, -0.3,  0.15, -0.1, 0.05, -0.25, // Triangle 6
      0.3, -0.2,  0.25, 0.1,  0.2,  0.05,  // Triangle 2
      0.5,  0.5,  0.45, 0.6,  0.55, 0.6,   // Triangle 4
  ];

  // Vertex data
  let color_data : [ f32 ; 18 ] =
  [
    // color 2x6x3
    0.9849, 0.0600, 0.0662, 0.1232, 0.9332, 0.4260, 0.6969, 0.5353, 0.1471,
    0.2899, 0.9056, 0.7799, 0.2565, 0.6451, 0.8498, 0.0969, 0.9353, 0.0471,
  ];

  // Offsets
  let offset_data : [ f32 ; 24 ] =
  [
     0.0, -0.5,
     0.0, -0.4,
     0.0, -0.3,
     0.0, -0.2,
     0.0, -0.1,
     0.0,  0.0,
     0.0,  0.1,
     0.0,  0.2,
     0.0,  0.3,
     0.0,  0.4,
     0.0,  0.5,
     0.0,  0.6,
  ];

  // create buffer and upload vertex data

  let position_slot = 0;
  let position_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &position_buffer, &position_data, GL::STATIC_DRAW );

  let color_slot = 1;
  let color_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &color_buffer, &color_data, GL::STATIC_DRAW );

  let offset_slot = 2;
  let offset_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &offset_buffer, &offset_data, GL::STATIC_DRAW );

  // Create vao.
  // And set attributes.
  // A divisor of 0 indicates that each vertex has its own unique attribute value.
  // A divisor of 1 means that the entire primitive shares the same attribute value.
  // A divisor of 2 or more specifies that the attribute value is shared across multiple primitives.

  let vao = gl::vao::create( &gl )?;
  gl.bind_vertex_array( Some( &vao ) );
  gl::BufferDescriptor::new::< [ f32 ; 2 ] >().stride( 2 ).offset( 0 ).divisor( 0 )
  .attribute_pointer( &gl, position_slot, &position_buffer )?;
  gl::BufferDescriptor::new::< [ f32 ; 3 ] >().stride( 3 ).offset( 0 ).divisor( 2 )
  .attribute_pointer( &gl, color_slot, &color_buffer )?;
  gl::BufferDescriptor::new::< [ f32 ; 2 ] >().stride( 2 ).offset( 0 ).divisor( 1 )
  .attribute_pointer( &gl, offset_slot, &offset_buffer )?;
  gl.bind_vertex_array( None );

  // Bind VAO and draw

  gl.bind_vertex_array( Some( &vao ) );
  // gl.draw_arrays( GL::TRIANGLES, 0, 3*4 );
  gl.draw_arrays_instanced( GL::TRIANGLES, 0, 3*6, 5 );
  gl.bind_vertex_array( None );

  Ok(())
}

fn main()
{
  app_run().unwrap();
}
