//! Core multidimensional mathematics library.
#![ doc( html_root_url = "https://docs.rs/mdmath_core/latest/mdmath_core/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Core multidimensional mathematics library" ) ]

#![allow(clippy::implicit_return)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::default_numeric_fallback)]
#![allow(clippy::missing_trait_methods)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::panic)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::as_conversions)]
#![allow(clippy::needless_maybe_sized)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::blanket_clippy_restriction_lints)]
#![allow(clippy::needless_return)]
#![allow(clippy::transmute_ptr_to_ptr)]
#![allow(clippy::elidable_lifetime_names)]
#![allow(clippy::if_then_some_else_none)]
#![allow(clippy::borrow_as_ptr)]

#[ cfg( feature = "enabled" ) ]
use ::mod_interface::mod_interface;

#[ cfg( feature = "enabled" ) ]
mod private
{
}

#[ cfg( feature = "enabled" ) ]
crate::mod_interface!
{

  /// Approximate equality for floating-point types can be determined using either relative difference
  /// or comparisons based on units in the last place (ULPs).
  #[ cfg( feature = "approx" ) ]
  layer approx;

  /// Multidimensional indices.
  #[ cfg( feature = "index" ) ]
  layer index;

  /// Describe general floats and operations on them.
  #[ cfg( feature = "float" ) ]
  layer float;

  /// General math traits.
  #[ cfg( feature = "general" ) ]
  layer general;

  /// Reusing nd_array.
  #[ cfg( feature = "nd" ) ]
  layer nd;

  /// Strides for plain multidemnsional space.
  layer plain;

  /// General traits, not necessarily special for math.
  layer traits;

  /// Univeral vector.
  layer vector;

}
