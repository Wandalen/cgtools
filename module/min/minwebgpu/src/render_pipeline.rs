/// Internal namespace.
mod private
{
  use crate::*;

  pub fn desc< 'a, T >( vertex : T ) -> RenderPipelineDescriptor< 'a >
    where  T : Into< web_sys::GpuVertexState >
  {
    RenderPipelineDescriptor::new(vertex)
  }

  pub fn create
  ( 
    device : &web_sys::GpuDevice ,
    descriptor : &web_sys::GpuRenderPipelineDescriptor
  ) -> Result< web_sys::GpuRenderPipeline, WebGPUError >
  {
    let pipeline = device.create_render_pipeline( descriptor )
    .map_err( | e | DeviceError::FailedToCreateRenderPipeline( format!( "{:?}", e ) ))?;
    
    Ok( pipeline )
  }

  pub async fn create_async
  ( 
    device : &web_sys::GpuDevice,
    descriptor : &web_sys::GpuRenderPipelineDescriptor
  ) -> Result< web_sys::GpuRenderPipeline, WebGPUError >
  {
    let pipeline = JsFuture::from( device.create_render_pipeline_async( descriptor ) ).await
    .map_err( | e | DeviceError::FailedToCreateRenderPipeline( format!( "{:?}", e ) ))?;

    let pipeline = web_sys::GpuRenderPipeline::from( pipeline );
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
