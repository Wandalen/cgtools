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
    let canvas = canvas::retrieve_or_make()?;
    from_canvas( &canvas )
  }
  
  /// Retrieve a WebGL2 context from an existing canvas or create a new canvas, configure it using a custom builder, 
  /// and retrieve the WebGL2 context from it.
  ///
  /// This function attempts to find a canvas element with an ID of "canvas". If it fails, it looks for a canvas 
  /// element with a class of "canvas". If no such canvas is found, it creates a new canvas element, adds it to 
  /// the document body, and stretches it to fill the entire screen. After obtaining the canvas, the provided 
  /// `ContexBuilder` implementation is used to configure the canvas before retrieving the WebGL2 context.
  ///
  /// # Type Parameters
  /// - `O`: A type that implements the `ContexBuilder` trait, used to configure the canvas before retrieving the context.
  ///
  /// # Arguments
  /// - `builder`: An instance of a type that implements the `ContexBuilder` trait, used to apply custom configurations 
  ///   to the canvas.
  ///
  /// # Returns
  /// - `Result<GL, Error>`: Returns a `GL` (WebGL2RenderingContext) on success, or an `Error` if the operation fails.
  ///
  /// # Errors
  /// - Returns an error if the canvas cannot be found or created.
  /// - Returns an error if the WebGL2 context cannot be retrieved or cast to the appropriate type.
  pub fn retrieve_or_make_with< O : ContexBuilder >( builder : O ) -> Result< GL, Error >
  {
    let canvas = canvas::retrieve_or_make()?;
    builder.build( &canvas );
    from_canvas( &canvas )
  }

  /// A trait for customizing the configuration of an HTML canvas element before retrieving a WebGL2 context.
  ///
  /// Types implementing this trait can define custom behavior for modifying the canvas element, such as adjusting
  /// its properties or applying specific configurations. This is useful when creating or retrieving a WebGL2 context
  /// with specific requirements.
  ///
  /// # Required Methods
  /// - `build`: This method is called to apply custom configurations to the provided canvas element.
  pub trait ContexBuilder
  {
    fn build( &self, canvas : &HtmlCanvasElement );
  }

  /// A builder that removes device pixel ratio (DPR) scaling from an HTML canvas element.
  ///
  /// The `ReducedDprBuilder` is an implementation of the `ContexBuilder` trait. It customizes the canvas
  /// by removing the scaling applied due to the device's pixel ratio. This can be useful in scenarios
  /// where you want to ensure consistent rendering behavior across devices with different pixel densities.
  pub struct ReducedDprBuilder;

  impl ContexBuilder for ReducedDprBuilder
  {
    fn build( &self, canvas : &HtmlCanvasElement )
    {
      canvas::remove_dpr_scaling( &canvas );
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
    ContexBuilder,
    ReducedDprBuilder,
  };

}
