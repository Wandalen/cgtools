/// Internal namespace.
mod private
{
  use crate::*;

  pub fn navigator() -> web_sys::Navigator
  {
    let window = web_sys::window().unwrap();
    let navigator = window.navigator();
    navigator
  }
  pub async fn request_adapter() -> web_sys::GpuAdapter
  {
    let navigator = navigator();
    let gpu = navigator.gpu();

    let adapter = JsFuture::from( gpu.request_adapter() ).await.unwrap();
    adapter.dyn_into().unwrap()
  }

  pub async fn request_device( adapter : &web_sys::GpuAdapter ) -> web_sys::GpuDevice
  {
    let device = JsFuture::from( adapter.request_device() ).await.unwrap();
    device.dyn_into().unwrap()
  }

  pub fn from_canvas( canvas : &web_sys::HtmlCanvasElement ) -> Result< GL, dom::Error >
  {
    let context = canvas
    .get_context( "webgpu" )
    .map_err( |_| dom::Error::ContextRetrievingError( "Failed to get webgpu context" ) )?
    .ok_or( dom::Error::ContextRetrievingError( "No webgpu context" ) )?;

    let gl : GL = context
    .dyn_into()
    .map_err( |_| dom::Error::ContextRetrievingError( "Failed to cast to GL" ) )?;

    Ok( gl ) 
  }

  pub fn configure( device : &web_sys::GpuDevice, context : &GL ) -> Result< (), error::CanvasError >
  {
    let navigator = navigator();
    let format = navigator.gpu().get_preferred_canvas_format();

    let configuration = web_sys::GpuCanvasConfiguration::new( device, format );

    context.configure( &configuration ).map_err( | e | error::CanvasError::ConfigurationError( format!( "{:?}", e ) ) )?;
    Ok( () )
  }
}

crate::mod_interface!
{
    orphan use 
    {
        request_adapter,
        request_device
    };

}
