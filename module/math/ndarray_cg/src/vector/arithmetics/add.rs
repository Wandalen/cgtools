mod private
{
  use crate::*;
  use vector::arithmetics::inner_product::*;

  impl< E, const LEN : usize > Add for Vector< E, LEN >  
  where
  E : MatEl + NdFloat
  {
    type Output = Self;
    
    fn add( self, rhs: Self ) -> Self::Output 
    {
        sum( &self, &rhs )
    }
  }

  impl< E, const LEN : usize > Add for &Vector< E, LEN >  
  where
    E : MatEl + NdFloat
  {
    type Output = Vector< E, LEN >;
    
    fn add( self, rhs: Self ) -> Self::Output {
      sum( self, rhs )
    }
  }

  impl< E, const LEN : usize > AddAssign for Vector< E, LEN >  
  where
  E : MatEl + NdFloat
  {
    fn add_assign( &mut self, rhs: Self ) 
    {
        *self = *self + rhs;
    }
  }
}

crate::mod_interface!
{
  
}