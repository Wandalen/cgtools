//! Verifies `web::file`'s pure URL helpers — `resolve_url`'s resolution rules
//! ( pass-through for self-contained URLs, origin joining for paths ) and
//! `data_url_base64_payload`'s validation of `data:` URLs. These pin the
//! natively-testable logic deliberately extracted from the wasm-only `load`
//! ( which needs a browser `window` for fetch/atob ). Relocated from inline
//! `src/web/file.rs` per the all-tests-in-tests/ convention; both helpers are
//! exported at the `web::file` module path for exactly this purpose.

use super::*;
use the_module::web::file::{ resolve_url, data_url_base64_payload };

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

#[ test ]
fn data_url_returns_base64_payload()
{
  assert_eq!
  (
    data_url_base64_payload( "data:application/octet-stream;base64,Z2xURg==" ),
    Ok( "Z2xURg==" )
  );
}

#[ test ]
fn data_url_with_empty_payload_is_ok()
{
  // A `;base64` header with nothing after the comma is a well-formed,
  // zero-length payload — `atob("")` returns the empty string.
  assert_eq!
  (
    data_url_base64_payload( "data:application/octet-stream;base64," ),
    Ok( "" )
  );
}

#[ test ]
fn data_url_without_comma_is_err()
{
  assert_eq!
  (
    data_url_base64_payload( "data:application/octet-stream;base64" ),
    Err( "Malformed data URL: missing comma" )
  );
}

#[ test ]
fn data_url_without_base64_marker_is_err()
{
  assert_eq!
  (
    data_url_base64_payload( "data:text/plain,Hello" ),
    Err( "Only base64-encoded data URLs are supported" )
  );
}
