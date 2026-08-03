//! Ndarray extensions for computer graphics mathematics.
#![ doc( html_root_url = "https://docs.rs/ndarray_cg/latest/ndarray_cg/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Ndarray extensions for computer graphics mathematics" ) ]

#![allow(clippy::implicit_return)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::default_numeric_fallback)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::similar_names)]
#![allow(clippy::same_name_method)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::blanket_clippy_restriction_lints)]
#![allow(clippy::needless_return)]
#![allow(clippy::transmute_ptr_to_ptr)]
#![allow(clippy::elidable_lifetime_names)]
#![allow(clippy::if_then_some_else_none)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::explicit_deref_methods)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::single_char_lifetime_names)]
#![allow(clippy::should_implement_trait)]

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

  // Re-export ndarray prelude items individually, excluding `ArrayRef`,
  // `LayoutRef`, `RawRef`, and the `ArrayRefN` aliases that ndarray 0.17
  // introduced — `ArrayRef` collides with our local `ArrayRef` trait
  // (see `mdmath_core::vector::ArrayRef`), which is used throughout this
  // crate as a trait bound.
  prelude use ::ndarray::prelude::
  {
    ArcArray,
    Array, Array0, Array1, Array2, Array3, Array4, Array5, Array6, ArrayD,
    ArrayBase,
    ArrayView, ArrayView0, ArrayView1, ArrayView2, ArrayView3, ArrayView4, ArrayView5, ArrayView6, ArrayViewD,
    ArrayViewMut, ArrayViewMut0, ArrayViewMut1, ArrayViewMut2, ArrayViewMut3, ArrayViewMut4, ArrayViewMut5, ArrayViewMut6, ArrayViewMutD,
    CowArray,
    RawArrayView, RawArrayViewMut,
    Axis, Dim, Dimension,
    Ix0, Ix1, Ix2, Ix3, Ix4, Ix5, Ix6, IxDyn,
    arr0, arr1, arr2, aview0, aview1, aview2, aview_mut1,
    array, azip, s,
    ShapeBuilder,
    NewAxis,
    AsArray,
    NdFloat,
  };

}
