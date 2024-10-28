/// Internal namespace.
mod private
{
  use crate::*;

  /// Trait for representing and manipulating rotations.
  ///
  /// This trait provides methods for creating and manipulating rotations, allowing
  /// for operations such as aligning vectors, rotating vectors, and inverting rotations.
  pub trait Rotation< const SIZE : usize >
  where
    Self : Collection,
  {
    /// The size of the vector space.
    const SIZE : usize = SIZE;

    /// Creates a rotation that aligns the `dir` vector with the forward direction,
    /// using `up` as the reference for the up direction.
    ///
    /// # Parameters
    /// - `dir`: The direction vector to align with the forward direction.
    /// - `up`: The reference up vector.
    ///
    /// # Returns
    /// - A rotation that aligns `dir` with the forward direction.
    fn look_at< Dir, Up >( dir : &Dir, up : &Up ) -> Self
    where
      Dir : VectorSpace< SIZE > + Collection< Scalar = < Self as Collection >::Scalar >,
      Up : VectorSpace< SIZE > + Collection< Scalar = < Self as Collection >::Scalar >;

    /// Creates a rotation that aligns vector `a` with vector `b`.
    ///
    /// # Parameters
    /// - `a`: The initial vector.
    /// - `b`: The target vector.
    ///
    /// # Returns
    /// - A rotation that aligns `a` with `b`.
    fn between_vectors< A, B >( a : &A, b : &B ) -> Self
    where
      A : VectorSpace< SIZE > + Collection< Scalar = < Self as Collection >::Scalar >,
      B : VectorSpace< SIZE > + Collection< Scalar = < Self as Collection >::Scalar >;

    /// Rotates a vector by this rotation.
    ///
    /// # Parameters
    /// - `vec`: The vector to rotate.
    ///
    /// # Returns
    /// - The rotated vector.
    fn rotate_vector< V >( &self, vec : &mut V )
    where
      V : VectorSpace< SIZE > + Collection< Scalar = < Self as Collection >::Scalar >;

    /// Inverts this rotation.
    ///
    /// # Returns
    /// - The inverse of this rotation.
    fn invert( &self ) -> Self;
  }

//   /// Creates a rotation that aligns vector `a` with vector `b` in place.
//   ///
//   /// # Parameters
//   /// - `dst`: The destination where the rotation will be stored.
//   /// - `a`: The initial vector.
//   /// - `b`: The target vector.
//   pub fn inplace_between_vectors< Dst, A, B, const SIZE : usize >( _dst : &mut Dst, _a : &A, _b : &B )
//   where
//     Dst : IndexingMut + MatWithShapeMut< SIZE, SIZE >,
//     A : VectorSpace< SIZE >,
//     B : VectorSpace< SIZE >,
//   {
//   }
//
//   /// Creates a rotation that aligns the `dir` vector with the forward direction in place,
//   /// using `up` as the reference for the up direction.
//   ///
//   /// # Parameters
//   /// - `dst`: The destination where the rotation will be stored.
//   /// - `dir`: The direction vector to align with the forward direction.
//   /// - `up`: The reference up vector.
//   pub fn inplace_look_at< Dst, Dir, Up, const SIZE : usize >( _dst : &mut Dst, _dir : &Dir, _up : &Up )
//   where
//     Dst : IndexingMut + MatWithShapeMut< SIZE, SIZE >,
//     Dir : VectorSpace< SIZE >,
//     Up : VectorSpace< SIZE >,
//   {
//   }

}

crate::mod_interface!
{
  exposed use
  {
    Rotation,
  };
  // own use
  // {
  //   inplace_between_vectors,
  //   inplace_look_at,
  // };
}
