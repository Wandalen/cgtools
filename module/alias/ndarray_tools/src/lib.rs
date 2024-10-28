#![ doc = include_str!( "../readme.md" ) ]

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
