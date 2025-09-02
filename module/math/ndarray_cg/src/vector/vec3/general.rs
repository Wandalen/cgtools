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

    /// The `x` component of vector
    #[ inline ]
    pub fn x( &self ) -> E
    {
      self.0[ 0 ]
    }

    /// The `y` component of vector
    #[ inline ]
    pub fn y( &self ) -> E
    {
      self.0[ 1 ]
    }

    /// The `z` component of vector
    #[ inline ]
    pub fn z( &self ) -> E
    {
      self.0[ 2 ]
    }

    /// Calculates cross product with another vector
    pub fn cross( self, rhs : Self ) -> Self
    {
      cross( &self, &rhs )
    }

    /// Calculates vector norm
    pub fn norm( &self ) -> E
    {
      let n = self.x().powi( 2 ) +
      self.y().powi( 2 ) +
      self.z().powi( 2 );

      n.sqrt()
    }

    /// Creates homogeneous vector from `self`
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
