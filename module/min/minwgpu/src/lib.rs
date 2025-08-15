#![ allow( clippy::implicit_return ) ]

//!
//! # `minwgpu`
//!
//! Minwgpu is a minimal, opinionated toolkit designed to simplify common
//! `wgpu` patterns. It provides convenient builders and helpers to reduce
//! boilerplate when setting up a `wgpu` context, managing buffers, and more,
//! making it easier to get a graphics application up and running.
//!

use mingl::mod_interface;

mod private
{
  /// The primary error type for all fallible operations within the crate.
  #[ derive( Debug, error_tools::Error ) ]
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
  layer helper;
  layer buffer;
  layer context;
  own use Error;
}
