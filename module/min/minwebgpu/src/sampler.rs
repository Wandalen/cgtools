/// Internal namespace.
mod private
{
  use crate::*;

  pub fn create< T : Into< web_sys::GpuSamplerDescriptor > >
  ( 
    device : &web_sys::GpuDevice, 
    descriptor : T 
  ) -> web_sys::GpuSampler
  {
    let sampler = device.create_sampler_with_descriptor( &descriptor.into() );
    sampler
  }
}

crate::mod_interface!
{
  own use
  {
    create
  };
}
