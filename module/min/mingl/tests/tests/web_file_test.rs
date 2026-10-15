//! Verifies `web::file`'s pure URL helpers — `url_resolve`'s resolution rules
//! ( pass-through for self-contained URLs, origin-absolute joining, and
//! document-relative joining against the current page's own directory, not
//! the origin root — see BUG-109 ) and `data_url_base64_payload`'s validation
//! of `data:` URLs. These pin the natively-testable logic deliberately
//! extracted from the wasm-only `load` ( which needs a browser `window` for
//! fetch/atob ). Relocated from inline `src/web/file.rs` per the
//! all-tests-in-tests/ convention; both helpers are exported at the
//! `web::file` module path for exactly this purpose.

use super::*;
use the_module::web::file::{ url_resolve, data_url_base64_payload };

#[ test ]
fn passes_https_url_through()
{
  assert_eq!
  (
    url_resolve( "https://app.example.com/minwebgl/text_msdf/", "https://cdn.example.com/foo.glb" ),
    "https://cdn.example.com/foo.glb"
  );
}

#[ test ]
fn passes_http_url_through()
{
  assert_eq!
  (
    url_resolve( "https://app.example.com/minwebgl/text_msdf/", "http://legacy.example.com/foo.glb" ),
    "http://legacy.example.com/foo.glb"
  );
}

#[ test ]
fn passes_protocol_relative_url_through()
{
  assert_eq!
  (
    url_resolve( "https://app.example.com/minwebgl/text_msdf/", "//cdn.example.com/foo.glb" ),
    "//cdn.example.com/foo.glb"
  );
}

#[ test ]
fn passes_blob_url_through()
{
  assert_eq!
  (
    url_resolve( "https://app.example.com/minwebgl/text_msdf/", "blob:https://app.example.com/uuid-1234" ),
    "blob:https://app.example.com/uuid-1234"
  );
}

#[ test ]
fn passes_data_url_through()
{
  assert_eq!
  (
    url_resolve( "https://app.example.com/minwebgl/text_msdf/", "data:application/octet-stream;base64,Z2xURg==" ),
    "data:application/octet-stream;base64,Z2xURg=="
  );
}

#[ test ]
fn joins_origin_absolute_path_discarding_page_directory()
{
  assert_eq!
  (
    url_resolve( "https://app.example.com/minwebgl/text_msdf/", "/assets/foo.glb" ),
    "https://app.example.com/assets/foo.glb"
  );
}

// test_kind: bug_reproducer(BUG-109)
/// ## Root Cause
/// `url_resolve` joined document-relative `file_name`s (no leading `/`, no
/// scheme) against `origin` (`window.location().origin()` — scheme+host+port
/// only) instead of the current page's own directory, so any page deployed
/// under a subpath resolved its relative asset fetches to the site root
/// instead of its own directory.
///
/// ## Why Not Caught
/// The original tests only ever exercised `base_href` values with no path
/// component (`"https://app.example.com"`), the domain-root case where
/// "origin" and "page directory" collapse to the same string — the
/// divergence only appears once `base_href` carries a subpath, which no
/// prior test constructed.
///
/// ## Fix Applied
/// `url_resolve` now takes the full current-document URL (`base_href`) and
/// resolves document-relative paths against `split_origin_and_dir(base_href)`'s
/// directory (path truncated after the final `/`) instead of the origin
/// alone; `load()` now passes `window.location().href()` instead of
/// `window.location().origin()`.
///
/// ## Prevention
/// RED state (empirically confirmed): reverting `url_resolve`'s
/// document-relative branch to join against `origin` alone (dropping `dir`)
/// makes this exact assertion fail, resolving to
/// `"https://app.example.com/static/foo.glb"` instead of the expected
/// subpath-prefixed URL.
///
/// ## Pitfall
/// Don't test `url_resolve` with only domain-root `base_href` values — that
/// input can't distinguish "joins against origin" from "joins against page
/// directory" since they're identical when there's no subpath; always
/// include at least one subpath-bearing `base_href` case.
#[ test ]
fn joins_document_relative_path_against_page_directory()
{
  assert_eq!
  (
    url_resolve( "https://app.example.com/minwebgl/text_msdf/", "static/foo.glb" ),
    "https://app.example.com/minwebgl/text_msdf/static/foo.glb"
  );
}

#[ test ]
fn joins_bare_filename_against_page_directory()
{
  assert_eq!
  (
    url_resolve( "https://app.example.com/minwebgl/text_msdf/", "foo.glb" ),
    "https://app.example.com/minwebgl/text_msdf/foo.glb"
  );
}

#[ test ]
fn truncates_base_href_filename_to_its_directory()
{
  assert_eq!
  (
    url_resolve( "https://app.example.com/minwebgl/text_msdf/index.html", "static/foo.glb" ),
    "https://app.example.com/minwebgl/text_msdf/static/foo.glb"
  );
}

#[ test ]
fn root_deployed_page_resolves_relative_path_at_origin_root()
{
  assert_eq!
  (
    url_resolve( "https://app.example.com/", "static/foo.glb" ),
    "https://app.example.com/static/foo.glb"
  );
}

#[ test ]
fn bare_origin_with_no_path_resolves_relative_path_at_origin_root()
{
  assert_eq!
  (
    url_resolve( "https://app.example.com", "static/foo.glb" ),
    "https://app.example.com/static/foo.glb"
  );
}

#[ test ]
fn empty_input_resolves_to_current_page_directory()
{
  assert_eq!
  (
    url_resolve( "https://app.example.com/minwebgl/text_msdf/", "" ),
    "https://app.example.com/minwebgl/text_msdf/"
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
