//! This module provides utility functions for interacting with browser-specific features,
//! such as event handling, that are common in WebAssembly applications.

use minwebgl as gl;
use gl::JsCast as _;
use web_sys::{ wasm_bindgen::prelude::Closure, EventTarget };

/// Disables the default browser context menu on a given event target.
///
/// This is useful to allow an application to implement its own right-click behavior
/// without being overridden by the browser's default menu.
///
/// # Arguments
/// * `target`: The `EventTarget` (e.g., an HTML element) on which to prevent the context menu.
pub fn prevent_rightclick( target : EventTarget )
{
  let prevent_default = | e : web_sys::Event | e.prevent_default();
  let prevent_default = Closure::< dyn Fn( _ ) >::new( prevent_default );
  target.add_event_listener_with_callback
  (
    "contextmenu",
    prevent_default.as_ref().unchecked_ref()
  ).unwrap();
  // The closure is intentionally "forgotten" to ensure it remains alive
  // and can be called by the browser's event loop.
  prevent_default.forget();
}
