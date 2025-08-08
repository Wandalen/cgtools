//! Ndarray extensions for computer graphics mathematics.
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

use ::mod_interface::mod_interface;

mod private
{
  // use super::*;
}

crate::mod_interface!
{

  /// Approximate equality for floating-point types can be determined using either relative difference
  /// or comparisons based on units in the last place (ULPs).
  layer approx;
  // own use super::approx;

  /// Derives.
  layer derive;
  // own use super::derive;
  // zzz : change to remove need to write explicitly that

  /// 2D entities, like matrix and vector.
  /// Not the same as 2D in CG
  layer d2;
  // own use super::d2;

  /// General math traits.
  layer general;

  /// Multidimensional space.
  layer md;
  // own use super::md;

  /// Multidimensional indices.
  layer index;
  // own use super::index;

  /// Memort-related things.
  layer mem;
  // own use super::mem;

  /// Ndarray things.
  layer nd;
  // own use super::nd;

  /// Vector things.
  layer vector;
  // own use super::vector;

  layer quaternion;

  prelude use ::ndarray::prelude::*;

}
