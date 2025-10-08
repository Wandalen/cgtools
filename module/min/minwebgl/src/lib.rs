//! WebGL wrapper providing browser-based graphics functionality.
#![ doc( html_root_url = "https://docs.rs/minwebgl/latest/minwebgl/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "WebGL wrapper providing browser-based graphics functionality" ) ]

#![allow(clippy::unreadable_literal)]
#![allow(clippy::implicit_return)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::exhaustive_enums)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::elidable_lifetime_names)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::exhaustive_structs)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::new_without_default)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::inconsistent_struct_constructor)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::expect_fun_call)]
#![allow(clippy::from_iter_instead_of_collect)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::len_zero)]
#![allow(clippy::explicit_counter_loop)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::unit_arg)]
#![allow(clippy::incompatible_msrv)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::question_mark)]
#![allow(clippy::min_ident_chars)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::to_string_trait_impl)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::manual_unwrap_or_default)]
#![allow(clippy::manual_string_new)]
#![allow(clippy::std_instead_of_alloc)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::from_over_into)]

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

  /// Panic-related.
  layer panic;

  #[ cfg( feature = "web" ) ]
  /// Web-related.
  layer web;

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
  /// Drawbuffers function
  layer drawbuffers;
  /// Simple geometry
  layer geometry;
  /// Clean up the state
  layer clean;
  /// Blob creation shortcut
  layer blob;

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

  /// Multi-dimensional math.
  #[ cfg( feature = "math" ) ]
  layer math;

}
