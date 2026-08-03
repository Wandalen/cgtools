mod private
{
  use crate::*;
// use vector::arithmetics::inner_product::*;
  use vector::{ sum, sum_scalar };

  impl< E, const LEN : usize > Add for Vector< E, LEN >
  where
  E : MatNum
  {
    type Output = Self;

    /// # Overflow
    /// For integer `E` the element-wise addition is not overflow-checked: it
    /// panics in debug / wraps in release once a sum leaves `E`'s range.
    fn add( self, rhs : Self ) -> Self::Output
    {
        sum( &self, &rhs )
    }
  }

  impl< E, const LEN : usize > Add< E > for Vector< E, LEN >
  where
  E : MatNum
  {
    type Output = Self;

    /// # Overflow
    /// For integer `E` the element-wise addition is not overflow-checked: it
    /// panics in debug / wraps in release once a sum leaves `E`'s range.
    fn add( self, rhs : E ) -> Self::Output
    {
        sum_scalar( &self, rhs )
    }
  }

  impl< E, const LEN : usize > Add for &Vector< E, LEN >
  where
    E : MatNum
  {
    type Output = Vector< E, LEN >;

    /// # Overflow
    /// For integer `E` the element-wise addition is not overflow-checked: it
    /// panics in debug / wraps in release once a sum leaves `E`'s range.
    fn add( self, rhs : Self ) -> Self::Output {
      sum( self, rhs )
    }
  }

  impl< E, const LEN : usize > AddAssign for Vector< E, LEN >
  where
  E : MatNum
  {
    /// # Overflow
    /// For integer `E` the element-wise addition is not overflow-checked: it
    /// panics in debug / wraps in release once a sum leaves `E`'s range.
    fn add_assign( &mut self, rhs : Self )
    {
        *self = *self + rhs;
    }
  }

}

crate::mod_interface!
{

}