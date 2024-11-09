/// Internal namespace.
mod private
{
  use crate::*;

  pub fn create
  ( 
    device : &web_sys::GpuDevice, 
    desc : &web_sys::GpuPipelineLayoutDescriptor 
  ) -> web_sys::GpuPipelineLayout
  {
    device.create_pipeline_layout( desc )
  }

  pub fn desc< 'a >() -> PipelineLayoutDescriptor< 'a >
  {
    PipelineLayoutDescriptor::new()
  }
}

crate::mod_interface!
{
  own use
  {
    desc
  };
}
