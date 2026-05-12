/// Internal namespace.
mod private
{
  use wasm_bindgen::JsCast;
  use crate::web::*;

  // qqq : implement typed errors
  /// Asynchronously fetches a file over HTTP using the browser's `fetch` API.
  ///
  /// The argument is used verbatim as a URL or path; no folder prefix is added.
  /// Three forms are accepted:
  /// * Absolute URL (`http://...`, `https://...`, or protocol-relative `//...`) — fetched as-is.
  /// * Origin-absolute path (`/assets/foo.png`) — joined to the window origin.
  /// * Origin-relative path (`static/foo.png`, `foo.png`) — joined to the window origin with `/`.
  ///
  /// Trunk-built examples in this repo expose assets under `static/` by default,
  /// so they pass arguments like `"static/foo.obj"`. Other deployments are free to
  /// pass full URLs or different folder prefixes.
  ///
  /// # Arguments
  /// * `file_name` - URL or origin-relative path of the file to fetch.
  ///
  /// # Returns
  /// A `Result` which is either a `Vec<u8>` containing the file's byte data on success,
  /// or a `JsValue` containing a JavaScript error on failure.
  ///
  /// # Panics
  /// This function will panic if it cannot access the browser's `window` object,
  /// if the URL is malformed, or if the fetch request promise is rejected.
  pub async fn load( file_name : &str ) -> Result< Vec< u8 >, JsValue >
  {

    let opts = web_sys::RequestInit::new();
    opts.set_method( "GET" );
    opts.set_mode( web_sys::RequestMode::Cors );

    let window = web_sys::window().unwrap();
    let url = if file_name.starts_with( "http://" )
           || file_name.starts_with( "https://" )
           || file_name.starts_with( "//" )
    {
      file_name.to_string()
    }
    else
    {
      let origin = window.location().origin().unwrap();
      if file_name.starts_with( '/' )
      {
        format!( "{}{}", origin, file_name )
      }
      else
      {
        format!( "{}/{}", origin, file_name )
      }
    };

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

}

crate::mod_interface!
{

  own use load;

}
