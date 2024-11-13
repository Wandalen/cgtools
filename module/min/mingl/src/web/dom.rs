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

  // Create HtmlImageElement
  pub fn create_image_element( path : &str ) -> Result< web_sys::HtmlImageElement, wasm_bindgen::JsValue >
  {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let origin = window.location().origin().unwrap();
    let url = format!( "{}/{}", origin, path );

    let image_element = document
    .create_element( "img" )?
    .dyn_into::< web_sys::HtmlImageElement >()?;

    image_element.set_src( &url );

    Ok( image_element )
  }

}

crate::mod_interface!
{
  layer image;

  own use
  {
    image,
    JsCast,
    Error,
  };
  own use create_image_element;

}
