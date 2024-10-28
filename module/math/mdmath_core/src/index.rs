//! Provides functionality for converting various types into multidimensional indices.
//! This module is particularly useful for operations involving multidimensional arrays or grids.

/// Internal namespace.
mod private
{
  use ::ndarray::
  {
    Ix2, Ix3,
  };

  /// Trait for converting a type into a 2-dimensional index (`Ix2`).
  ///
  /// This trait is implemented for various types, allowing them to be converted
  /// into a 2D index, which is useful for indexing into 2D arrays or matrices.
  pub trait AsIx2
  {
    /// Converts the implementing type into a 2D index (`Ix2`).
    ///
    /// # Returns
    /// - `Ix2`: The 2-dimensional index.
    fn as_ix2( self ) -> Ix2;
  }

  /// Trait for converting a type into a 3-dimensional index (`Ix3`).
  ///
  /// This trait is implemented for various types, allowing them to be converted
  /// into a 3D index, which is useful for indexing into 3D arrays or grids.
  pub trait AsIx3
  {
    /// Converts the implementing type into a 3D index (`Ix3`).
    ///
    /// # Returns
    /// - `Ix3`: The 3-dimensional index.
    fn as_ix3( self ) -> Ix3;
  }
}

mod array;
mod prime;
mod slice;
mod tuple;

crate::mod_interface!
{
  exposed use
  {
    AsIx2, AsIx3,
  };
  exposed use ::ndarray::
  {
    Ix,
    Ix0, Ix1, Ix2, Ix3, Ix4, Ix5, Ix6,
  };
}