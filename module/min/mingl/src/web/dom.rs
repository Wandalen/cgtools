/// Internal namespace.
mod private
{
  use crate::error;
  // The `error::typed::Error` derive resolves to `thiserror::Error` through several
  // re-export layers (see `error_tools::error::typed`). Its generated `Display` impl
  // contains its own internal `use thiserror::__private::AsDisplay as _;`, which needs
  // the bare name `thiserror` resolvable here — `mingl` depends on `error_tools`, not
  // `thiserror` directly, so it isn't in the extern prelude without this import.
  use error::thiserror;
  pub use web_sys::
  {
    wasm_bindgen::
    {
      JsCast,
    },
  };

  /// Represents errors related to dom elements handling.
  // Variants are constructed directly by sibling crates (`minwebgpu::context`,
  // `minwebgl::context`) across the crate boundary — `#[non_exhaustive]` would
  // break that construction, so this is a deliberate public contract instead.
  #[ derive( Debug, error::typed::Error ) ]
  pub enum Error
  {

    /// Error when failing to find or create a canvas.
    #[ error( "Failed to find or create a canvas\n{0}" ) ]
    CanvasRetrievingError( &'static str ),

    /// Error when failing to get WebGL2 context.
    #[ error( "Failed to get WebGL2 context\n{0}" ) ]
    ContextRetrievingError( &'static str ),

    /// Error when dealing with bingen functionality
    #[ error( "Bindgen error: {0}\n{1}" ) ]
    BindgenError( &'static str, String ),

  }

  /// Create HtmlVideoElement configure and set source of video resource
  ///
  /// # Parameters
  /// - `path`: Path to the video resource
  /// - `video_width`: Desired width of the video element
  /// - `video_height`: Desired height of the video element
  ///
  /// # Returns
  /// A `Result` containing the created `HtmlVideoElement` or a `JsValue` error
  ///
  /// # Behavior
  /// - Creates video element from document
  /// - Sets video source, resolving `path` against the current document's own
  ///   URL — document-relative paths join to the page's directory, not the
  ///   origin root; see `web::resolve_url` for the exact rule
  /// - Configures video to loop and mute
  /// - Automatically starts video playback
  ///
  /// # When it useful
  /// - Create an video element for the natural and cheapest video upload to the web
  ///
  /// # Errors
  /// Returns an error if the window's location has no href, or if the video
  /// element cannot be created/cast, or if playback fails to start.
  ///
  /// # Panics
  /// Panics if the global `window`/`document` is unavailable.
  //
  // Fix(BUG-109): joined `path` against `window.location().origin()` alone,
  // discarding the current page's own directory — every subpath-deployed
  // example's video source resolved to the site root instead of its own
  // directory.
  // Root cause: same as `web::resolve_url`'s ( origin never carries a path;
  // relative references must resolve against the document's directory ) —
  // `dom` compiles under a strictly weaker feature gate than `file`
  // ( `web` alone vs. `web` + `web_future` + `web_file` ), so this couldn't
  // simply call `web::file::url_resolve` directly; both now share
  // `web::resolve_url` instead.
  // Pitfall: don't reach for `location().origin()` when resolving a path that
  // might be document-relative — only an absolute-path reference (leading
  // `/`) targets the origin.
  #[ inline ]
  pub fn video_element_create( path : &str, video_width : u32, video_height : u32 ) -> Result< web_sys::HtmlVideoElement, wasm_bindgen::JsValue >
  {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let href = window.location().href()?;
    let url = crate::web::resolve_url( &href, path );

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

  /// Create HtmlImageElement and set source of image resource
  ///
  /// # Parameters
  /// - `path`: Path to the image resource
  ///
  /// # Returns
  /// A `Result` containing the created `HtmlImageElement` or a `JsValue` error
  ///
  /// # Behavior
  /// - Creates image element from document
  /// - Sets image source, resolving `path` against the current document's own
  ///   URL — document-relative paths join to the page's directory, not the
  ///   origin root; see `web::resolve_url` for the exact rule
  ///
  /// # When it useful
  /// - Create an image element for the natural and cheapest image upload to the web
  ///
  /// # Errors
  /// Returns an error if the window's location has no href, or if the image
  /// element cannot be created or cast.
  ///
  /// # Panics
  /// Panics if the global `window`/`document` is unavailable.
  //
  // Fix(BUG-109): joined `path` against `window.location().origin()` alone,
  // discarding the current page's own directory — every subpath-deployed
  // example's image source resolved to the site root instead of its own
  // directory.
  // Root cause: same as `video_element_create`'s, above — see its comment.
  // Pitfall: don't reach for `location().origin()` when resolving a path that
  // might be document-relative — only an absolute-path reference (leading
  // `/`) targets the origin.
  #[ inline ]
  pub fn image_element_create( path : &str ) -> Result< web_sys::HtmlImageElement, wasm_bindgen::JsValue >
  {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let href = window.location().href()?;
    let url = crate::web::resolve_url( &href, path );

    let image_element = document
    .create_element( "img" )?
    .dyn_into::< web_sys::HtmlImageElement >()?;

    image_element.set_src( &url );

    Ok( image_element )
  }

}

crate::mod_interface!
{

  own use
  {
    JsCast,
    Error,
  };
  own use video_element_create;
  own use image_element_create;

}
