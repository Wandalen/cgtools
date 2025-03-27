//! Indices.

/// Internal namespace.
mod private
{
  use crate::*;

  /// A vector structure.
  #[ derive( Clone, Copy, PartialEq, PartialOrd, Hash, Debug ) ]
  pub struct Vector< E, const LEN : usize >( pub [ E; LEN ] )
  where E : MatEl;
  pub type F32x2 = Vector< f32, 2 >;
  pub type F32x3 = Vector< f32, 3 >;
  pub type F32x4 = Vector< f32, 4 >;
  pub type F64x2 = Vector< f64, 2 >;
  pub type F64x3 = Vector< f64, 3 >;
  pub type F64x4 = Vector< f64, 4 >;

  impl< E : MatEl, const LEN : usize > Default for Vector< E, LEN >
  {
    #[ inline( always ) ]
    fn default() -> Self
    {
      Vector( [ E::default() ; LEN ] )
    }
  }

  pub trait VectorSpace< const SIZE : usize >
  where
    Self : Collection + Indexable + VectorIter< < Self as Collection >::Scalar, SIZE >,
  {
  }

  impl< T, const SIZE : usize > VectorSpace< SIZE > for T
  where
    Self : Collection + Indexable + VectorIter< < Self as Collection >::Scalar, SIZE >,
  {
  }

  pub trait VectorSpaceMut< const SIZE : usize >
  where
    Self : VectorSpace< SIZE > + VectorIterMut< < Self as Collection >::Scalar, SIZE >,
  {
  }

  impl< T, const SIZE : usize > VectorSpaceMut< SIZE > for T
  where
    Self : VectorSpace< SIZE > + VectorIterMut< < Self as Collection >::Scalar, SIZE >,
  {
  }

}

crate::mod_interface!
{
  /// General trait implementation for the vector type
  layer general;
  /// General arithmetics for the vector type
  layer arithmetics;
  /// Conversions from `Array` type to `Vector`
  layer array;
  /// Functionality related to 2D vectors
  layer vec2;
  /// Functionality related to 3D vectors
  layer vec3;
  /// Functionality related to 4D vectors
  layer vec4;

  reuse ::mdmath_core::vector;

  exposed use
  {
    VectorSpace,
    VectorSpaceMut,
    Vector,
    F32x2,
    F32x3,
    F32x4,
    F64x2,
    F64x3,
    F64x4
  };

}
