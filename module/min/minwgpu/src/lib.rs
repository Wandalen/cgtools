//! Minwgpu - A minimal WGPU toolkit

use mingl::mod_interface;

mod private
{
  /// Represents errors resulting from usage of Minwgpu APIs.
  #[ derive( Debug, error_tools::Error ) ]
  pub enum Error
  {
    /// Error resulting from usage of WGPU APIs.
    #[ error( "{0}" ) ]
    WGPUError( #[ from ] wgpu::Error ),
    /// Error when [`Instance::request_adapter()`] fails.
    #[ error( "{0}" ) ]
    RequestAdapterError( #[ from ] wgpu::RequestAdapterError ),
    /// Requesting a device from an [`Adapter`] failed.
    #[ error( "{0}" ) ]
    RequestDeviceError( #[ from ] wgpu::RequestDeviceError ),
  }
}

mod_interface!
{
  layer helper;
  own use Error;
}
