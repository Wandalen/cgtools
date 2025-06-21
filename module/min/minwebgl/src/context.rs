/// Internal namespace.
mod private
{
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

    /// Error when failing to find or create a canvas.
    #[ error( "Failed to create resource {0}" ) ]
    FailedToAllocateResource( &'static str ),
    #[ error( "Cant upload uniform {0} with {1} of length {2}.\nKnown length : [ {3} ]" ) ]
    CantUploadUniform( &'static str, &'static str, usize, &'static str ),
    #[ error( "Not supported for type {0}" ) ]
    NotSupportedForType( &'static str ),

    #[ error( "Data type error :: {0}" ) ]
    DataType( #[ from ] data_type::Error ),
    #[ error( "Dom error :: {0}" ) ]
    DomError( #[ from ] dom::Error ),
    #[ error( "Shader error :: {0}" ) ]
    ShaderError( #[ from ] shader::Error ),

  }

  pub fn from_canvas( canvas : &HtmlCanvasElement ) -> Result< GL, Error >
  {
    from_canvas_with( canvas, ContexOptions::default() )
  }

  /// Create a WebGL2 context from a canvas.
  pub fn from_canvas_with( canvas: &HtmlCanvasElement, o : ContexOptions ) -> Result< GL, Error >
  {
    if o.remove_dpr_scaling
    {
      canvas::remove_dpr_scaling( &canvas );
    }

    let context_options : js_sys::Object = o.into();
    let context = canvas
    .get_context_with_context_options( "webgl2", &context_options )
    .map_err( |_| Error::ContextRetrievingError( "Failed to get webgl2 context" ) )?
    .ok_or( Error::ContextRetrievingError( "No webgl2 context" ) )?;

    let gl : GL = context
    .dyn_into()
    .map_err( |_| Error::ContextRetrievingError( "Failed to cast to GL" ) )?;

    Ok( gl )
  }

  /// Create a 2d context from a canvas.
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
  pub fn retrieve_or_make() -> Result< GL, Error >
  {
    retrieve_or_make_with( Default::default() )
  }

  // aaa : explain difference between similar functions
  /// Retrieves a WebGL2 context from an existing canvas or creates a new canvas if none is found,
  /// applying the specified `ContexOptions`.
  ///
  /// # Arguments
  /// - `o`: A `ContexOptions` instance to configure the behavior of the canvas, such as
  ///   reducing device pixel ratio scaling.
  ///
  /// # Errors
  /// - Returns an error if the canvas cannot be found, created, or if the WebGL2 context cannot
  ///   be retrieved.
  // aaa : use o instead of long name in such cases
  pub fn retrieve_or_make_with( o : ContexOptions ) -> Result< GL, Error >
  {
    let canvas = canvas::retrieve_or_make()?;
    // aaa : no, opposite retrieve_or_make is shortcut for retrieve_or_make_with
    from_canvas_with( &canvas, o )
  }

  #[ derive( Debug, Clone, Copy, Default ) ]
  pub enum PowerPreference
  {
    LowPower,
    HighPerformance,
    #[ default ]
    Default,
  }

  impl ToString for PowerPreference
  {
    fn to_string( &self ) -> String
    {
      match self
      {
        PowerPreference::LowPower => "low-power".to_string(),
        PowerPreference::HighPerformance => "high-performance".to_string(),
        PowerPreference::Default => "default".to_string(),
      }
    }
  }

  /// `ContexOptions` is a configuration struct used to customize the behavior of canvas creation
  /// and WebGL2 context retrieval.
  #[ derive( Debug, Clone ) ]
  pub struct ContexOptions
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

  impl ContexOptions
  {
    pub fn remove_dpr_scaling( mut self, val : bool ) -> Self
    {
      self.remove_dpr_scaling = val;
      self
    }

    pub fn preserve_drawing_buffer( mut self, val : bool ) -> Self
    {
      self.preserve_drawing_buffer = val;
      self
    }

    pub fn alpha( mut self, val : bool ) -> Self
    {
      self.alpha = val;
      self
    }

    pub fn antialias( mut self, val : bool ) -> Self
    {
      self.antialias = val;
      self
    }

    pub fn depth( mut self, val : bool ) -> Self
    {
      self.depth = val;
      self
    }

    pub fn stencil( mut self, val : bool ) -> Self
    {
      self.stencil = val;
      self
    }

    pub fn premultiplied_alpha( mut self, val : bool ) -> Self
    {
      self.premultiplied_alpha = val;
      self
    }

    pub fn fail_if_major_performance_caveat( mut self, val : bool ) -> Self
    {
      self.fail_if_major_performance_caveat = val;
      self
    }

    pub fn power_preference( mut self, val : PowerPreference ) -> Self
    {
      self.power_preference = val;
      self
    }

    pub fn desynchronized( mut self, val : bool ) -> Self
    {
      self.desynchronized = val;
      self
    }
  }

  impl Default for ContexOptions
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

  impl Into< js_sys::Object > for ContexOptions
  {
    fn into( self ) -> js_sys::Object
    {
      let context_options = js_sys::Object::new();
      js_sys::Reflect::set( &context_options, &"preserveDrawingBuffer".into(), &self.preserve_drawing_buffer.into() ).unwrap();
      js_sys::Reflect::set( &context_options, &"alpha".into(), &self.alpha.into() ).unwrap();
      js_sys::Reflect::set( &context_options, &"antialias".into(), &self.antialias.into() ).unwrap();
      js_sys::Reflect::set( &context_options, &"depth".into(), &self.depth.into() ).unwrap();
      js_sys::Reflect::set( &context_options, &"stencil".into(), &self.stencil.into() ).unwrap();
      js_sys::Reflect::set( &context_options, &"premultipliedAlpha".into(), &self.premultiplied_alpha.into() ).unwrap();
      js_sys::Reflect::set( &context_options, &"failIfMajorPerformanceCaveat".into(), &self.fail_if_major_performance_caveat.into() ).unwrap();
      js_sys::Reflect::set( &context_options, &"powerPreference".into(), &self.power_preference.to_string().into() ).unwrap();
      js_sys::Reflect::set( &context_options, &"desynchronized".into(), &self.desynchronized.into() ).unwrap();

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
    ContexOptions,
    PowerPreference,
  };

}
