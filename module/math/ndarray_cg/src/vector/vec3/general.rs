mod private
{
  use crate::*;
  use vector::{ cross };

  impl< E > Vector< E, 3 >
  where
    E : MatEl + NdFloat,
  {

    /// Create a new vector
    #[ inline( always ) ]
    pub const fn new( x : E, y : E, z : E ) -> Self
    {
      Self( [ x, y, z ] )
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

    pub fn cross( self, rhs : Self ) -> Self
    {
      cross( &self, &rhs )
    }
  }

}

crate::mod_interface!
{
}
