//! Minwebgpu

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

#[ cfg( feature = "enabled" ) ]
pub use mingl::mod_interface;

#[ cfg( feature = "enabled" ) ]
mod private {}

#[ cfg( feature = "enabled" ) ]
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