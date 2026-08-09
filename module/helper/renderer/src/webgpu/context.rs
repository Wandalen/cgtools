mod private
{
  use minwebgpu as gl;
  use gl::web_sys;
  use gpu_hal::{ Device, Queue, Surface, Error };

  /// Owned `gpu_hal` handles of a configured GPU context.
  pub struct GpuContext
  {
    /// Canvas the context presents to.
    pub canvas : web_sys::HtmlCanvasElement,
    /// Logical device.
    pub device : Device,
    /// Default queue of the device.
    pub queue : Queue,
    /// Canvas presentation surface.
    pub surface : Surface
  }

  impl GpuContext
  {
    /// Requests an adapter and a device, then configures `canvas` for
    /// presentation in the browser's preferred canvas format.
    pub async fn new_webgpu( canvas : &web_sys::HtmlCanvasElement ) -> Result< Self, Error >
    {
      let ( device, queue, surface ) = Device::new_webgpu( canvas ).await?;

      Ok
      (
        Self
        {
          canvas : canvas.clone(),
          device,
          queue,
          surface
        }
      )
    }

    /// Wraps `canvas`' WebGL2 context in the same handle set. Requires the
    /// `EXT_color_buffer_float` extension ( float render targets ).
    ///
    /// Projections fed to the renderer must match `device.depth_range()` —
    /// -1..1 here, unlike the WebGPU backend's 0..1.
    pub fn new_webgl( canvas : &web_sys::HtmlCanvasElement ) -> Result< Self, Error >
    {
      let ( device, queue, surface ) = Device::new_webgl( canvas )?;

      Ok
      (
        Self
        {
          canvas : canvas.clone(),
          device,
          queue,
          surface
        }
      )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    GpuContext
  };
}
