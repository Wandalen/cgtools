//! Scalar conversions for `Vector` between numeric element types.
//!
//! Two flavors are provided:
//! - [`Vector::cast`] — component-wise conversion via [`From`]. Only available
//!   when the target type provides `From` for the source type (i.e. lossless).
//! - [`Vector::cast_as`] — component-wise conversion via
//!   [`num_traits::AsPrimitive`]. Used for lossy / explicit `as`-style
//!   conversions (`f64` → `i32`, `i64` → `i32`, etc.).
//!
//! For the common lossless pairings (matching `std::convert::From` between
//! primitives), concrete `From` impls are generated so `.into()` works without
//! ascription.

mod private
{
  use crate::*;
  use ::num_traits::AsPrimitive;

  impl< E, const N : usize > Vector< E, N >
  where
    E : MatEl,
  {

    /// Component-wise scalar conversion via [`From`]. Reserved for lossless
    /// primitive conversions (e.g. `i32 -> f64`, `u32 -> u64`). For lossy
    /// conversions use [`Vector::cast_as`].
    #[ inline ]
    pub fn cast< T >( self ) -> Vector< T, N >
    where
      T : MatEl + From< E >,
    {
      Vector( self.0.map( T::from ) )
    }

    /// Component-wise scalar conversion via [`num_traits::AsPrimitive`].
    ///
    /// Mirrors the `as` keyword. For float → integer conversions this is *not*
    /// plain truncation at the edges: in-range floats truncate toward zero,
    /// while out-of-range values follow Rust's saturating cast rules
    /// (`NaN → 0`, `+∞`/above-max → `T::MAX`, `-∞`/below-min → `T::MIN`). See the
    /// Rust Reference, "Type cast expressions", for the full specification.
    #[ inline ]
    pub fn cast_as< T >( self ) -> Vector< T, N >
    where
      T : MatEl + 'static + Copy,
      E : AsPrimitive< T >,
    {
      Vector( self.0.map( AsPrimitive::as_ ) )
    }
  }

  /// Concrete lossless `From` impls between element types.
  ///
  /// The set of `( SrcE, DstE )` pairs mirrors `std::convert::From` between the
  /// corresponding primitives, so `.into()` resolves without ascription.
  macro_rules! impl_vector_from
  {
    ( $( $src:ty => $dst:ty ),* $(,)? ) =>
    {
      $(
        impl< const N : usize > From< Vector< $src, N > > for Vector< $dst, N >
        {
          #[ inline ]
          fn from( value : Vector< $src, N > ) -> Self
          {
            value.cast::< $dst >()
          }
        }
      )*
    };
  }

  impl_vector_from!
  {
    // Integer widening
    i32 => i64,
    u32 => u64,
    u32 => i64,
    // Integer to float (lossless under 2^53 for i64 → f64; std mirrors this
    // by only providing the i32/u32 → f64 directions losslessly).
    i32 => f64,
    u32 => f64,
    // Float widening
    f32 => f64,
  }
}

crate::mod_interface!
{
}
