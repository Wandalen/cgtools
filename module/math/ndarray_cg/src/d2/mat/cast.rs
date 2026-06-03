//! Scalar conversions for `Mat` between numeric element types.
//!
//! Mirrors the vector conversions in [`crate::vector::cast`].

mod private
{
  use crate::*;
  use ::num_traits::AsPrimitive;

  impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > Mat< ROWS, COLS, E, Descriptor >
  where
    E : MatEl,
    Self : RawSlice< Scalar = E >,
  {

    /// Component-wise scalar conversion via [`From`]. Reserved for lossless
    /// primitive conversions. For lossy conversions use [`Mat::cast_as`].
    #[ inline ]
    pub fn cast< T >( self ) -> Mat< ROWS, COLS, T, Descriptor >
    where
      T : MatEl + From< E >,
      Mat< ROWS, COLS, T, Descriptor > : RawSliceMut< Scalar = T >,
    {
      let mut out : Mat< ROWS, COLS, T, Descriptor > = Mat::default();
      for ( o, s ) in out.raw_slice_mut().iter_mut().zip( self.raw_slice().iter() )
      {
        *o = T::from( *s );
      }
      out
    }

    /// Component-wise scalar conversion via [`num_traits::AsPrimitive`].
    ///
    /// Mirrors the `as` keyword. For float → integer conversions this is *not*
    /// plain truncation at the edges: in-range floats truncate toward zero,
    /// while out-of-range values follow Rust's saturating cast rules
    /// (`NaN → 0`, `+∞`/above-max → `T::MAX`, `-∞`/below-min → `T::MIN`). See the
    /// Rust Reference, "Type cast expressions", for the full specification.
    #[ inline ]
    pub fn cast_as< T >( self ) -> Mat< ROWS, COLS, T, Descriptor >
    where
      T : MatEl + 'static + Copy,
      E : AsPrimitive< T >,
      Mat< ROWS, COLS, T, Descriptor > : RawSliceMut< Scalar = T >,
    {
      let mut out : Mat< ROWS, COLS, T, Descriptor > = Mat::default();
      for ( o, s ) in out.raw_slice_mut().iter_mut().zip( self.raw_slice().iter() )
      {
        *o = s.as_();
      }
      out
    }
  }

  /// Concrete lossless `From` impls between element types for `Mat`.
  macro_rules! impl_mat_from
  {
    ( $( $src:ty => $dst:ty ),* $(,)? ) =>
    {
      $(
        impl< const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor >
          From< Mat< ROWS, COLS, $src, Descriptor > > for Mat< ROWS, COLS, $dst, Descriptor >
        where
          Mat< ROWS, COLS, $src, Descriptor > : RawSlice< Scalar = $src >,
          Mat< ROWS, COLS, $dst, Descriptor > : RawSliceMut< Scalar = $dst >,
        {
          #[ inline ]
          fn from( value : Mat< ROWS, COLS, $src, Descriptor > ) -> Self
          {
            value.cast::< $dst >()
          }
        }
      )*
    };
  }

  impl_mat_from!
  {
    i32 => i64,
    u32 => u64,
    u32 => i64,
    i32 => f64,
    u32 => f64,
    f32 => f64,
  }
}

crate::mod_interface!
{
}
