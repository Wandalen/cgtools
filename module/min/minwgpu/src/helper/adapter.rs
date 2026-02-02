//! Adapter helper functions

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
}

mod_interface!
{
  own use request_adapter;
}
