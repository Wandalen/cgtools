/// Internal namespace.
mod private
{
  use crate::*;

  pub fn desc< 'a, T >( compute : T ) -> ComputePipelineDescriptor< 'a >
    where  T : Into< web_sys::GpuProgrammableStage >
  {
    ComputePipelineDescriptor::new( compute )
  }

  pub fn create
  ( 
    device : &web_sys::GpuDevice ,
    descriptor : &web_sys::GpuComputePipelineDescriptor
  ) -> web_sys::GpuComputePipeline
  {
    let pipeline = device.create_compute_pipeline( descriptor );
    pipeline
  }

  pub async fn create_async
  ( 
    device : &web_sys::GpuDevice,
    descriptor : &web_sys::GpuComputePipelineDescriptor
  ) -> Result< web_sys::GpuComputePipeline, WebGPUError >
  {
    let pipeline = JsFuture::from( device.create_compute_pipeline_async( descriptor ) ).await
    .map_err( | e | DeviceError::FailedToCreateRenderPipeline( format!( "{:?}", e ) ))?;

    let pipeline = web_sys::GpuComputePipeline::from( pipeline );
    Ok( pipeline )
  }
}

crate::mod_interface!
{
  own use
  {
    create,
    create_async,
    desc
  };
}
