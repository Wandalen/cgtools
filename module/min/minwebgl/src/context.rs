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
    CanUploadUniform( &'static str, &'static str, usize, &'static str ),
    #[ error( "Not supported for type {0}" ) ]
    NotSupportedForType( &'static str ),

    #[ error( "Data type error :: {0}" ) ]
    DataType( #[ from ] data_type::Error ),
    #[ error( "Dom error :: {0}" ) ]
    DomError( #[ from ] dom::Error ),
    #[ error( "Shader error :: {0}" ) ]
    ShaderError( #[ from ] shader::Error ),

  }

  /// Create a WebGL2 context from a canvas.
  pub fn from_canvas( canvas: &HtmlCanvasElement, o : ContexOptions ) -> Result< GL, Error >
  {
    let context_options = js_sys::Object::new();
    js_sys::Reflect::set( &context_options, &"preserveDrawingBuffer".into(), &o.preserve_drawing_buffer.into() ).unwrap();
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
    // let canvas = canvas::retrieve_or_make()?;
    // from_canvas( &canvas )
  }

  // qqq : explain difference between similar functions
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
    if o.reduce_dpr
    {
      canvas::remove_dpr_scaling( &canvas );
    }
    from_canvas( &canvas, o )
  }

  /// `ContexOptions` is a configuration struct used to customize the behavior of canvas creation
  /// and WebGL2 context retrieval. It allows for optional adjustments, such as reducing the canvas
  /// scaling based on the device's pixel ratio.
  #[ derive( Debug, Clone, Copy, Default ) ]
  pub struct ContexOptions
  {
    /// If set to true, the canvas will be scaled down by the device's pixel ratio, which can help
    /// in achieving consistent rendering across devices with different pixel densities.
    pub reduce_dpr : bool,
    /// If set to true, the drawing buffer will be preserved, allowing for the contents of the
    /// canvas to be retained after rendering. This can be useful for certain applications where
    /// you want to keep the rendered content visible even after the next frame is drawn.
    /// Note that this may have performance implications.
    pub preserve_drawing_buffer : bool,
  }

  impl ContexOptions
  {
    pub fn new() -> Self
    {
      Self::default()
    }

    pub fn reduce_dpr( mut self, val : bool ) -> Self
    {
      self.reduce_dpr = val;
      self
    }

    pub fn preserve_drawing_buffer( mut self, val : bool ) -> Self
    {
      self.preserve_drawing_buffer = val;
      self
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
    retrieve_or_make,
    from_canvas_2d,
    retrieve_or_make_with,
    ContexOptions,
  };

}
