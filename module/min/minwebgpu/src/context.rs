/// Internal namespace.
mod private
{
  use crate::{ web_sys, JsCast, GL, dom, GpuTextureFormat, WebGPUError, texture };
  use wasm_bindgen_futures::JsFuture;

  /// Retrieves the `web_sys::Navigator` object from the current window.
  ///
  /// # Panics
  /// Panics if no global `window` object exists (e.g. not running inside a browser window).
  #[ inline ]
  #[ must_use ]
  pub fn navigator() -> web_sys::Navigator
  {
    let window = web_sys::window().unwrap();
    window.navigator()
  }

  // Fix(BUG-164): `adapter_request`/`preferred_format` used to call `navigator.gpu()`
  // unconditionally and immediately invoke a method on the result, panicking at the wasm-bindgen
  // FFI boundary ("can't access property ..., arg0 is undefined") when the browser has no
  // WebGPU support at all -- `navigator.gpu` is then JS `undefined`, not a `Gpu` object.
  // Root cause: `web_sys::Navigator::gpu()` is a raw, unchecked property getter (it returns
  // whatever is there, even `undefined`, typed as `Gpu` regardless) -- discovered via BUG-162's
  // own regression test throwing this exact FFI error in a real, WebGPU-less headless test
  // browser, one call earlier than anything BUG-162 touched.
  // Pitfall: `web_sys` types the return of a getter like `.gpu()` as non-`Option` even when the
  // underlying browser feature is experimental/optional -- the crate calling it is responsible
  // for feature-detecting first (`JsValue::is_undefined`), the binding itself won't.
  /// Returns the browser's `Gpu` interface, or `ContextError::WebGpuUnsupported` if this browser
  /// has no WebGPU support at all (`navigator.gpu` itself is `undefined`, not merely a failed
  /// or empty adapter/device request).
  fn gpu_or_unsupported() -> Result< web_sys::Gpu, WebGPUError >
  {
    let gpu = navigator().gpu();
    if AsRef::< wasm_bindgen::JsValue >::as_ref( &gpu ).is_undefined()
    {
      return Err( crate::error::ContextError::WebGpuUnsupported.into() );
    }

    Ok( gpu )
  }

  // Fix(BUG-162): `adapter_request`/`device_request` used to unconditionally `.unwrap()` both
  // the JsFuture result and the dyn_into cast, panicking on two ordinary, reachable outcomes
  // ("no adapter available", "device request rejected") that this crate's own written invariant
  // (docs/invariant/001_result_based_error_handling.md) requires surfacing as `Result::Err`.
  // Root cause: `Gpu::request_adapter()` resolves (never rejects) with `null` on "no adapter" --
  // a normal spec-defined outcome, not an exception -- so a blind `.unwrap()` on the outer
  // JsFuture result masked that this specific failure mode needed its own check, not error
  // handling on the wrong side of the Result.
  // Pitfall: a Promise's own resolve/reject shape doesn't map 1:1 onto "success/failure" --
  // `request_adapter` communicates its one failure mode through a resolved `null`, while
  // `request_device` communicates its failure mode through rejection. Each needs its own check
  // matching its actual signature, not a uniform `.unwrap()` on the outer Result.
  /// Asynchronously requests a WebGPU adapter from the browser.
  ///
  /// # Errors
  /// Returns `error::ContextError::WebGpuUnsupported` if `navigator.gpu` itself is absent (this
  /// browser has no WebGPU support at all). Returns `error::ContextError::NoAdapterAvailable` if
  /// `navigator.gpu.requestAdapter()` resolves to `null` -- the browser has WebGPU infrastructure
  /// but no compatible `GPUAdapter` available. Both are normal, spec-defined outcomes, not
  /// browser exceptions.
  ///
  /// # Panics
  /// Panics if the non-null value the browser resolves the request with does not cast to
  /// `web_sys::GpuAdapter` -- unreachable per the WebGPU spec, which guarantees
  /// `requestAdapter()` resolves with either `null` or a valid `GPUAdapter`.
  #[ inline ]
  pub async fn adapter_request() -> Result< web_sys::GpuAdapter, WebGPUError >
  {
    let gpu = gpu_or_unsupported()?;

    let adapter = JsFuture::from( gpu.request_adapter() ).await.unwrap();
    if adapter.is_null()
    {
      return Err( crate::error::ContextError::NoAdapterAvailable.into() );
    }

    Ok( adapter.dyn_into().unwrap() )
  }

  /// Asynchronously requests a logical GPU device from a given adapter.
  ///
  /// # Errors
  /// Returns `error::ContextError::DeviceRequestRejected` if the device request's promise is
  /// rejected by the browser.
  ///
  /// # Panics
  /// Panics if the value the browser resolves the request with does not cast to
  /// `web_sys::GpuDevice` -- unreachable per the WebGPU spec, which guarantees
  /// `requestDevice()` resolves only with a valid `GPUDevice` on success.
  #[ inline ]
  pub async fn device_request( adapter : &web_sys::GpuAdapter ) -> Result< web_sys::GpuDevice, WebGPUError >
  {
    let device = JsFuture::from( adapter.request_device() )
    .await
    .map_err( | e | crate::error::ContextError::DeviceRequestRejected( format!( "{e:?}" ) ) )?;

    Ok( device.dyn_into().unwrap() )
  }

  /// Retrieves the WebGPU context from an HTML canvas element.
  ///
  /// # Errors
  /// Returns `dom::Error::ContextRetrievingError` if the canvas has no `"webgpu"` context
  /// (e.g. it was already configured for a different context type, or WebGPU is unsupported),
  /// or if the returned context fails to cast to [`GL`].
  #[ inline ]
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
  ///
  /// # Errors
  /// Returns `error::CanvasError::ConfigurationError` if the underlying
  /// `GPUCanvasContext.configure` call throws.
  #[ inline ]
  pub fn configure( device : &web_sys::GpuDevice, context : &GL, format : GpuTextureFormat ) -> Result< (), WebGPUError >
  {
    let configuration = web_sys::GpuCanvasConfiguration::new( device, format );

    context.configure( &configuration ).map_err( | e | crate::error::CanvasError::ConfigurationError( format!( "{e:?}" ) ) )?;
    Ok( () )
  }

  // Fix(BUG-164): see `gpu_or_unsupported`'s own comment above `adapter_request` -- this
  // function had the identical unconditional-`navigator.gpu()` panic risk, independently
  // reachable since callers may query the preferred format before ever calling
  // `adapter_request` (the WebGPU spec defines `getPreferredCanvasFormat()` as a standalone
  // capability query, not dependent on a live adapter/device).
  /// Retrieves the preferred texture format for the current canvas.
  ///
  /// # Errors
  /// Returns `error::ContextError::WebGpuUnsupported` if `navigator.gpu` itself is absent (this
  /// browser has no WebGPU support at all).
  #[ inline ]
  pub fn preferred_format() -> Result< GpuTextureFormat, WebGPUError >
  {
    Ok( gpu_or_unsupported()?.get_preferred_canvas_format() )
  }

  /// Gets the current texture from the WebGPU context.
  ///
  /// # Errors
  /// Returns `error::ContextError::FailedToGetCurrentTextureError` if the underlying
  /// `GPUCanvasContext.getCurrentTexture` call throws.
  #[ inline ]
  pub fn current_texture( context : &GL ) -> Result< web_sys::GpuTexture, WebGPUError >
  {
    let format = context.get_current_texture()
    .map_err( | e | crate::error::ContextError::FailedToGetCurrentTextureError( format!( "{e:?}" ) ) )?;

    Ok( format )
  }

  /// Gets a default view of the context's current texture in one call.
  /// Equivalent to [`current_texture`] followed by [`texture::view`].
  ///
  /// # Errors
  /// Returns whatever [`current_texture`] or [`texture::view`] returns on failure.
  #[ inline ]
  pub fn current_view( context : &GL ) -> Result< web_sys::GpuTextureView, WebGPUError >
  {
    let texture = current_texture( context )?;
    texture::view( &texture )
  }

  /// The objects produced by the one-shot [`setup`] convenience.
  ///
  /// Every field is the plain native `web_sys` type you would have gotten by
  /// calling [`from_canvas`], [`adapter_request`], [`device_request`] and
  /// [`preferred_format`] yourself — this struct only aggregates their
  /// results, it does not wrap or hide them.
  #[ non_exhaustive ]
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
  /// sequence in one call: [`from_canvas`], [`adapter_request`],
  /// [`device_request`], [`preferred_format`], then [`configure`].
  ///
  /// This is pure sequencing — nothing is defaulted beyond what those
  /// functions already do on their own. Call them individually instead when
  /// you need to inspect the adapter before requesting a device, request
  /// specific device features or limits, or otherwise need to intervene
  /// partway through.
  ///
  /// # Errors
  /// Returns whatever [`from_canvas`], [`adapter_request`], [`device_request`] or
  /// [`configure`] returns on failure.
  #[ inline ]
  pub async fn setup( canvas : &web_sys::HtmlCanvasElement ) -> Result< GpuSetup, WebGPUError >
  {
    let context = from_canvas( canvas )?;
    let adapter = adapter_request().await?;
    let device = device_request( &adapter ).await?;
    let queue = device.queue();
    let format = preferred_format()?;
    configure( &device, &context, format )?;

    Ok( GpuSetup { context, adapter, device, queue, format } )
  }
}

crate::mod_interface!
{
  own use
  {
    adapter_request,
    device_request,
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
