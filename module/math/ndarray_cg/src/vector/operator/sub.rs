mod private
{
  use crate::*;
  // use vector::arithmetics::inner_product::*;
  use vector::{ sub, sub_scalar };

  // Vector - Vector
  impl< E, const LEN : usize > Sub for Vector< E, LEN >
  where
    E : MatNum
  {
    type Output = Self;

    /// # Overflow
    /// For integer `E` the element-wise subtraction is not overflow-checked: it
    /// panics in debug / wraps in release on under/overflow — e.g. unsigned
    /// underflow when a component of `rhs` exceeds the matching component.
    fn sub( self, rhs : Self ) -> Self::Output
    {
      sub( &self, &rhs )
    }
  }

  // &Vector - &Vector
  impl< E, const LEN : usize > Sub for &Vector< E, LEN >
  where
    E : MatNum
  {
    type Output = Vector< E, LEN >;

    /// # Overflow
    /// For integer `E` the element-wise subtraction is not overflow-checked: it
    /// panics in debug / wraps in release on under/overflow — e.g. unsigned
    /// underflow when a component of `rhs` exceeds the matching component.
    fn sub( self, rhs : Self ) -> Self::Output
    {
      sub( self, rhs )
    }
  }

  // Vector -= Vector
  impl< E, const LEN : usize > SubAssign for Vector< E, LEN >
  where
    E : MatNum
  {
    /// # Overflow
    /// For integer `E` the element-wise subtraction is not overflow-checked: it
    /// panics in debug / wraps in release on under/overflow — e.g. unsigned
    /// underflow when a component of `rhs` exceeds the matching component.
    fn sub_assign( &mut self, rhs : Self )
    {
      *self = *self - rhs;
    }
  }

  // Vector - scalar
  impl< E, const LEN : usize > Sub< E > for Vector< E, LEN >
  where
    E : MatNum
  {
    type Output = Self;

    /// # Overflow
    /// For integer `E` the element-wise subtraction is not overflow-checked: it
    /// panics in debug / wraps in release on under/overflow — e.g. unsigned
    /// underflow when `rhs` exceeds a component.
    fn sub( self, rhs : E ) -> Self::Output
    {
      sub_scalar( &self, rhs )
    }
  }
}

crate::mod_interface!
{

}