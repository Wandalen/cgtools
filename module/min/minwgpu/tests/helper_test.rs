//! Native tests for the `helper` layer's public surface (established by task 070): the pure
//! `attr` shortcut, whose output fields are directly observable, and the synchronous
//! `adapter_request` shortcut's error path, which is deterministic without a GPU (an
//! empty-backends instance can never provide an adapter).

use minwgpu::{ helper, Error };

/// `attr` must map its three arguments onto the corresponding `wgpu::VertexAttribute`
/// fields unchanged.
#[ test ]
fn attr_maps_arguments_onto_vertex_attribute_fields()
{
  let attribute = helper::attr( wgpu::VertexFormat::Float32x2, 8, 3 );
  assert_eq!( attribute.format, wgpu::VertexFormat::Float32x2 );
  assert_eq!( attribute.offset, 8 );
  assert_eq!( attribute.shader_location, 3 );
}

/// The synchronous `adapter_request` shortcut must surface an adapter-request failure as
/// the crate's own error type, not a panic.
#[ test ]
fn request_adapter_shortcut_errors_on_empty_backends()
{
  let descriptor = wgpu::InstanceDescriptor
  {
    backends : wgpu::Backends::empty(),
    ..wgpu::InstanceDescriptor::new_without_display_handle()
  };
  let instance = wgpu::Instance::new( descriptor );

  let result = helper::adapter::adapter_request( &instance, &wgpu::RequestAdapterOptions::default() );
  assert!
  (
    matches!( &result, Err( Error::RequestAdapterError( _ ) ) ),
    "expected Err( Error::RequestAdapterError ), got {result:?}"
  );
}
