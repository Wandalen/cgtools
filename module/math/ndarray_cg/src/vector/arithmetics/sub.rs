mod private
{
  use crate::*;
  use vector::arithmetics::inner_product::*;

  impl< E, const LEN : usize > Sub for Vector< E, LEN >  
  where
    E : MatEl + NdFloat
  {
    type Output = Self;
    
    fn sub( self, rhs: Self ) -> Self::Output 
    {
      sub( &self, &rhs )
    }
  }

  impl< E, const LEN : usize > Sub for &Vector< E, LEN >  
  where
    E : MatEl + NdFloat
  {
    type Output = Vector< E, LEN >;
    
    fn sub( self, rhs: Self ) -> Self::Output 
    {
      sub( self, rhs )
    }
  }

  impl< E, const LEN : usize > SubAssign for Vector< E, LEN >  
  where
    E : MatEl + NdFloat
  {
    fn sub_assign( &mut self, rhs: Self ) 
    {
      *self = *self - rhs;
    }
  }
}

crate::mod_interface!
{
  
}