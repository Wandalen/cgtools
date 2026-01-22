//! This module provides utility structures and functions for interfacing Rust data
//! with a web browser environment, particularly for use with `wasm-bindgen`.
//! It includes a generic wrapper `ForBrowser` to make Rust types more accessible to JavaScript.

/// Internal namespace for implementation details.
mod private
{
  use std::fmt::Debug;

  /// A generic wrapper struct designed to make Rust types compatible with browser-side code.
  ///
  /// This is often used with `#[wasm_bindgen]` to expose a Rust type and its data to JavaScript.
  /// The wrapped type `T` must implement `Debug`.
  #[ derive( Debug ) ]
  pub struct ForBrowser< T : Debug >
  {
    /// The wrapped data report or object.
    pub report : T
  }

  impl< T : Debug > ForBrowser< T >
  {
    /// Creates a new `ForBrowser` instance, wrapping the provided data.
    pub fn new( r : T ) -> Self
    {
      ForBrowser
      {
        report : r
      }
    }

    /// A convenience constructor to create a `ForBrowser` instance from a report.
    pub fn from_report( r : T ) -> Self
    {
      ForBrowser::new( r )
    }

    /// Converts a `Vec` of items into a `Vec` of `ForBrowser`-wrapped items.
    pub fn from_reports( r : Vec< T > ) -> Vec< Self >
    {
      r.into_iter().map( | r | ForBrowser::new( r ) ).collect()
    }
  }
}

// This macro organizes and exposes the public interface of the module.
crate::mod_interface!
{
  /// Provides web-specific utilities for handling OBJ model data.
  /// This layer is only available when the "web_model_obj" feature is enabled.
  #[ cfg( feature = "web_model_obj" ) ]
  layer obj;

  /// Exposes the `ForBrowser` wrapper for public use.
  own use
  {
    ForBrowser
  };
}
