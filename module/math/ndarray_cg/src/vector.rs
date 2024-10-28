//! Indices.

/// Internal namespace.
mod private
{
  use crate::*;

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

  reuse ::mdmath_core::vector;

  exposed use
  {
    VectorSpace,
    VectorSpaceMut,
  };

}
