mod private
{
  use crate::*;
  // use vector::arithmetics::inner_product::*;
  use vector::{ div_scalar, div_mut };

  impl< E, const LEN : usize > Div< E > for Vector< E, LEN >
  where
    E : MatNum
  {
    type Output = Self;

    /// # Panics
    /// For integer `E` this panics if `rhs` is zero, in both debug and release
    /// mode. For float `E`, division by zero yields `INFINITY` or `NAN` instead.
    fn div(self, rhs : E) -> Self::Output
    {
      div_scalar( &self, rhs )
    }
  }

  impl< E, const LEN : usize > DivAssign< E > for Vector< E, LEN >
  where
    E : MatNum
  {
    /// # Panics
    /// For integer `E` this panics if `rhs` is zero, in both debug and release
    /// mode. For float `E`, division by zero yields `INFINITY` or `NAN` instead.
    fn div_assign( &mut self, rhs : E )
    {
        *self = *self / rhs;
    }
  }

  impl< E, const LEN : usize > DivAssign for Vector< E, LEN >
  where
    E : MatNum
  {
    /// # Panics
    /// For integer `E` this panics if any component of `rhs` is zero, in both
    /// debug and release mode. For float `E`, division by zero yields
    /// `INFINITY` or `NAN` instead.
    fn div_assign( &mut self, rhs : Self )
    {
      div_mut( self, &rhs );
    }
  }

}

crate::mod_interface!
{

}