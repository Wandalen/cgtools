//! This crate serves as a facade for common web-related functionalities,
//! designed for use in WebAssembly applications. It re-exports essential web-sys
//! and js-sys types and organizes features into distinct layers, which can be
//! enabled via feature flags.

/// Internal namespace for implementation details.
mod private
{
  /// Splits an absolute `href` ( e.g. `window.location().href()` ) into its
  /// origin ( `scheme://host[:port]` ) and the directory portion of its path —
  /// the path truncated after its final `/`. `https://host/a/b/c.html` splits
  /// into `https://host` and `/a/b/`; `https://host/a/b/` splits into the same
  /// pair unchanged; `https://host` ( no path at all ) yields directory `/`.
  pub( crate ) fn split_origin_and_dir( href : &str ) -> ( &str, &str )
  {
    let path_start = href.find( "://" )
    .and_then( | scheme_end | href[ scheme_end + 3.. ].find( '/' ).map( | i | scheme_end + 3 + i ) )
    .unwrap_or( href.len() );
    let ( origin, path ) = href.split_at( path_start );
    if path.is_empty()
    {
      return ( origin, "/" );
    }
    let dir_end = path.rfind( '/' ).map_or( 0, | i | i + 1 );
    ( origin, &path[ ..dir_end ] )
  }

  /// Returns `true` for URLs that already carry their own location and must never
  /// be prefixed with an origin or a folder path — doing so mangles them into an
  /// unresolvable same-origin path. Two subcategories qualify:
  /// * absolute (`http://`, `https://`), protocol-relative (`//`), and `blob:` URLs,
  ///   which reach `fetch` verbatim, and
  /// * self-contained `data:` payloads, which decode inline and never reach the
  ///   network at all.
  ///
  /// Note that origin-absolute paths (a leading `/`) are deliberately *not* covered
  /// here: they carry no scheme and the caller still has to join them to an origin
  /// or pass them through, depending on context.
  pub( crate ) fn is_self_contained_url( url : &str ) -> bool
  {
    url.starts_with( "http://" )
    || url.starts_with( "https://" )
    || url.starts_with( "//" )
    || url.starts_with( "blob:" )
    || url.starts_with( "data:" )
  }

  /// Resolves `file_name` against `base_href` — the current document's full
  /// URL.
  ///
  /// * Absolute URLs (`http://`, `https://`, `//`) pass through unchanged.
  /// * Self-contained URLs (`blob:`, `data:`) pass through unchanged — these carry
  ///   their own payload and must reach `fetch` verbatim; prefixing anything onto
  ///   them mangles them into an unresolvable path.
  /// * Origin-absolute paths (leading `/`) are appended to `base_href`'s origin,
  ///   discarding its path — matching how a browser resolves an absolute-path
  ///   URL reference.
  /// * Anything else is document-relative: joined to `base_href`'s own directory
  ///   ( its path truncated after the final `/` ), NOT to the origin root. A page
  ///   served at `/minwebgl/text_msdf/` resolves `"static/foo.json"` to
  ///   `/minwebgl/text_msdf/static/foo.json` — matching how a plain `<img src>`
  ///   or `fetch()` call on that page would resolve the same relative path.
  //
  // Fix(BUG-109): resolving document-relative paths against `origin` alone
  // (dropping the current page's own path) sent every subpath-deployed
  // example's asset fetches to the site root instead of its own directory.
  // Root cause: `window.location().origin()` never includes a path component
  // by definition — using it as the sole join target only works for pages
  // served at the domain root. Shared here (rather than kept local to
  // `web::file`) because `web::dom`'s element-creating functions have the
  // identical join requirement but compile under a strictly weaker feature
  // gate ( `web` alone ) than `web::file` ( `web` + `web_future` + `web_file` ) —
  // a `dom`-only build has no `file` module to call into.
  // Pitfall: don't reach for `origin` when resolving a *relative* reference —
  // relative references resolve against the current document's directory, not
  // its origin; only an absolute-path reference (leading `/`) targets the origin.
  pub( crate ) fn resolve_url( base_href : &str, file_name : &str ) -> String
  {
    if is_self_contained_url( file_name )
    {
      file_name.to_string()
    }
    else
    {
      let ( origin, dir ) = split_origin_and_dir( base_href );
      if file_name.starts_with( '/' )
      {
        format!( "{origin}{file_name}" )
      }
      else
      {
        format!( "{origin}{dir}{file_name}" )
      }
    }
  }
}

// Crate-internal re-export so sibling layers ( `dom`, `file` ) can share the
// URL-resolution logic above regardless of which layers their own feature
// gates enable — see `resolve_url`'s doc comment for why this can't live
// solely in `file.rs`.
pub( crate ) use private::{ resolve_url, is_self_contained_url };

// This macro organizes and exposes the public API of the module.
crate::mod_interface!
{
  // Re-exports of core WebAssembly and JavaScript interop crates and types.
  own use ::wasm_bindgen;
  own use ::web_sys;
  own use ::js_sys;
  own use ::wasm_bindgen::JsValue;

  /// Provides utilities for creating and managing the main application/render loop.
  layer exec_loop;
  /// Contains functions for interacting with the HTML5 Canvas element.
  layer canvas;
  /// Includes helpers for manipulating the Document Object Model (DOM).
  layer dom;

  /// Provides utilities for working with Rust Futures in a `wasm-bindgen` context.
  #[ cfg( feature = "web_future"  ) ]
  layer future;

  /// Offers tools for file handling, such as loading files from a web server.
  #[ cfg( all( feature = "web_future", feature = "web_file" ) ) ]
  layer file;

  /// Contains web-specific utilities for handling and reporting on 3D models.
  #[ cfg( all( feature = "math", feature = "web_future", feature = "web_file" ) ) ]
  layer model;

  /// Provides integration with the `console.log` API for logging from Rust.
  #[ cfg( feature = "web_log"  ) ]
  layer log;
}
