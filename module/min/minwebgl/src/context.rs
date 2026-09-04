/// Internal namespace.
mod private
{
  #[ allow( clippy::wildcard_imports, reason = "crate-root prelude from mod_interface!; enumerating would break on every layer change" ) ]
  use crate::*;
  // pub use ::web_sys::{ WebGl2RenderingContext, WebGl2RenderingContext as GL };
  pub use web_sys::
  {
    HtmlCanvasElement,
    wasm_bindgen::
    {
      JsCast,
    },
  };

  pub use dom::Error;

  /// Represents errors related to dom elements handling.
  #[ derive( Debug, error::typed::Error ) ]
  pub enum WebglError
  {
    /// Occurs when the system fails to create a WebGL resource, such as a buffer, texture, or vertex array object (VAO),
    /// which can happen if the browser's WebGL context is lost or resources are exhausted.
    #[ error( "Failed to create resource {0}" ) ]
    FailedToAllocateResource( &'static str ),
    /// Error when trying to upload a uniform with incorrect data.
    #[ error( "Cant upload uniform {0} with {1} of length {2}.\nKnown length : [ {3} ]" ) ]
    CantUploadUniform( &'static str, &'static str, usize, &'static str ),
    /// Error when operation is not supported for the given type.
    #[ error( "Not supported for type {0}" ) ]
    NotSupportedForType( &'static str ),
    /// Error related to data type conversion.
    #[ error( "Data type error :: {0}" ) ]
    DataType( #[ from ] data_type::Error ),
    /// Error related to DOM operations.
    #[ error( "Dom error :: {0}" ) ]
    DomError( #[ from ] dom::Error ),
    /// Error related to shader compilation or linking.
    #[ error( "Shader error :: {0}" ) ]
    ShaderError( #[ from ] shader::Error ),
    /// Error when required data is missing.
    #[ error( "Can't find {0}" ) ]
    MissingDataError( &'static str ),
    /// Error when a numeric id (e.g. a framebuffer attachment index) does not fit into the
    /// `u32` range a WebGL id requires. Ids computed at runtime (e.g. while iterating a
    /// dynamically sized framebuffer configuration) can legitimately be out of range, so
    /// this is surfaced as a recoverable error instead of panicking.
    #[ error( "{0}" ) ]
    IdOutOfRange( String ),
    /// General error type
    #[ error( "{0}" ) ]
    Other( &'static str ),
  }

  /// Create a WebGL2 context from a canvas element with default options.
  ///
  /// # Errors
  /// Returns an error if the canvas cannot provide a WebGL2 context.
  pub fn from_canvas( canvas : &HtmlCanvasElement ) -> Result< GL, Error >
  {
    from_canvas_with( canvas, ContextOptions::default() )
  }

  /// Create a WebGL2 context from a canvas.
  ///
  /// # Errors
  /// Returns an error if the canvas cannot provide a WebGL2 context or if the retrieved
  /// context cannot be cast to `GL`.
  //
  // Fix(BUG-423): a canvas resized via `mingl::web::canvas`'s `ResizeObserver` ( the one
  // `canvas::make()` sets up ) had its `canvas.width()`/`height()` kept in sync with its CSS
  // box, but any WebGL2 context already bound to that canvas kept rendering into its
  // *original* viewport -- `gl.viewport(..)` is never implied by a drawing-buffer resize, so
  // the visible image stayed clipped/stretched into a stale rectangle after any CSS-driven
  // resize ( window resize, flex reflow, devtools docking, ... ).
  // Root cause: `mingl::web::canvas` is deliberately GL-unaware ( shared substrate reused by
  // both `minwebgl::canvas` and `minwebgpu::canvas`, and used internally by
  // `minwebgl::texture::d2::sprite_upload`'s own temporary 2D-context canvas ) -- it cannot
  // call `gl.viewport(..)` itself without hard-coding a WebGL assumption into shared
  // infrastructure that WebGPU consumers and 2D-context call sites also depend on. The fix
  // therefore lives here instead, in the one place that already knows both the canvas *and*
  // the GL context bound to it: a second, GL-aware `ResizeObserver` is attached once `gl`
  // exists, reusing `canvas::canvas_resize`'s exact width/height computation ( now `pub` for
  // this purpose ) so the buffer size and the viewport can never disagree, then applying it as
  // the viewport.
  // Pitfall: don't move this ( or any `gl.viewport(..)` call ) into `mingl` itself -- a blind
  // `canvas.get_context("webgl2")` in shared canvas-setup code would either silently discard a
  // caller's custom `ContextOptions` ( `getContext` returns the *first* context ever created
  // for a given type, ignoring new options on later calls ), or permanently lock a
  // WebGPU-destined canvas out of `getContext("webgpu")` ( a canvas can only ever bind one
  // context type ), or break `sprite_upload`'s own `context::from_canvas_2d` call on its
  // temporary canvas. See `task/bug/completed/423_*.md` for the full verification record,
  // including why a fully pixel-verified live-browser reproduction of the resize path itself
  // was not achievable from this workspace's existing, unmodified example crates.
  pub fn from_canvas_with( canvas: &HtmlCanvasElement, o : ContextOptions ) -> Result< GL, Error >
  {
    if o.remove_dpr_scaling
    {
      canvas::dpr_scaling_remove( canvas );
    }

    let context_options : js_sys::Object = o.into();
    let context = canvas
    .get_context_with_context_options( "webgl2", &context_options )
    .map_err( |_| Error::ContextRetrievingError( "Failed to get webgl2 context" ) )?
    .ok_or( Error::ContextRetrievingError( "No webgl2 context" ) )?;

    let gl : GL = context
    .dyn_into()
    .map_err( |_| Error::ContextRetrievingError( "Failed to cast to GL" ) )?;

    // Match the viewport to the drawing buffer right away -- `get_context_with_context_options`
    // already sized the buffer to whatever `canvas.width()`/`height()` were at creation time,
    // so an explicit initial `viewport` call keeps this function's own postcondition ( "viewport
    // matches the buffer" ) true from the very first frame, not just after the first resize.
    gl.viewport( 0, 0, canvas.width() as i32, canvas.height() as i32 );

    let canvas_clone = canvas.clone();
    let gl_clone = gl.clone();
    let closure = wasm_bindgen::closure::Closure::wrap( Box::new( move ||
    {
      // Recompute ( not just re-read ) the drawing-buffer size here, redundantly with
      // whatever other `ResizeObserver` may also be watching this canvas ( e.g. the one
      // `canvas::make()` attaches ) -- `canvas_resize` is a pure, idempotent function of the
      // canvas's current CSS box and device pixel ratio, so calling it again is always safe,
      // and doing so makes this callback correct on its own without depending on cross-observer
      // callback ordering, which the `ResizeObserver` spec does not strongly guarantee across
      // independently-registered observer instances.
      canvas::canvas_resize( &canvas_clone );
      gl_clone.viewport( 0, 0, canvas_clone.width() as i32, canvas_clone.height() as i32 );
    }) as Box< dyn Fn() > );
    let observer = web_sys::ResizeObserver::new( closure.as_ref().unchecked_ref() )
    .map_err( | e | Error::BindgenError( "Cant create ResizeObserver", format!( "{e:?}" ) ) )?;
    observer.observe( canvas );

    // Both the callback and the observer must live for the whole app; dropping the observer
    // would let the browser stop delivering resize callbacks.
    closure.forget();
    core::mem::forget( observer );

    Ok( gl )
  }

  /// Create a 2d context from a canvas.
  ///
  /// # Errors
  /// Returns an error if the canvas cannot provide a 2d context or if the retrieved
  /// context cannot be cast to `CanvasRenderingContext2d`.
  pub fn from_canvas_2d( canvas : &HtmlCanvasElement ) -> Result< web_sys::CanvasRenderingContext2d, Error >
  {
    let context = canvas
    .get_context( "2d" )
    .map_err( |_| Error::ContextRetrievingError( "Failed to get 2d context" ) )?
    .ok_or( Error::ContextRetrievingError( "No 2d context" ) )?;

    let context_2d : web_sys::CanvasRenderingContext2d = context
    .dyn_into()
    .map_err( |_| Error::ContextRetrievingError( "Failed to cast to CanvasRenderingContext2d" ) )?;

    Ok( context_2d )
  }

  /// Retrieve WebGL2 context from a canvas or create a new canvas and retrives from it the context.
  ///
  /// Trying to find a canvas with id "canvas",
  /// if fails to find it's looking for canvas with class "canvas",
  /// if fails to find it create a canvas, add it to document body and stretch it to fill whole screen.
  /// Retrtuve from canvas WebGL2 context.
  ///
  /// # Errors
  /// Returns an error if the canvas cannot be found, created, or if the WebGL2 context
  /// cannot be retrieved from it (see [`retrieve_or_make_with`]).
  pub fn retrieve_or_make() -> Result< GL, Error >
  {
    retrieve_or_make_with( ContextOptions::default() )
  }

  /// Retrieves a WebGL2 context from an existing canvas or creates a new canvas if none is found,
  /// applying the specified `ContextOptions`.
  ///
  /// # Arguments
  /// - `o`: A `ContextOptions` instance to configure the behavior of the canvas, such as
  ///   reducing device pixel ratio scaling.
  ///
  /// # Errors
  /// - Returns an error if the canvas cannot be found, created, or if the WebGL2 context cannot
  ///   be retrieved.
  pub fn retrieve_or_make_with( o : ContextOptions ) -> Result< GL, Error >
  {
    let canvas = canvas::retrieve_or_make()?;
    from_canvas_with( &canvas, o )
  }

  /// WebGL power preference options for context creation.
  #[ derive( Debug, Clone, Copy, Default ) ]
  pub enum PowerPreference
  {
    /// Prefer low power consumption.
    LowPower,
    /// Prefer high performance.
    HighPerformance,
    /// Use default power preference.
    #[ default ]
    Default,
  }

  /// Converts the `PowerPreference` enum variant to its corresponding string representation.
  impl std::fmt::Display for PowerPreference
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      let s = match self
      {
        PowerPreference::LowPower => "low-power",
        PowerPreference::HighPerformance => "high-performance",
        PowerPreference::Default => "default",
      };
      write!( f, "{s}" )
    }
  }

  /// `ContextOptions` is a configuration struct used to customize the behavior of canvas creation
  /// and WebGL2 context retrieval.
  #[ derive( Debug, Clone ) ]
  pub struct ContextOptions
  {
    /// If set to true, the canvas will be scaled down by the device's pixel ratio, which can help
    /// in achieving consistent rendering across devices with different pixel densities.
    pub remove_dpr_scaling : bool,
    /// If set to true, the drawing buffer will be preserved, allowing for the contents of the
    /// canvas to be retained after rendering. This can be useful for certain applications where
    /// you want to keep the rendered content visible even after the next frame is drawn.
    /// Note that this may have performance implications.
    pub preserve_drawing_buffer : bool,
    /// If set to true, the canvas will have an alpha channel, allowing for transparency in the
    /// rendered content. This is useful for applications that require blending with the background
    /// or other elements.
    pub alpha : bool,
    /// If set to true, antialiasing will be enabled for smoother edges in the rendered content.
    pub antialias : bool,
    /// If set to true, a depth buffer will be created, allowing for proper depth testing in 3D
    /// rendering. This is important for applications that require accurate depth representation,
    /// such as 3D games or simulations.
    pub depth : bool,
    /// If set to true, a stencil buffer will be created, allowing for advanced rendering techniques
    /// that require stencil operations. This is useful for applications that need to perform
    /// complex masking or rendering effects.
    pub stencil : bool,
    /// If set to true, the canvas will use premultiplied alpha, which can improve performance in
    /// certain scenarios. This is particularly relevant for applications that involve blending
    /// operations, as premultiplied alpha can lead to more efficient rendering.
    pub premultiplied_alpha : bool,
    /// If set to true, the context will fail if there are major performance caveats, which can be
    /// useful for debugging or ensuring that the application runs optimally. This can help
    /// identify potential performance issues early in the development process.
    pub fail_if_major_performance_caveat : bool,
    /// Specifies the power preference for the WebGL2 context. This can be set to "low-power",
    /// "high-performance", or "default". This option allows developers to indicate their
    /// preference for power consumption versus performance, which can be important for
    /// applications that run on battery-powered devices or require high-performance rendering.
    pub power_preference : PowerPreference,
    /// If set to true, the context will be desynchronized, which can improve performance in
    /// certain scenarios. This option is useful for applications that require high-frequency
    /// rendering or need to minimize latency in rendering operations. Desynchronization can help
    /// reduce the overhead of synchronization between the CPU and GPU, leading to smoother
    /// rendering in some cases.
    pub desynchronized : bool,
  }

  impl ContextOptions
  {
    /// Set whether to remove device pixel ratio scaling.
    #[ must_use ]
    pub fn dpr_scaling_remove( mut self, val : bool ) -> Self
    {
      self.remove_dpr_scaling = val;
      self
    }

    /// Set whether to preserve the drawing buffer.
    #[ must_use ]
    pub fn drawing_buffer_preserve( mut self, val : bool ) -> Self
    {
      self.preserve_drawing_buffer = val;
      self
    }

    /// Set whether the canvas has an alpha channel.
    #[ must_use ]
    pub fn alpha( mut self, val : bool ) -> Self
    {
      self.alpha = val;
      self
    }

    /// Set whether antialiasing is enabled.
    #[ must_use ]
    pub fn antialias( mut self, val : bool ) -> Self
    {
      self.antialias = val;
      self
    }

    /// Set whether a depth buffer is created.
    #[ must_use ]
    pub fn depth( mut self, val : bool ) -> Self
    {
      self.depth = val;
      self
    }

    /// Set whether a stencil buffer is created.
    #[ must_use ]
    pub fn stencil( mut self, val : bool ) -> Self
    {
      self.stencil = val;
      self
    }

    /// Set whether to use premultiplied alpha.
    #[ must_use ]
    pub fn premultiplied_alpha( mut self, val : bool ) -> Self
    {
      self.premultiplied_alpha = val;
      self
    }

    /// Set whether to fail on major performance caveats.
    #[ must_use ]
    pub fn fail_if_major_performance_caveat( mut self, val : bool ) -> Self
    {
      self.fail_if_major_performance_caveat = val;
      self
    }

    /// Set the power preference for the WebGL context.
    #[ must_use ]
    pub fn power_preference( mut self, val : PowerPreference ) -> Self
    {
      self.power_preference = val;
      self
    }

    /// Set whether the context is desynchronized.
    #[ must_use ]
    pub fn desynchronized( mut self, val : bool ) -> Self
    {
      self.desynchronized = val;
      self
    }
  }

  /// Provides a default implementation for `ContextOptions`.
  impl Default for ContextOptions
  {
    fn default() -> Self
    {
      Self
      {
        remove_dpr_scaling : false,
        preserve_drawing_buffer : false,
        alpha : true,
        antialias : true,
        depth : true,
        stencil : false,
        premultiplied_alpha : true,
        fail_if_major_performance_caveat : false,
        power_preference : PowerPreference::Default,
        desynchronized : false,
      }
    }
  }

  /// Converts `ContextOptions` into a `js_sys::Object` for use with JavaScript.
  impl From< ContextOptions > for js_sys::Object
  {
    fn from( value : ContextOptions ) -> Self
    {
      let context_options = js_sys::Object::new();
      js_sys::Reflect::set( &context_options, &"preserveDrawingBuffer".into(), &value.preserve_drawing_buffer.into() ).unwrap();
      js_sys::Reflect::set( &context_options, &"alpha".into(), &value.alpha.into() ).unwrap();
      js_sys::Reflect::set( &context_options, &"antialias".into(), &value.antialias.into() ).unwrap();
      js_sys::Reflect::set( &context_options, &"depth".into(), &value.depth.into() ).unwrap();
      js_sys::Reflect::set( &context_options, &"stencil".into(), &value.stencil.into() ).unwrap();
      js_sys::Reflect::set( &context_options, &"premultipliedAlpha".into(), &value.premultiplied_alpha.into() ).unwrap();
      js_sys::Reflect::set( &context_options, &"failIfMajorPerformanceCaveat".into(), &value.fail_if_major_performance_caveat.into() ).unwrap();
      js_sys::Reflect::set( &context_options, &"powerPreference".into(), &value.power_preference.to_string().into() ).unwrap();
      js_sys::Reflect::set( &context_options, &"desynchronized".into(), &value.desynchronized.into() ).unwrap();

      context_options
    }
  }
}

crate::mod_interface!
{

  orphan use
  {
    // GL,
    // WebGl2RenderingContext,
    WebglError,
  };

  own use
  {
    Error,
    from_canvas,
    from_canvas_with,
    retrieve_or_make,
    from_canvas_2d,
    retrieve_or_make_with,
    ContextOptions,
    PowerPreference,
  };

}
