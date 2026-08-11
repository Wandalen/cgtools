/// Internal namespace.
mod private
{
  use crate::{ web_sys, PipelineLayoutDescriptor };

  /// Creates a new GPU pipeline layout.
  #[ inline ]
  #[ must_use ]
  pub fn create
  ( 
    device : &web_sys::GpuDevice, 
    desc : &web_sys::GpuPipelineLayoutDescriptor 
  ) -> web_sys::GpuPipelineLayout
  {
    device.create_pipeline_layout( desc )
  }

  /// Creates a new pipeline layout descriptor builder.
  #[ inline ]
  #[ must_use ]
  pub fn desc< 'a >() -> PipelineLayoutDescriptor< 'a >
  {
    PipelineLayoutDescriptor::new()
  }
}

crate::mod_interface!
{
  own use
  {
    desc,
    create
  };
}
