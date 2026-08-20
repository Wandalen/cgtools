//! Tests for the buffer builders' state accumulation — `BufferBuilder` and
//! `VertexBufferBuilder` setters pinned through the `*_get` getters — runnable without
//! a GPU ( only `build` needs a live `wgpu::Device` ).
//!
//! Relocated from `src/buffer.rs` ( formerly a documented task-070 exception that read
//! `pub( super )` fields directly ); the getters now make the same state publicly
//! observable, per the all-tests-in-tests/ convention.

use minwgpu::buffer::{ buffer, vertex_buffer };

#[ test ]
fn buffer_builder_sets_label()
{
  let builder = buffer( wgpu::BufferUsages::empty() ).label( "test_label" );
  assert_eq!( builder.label_get(), Some( "test_label" ) );
}

#[ test ]
fn buffer_builder_sets_data()
{
  let test_data : &[ f32 ] = &[ 1.0, 2.0, 3.0 ];
  let builder = buffer( wgpu::BufferUsages::empty() ).data( test_data );
  assert_eq!( builder.data_get(), Some( asbytes::cast_slice( test_data ) ) );
}

#[ test ]
fn buffer_builder_sets_size_from_type()
{
  struct MyType { _a : f32, _b : u64 }
  let builder = buffer( wgpu::BufferUsages::empty() ).size::< MyType >();
  assert_eq!( builder.size_get(), core::mem::size_of::< MyType >() as u64 );
}

#[ test ]
fn buffer_builder_sets_size_from_var()
{
  let my_var = [ 0u32; 10 ];
  let builder = buffer( wgpu::BufferUsages::empty() ).size_from_var( &my_var );
  assert_eq!( builder.size_get(), core::mem::size_of_val( &my_var ) as u64 );
}

#[ test ]
fn buffer_builder_sets_size_from_value()
{
  let builder = buffer( wgpu::BufferUsages::empty() ).size_from_value( 128 );
  assert_eq!( builder.size_get(), 128 );
}

#[ test ]
fn buffer_builder_sets_mapped_at_creation()
{
  let builder = buffer( wgpu::BufferUsages::empty() ).mapped_at_creation( true );
  assert!( builder.mapped_at_creation_get() );
}

#[ test ]
fn buffer_builder_sets_usage()
{
  let usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
  let builder = buffer( wgpu::BufferUsages::empty() ).usage( usage );
  assert_eq!( builder.usage_get(), usage );
}

#[ test ]
fn vertex_buffer_builder_defaults()
{
  let builder = vertex_buffer();
  assert_eq!( builder.usage_get(), wgpu::BufferUsages::VERTEX );
  assert_eq!( builder.step_mode_get(), wgpu::VertexStepMode::Vertex );
  assert_eq!( builder.array_stride_get(), 0 );
  assert!( builder.attributes_get().is_empty() );
}

#[ test ]
fn vertex_buffer_builder_sets_array_stride()
{
  let builder = vertex_buffer().array_stride( 32 );
  assert_eq!( builder.array_stride_get(), 32 );
}

#[ test ]
fn vertex_buffer_builder_sets_step_mode()
{
  let builder = vertex_buffer().step_mode( wgpu::VertexStepMode::Instance );
  assert_eq!( builder.step_mode_get(), wgpu::VertexStepMode::Instance );
}

#[ test ]
fn vertex_buffer_builder_sets_attributes()
{
  let attrs = &[ wgpu::VertexAttribute { format : wgpu::VertexFormat::Float32x2, offset : 0, shader_location : 0 } ];
  let builder = vertex_buffer().attributes( attrs );
  assert_eq!( builder.attributes_get(), attrs );
}

#[ test ]
fn vertex_buffer_builder_chains_buffer_methods()
{
  let test_data : &[ i32 ] = &[ 5, 10, 15 ];
  let builder = vertex_buffer()
  .label( "vertex_test" )
  .data( test_data )
  .vertex_usage( wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST );

  assert_eq!( builder.label_get(), Some( "vertex_test" ) );
  assert_eq!( builder.data_get(), Some( asbytes::cast_slice( test_data ) ) );
  assert_eq!( builder.usage_get(), wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST );
}
