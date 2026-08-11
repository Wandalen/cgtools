/// Internal namespace.
mod private
{
  use crate::{ Into, web_sys, RenderPipelineDescriptor, WebGPUError, DeviceError };
  use wasm_bindgen_futures::JsFuture;

  /// Creates a new `RenderPipelineDescriptor` with the specified vertex state.
  #[ inline ]
  pub fn desc< 'a, T >( vertex : T ) -> RenderPipelineDescriptor< 'a >
    where  T : Into< web_sys::GpuVertexState >
  {
    RenderPipelineDescriptor::new(vertex)
  }

  /// Creates a new `RenderPipeline` synchronously.
  ///
  /// # Errors
  /// Returns `error::DeviceError::FailedToCreateRenderPipeline` if the underlying
  /// `GPUDevice.createRenderPipeline` call throws.
  #[ inline ]
  pub fn create
  ( 
    device : &web_sys::GpuDevice ,
    descriptor : &web_sys::GpuRenderPipelineDescriptor
  ) -> Result< web_sys::GpuRenderPipeline, WebGPUError >
  {
    let pipeline = device.create_render_pipeline( descriptor )
    .map_err( | e | DeviceError::FailedToCreateRenderPipeline( format!( "{e:?}" ) ))?;
    
    Ok( pipeline )
  }

  /// Creates a new `RenderPipeline` asynchronously.
  ///
  /// # Errors
  /// Returns `error::DeviceError::FailedToCreateRenderPipeline` if the underlying
  /// `GPUDevice.createRenderPipelineAsync` call rejects.
  #[ inline ]
  pub async fn create_async
  ( 
    device : &web_sys::GpuDevice,
    descriptor : &web_sys::GpuRenderPipelineDescriptor
  ) -> Result< web_sys::GpuRenderPipeline, WebGPUError >
  {
    let pipeline = JsFuture::from( device.create_render_pipeline_async( descriptor ) ).await
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
