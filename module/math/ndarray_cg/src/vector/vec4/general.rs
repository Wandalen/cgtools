mod private
{
  use crate::*;
  //use vector::arithmetics::inner_product::*;

  impl< E > Vector< E, 4 >
  where
    E : MatEl + NdFloat,
  {

    /// Create a new vector
    #[ inline( always ) ]
    pub const fn new( x : E, y : E, z : E, w : E ) -> Self
    {
      Self( [ x, y, z, w ] )
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

    #[ inline ]
    pub fn z( &self ) -> E
    {
      self.0[ 2 ]
    }

    #[ inline ]
    pub fn w( &self ) -> E
    {
      self.0[ 2 ]
    }
  }

}

crate::mod_interface!
{
}
