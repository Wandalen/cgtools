//! This module contains helper shortcuts.

use mingl::mod_interface;

mod private
{
  /// Requests adapter synchronously.
  ///
  /// # Errors
  ///
  /// Return error in case of `Instance::request_adapter` returns error.
  #[ inline ]
  pub fn request_adapter( instance : &wgpu::Instance, options : &wgpu::RequestAdapterOptions< '_, '_ > )
  -> Result< wgpu::Adapter, crate::Error >
  {
    Ok( pollster::block_on( instance.request_adapter( options ) )? )
  }

  /// Requests device synchronously.
  ///
  /// # Errors
  ///
  /// Return error in case of `Adapter::request_device` returns error.
  #[ inline ]
  pub fn request_device( adapter : &wgpu::Adapter, device_descriptor : &wgpu::DeviceDescriptor< '_ > )
  -> Result< ( wgpu::Device, wgpu::Queue ), crate::Error >
  {
    Ok( pollster::block_on( adapter.request_device( device_descriptor ) )? )
  }

  /// A shortcut for creating a `wgpu::VertexAttribute`.
  #[ inline ]
  #[ must_use ]
  pub const fn attr
  (
    format : wgpu::VertexFormat,
    offset : wgpu::BufferAddress,
    shader_location : wgpu::ShaderLocation
  ) -> wgpu::VertexAttribute
  {
    wgpu::VertexAttribute
    {
      format,
      offset,
      shader_location,
    }
  }
}

mod_interface!
{
  own use request_adapter;
  own use request_device;
  own use attr;
}
