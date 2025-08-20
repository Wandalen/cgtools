//! Device helper functions

use mingl::mod_interface;

mod private
{
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
}

mod_interface!
{
  own use request_device;
}
