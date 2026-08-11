/// Internal namespace.
mod private
{
  use crate::{ Into, web_sys, ComputePipelineDescriptor, WebGPUError, DeviceError };
  use wasm_bindgen_futures::JsFuture;


  /// Creates a `GpuComputePipelineDescriptor` with the specified compute stage.
  #[ inline ]
  pub fn desc< 'a, T >( compute : T ) -> ComputePipelineDescriptor< 'a >
    where  T : Into< web_sys::GpuProgrammableStage >
  {
    ComputePipelineDescriptor::new( compute )
  }

  /// Creates a GPU compute pipeline synchronously.
  #[ inline ]
  #[ must_use ]
  pub fn create
  ( 
    device : &web_sys::GpuDevice ,
    descriptor : &web_sys::GpuComputePipelineDescriptor
  ) -> web_sys::GpuComputePipeline
  {
    device.create_compute_pipeline( descriptor )
  }

  /// Creates a GPU compute pipeline asynchronously.
  #[ inline ]
  pub async fn create_async
  ( 
    device : &web_sys::GpuDevice,
    descriptor : &web_sys::GpuComputePipelineDescriptor
  ) -> Result< web_sys::GpuComputePipeline, WebGPUError >
  {
    let pipeline = JsFuture::from( device.create_compute_pipeline_async( descriptor ) ).await
    .map_err( | e | DeviceError::FailedToCreateRenderPipeline( format!( "{e:?}" ) ))?;

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
