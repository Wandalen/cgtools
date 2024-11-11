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

  // Create HtmlVideoElement and set source of video resource
  pub fn create_video_element( path : &str, video_width : u32, video_height : u32 ) -> Result< web_sys::HtmlVideoElement, wasm_bindgen::JsValue >
  {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let origin = window.location().origin().unwrap();
    let url = format!( "{}/{}", origin, path );

    let video_element = document
    .create_element( "video" )?
    .dyn_into::< web_sys::HtmlVideoElement >()?;

    video_element.set_src( &url );
    video_element.set_width( video_width );
    video_element.set_height( video_height );
    video_element.set_loop( true );
    video_element.set_muted( true );
    let _ = video_element.play()?;

    Ok( video_element )
  }

}

crate::mod_interface!
{

  own use
  {
    JsCast,
    Error,
  };
  own use create_video_element;

}
