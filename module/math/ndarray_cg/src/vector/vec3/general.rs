mod private
{
  use crate::*;
  use vector::arithmetics::inner_product::*;

  impl< E : MatEl + NdFloat > Vector< E, 3 >
  {
    pub fn cross( self, rhs : Self ) -> Self
    {
      cross( &self, &rhs )
    }
  }
}

crate::mod_interface!
{
  
}
