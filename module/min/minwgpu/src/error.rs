//! Contains primary error type for all fallible operations within the crate.

use mingl::mod_interface;

mod private
{
  use thiserror::Error;

  /// The primary error type for all fallible operations within the crate.
  #[ derive( Debug, Error ) ]
  #[ non_exhaustive ]
  pub enum Error
  {
    /// Error resulting from usage of WGPU APIs.
    #[ error( "{0}" ) ]
    WGPUError( #[ from ] wgpu::Error ),
    /// Error when `Instance::request_adapter` fails.
    #[ error( "{0}" ) ]
    RequestAdapterError( #[ from ] wgpu::RequestAdapterError ),
    /// Error when `Adapter::request_device` fails.
    #[ error( "{0}" ) ]
    RequestDeviceError( #[ from ] wgpu::RequestDeviceError ),
  }
}

mod_interface!
{
  exposed use Error;
}
