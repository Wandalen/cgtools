/// Internal namespace.
mod private
{
  use crate::*;
  pub use web_sys::
  {
    wasm_bindgen::
    {
      JsCast,
    },
  };

  /// Represents errors related to dom elements handling.
  #[ derive( Debug, error::typed::Error ) ]
  pub enum Error
  {

    /// Error when failing to find or create a canvas.
    #[ error( "Failed to find or create a canvas\n{0}" ) ]
    CanvasRetrievingError( &'static str ),

    /// Error when failing to get WebGL2 context.
    #[ error( "Failed to get WebGL2 context" )]
    ContextRetrievingError( &'static str ),

    #[ error( "Bindgen error : {0}\n{1}" )]
    BindgenError( &'static str, String ),

  }

}

crate::mod_interface!
{

  own use
  {
    JsCast,
    Error,
  };

}
