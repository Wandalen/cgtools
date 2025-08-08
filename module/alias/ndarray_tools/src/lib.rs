//! Ndarray-based tools and utilities for numerical computing.
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

use ::mod_interface::mod_interface;

mod private
{
  // use super::*;
}

crate::mod_interface!
{
  /// Reusing main crate.
  reuse ::ndarray_cg;
}
