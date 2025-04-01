//! Provides functionality for converting multidimensional indices
//! into flat offsets, particularly useful for operations involving
//! multidimensional arrays or grids.

/// Internal namespace.
mod private
{
  use crate::*;
  use core::
  {
    fmt,
    ops::{ Add, Mul },
    cmp::PartialOrd,
  };

  /// Trait for converting a multidimensional index into a flat offset.
  ///
  /// This trait is implemented for arrays of any dimension, allowing conversion of
  /// a multidimensional index into a single linear offset. It is useful for mapping
  /// coordinates in a multidimensional space to a flat array.
  pub trait DimOffset< const N : usize > : Collection
  {
    /// Converts a multidimensional index into a flat offset.
    ///
    /// # Arguments
    ///
    /// - `md_index`: A reference to a vector representing the multidimensional index.
    ///
    /// # Returns
    ///
    /// A scalar value representing the flat offset.
    fn offset< V2 >( &self, md_index : &V2 ) -> Self::Scalar
    where
      V2 : ArrayRef< Self::Scalar, N > + Collection< Scalar = Self::Scalar > + fmt::Debug + ?Sized;
  }

  /// Implementation of `DimOffset` for arrays of any dimension.
  ///
  /// This implementation calculates the flat offset by iterating over the dimensions
  /// in reverse order, ensuring that the most significant dimension is processed last.
  impl< E, V, const N : usize > DimOffset< N > for V
  where
    Self : Collection< Scalar = E > + fmt::Debug,
    E : Mul< E, Output = E > + Add< E, Output = E > + PartialOrd + Copy + Default + From< u8 >,
    V : ArrayRef< E, N > + Collection< Scalar = E > + fmt::Debug + ?Sized,
  {
    #[ inline ]
    fn offset< V2 >( &self, md_index : &V2 ) -> Self::Scalar
    where
      V2 : ArrayRef< E, N > + Collection< Scalar = Self::Scalar > + fmt::Debug + ?Sized,
    {
      let mut offset = E::default();
      let mut stride = E::from( 1 ); // Start with a stride of 1
      for i in 0..N
      {
        let dim_index = N - 1 - i; // Use components in reverse order
        debug_assert!
        (
          md_index.array_ref()[ dim_index ] < self.array_ref()[ dim_index ],
          "md_index : {md_index:?} | md_size : {self:?}"
        );
        offset = offset + stride * md_index.array_ref()[ dim_index ];
        stride = stride * self.array_ref()[ dim_index ];
      }
      offset
    }
  }

}

crate::mod_interface!
{
  own use DimOffset;
}
