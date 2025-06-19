mod private
{
  use crate::*;

  impl< E > Quat< E >
  where E : MatEl
  {
    pub fn x( &self ) -> E
    {
      self.0[ 0 ]
    }

    pub fn y( &self ) -> E
    {
      self.0[ 1 ]
    }

    pub fn z( &self ) -> E
    {
      self.0[ 2 ]
    }

    pub fn w( &self ) -> E
    {
      self.0[ 3 ]
    }
  }

  impl< E > Quat< E >
  where E : MatEl + NdFloat
  {
    /// Creates a quaternion from the normalized axis and the angle in radians
    pub fn from_axis_angle< T >( axis : T, angle : E ) -> Self
    where
      T : VectorIter< E, 3 >
    {
        let ( s, c ) = angle.sin_cos();

        let mut iter = axis.vector_iter();
        let x = *iter.next().unwrap() * s;
        let y = *iter.next().unwrap() * s;
        let z = *iter.next().unwrap() * s;
        Self( Vector::< E, 4 >::from( [ x, y, z, c ] ) )
    }    

    pub fn normalize( self ) -> Self
    {
      Self( self.0.normalize() )
    }

    pub fn to_array( &self ) -> [ E; 4 ]
    {
      self.0.into()
    }

    pub fn conjugate( mut self ) -> Self
    {
      self.0[ 0 ] = -self.0[ 0 ]; 
      self.0[ 1 ] = -self.0[ 1 ]; 
      self.0[ 2 ] = -self.0[ 2 ];
      self 
    }

    pub fn mag2( &self ) -> E
    {
      self.0.mag2()
    }

    pub fn mag( &self ) -> E
    {
      self.0.mag()
    }

    pub fn slerp( self, other : &Self, s : f32 )
    {
      
    }

    // pub fn from_rotation_matrix( rot : F32x3x3 )
    // {

    // }
  }
}

crate::mod_interface!
{
  
}
