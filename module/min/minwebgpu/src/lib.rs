//! Minwebgpu - A minimal WebGPU toolkit for browser environments
//!
//! This crate is designed specifically for WebAssembly targets and browser environments.
//! It provides WebGPU bindings and utilities that only work when compiled for `wasm32-unknown-unknown`.
//!
//! For native targets, this crate provides stub implementations to enable compilation
//! without runtime functionality.

// WebAssembly target - full WebGPU functionality
#[ cfg( all( feature = "enabled", target_arch = "wasm32" ) ) ]
pub use mingl::mod_interface;

#[ cfg( all( feature = "enabled", target_arch = "wasm32" ) ) ]
mod private {}

#[ cfg( all( feature = "enabled", target_arch = "wasm32" ) ) ]
mod_interface!
{
  own use ::wasm_bindgen;
  own use ::web_sys;
  own use ::js_sys;
  own use ::wasm_bindgen::JsValue;
  // `Into`/`IntoIterator`/`TryInto` must exist at the crate root
  // unconditionally: dozens of modules import them via
  // `use crate::{ ..., Into, ... }`, but without these lines they are only
  // present when the optional `math` layer's `reuse ::mingl::math` happens
  // to carry them in transitively — every no-math build fails.
  own use ::mingl::Into;
  own use ::mingl::IntoIterator;
  own use ::mingl::TryInto;

  /// Error related stuff
  layer error;
  /// Canvas related stuff
  layer canvas;
  /// Browser realted stuff
  layer browser;
  /// Functionality for executing the rendering loop
  layer exec_loop;

  /// Functionality for asynchronous programmimng
  #[ cfg( feature = "future" ) ]
  layer future;
  
  /// Dom related
  layer dom;
  /// Webgpu Textures
  layer texture;
  /// Webgpu Descriptors
  layer descriptor;
  /// Context related
  layer context;

  /// Reimported types from web_sys
  layer webgpu;

  /// Webgpu sampler
  layer sampler;
  /// Mingl model handling
  layer model;
  /// Functionality for hangling files
  #[ cfg( feature = "file" ) ]
  layer file;
  /// Webgpu layouts
  layer layout;
  /// State objects
  layer state;
  /// Shader objects
  layer shader;
  /// Types of bindings
  layer binding_type;
  /// Render pipeline related
  layer render_pipeline;
  /// Render pass related
  layer render_pass;
  /// Queue related
  layer queue;
  /// Bindgroup related
  layer bind_group;
  /// Bind group entry related
  layer bind_group_entry;
  /// Module for converting crate types to web_sys types
  layer transform;
  /// Buffer related
  layer buffer;
  /// Low level data manipulation
  layer mem;
  /// Logging
  layer log;
  /// Math functionality
  #[ cfg( feature = "math" ) ]
  layer math;
  /// Compute pipeline related
  layer compute_pipeline;
}

// Native target stub - provides minimal compatibility without WebGPU functionality
#[ cfg( all( feature = "enabled", not( target_arch = "wasm32" ) ) ) ]
pub mod stub
{
  //! Stub implementations for native targets
  //!
  //! This module provides empty/stub implementations of the minwebgpu API
  //! to allow compilation on native targets without WebGPU support.
  //! All functions will return appropriate errors when called.

  /// Stub error type for native targets.
  // Constructed via `new()`, not the unit-struct literal, so this stays
  // `#[non_exhaustive]` — downstream native-target stub code can match on it
  // without depending on it having exactly zero fields forever.
  #[ non_exhaustive ]
  #[ derive( Debug, Default ) ]
  pub struct WebGPUNotAvailableError;

  impl WebGPUNotAvailableError
  {
    /// Creates a new `WebGPUNotAvailableError`.
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self::default()
    }
  }

  impl std::fmt::Display for WebGPUNotAvailableError
  {
    #[ inline ]
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      write!( f, "WebGPU functionality is only available on WebAssembly targets" )
    }
  }

  impl std::error::Error for WebGPUNotAvailableError {}
}

// Re-export stub for non-wasm targets when enabled
#[ cfg( all( feature = "enabled", not( target_arch = "wasm32" ) ) ) ]
pub use stub::*;

// Math is pure CPU-side linear algebra with nothing browser-bound, so it
// stays reachable off-wasm — native consumers ( e.g. the renderer's wgpu
// backend ) keep the same `minwebgpu::math` path browser code uses.
#[ cfg( all( feature = "enabled", feature = "math", not( target_arch = "wasm32" ) ) ) ]
pub use mingl::math;