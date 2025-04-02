mod private
{
  use crate::*;
  // use vector::arithmetics::inner_product::*;
  use vector::{ div_scalar };

  impl< E, const LEN : usize > Div< E > for Vector< E, LEN >
  where
    E : MatEl + NdFloat
  {
    type Output = Self;

    fn div(self, rhs : E) -> Self::Output
    {
      div_scalar( &self, rhs )
    }
  }

  impl< E, const LEN : usize > DivAssign< E > for Vector< E, LEN >
  where
    E : MatEl + NdFloat
  {
    fn div_assign( &mut self, rhs : E )
    {
        *self = *self / rhs;
    }
  }

}

crate::mod_interface!
{

}