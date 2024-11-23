mod private
{
  use crate::*;
  use vector::arithmetics::inner_product::*;

  // Vector / Scalar
  impl< E, const LEN : usize > Div< E > for Vector< E, LEN >  
  where
    E : MatEl + NdFloat
  {
    type Output = Self;
    
    fn div( self, rhs: E ) -> Self::Output 
    {
      div_scalar( &self, rhs )
    }
  }

  // Vector /= Scalar
  impl< E, const LEN : usize > DivAssign< E > for Vector< E, LEN >  
  where
    E : MatEl + NdFloat
  {
    fn div_assign( &mut self, rhs: E ) 
    {
        *self = *self / rhs;
    }
  }

  impl< E, const LEN : usize > Div for Vector< E, LEN >  
  where
    E : MatEl + NdFloat
  {
    type Output = Self;
    
    fn div( self, rhs: Self ) -> Self::Output 
    {
      div( &self, &rhs )
    }
  }

  impl< E, const LEN : usize > DivAssign for Vector< E, LEN >  
  where
    E : MatEl + NdFloat
  {
    fn div_assign( &mut self, rhs: Self ) 
    {
      div_mut( self, &rhs );
    }
  }

}

crate::mod_interface!
{
  
}