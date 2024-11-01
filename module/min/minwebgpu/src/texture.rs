/// Internal namespace.
mod private
{
  use crate::*;

  pub fn create< T : Into< web_sys::GpuTextureDescriptor > >
  ( 
    device : &web_sys::GpuDevice, 
    descriptor : T 
  ) -> Result< web_sys::GpuTexture, WebGPUError >
  {
    let texture = device.create_texture( &descriptor.into() )
      .map_err( | e | DeviceError::FailedToCreateTexture( format!( "{:?}", e ) ) )?;

    Ok( texture )
  }
}

crate::mod_interface!
{
  own use
  {
    create
  };
}
