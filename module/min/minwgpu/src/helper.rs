//! This module contains helper shortcuts

use mingl::mod_interface;

mod private
{
  /// Requests adapter synchronously.
  pub fn request_adapter( instance : &wgpu::Instance, options : &wgpu::RequestAdapterOptions< '_, '_ > )
  -> Result< wgpu::Adapter, crate::Error >
  {
    Ok( pollster::block_on( instance.request_adapter( options ) )? )
  }

  /// Requests device synchronously.
  pub fn request_device( adapter : &wgpu::Adapter, device_descriptor : &wgpu::DeviceDescriptor< '_ > )
  -> Result< ( wgpu::Device, wgpu::Queue ), crate::Error >
  {
    Ok( pollster::block_on( adapter.request_device( device_descriptor ) )? )
  }
}

mod_interface!
{
  own use request_adapter;
  own use request_device;
}
