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

    #[ inline ]
    pub fn to_homogenous( self ) -> Vector< E, 4 >
    {
      Vector::< E, 4 >::new( self.x(), self.y(), self.z(), E::one() )
    }
  }

}

crate::mod_interface!
{
}
