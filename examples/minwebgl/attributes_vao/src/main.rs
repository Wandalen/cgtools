//! Just draw a large point in the middle of the screen.

use minwebgl as gl;
use gl::{ GL };

fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make()?;

  // Vertex and fragment shader source code
  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  // Vertex data
  let vert_data : [ f32 ; 30 ] =
  [
     // position    point size    color
    -0.6618, -0.7687, 200.0, 1.5849, 0.0600, 0.0662,
    -0.3149, 0.7417, 40.0, 0.9232, 0.9332, 0.4260,
     0.9749, -0.8996, 160.0, 0.6969, 0.5353, 0.1471,
    -0.9202, -0.2956, 360.0, 0.2899, 0.9056, 0.7799,
     0.4550, -0.0642, 80.0, 0.2565, 0.6451, 0.8498,
  ];

  // Vertex data
  let vert_data2 : [ f32 ; 30 ] =
  [
     // position    point size    color
     0.6192, 0.5755, 280.0, 0.6133, 0.8137, 0.4046,
    -0.5946, 0.7057, 80.0, 0.6745, 0.5229, 0.4518,
     0.6365, 0.7236, 280.0, 0.4690, 0.0542, 0.7396,
     0.8625, -0.0835, 80.0, 0.3708, 0.6588, 0.8611,
     0.7997, 0.4695, 280.0, 0.7490, 0.3797, 0.6879,
  ];

  // create buffer and upload vertex data

  let vert_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &vert_buffer, &vert_data, GL::STATIC_DRAW );

  let vert_buffer2 = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &vert_buffer2, &vert_data2, GL::STATIC_DRAW );

  // create vao

  let vao = gl::vao::create( &gl )?;
  gl.bind_vertex_array( Some( &vao ) );
  gl::BufferDescriptor::new::< [ f32 ; 2 ] >().stride( 6 ).offset( 0 ).attribute_pointer( &gl, 0, &vert_buffer2 )?;
  gl::BufferDescriptor::new::< [ f32 ; 1 ] >().stride( 6 ).offset( 2 ).attribute_pointer( &gl, 1, &vert_buffer2 )?;
  gl::BufferDescriptor::new::< [ f32 ; 3 ] >().stride( 6 ).offset( 3 ).attribute_pointer( &gl, 2, &vert_buffer )?;
  gl.bind_vertex_array( None );

  // Bind VAO and draw

  gl.bind_vertex_array( Some( &vao ) );
  gl.draw_arrays( GL::POINTS, 0, 5 );
  gl.bind_vertex_array( None );

  Ok(())
}

fn main()
{
  run().unwrap()
}
