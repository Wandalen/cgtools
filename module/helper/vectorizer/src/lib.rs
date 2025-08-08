//! Vector graphics processing and conversion utilities.
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::as_conversions)]
#![allow(clippy::min_ident_chars)]
#![allow(clippy::question_mark_used)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::implicit_return)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::exhaustive_enums)]
#![allow(clippy::print_stdout)]
#![allow(clippy::default_numeric_fallback)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::missing_panics_doc)]

#[ cfg( feature = "enabled" ) ]
mod private {}

#[ cfg( feature = "enabled" ) ]
::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;
  own use ::visioncortex;
  own use ::palette;
  own use ::fastrand;
  
  reuse ::error_tools as error;

  /// Module for converting to svg
  layer svg;
  /// Vectorization functionality
  layer actions;
  /// Module for dealing with CLI commands
  layer commands;
}