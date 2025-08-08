//!
//! This crate provides a foundational toolkit for graphics and rendering applications.
//! It offers a modular structure with layers for derives, error handling, buffer and memory management,
//! data type descriptors, and optional features for camera controls and diagnostics.
//!
#![ doc( html_root_url = "https://docs.rs/mingl/latest/mingl/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Set of tools and helpers for working with WebGL and WebGPU" ) ]

// use error::prelude::*;
// use former::Former;
// use derive_tools::{ New };

#[ cfg( feature = "enabled" ) ]
mod private {}

#[ cfg( feature = "enabled" ) ]
::mod_interface::mod_interface!
{

  // own use ::error;
  // own use ::log;
  /// Re-exports the `mod_interface` macro for use in other modules.
  own use ::mod_interface::mod_interface;

  /// Derives.
  layer derive;

  /// Errors handling.
  layer error;
  /// Buffer-related.
  layer buffer;
  /// Descriptors of primitive data types.
  layer data_type;
  /// Memory-related entities.
  layer mem;

  /// Provides an orbit-style camera controller.
  #[ cfg( all( feature = "math", feature = "camera_orbit_controls" ) ) ]
  layer camera_orbit_controls;

  /// Includes diagnostic tools.
  #[ cfg( all( feature = "math", feature = "diagnostics" ) ) ]
  layer diagnostics;

  /// Utilities related to different models.
  #[ cfg( feature = "math" ) ]
  layer model;

  /// Multi-dimensional math.
  #[ cfg( feature = "math" ) ]
  layer math;

  /// Geometry math.
  #[ cfg( feature = "math" ) ]
  layer geometry;

  /// Web related stuff
  #[ cfg( feature = "web" ) ]
  layer web;

}
