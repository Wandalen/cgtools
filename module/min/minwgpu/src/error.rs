//! Contains primary error type for all fallible operations within the crate.

use mingl::mod_interface;

mod private
{
  use error_tools::dependency::thiserror;

  /// The primary error type for all fallible operations within the crate.
  #[ derive( Debug, thiserror::Error ) ]
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
    /// Error when polling the device for completed GPU work fails.
    #[ error( "{0}" ) ]
    PollError( #[ from ] wgpu::PollError ),
    /// Error when asynchronously mapping a buffer for host access fails.
    #[ error( "{0}" ) ]
    BufferAsyncError( #[ from ] wgpu::BufferAsyncError ),
    /// Error when accessing the mapped range of a buffer fails.
    #[ error( "{0}" ) ]
    MapRangeError( #[ from ] wgpu::MapRangeError ),
    /// Error when a texture format is not supported by the requested operation.
    #[ error( "texture format {0:?} is not supported : {1}" ) ]
    UnsupportedTextureFormat( wgpu::TextureFormat, &'static str ),
    /// Error when `surface_configure` is called with a zero-sized drawable area.
    #[ error( "surface_configure called with a zero-sized drawable area: {0}x{1}" ) ]
    ZeroSizeSurface( u32, u32 ),
  }
}

mod_interface!
{
  exposed use Error;
}
