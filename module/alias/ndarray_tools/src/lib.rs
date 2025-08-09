//! Ndarray-based tools and utilities for numerical computing.
#![ doc( html_root_url = "https://docs.rs/ndarray_tools/latest/ndarray_tools/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Ndarray-based tools and utilities for numerical computing" ) ]

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
