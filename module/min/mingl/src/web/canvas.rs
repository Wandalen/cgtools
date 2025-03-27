/// Internal namespace.
mod private
{
  use crate::*;
  use crate::web::*;
  pub use web_sys::
  {
    Element,
    HtmlCanvasElement,
    wasm_bindgen::
    {
      JsCast,
      closure::Closure,
    },
  };

  // use wasm_bindgen::prelude::*;
  // use wasm_bindgen::JsCast;
  // use web_sys::{ window, HtmlCanvasElement, Element };

  pub use dom::Error;

  /// Trying to find a canvas with id "canvas",
  /// if fails to find it's looking for canvas with class "canvas",
  /// if fails to find it return error.
  pub fn retrieve() -> Result< HtmlCanvasElement, Error >
  {
    let window = web_sys::window().ok_or( Error::CanvasRetrievingError( "Failed to get window" ) )?;
    let document = window.document().ok_or( Error::CanvasRetrievingError( "Failed to get document" ) )?;

    // Try to find the canvas by id
    if let Some( canvas ) = document.get_element_by_id( "canvas" )
    {
      return canvas.dyn_into::< HtmlCanvasElement >().map_err( |_| Error::CanvasRetrievingError( "DOM Element with id `canvas` exist, but it's not canvas" ) );
    }

    // Try to find the canvas by class
    if let Some( canvas ) = document.get_elements_by_class_name( "canvas" ).item( 0 )
    {
      return canvas.dyn_into::< HtmlCanvasElement >().map_err( |_| Error::CanvasRetrievingError( "DOM Element with class `canvas` exist, but it's not canvas" ) );
    }

    return Err( Error::CanvasRetrievingError( "Canvas was not found" ) )
  }

  /// Add canvas to document body and stretch it to fill whole screen. Also bind resize handler on parent.
  pub fn make() -> Result< HtmlCanvasElement, Error >
  {
    let window = web_sys::window().ok_or( Error::CanvasRetrievingError( "Failed to get window" ) )?;
    let document = window.document().ok_or( Error::CanvasRetrievingError( "Failed to get document" ) )?;

    // Create a new canvas if not found
    let canvas = document.create_element( "canvas" ).map_err( |_| Error::CanvasRetrievingError( "Failed to create a new canvas" ) )?;
    let canvas : HtmlCanvasElement = canvas.dyn_into().map_err( |_| Error::CanvasRetrievingError( "Failed to create a new canvas" ) )?;
    canvas.class_list().add_1( "canvas" ).map_err( |_| Error::CanvasRetrievingError( "Failed to assign a class to the canvas" ) )?;

    // Add the canvas to the document body
    document.body()
    .ok_or( Error::CanvasRetrievingError( "Failed to get body of the document" ) )?
    .append_child( &canvas )
    .map_err( |_| Error::CanvasRetrievingError( "Failed to append canvas to the document" ) )?;

    // Set CSS styles to stretch the canvas to fill the whole screen
    canvas.style().set_property( "width", "100%" ).map_err( |_| Error::CanvasRetrievingError( "Failed to set css width of canvas" ) )?;
    canvas.style().set_property( "height", "100%" ).map_err( |_| Error::CanvasRetrievingError( "Failed to set css height of canvas" ) )?;
    canvas.style().set_property( "display", "block" ).map_err( |_| Error::CanvasRetrievingError( "Failed to set css display of canvas" ) )?;

    // Try to get the parent element, and do nothing if there's no parent
    if let Some( parent ) = canvas.parent_element()
    {
      // Resize the canvas initially
      resize_canvas( &canvas, &parent );

      // Create a closure to handle window resizing
      let _canvas = canvas.clone();
      let closure = Closure::wrap( Box::new( move ||
      {
        resize_canvas( &_canvas, &parent );
      }) as Box< dyn Fn() > );

      // Add the closure as a listener to the resize event
      window
      .add_event_listener_with_callback( "resize", closure.as_ref().unchecked_ref() )
      .map_err( | e | Error::BindgenError( "Cant bind resize", format!( "{:?}", e ) ) )?;

      // Keep the closure alive for the duration of the app
      closure.forget();
    }
    else
    {
      // Do nothing if no parent exists
      web_sys::console::log_1( &"Canvas has no parent, skipping resize.".into() );
    }

    Ok( canvas )
  }

  /// Trying to find a canvas with id "canvas",
  /// if fails to find it's looking for canvas with class "canvas",
  /// if fails to find it create a canvas, add it to document body and stretch it to fill whole screen. Also bind resize handler on parent.
  pub fn retrieve_or_make() -> Result< HtmlCanvasElement, Error >
  {
    if let Ok( canvas ) = retrieve()
    {
      return Ok( canvas );
    }
    make()
  }

  // Function to resize the canvas
  fn resize_canvas( canvas: &HtmlCanvasElement, parent: &Element )
  {
    // Set the canvas dimensions to match the parent element's size
    let width = parent.client_width() as u32;
    let height = parent.client_height() as u32;

    // log::info!( "resize : {width}x{height}" );

    canvas.set_width( width );
    canvas.set_height( height );
  }

  /// Sets canvas width and height in CSS style as width / dpr and height / dpr. This removes scaling of operating system.
  /// If you resize the canvas after calling this funciton, don't forget to update CSS width and height.
  pub fn remove_dpr_scaling( canvas: &HtmlCanvasElement )
  {
    let width = canvas.width();
    let height = canvas.height();
    let dpr = web_sys::window().expect( "Should have a window" ).device_pixel_ratio();
    let css_width = format!( "{}px", width as f64 / dpr );
    let css_height = format!( "{}px", height as f64 / dpr );
    canvas.style().set_property( "width", &css_width ).unwrap();
    canvas.style().set_property( "height", &css_height ).unwrap();
  }
}

crate::mod_interface!
{

  own use
  {
    Error,
    HtmlCanvasElement,
    retrieve,
    make,
    retrieve_or_make,
    remove_dpr_scaling
  };

}
