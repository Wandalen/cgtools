/// Internal namespace.
mod private
{
  use crate::*;
  use wasm_bindgen_futures::JsFuture;

  /// Retrieves the `web_sys::Navigator` object from the current window.
  pub fn navigator() -> web_sys::Navigator
  {
    let window = web_sys::window().unwrap();
    let navigator = window.navigator();
    navigator
  }

  /// Asynchronously requests a WebGPU adapter from the browser.
  pub async fn request_adapter() -> web_sys::GpuAdapter
  {
    let navigator = navigator();
    let gpu = navigator.gpu();

    let adapter = JsFuture::from( gpu.request_adapter() ).await.unwrap();
    adapter.dyn_into().unwrap()
  }

  /// Asynchronously requests a logical GPU device from a given adapter.
  pub async fn request_device( adapter : &web_sys::GpuAdapter ) -> web_sys::GpuDevice
  {
    let device = JsFuture::from( adapter.request_device() ).await.unwrap();
    device.dyn_into().unwrap()
  }

  /// Retrieves the WebGPU context from an HTML canvas element.
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

  /// Configures the WebGPU canvas context for rendering.
  pub fn configure( device : &web_sys::GpuDevice, context : &GL, format : GpuTextureFormat ) -> Result< (), WebGPUError >
  {
    let configuration = web_sys::GpuCanvasConfiguration::new( device, format );

    context.configure( &configuration ).map_err( | e | error::CanvasError::ConfigurationError( format!( "{:?}", e ) ) )?;
    Ok( () )
  }

  /// Retrieves the preferred texture format for the current canvas.
  pub fn preferred_format() -> GpuTextureFormat
  {
    let navigator = navigator();
    let format = navigator.gpu().get_preferred_canvas_format();
    format
  }

  /// Gets the current texture from the WebGPU context.
  pub fn current_texture( context : &GL ) -> Result< web_sys::GpuTexture, WebGPUError >
  {
    let format = context.get_current_texture()
    .map_err( | e | error::ContextError::FailedToGetCurrentTextureError( format!( "{:?}", e ) ) )?;

    Ok( format )
  }

  /// Gets a default view of the context's current texture in one call.
  /// Equivalent to [`current_texture`] followed by [`texture::view`].
  pub fn current_view( context : &GL ) -> Result< web_sys::GpuTextureView, WebGPUError >
  {
    let texture = current_texture( context )?;
    texture::view( &texture )
  }

  /// The objects produced by the one-shot [`setup`] convenience.
  ///
  /// Every field is the plain native `web_sys` type you would have gotten by
  /// calling [`from_canvas`], [`request_adapter`], [`request_device`] and
  /// [`preferred_format`] yourself — this struct only aggregates their
  /// results, it does not wrap or hide them.
  pub struct GpuSetup
  {
    /// The configured WebGPU canvas context.
    pub context : GL,
    /// The adapter the device was requested from.
    pub adapter : web_sys::GpuAdapter,
    /// The logical GPU device.
    pub device : web_sys::GpuDevice,
    /// The device's default command queue.
    pub queue : web_sys::GpuQueue,
    /// The canvas's preferred texture format, already passed to [`configure`].
    pub format : GpuTextureFormat
  }

  /// Runs the common "get a WebGPU device and a configured canvas context"
  /// sequence in one call: [`from_canvas`], [`request_adapter`],
  /// [`request_device`], [`preferred_format`], then [`configure`].
  ///
  /// This is pure sequencing — nothing is defaulted beyond what those
  /// functions already do on their own. Call them individually instead when
  /// you need to inspect the adapter before requesting a device, request
  /// specific device features or limits, or otherwise need to intervene
  /// partway through.
  pub async fn setup( canvas : &web_sys::HtmlCanvasElement ) -> Result< GpuSetup, WebGPUError >
  {
    let context = from_canvas( canvas )?;
    let adapter = request_adapter().await;
    let device = request_device( &adapter ).await;
    let queue = device.queue();
    let format = preferred_format();
    configure( &device, &context, format )?;

    Ok( GpuSetup { context, adapter, device, queue, format } )
  }
}

crate::mod_interface!
{
  own use
  {
    request_adapter,
    request_device,
    from_canvas,
    navigator,
    preferred_format,
    configure,
    current_texture,
    current_view,
    setup
  };

  exposed use
  {
    GpuSetup
  };

}
