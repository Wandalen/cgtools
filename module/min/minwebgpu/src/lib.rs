//! Minwebgpu - A minimal WebGPU toolkit for browser environments
//!
//! This crate is designed specifically for WebAssembly targets and browser environments.
//! It provides WebGPU bindings and utilities that only work when compiled for `wasm32-unknown-unknown`.
//!
//! For native targets, this crate provides stub implementations to enable compilation
//! without runtime functionality.

#![allow(clippy::wildcard_imports)]
#![allow(clippy::exhaustive_enums)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::implicit_return)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::min_ident_chars)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::let_and_return)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::elidable_lifetime_names)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::new_without_default)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::inconsistent_struct_constructor)]
#![allow(clippy::exhaustive_structs)]
#![allow(clippy::len_zero)]
#![allow(clippy::explicit_counter_loop)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::redundant_closure_for_method_calls)]

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
pub mod stub {
  //! Stub implementations for native targets
  //! 
  //! This module provides empty/stub implementations of the minwebgpu API
  //! to allow compilation on native targets without WebGPU support.
  //! All functions will return appropriate errors when called.
  
  /// Stub error type for native targets
  #[derive(Debug)]
  pub struct WebGPUNotAvailableError;
  
  impl std::fmt::Display for WebGPUNotAvailableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "WebGPU functionality is only available on WebAssembly targets")
    }
  }
  
  impl std::error::Error for WebGPUNotAvailableError {}
}

// Re-export stub for non-wasm targets when enabled
#[ cfg( all( feature = "enabled", not( target_arch = "wasm32" ) ) ) ]
pub use stub::*;