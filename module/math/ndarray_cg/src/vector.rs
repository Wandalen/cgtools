//! Indices.

/// Internal namespace.
mod private
{
  use crate::*;

  /// A vector structure.
  #[ derive( Clone, Copy, PartialEq, PartialOrd, Hash ) ]
  pub struct Vector< E, const LEN : usize >( pub [ E; LEN ] )
  where E : MatEl;
  pub type Vec2< E > = Vector< E, 2 >;
  pub type Vec3< E > = Vector< E, 3 >;
  pub type Vec4< E > = Vector< E, 4 >;

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
  layer general;
  layer arithmetics;
  layer array;
  layer vec3;

  reuse ::mdmath_core::vector;

  exposed use
  {
    VectorSpace,
    VectorSpaceMut,
    Vector,
    Vec2,
    Vec3,
    Vec4
  };

}
