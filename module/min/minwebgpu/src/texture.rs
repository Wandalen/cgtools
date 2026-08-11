/// Internal namespace.
mod private
{
  use crate::{ TextureDescriptor, web_sys, WebGPUError, DeviceError, TextureError };

  /// Returns a new `TextureDescriptor` with default settings.
  #[ inline ]
  #[ must_use ]
  pub fn desc< 'a >() -> TextureDescriptor< 'a >
  {
    TextureDescriptor::new()
  }

  /// Creates a new `GpuTexture` on a WebGPU device.
  ///
  /// # Errors
  /// Returns `error::DeviceError::FailedToCreateTexture` if the underlying
  /// `GPUDevice.createTexture` call throws.
  #[ inline ]
  pub fn create
  ( 
    device : &web_sys::GpuDevice, 
    descriptor : &web_sys::GpuTextureDescriptor 
  ) -> Result< web_sys::GpuTexture, WebGPUError >
  {
    let texture = device.create_texture( descriptor )
    .map_err( | e | DeviceError::FailedToCreateTexture( format!( "{e:?}" ) ) )?;

    Ok( texture )
  }

  /// Creates a default `GpuTextureView` for a given texture.
  ///
  /// # Errors
  /// Returns `error::TextureError::FailedToCreateView` if the underlying
  /// `GPUTexture.createView` call throws.
  #[ inline ]
  pub fn view( texture : &web_sys::GpuTexture ) -> Result< web_sys::GpuTextureView, WebGPUError >
  {
    let view = texture.create_view()
    .map_err( | e | TextureError::FailedToCreateView( format!( "{e:?}" ) ) )?;

    Ok( view )
  }

  /// Creates a `GpuTextureView` with a specific descriptor.
  ///
  /// # Errors
  /// Returns `error::TextureError::FailedToCreateView` if the underlying
  /// `GPUTexture.createView` call throws.
  #[ inline ]
  pub fn view_with_descriptor
  ( 
    texture : &web_sys::GpuTexture,
    descriptor : &web_sys::GpuTextureViewDescriptor
   ) -> Result< web_sys::GpuTextureView, WebGPUError >
  {
    let view = texture.create_view_with_descriptor( descriptor )
    .map_err( | e | TextureError::FailedToCreateView( format!( "{e:?}" ) ) )?;

    Ok( view )
  }
}

crate::mod_interface!
{
  own use
  {
    create,
    desc,
    view,
    view_with_descriptor
  };
}
