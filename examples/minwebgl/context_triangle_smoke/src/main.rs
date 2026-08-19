//! Draws one solid-colored triangle through `minwebgl`'s `context::from_canvas` entry
//! point plus a minimal shader/buffer/draw sequence — the browser-side pixel-verified
//! smoke test for the crate's most foundational, currently browser-untested entry point.

use minwebgl as gl;
use gl::GL;

fn app_run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let canvas = gl::canvas::make()?;
  let context = gl::context::from_canvas( &canvas )?;

  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let program = gl::Program::new( context.clone(), vertex_shader_src, fragment_shader_src )?;
  program.activate();

  let vertex_data : [ f32 ; 6 ] = [ -0.5, -0.5, 0.5, -0.5, 0.0, 0.5 ];
  let vertex_buffer = gl::buffer::create( &context )?;
  gl::buffer::upload( &context, &vertex_buffer, &vertex_data, GL::STATIC_DRAW );

  let vao = gl::vao::create( &context )?;
  context.bind_vertex_array( Some( &vao ) );
  let position_attr = mingl::VertexAttribute::new( 0, mingl::VectorDataType::new( mingl::DataType::F32, 2, 1 ), 0 );
  gl::BufferDescriptor::from_vector( position_attr.vector ).stride( 2 ).offset( position_attr.offset ).attribute_pointer( &context, position_attr.location, &vertex_buffer )?;

  context.clear_color( 0.0, 0.0, 0.0, 1.0 );
  context.clear( GL::COLOR_BUFFER_BIT );
  context.draw_arrays( GL::TRIANGLES, 0, 3 );
  context.bind_vertex_array( None );

  Ok( () )
}

fn main()
{
  app_run().unwrap();
}
