mod private
{
  use crate::*;
  //use vector::arithmetics::inner_product::*;

  impl< E : MatEl + NdFloat > Vector< E, 2 >
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
  }
}

crate::mod_interface!
{
  
}
