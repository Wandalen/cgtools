/// Internal namespace.
mod private
{
  use wasm_bindgen::JsCast;
  use crate::error;
  // The `error::typed::Error` derive resolves to `thiserror::Error` through several
  // re-export layers (see `error_tools::error::typed`). Its generated `Display` impl
  // contains its own internal `use thiserror::__private::AsDisplay as _;`, which needs
  // the bare name `thiserror` resolvable here — `mingl` depends on `error_tools`, not
  // `thiserror` directly, so it isn't in the extern prelude without this import.
  use error::thiserror;
  use crate::web::{ JsFuture, JsValue };

  /// Error returned by `load` when fetching or decoding a file fails.
  #[ non_exhaustive ]
  #[ derive( Debug, error::typed::Error ) ]
  pub enum Error
  {
    /// The `data:` URL is malformed or uses an unsupported ( non-base64 ) encoding.
    #[ error( "Data URL error :: {0}" ) ]
    DataUrl( &'static str ),
    /// The underlying browser API rejected — request construction, `fetch`, or body read.
    #[ error( "Web API error :: {0:?}" ) ]
    Js( JsValue ),
  }

  impl From< JsValue > for Error
  {
    #[ inline ]
    fn from( value : JsValue ) -> Self
    {
      Error::Js( value )
    }
  }

  // Callers in wasm contexts commonly return `Result< _, JsValue >` and rely on `?` —
  // this conversion keeps that pattern working, preserving the original browser error
  // object where one exists.
  impl From< Error > for JsValue
  {
    #[ inline ]
    fn from( error : Error ) -> Self
    {
      match error
      {
        Error::Js( value ) => value,
        other @ Error::DataUrl( _ ) => JsValue::from_str( &other.to_string() ),
      }
    }
  }

  /// Resolves `file_name` against `origin` according to `load`'s contract.
  ///
  /// * Absolute URLs (`http://`, `https://`, `//`) pass through unchanged.
  /// * Self-contained URLs (`blob:`, `data:`) pass through unchanged — these carry
  ///   their own payload and must reach `fetch` verbatim; prefixing the origin
  ///   mangles them into an unresolvable same-origin path.
  /// * Origin-absolute paths (leading `/`) are appended to the origin as-is.
  /// * Anything else is treated as origin-relative and joined with a single `/`.
  #[ must_use ]
  pub fn resolve_url( origin : &str, file_name : &str ) -> String
  {
    if is_self_contained_url( file_name )
    {
      file_name.to_string()
    }
    else if file_name.starts_with( '/' )
    {
      format!( "{origin}{file_name}" )
    }
    else
    {
      format!( "{origin}/{file_name}" )
    }
  }

  /// Returns `true` for URLs that already carry their own location and must never
  /// be prefixed with an origin or a folder path — doing so mangles them into an
  /// unresolvable same-origin path. Two subcategories qualify:
  /// * absolute (`http://`, `https://`), protocol-relative (`//`), and `blob:` URLs,
  ///   which reach `fetch` verbatim, and
  /// * self-contained `data:` payloads, which `load` decodes inline (see its `data:`
  ///   branch) and which never reach the network at all — `fetch` rejects `cors`
  ///   mode for `data:` URIs.
  ///
  /// Note that origin-absolute paths (a leading `/`) are deliberately *not* covered
  /// here: they carry no scheme and the caller still has to join them to an origin
  /// or pass them through, depending on context.
  #[ inline ]
  #[ must_use ]
  pub fn is_self_contained_url( url : &str ) -> bool
  {
    url.starts_with( "http://" )
    || url.starts_with( "https://" )
    || url.starts_with( "//" )
    || url.starts_with( "blob:" )
    || url.starts_with( "data:" )
  }

  /// Validates a `data:` URL and returns its base64-encoded payload (the text
  /// after the comma), without decoding it.
  ///
  /// `url` is expected to begin with the `data:` scheme — `load` only calls this
  /// after checking that prefix. Returns `Err` if the URL is malformed (no comma
  /// separating the header from the payload) or if the payload is not declared as
  /// base64 (`;base64` is the only supported encoding). The actual base64 decode
  /// is left to the caller, because it relies on the browser's `window.atob`.
  ///
  /// # Errors
  /// Returns `Err` when the URL has no comma separating header from payload, or
  /// when the header lacks the `;base64` marker.
  pub fn data_url_base64_payload( url : &str ) -> Result< &str, &'static str >
  {
    let comma_pos = url.find( ',' ).ok_or( "Malformed data URL: missing comma" )?;
    let header = &url[ "data:".len()..comma_pos ];
    if !header.ends_with( ";base64" )
    {
      return Err( "Only base64-encoded data URLs are supported" );
    }
    Ok( &url[ comma_pos + 1.. ] )
  }

  /// Asynchronously fetches a file over HTTP using the browser's `fetch` API,
  /// or decodes a `data:` URL inline without a network round-trip.
  ///
  /// The argument is used verbatim as a URL or path; no folder prefix is added.
  /// These forms are accepted:
  /// * Absolute URL (`http://...`, `https://...`, or protocol-relative `//...`) — fetched as-is.
  /// * `blob:` URL — fetched as-is using `fetch`.
  /// * `data:` URL (e.g. `data:application/octet-stream;base64,...`) — decoded directly
  ///   without a network request. Only base64-encoded payloads are supported; non-base64
  ///   data URLs return `Err`.
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
  /// * `file_name` - URL, data URI, or origin-relative path of the file to load.
  ///
  /// # Returns
  /// A `Result` which is either a `Vec<u8>` containing the file's byte data on success,
  /// or a typed [`Error`] on failure.
  ///
  /// # Errors
  /// Returns [`Error::Js`] wrapping the `JsValue` that the underlying request
  /// construction, `fetch`, or `Response::array_buffer` rejected with — typically a
  /// `TypeError` for network / CORS failures or a `DOMException` for aborted reads.
  /// HTTP error status codes (4xx, 5xx) do **not** produce an `Err` here; `fetch`
  /// resolves successfully and the caller receives the response body as `Ok`.
  /// For `data:` URLs, returns [`Error::DataUrl`] if the URL is malformed or does not
  /// use base64 encoding. `Error` converts into `JsValue` (preserving the original
  /// browser error object), so callers returning `Result< _, JsValue >` can keep using `?`.
  ///
  /// # Panics
  /// Panics only if the browser `window` object is unavailable (i.e. not running in
  /// a browsing context).
  #[ inline ]
  pub async fn load( file_name : &str ) -> Result< Vec< u8 >, Error >
  {
    let window = web_sys::window().unwrap();
    let origin = window.location().origin()?;
    let url = resolve_url( &origin, file_name );

    // `fetch()` rejects `cors` mode for `data:` URLs — decode them directly instead.
    if url.starts_with( "data:" )
    {
      let payload = data_url_base64_payload( &url ).map_err( Error::DataUrl )?;
      let decoded = window.atob( payload )?;
      // `atob` returns a DOMString whose code points are Latin-1 bytes
      // (U+0000–U+00FF per Web IDL). Each `char` scalar therefore fits in a `u8`,
      // so `c as u8` truncation is lossless and reconstructs the original binary
      // payload. Do not "simplify" this to byte-length arithmetic over the string:
      // wasm-bindgen transports the DOMString as UTF-8, where every U+0080–U+00FF
      // code point is two bytes, so `.bytes()` would corrupt non-ASCII payloads.
      return Ok( decoded.chars().map( | c | c as u8 ).collect() );
    }

    let opts = web_sys::RequestInit::new();
    opts.set_method( "GET" );
    opts.set_mode( web_sys::RequestMode::Cors );

    // Propagate request-construction and fetch failures as `Err` rather than
    // panicking: a rejected fetch (network error, CORS block, rate limit) must
    // not abort the whole wasm module — callers can recover from a returned error.
    let request = web_sys::Request::new_with_str_and_init( &url, &opts )?;

    let resp_value = JsFuture::from( window.fetch_with_request( &request ) ).await?;
    let resp : web_sys::Response = resp_value.dyn_into()?;
    let array_buffer_promise = resp.array_buffer()?;
    let array_buffer = JsFuture::from( array_buffer_promise ).await?;

    let uint8_array = js_sys::Uint8Array::new( &array_buffer );
    Ok( uint8_array.to_vec() )
  }
}

crate::mod_interface!
{

  own use load;
  own use resolve_url;
  own use data_url_base64_payload;
  own use is_self_contained_url;
  own use Error;

}
