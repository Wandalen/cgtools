/// Internal namespace.
mod private
{
  use crate::{ SamplerDescriptor, Into, web_sys };

  /// Creates a new `SamplerDescriptor` with default settings.
  #[ inline ]
  #[ must_use ]
  pub fn desc< 'a >() -> SamplerDescriptor< 'a >
  {
    SamplerDescriptor::new()
  }

  /// Creates a `GpuSampler` from a descriptor.
  #[ inline ]
  pub fn create< T : Into< web_sys::GpuSamplerDescriptor > >
  ( 
    device : &web_sys::GpuDevice, 
    descriptor : T 
  ) -> web_sys::GpuSampler
  {
    device.create_sampler_with_descriptor( &descriptor.into() )
  }
}

crate::mod_interface!
{
  own use
  {
    create,
    desc
  };
}
