//! Minwebgpu

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