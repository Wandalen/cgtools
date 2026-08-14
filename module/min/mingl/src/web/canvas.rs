/// Internal namespace.
mod private
{
  use crate::web::{ dom, web_sys };
  pub use web_sys::
  {
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
  ///
  /// # Errors
  /// Returns an error if the window/document cannot be accessed, or if an element
  /// matching id/class `canvas` exists but is not an `HtmlCanvasElement`, or if no
  /// matching element is found at all.
  #[ inline ]
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

    Err( Error::CanvasRetrievingError( "Canvas was not found" ) )
  }

  /// Create a canvas, add it to the document body, and keep it filling the whole of its
  /// parent: CSS stretches it to 100% x 100%, `<html>`/`<body>` are pinned to the viewport
  /// height ( so the percentage resolves against something on a bare page ), and a
  /// `ResizeObserver` keeps the drawing buffer matched to the canvas's CSS box at
  /// `devicePixelRatio` resolution through any window- or layout-driven resize.
  ///
  /// # Errors
  /// Returns an error if the window/document/body cannot be accessed, or if the new
  /// canvas element cannot be created, classed, appended, styled, or observed.
  #[ inline ]
  pub fn make() -> Result< HtmlCanvasElement, Error >
  {
    let window = web_sys::window().ok_or( Error::CanvasRetrievingError( "Failed to get window" ) )?;
    let document = window.document().ok_or( Error::CanvasRetrievingError( "Failed to get document" ) )?;

    // Create a new canvas if not found
    let canvas = document.create_element( "canvas" ).map_err( |_| Error::CanvasRetrievingError( "Failed to create a new canvas" ) )?;
    let canvas : HtmlCanvasElement = canvas.dyn_into().map_err( |_| Error::CanvasRetrievingError( "Failed to create a new canvas" ) )?;
    canvas.class_list().add_1( "canvas" ).map_err( |_| Error::CanvasRetrievingError( "Failed to assign a class to the canvas" ) )?;

    // Add the canvas to the document body
    let body = document.body().ok_or( Error::CanvasRetrievingError( "Failed to get body of the document" ) )?;
    body.append_child( &canvas ).map_err( |_| Error::CanvasRetrievingError( "Failed to append canvas to the document" ) )?;

    // "100% of the parent" is ill-defined on a default page: <html> and <body> both have
    // auto height, so a percentage height would resolve against nothing. Pin both to the
    // viewport and drop the default body margin so a bare-page canvas genuinely fills the
    // window.
    if let Some( html ) = document.document_element()
    {
      if let Ok( html ) = html.dyn_into::< web_sys::HtmlElement >()
      {
        html.style().set_property( "height", "100%" ).map_err( |_| Error::CanvasRetrievingError( "Failed to set css height of <html>" ) )?;
      }
    }
    body.style().set_property( "height", "100%" ).map_err( |_| Error::CanvasRetrievingError( "Failed to set css height of <body>" ) )?;
    body.style().set_property( "margin", "0" ).map_err( |_| Error::CanvasRetrievingError( "Failed to set css margin of <body>" ) )?;

    // Set CSS styles to stretch the canvas to fill the whole parent
    canvas.style().set_property( "width", "100%" ).map_err( |_| Error::CanvasRetrievingError( "Failed to set css width of canvas" ) )?;
    canvas.style().set_property( "height", "100%" ).map_err( |_| Error::CanvasRetrievingError( "Failed to set css height of canvas" ) )?;
    canvas.style().set_property( "display", "block" ).map_err( |_| Error::CanvasRetrievingError( "Failed to set css display of canvas" ) )?;

    // Size the drawing buffer synchronously — callers read canvas.width() right after
    // make() — then keep it in sync via a ResizeObserver on the canvas itself. A window
    // `resize` listener would miss parent-driven layout changes ( devtools docking, flex
    // reflow, splitters ); the observer reports exactly the box the canvas occupies.
    canvas_resize( &canvas );

    let canvas_clone = canvas.clone();
    let closure = Closure::wrap( Box::new( move ||
    {
      canvas_resize( &canvas_clone );
    }) as Box< dyn Fn() > );
    let observer = web_sys::ResizeObserver::new( closure.as_ref().unchecked_ref() )
    .map_err( | e | Error::BindgenError( "Cant create ResizeObserver", format!( "{e:?}" ) ) )?;
    observer.observe( &canvas );

    // Both the callback and the observer must live for the whole app; dropping the
    // observer would let the browser stop delivering resize callbacks.
    closure.forget();
    core::mem::forget( observer );

    Ok( canvas )
  }

  /// Trying to find a canvas with id "canvas",
  /// if fails to find it's looking for canvas with class "canvas",
  /// if fails to find it create a canvas that fills its parent and tracks resizes
  /// ( see [`make`] ).
  ///
  /// # Errors
  /// Returns an error under the same conditions as [`make`], since it is called
  /// as a fallback when [`retrieve`] does not find an existing canvas.
  #[ inline ]
  pub fn retrieve_or_make() -> Result< HtmlCanvasElement, Error >
  {
    if let Ok( canvas ) = retrieve()
    {
      return Ok( canvas );
    }
    make()
  }

  // Match the drawing buffer to the canvas's own CSS box, at device-pixel-ratio
  // resolution so hiDPI displays get a crisp buffer rather than an upscaled one.
  // Measuring the canvas itself ( not its parent ) sidesteps auto-height parents and
  // works for `retrieve()`d canvases too.
  fn canvas_resize( canvas : &HtmlCanvasElement )
  {
    let dpr = web_sys::window().map_or( 1.0, | window | window.device_pixel_ratio() );
    // `client_width`/`client_height` return `i32` for historical WebIDL reasons, but the
    // DOM spec guarantees both are always non-negative for a connected element.
    let width = ( f64::from( canvas.client_width() ) * dpr ).round() as u32;
    let height = ( f64::from( canvas.client_height() ) * dpr ).round() as u32;

    canvas.set_width( width );
    canvas.set_height( height );
  }

  /// Sets canvas width and height in CSS style as width / dpr and height / dpr. This removes scaling of operating system.
  /// If you resize the canvas after calling this funciton, don't forget to update CSS width and height.
  ///
  /// # Panics
  /// Panics if the global `window` object is unavailable, or if setting the `width`/`height`
  /// CSS properties on the canvas style fails.
  #[ inline ]
  pub fn dpr_scaling_remove( canvas: &HtmlCanvasElement )
  {
    let width = canvas.width();
    let height = canvas.height();
    let dpr = web_sys::window().expect( "Should have a window" ).device_pixel_ratio();
    let css_width = format!( "{}px", f64::from( width ) / dpr );
    let css_height = format!( "{}px", f64::from( height ) / dpr );
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
    dpr_scaling_remove
  };

}
