//!
//! This crate provides a foundational toolkit for graphics and rendering applications.
//! It offers a modular structure with layers for derives, error handling, buffer and memory management,
//! data type descriptors, and optional features for camera controls and diagnostics.
//!
#![ doc = include_str!( "../readme.md" ) ]

#![allow(clippy::implicit_return)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::exhaustive_structs)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::min_ident_chars)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::identity_op)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::non_ascii_literal)]
#![allow(clippy::unnecessary_semicolon)]
#![allow(clippy::inconsistent_struct_constructor)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::std_instead_of_alloc)]
#![allow(clippy::new_without_default)]
#![allow(clippy::needless_return)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::exhaustive_enums)]
#![allow(clippy::elidable_lifetime_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::redundant_closure)]

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
