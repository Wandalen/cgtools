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
  pub fn from_canvas( canvas: &HtmlCanvasElement ) -> Result< GL, Error >
  {
    let context = canvas
    .get_context( "webgl2" )
    .map_err( |_| Error::ContextRetrievingError( "Failed to get webgl2 context" ) )?
    .ok_or( Error::ContextRetrievingError( "No webgl2 context" ) )?;

    let gl : GL = context
    .dyn_into()
    .map_err( |_| Error::ContextRetrievingError( "Failed to cast to GL" ) )?;

    Ok( gl )
  }

  /// Retrieve WebGL2 context from a canvas or create a new canvas and retrives from it the context.
  ///
  /// Trying to find a canvas with id "canvas",
  /// if fails to find it's looking for canvas with class "canvas",
  /// if fails to find it create a canvas, add it to document body and stretch it to fill whole screen.
  /// Retrtuve from canvas WebGL2 context.
  pub fn retrieve_or_make() -> Result< GL, Error >
  {
    let canvas = canvas::retrieve_or_make()?;
    from_canvas( &canvas )
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
  };

}
