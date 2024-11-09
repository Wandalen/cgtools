mod private
{
  use crate::*;
  use vector::arithmetics::inner_product::*;

  impl< E : MatEl + NdFloat > Vector< E, 3 >
  {
    #[ inline ]
    pub fn x( &self ) -> E
    {
      self.0[ 0 ]
    }

    #[ inline ]
    pub fn y( &self ) -> E
    {
      self.0[ 1 ]
    }

    #[ inline ]
    pub fn z( &self ) -> E
    {
      self.0[ 2 ]
    }

    pub fn cross( self, rhs : Self ) -> Self
    {
      cross( &self, &rhs )
    }
  }
}

crate::mod_interface!
{
  
}
