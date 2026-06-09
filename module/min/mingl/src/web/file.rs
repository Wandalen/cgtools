/// Internal namespace.
mod private
{
  use wasm_bindgen::JsCast;
  use crate::web::*;

  /// Resolves `file_name` against `origin` according to `load`'s contract.
  ///
  /// * Absolute URLs (`http://`, `https://`, `//`) pass through unchanged.
  /// * Self-contained URLs (`blob:`, `data:`) pass through unchanged — these carry
  ///   their own payload and must reach `fetch` verbatim; prefixing the origin
  ///   mangles them into an unresolvable same-origin path.
  /// * Origin-absolute paths (leading `/`) are appended to the origin as-is.
  /// * Anything else is treated as origin-relative and joined with a single `/`.
  fn resolve_url( origin : &str, file_name : &str ) -> String
  {
    if file_name.starts_with( "http://" )
    || file_name.starts_with( "https://" )
    || file_name.starts_with( "//" )
    || file_name.starts_with( "blob:" )
    || file_name.starts_with( "data:" )
    {
      file_name.to_string()
    }
    else if file_name.starts_with( '/' )
    {
      format!( "{}{}", origin, file_name )
    }
    else
    {
      format!( "{}/{}", origin, file_name )
    }
  }

  // qqq : implement typed errors
  /// Asynchronously fetches a file over HTTP using the browser's `fetch` API.
  ///
  /// The argument is used verbatim as a URL or path; no folder prefix is added.
  /// These forms are accepted:
  /// * Absolute URL (`http://...`, `https://...`, or protocol-relative `//...`) — fetched as-is.
  /// * Self-contained URL (`blob:...`, `data:...`) — fetched as-is.
  /// * Origin-absolute path (`/assets/foo.png`) — joined to the window origin.
  /// * Origin-relative path (`static/foo.png`, `foo.png`) — joined to the window origin with `/`.
  ///
  /// Trunk-built examples in this repo expose assets under `static/` by default,
  /// so they pass arguments like `"static/foo.obj"`. Other deployments are free to
  /// pass full URLs or different folder prefixes.
  ///
  /// An empty `file_name` resolves to `{origin}/` and is almost certainly a caller bug.
  ///
  /// # Arguments
  /// * `file_name` - URL or origin-relative path of the file to fetch.
  ///
  /// # Returns
  /// A `Result` which is either a `Vec<u8>` containing the file's byte data on success,
  /// or a `JsValue` containing a JavaScript error on failure.
  ///
  /// # Errors
  /// Returns the `JsValue` that the underlying `fetch` or `Response::array_buffer`
  /// promise rejected with — typically a `TypeError` for network / CORS failures or
  /// a `DOMException` for aborted reads. HTTP error status codes (4xx, 5xx) do
  /// **not** produce an `Err` here; `fetch` resolves successfully and the caller
  /// receives the response body as `Ok`.
  ///
  /// # Panics
  /// Panics if the browser `window` object is unavailable, if the constructed URL
  /// is rejected by `Request::new_with_str_and_init`, or if the initial `fetch`
  /// promise itself rejects (as opposed to resolving with an error response).
  pub async fn load( file_name : &str ) -> Result< Vec< u8 >, JsValue >
  {

    let opts = web_sys::RequestInit::new();
    opts.set_method( "GET" );
    opts.set_mode( web_sys::RequestMode::Cors );

    let window = web_sys::window().unwrap();
    let origin = window.location().origin().unwrap();
    let url = resolve_url( &origin, file_name );

    let request = web_sys::Request::new_with_str_and_init( &url, &opts ).expect( "Invalid url" );

    let resp_value = JsFuture::from( window.fetch_with_request( &request ) ).await.expect( "Fetch request fail" );
    let resp : web_sys::Response = resp_value.dyn_into().unwrap();
    let array_buffer_promise = resp.array_buffer()?;
    let array_buffer = JsFuture::from( array_buffer_promise ).await?;

    let uint8_array = js_sys::Uint8Array::new( &array_buffer );
    let mut data = vec![ 0; uint8_array.length() as usize ];
    uint8_array.copy_to( &mut data[ .. ] );

    Ok( data )
  }

  #[ cfg( test ) ]
  mod tests
  {
    use super::resolve_url;

    #[ test ]
    fn passes_https_url_through()
    {
      assert_eq!
      (
        resolve_url( "https://app.example.com", "https://cdn.example.com/foo.glb" ),
        "https://cdn.example.com/foo.glb"
      );
    }

    #[ test ]
    fn passes_http_url_through()
    {
      assert_eq!
      (
        resolve_url( "https://app.example.com", "http://legacy.example.com/foo.glb" ),
        "http://legacy.example.com/foo.glb"
      );
    }

    #[ test ]
    fn passes_protocol_relative_url_through()
    {
      assert_eq!
      (
        resolve_url( "https://app.example.com", "//cdn.example.com/foo.glb" ),
        "//cdn.example.com/foo.glb"
      );
    }

    #[ test ]
    fn passes_blob_url_through()
    {
      assert_eq!
      (
        resolve_url( "https://app.example.com", "blob:https://app.example.com/uuid-1234" ),
        "blob:https://app.example.com/uuid-1234"
      );
    }

    #[ test ]
    fn passes_data_url_through()
    {
      assert_eq!
      (
        resolve_url( "https://app.example.com", "data:application/octet-stream;base64,Z2xURg==" ),
        "data:application/octet-stream;base64,Z2xURg=="
      );
    }

    #[ test ]
    fn joins_origin_absolute_path_without_extra_slash()
    {
      assert_eq!
      (
        resolve_url( "https://app.example.com", "/assets/foo.glb" ),
        "https://app.example.com/assets/foo.glb"
      );
    }

    #[ test ]
    fn joins_origin_relative_path_with_slash()
    {
      assert_eq!
      (
        resolve_url( "https://app.example.com", "static/foo.glb" ),
        "https://app.example.com/static/foo.glb"
      );
    }

    #[ test ]
    fn joins_bare_filename_with_slash()
    {
      assert_eq!
      (
        resolve_url( "https://app.example.com", "foo.glb" ),
        "https://app.example.com/foo.glb"
      );
    }

    #[ test ]
    fn empty_input_resolves_to_origin_root()
    {
      assert_eq!
      (
        resolve_url( "https://app.example.com", "" ),
        "https://app.example.com/"
      );
    }
  }

}

crate::mod_interface!
{

  own use load;

}
