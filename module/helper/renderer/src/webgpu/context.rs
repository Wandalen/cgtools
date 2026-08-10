mod private
{
  // Inside this layer, wasm32 implies the `webgpu` feature and non-wasm32
  // implies `native` ( see the layer gate in `lib.rs` ), so plain target
  // gates suffice here.
  #[ cfg( target_arch = "wasm32" ) ]
  use minwebgpu as gl;
  #[ cfg( target_arch = "wasm32" ) ]
  use gl::web_sys;
  use gpu_hal::{ Device, Queue, Surface, Error };

  /// Owned `gpu_hal` handles of a configured GPU context.
  pub struct GpuContext
  {
    /// Canvas the context presents to.
    #[ cfg( target_arch = "wasm32" ) ]
    pub canvas : web_sys::HtmlCanvasElement,
    /// Pixel size of the offscreen surface.
    #[ cfg( not( target_arch = "wasm32" ) ) ]
    surface_size : [ u32; 2 ],
    /// Logical device.
    pub device : Device,
    /// Default queue of the device.
    pub queue : Queue,
    /// Presentation surface ( canvas in the browser, offscreen texture
    /// natively ).
    pub surface : Surface
  }

  impl GpuContext
  {
    /// Requests an adapter and a device, then configures `canvas` for
    /// presentation in the browser's preferred canvas format.
    #[ cfg( target_arch = "wasm32" ) ]
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
    #[ cfg( target_arch = "wasm32" ) ]
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

    /// Builds a native context over the machine's Vulkan driver, rendering
    /// into an offscreen `width` x `height` surface whose pixels
    /// `Surface::read_pixels` returns — no browser, no window.
    #[ cfg( not( target_arch = "wasm32" ) ) ]
    pub fn new_native( width : u32, height : u32 ) -> Result< Self, Error >
    {
      let ( device, queue, surface ) = Device::new_native( width, height )?;

      Ok
      (
        Self
        {
          surface_size : [ width, height ],
          device,
          queue,
          surface
        }
      )
    }

    /// Pixel size of the render target — the canvas in the browser, the
    /// offscreen surface natively.
    pub fn size( &self ) -> [ u32; 2 ]
    {
      #[ cfg( target_arch = "wasm32" ) ]
      {
        [ self.canvas.width(), self.canvas.height() ]
      }
      #[ cfg( not( target_arch = "wasm32" ) ) ]
      {
        self.surface_size
      }
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
