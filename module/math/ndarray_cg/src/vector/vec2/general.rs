mod private
{
  use crate::*;
  //use vector::arithmetics::inner_product::*;

  impl< E > Vector< E, 2 >
  where
    E : MatEl + NdFloat,
  {

    /// Create a new vector
    #[ inline( always ) ]
    pub const fn new( x : E, y : E ) -> Self
    {
      Self( [ x, y ] )
    }

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
