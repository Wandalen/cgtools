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

    /// Creates homogeneous vector from `self`
    #[ inline ]
    pub fn to_homogenous( self ) -> Vector< E, 4 >
    {
      Vector::< E, 4 >::new( self.x(), self.y(), self.z(), E::one() )
    }

    /// Converts spherical coords to decart
    pub fn from_spherical( radius : E, theta : E, phi : E ) -> Self
    {
      let phi = phi.to_radians();
      let theta = theta.to_radians();
      let cos_phi = phi.cos();

      Self
      (
        [
          radius * cos_phi * theta.sin(),
          radius * phi.sin(),
          radius * cos_phi * theta.cos(),
        ]
      )
    }

    /// Converts decart coords to spherical and returns ( radius, theta, phi )
    pub fn to_spherical( self ) -> ( E, E, E )
    {
      let radius = self.mag();
      let [ x, y, z ] = self.0;
      let phi = y.atan2( ( x * x + z * z ).sqrt() );
      let theta = x.atan2( z );

      let phi = phi.to_degrees();
      let theta = theta.to_degrees();

      ( radius, theta, phi )
    }
  }

}

crate::mod_interface!
{
}
