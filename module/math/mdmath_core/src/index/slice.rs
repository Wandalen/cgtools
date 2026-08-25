//! `AsIx2`/`AsIx3` for `&[ Ix ]`.
//!
//! UX/DX: unlike every other `AsIx2`/`AsIx3` implementor in this module (`array.rs`,
//! `prime.rs`, `tuple.rs` -- all fixed-length types, so conversion is infallible), a slice's
//! length is a runtime property. Both impls below panic when the input length doesn't match
//! the target dimensionality; each method now documents that contract explicitly via its own
//! `# Panics` section rather than leaving it discoverable only by reading the `match` arm.

use super::{ AsIx2, Ix, Ix2, AsIx3, Ix3 };

impl AsIx2 for &[ Ix ]
{
  /// Converts a 2-element slice into an [`Ix2`].
  ///
  /// # Panics
  /// Panics if `self.len() != 2`. Unlike the array/tuple/prime-type `AsIx2` impls, this
  /// conversion is fallible because a slice's length isn't known at compile time -- callers
  /// with a slice of unverified length must check `self.len()` themselves before calling, or
  /// use a fixed-size type (`[ Ix ; 2 ]`, `( Ix, Ix )`) instead where the compiler enforces it.
  #[ inline( always ) ]
  fn as_ix2( self ) -> Ix2
  {
    match self
    {
      &[ a, b ] => Ix2( a, b ),
      _ => panic!( "Slice must have exactly 2 elements for Ix2 conversion" ),
    }
  }
}

impl AsIx3 for &[ Ix ]
{
  /// Converts a 3-element slice into an [`Ix3`].
  ///
  /// # Panics
  /// Panics if `self.len() != 3`. Unlike the array/tuple/prime-type `AsIx3` impls, this
  /// conversion is fallible because a slice's length isn't known at compile time -- callers
  /// with a slice of unverified length must check `self.len()` themselves before calling, or
  /// use a fixed-size type (`[ Ix ; 3 ]`, `( Ix, Ix, Ix )`) instead where the compiler enforces it.
  #[ inline( always ) ]
  fn as_ix3( self ) -> Ix3
  {
    match self
    {
      &[ a, b, c ] => Ix3( a, b, c ),
      _ => panic!( "Slice must have exactly 3 elements for Ix3 conversion" ),
    }
  }
}
