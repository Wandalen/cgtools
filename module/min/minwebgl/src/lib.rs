#![ doc = include_str!( "../readme.md" ) ]

#[ cfg( feature = "enabled" ) ]
pub use mingl::mod_interface;

#[ cfg( feature = "enabled" ) ]
mod private
{

  // use ::web_sys::WebGl2RenderingContext::DYNAMIC_DRAW;

}

#[ cfg( feature = "enabled" ) ]
mod_interface!
{

  own use ::wasm_bindgen;
  own use ::web_sys;
  own use ::js_sys;
  own use ::wasm_bindgen::JsValue;

  /// Attribute-related.
  layer attribute;
  /// Derive-related.
  layer derive;
  /// Error-related.
  layer error;
  /// Browser-related.
  layer browser;
  /// Buffer-related.
  layer buffer;
  /// Operations on canvas.
  layer canvas;
  /// Operations on WebGL context.
  layer context;
  /// Descriptors of primitive data types.
  layer data_type;
  /// Operations on DOM elements.
  layer dom;
  /// Loop-related.
  layer exec_loop;

  /// Vertex indices.
  layer index;

  /// Logger-related.
  layer log;
  /// Memory-related entities.
  layer mem;
  /// Multi-dimensional math.
  #[ cfg( feature = "math" ) ]
  layer math;
  /// Panic-related.
  layer panic;
  /// Program-related entities and functionality.
  layer program;
  /// Program-related entities and functionality.
  layer shader;
  /// Vertex array object related.
  layer vao;
  /// Uniform buffer object related.
  layer ubo;
  /// Uniform-related.
  layer uniform;
  /// General WebGL things. Unfortunetely `web-sys` does not allow reuse constants, so them are duplicated in this file.
  layer webgl;
  /// Texture related functions
  layer texture;
  /// Color Attachment structure
  layer color_attachments;

  /// Useful information about your objects
  #[ cfg( feature = "diagnostics" ) ]
  layer diagnostics;

  /// Utils related to different models
  #[ cfg( all( feature = "future", feature = "file" ) ) ]
  layer model;

  /// File processing.
  #[ cfg( all( feature = "future", feature = "file" ) ) ]
  layer file;

  /// Future processing.
  #[ cfg( feature = "future" ) ]
  layer future;

}
