//! This crate serves as a facade for common web-related functionalities,
//! designed for use in WebAssembly applications. It re-exports essential web-sys
//! and js-sys types and organizes features into distinct layers, which can be
//! enabled via feature flags.

/// Internal namespace for implementation details.
mod private
{
}

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
