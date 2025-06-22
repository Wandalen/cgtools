mod private
{
  use crate::*;

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
    
    pub fn dot( &self, other : &Self ) -> E
    {
      self.0.dot( &other.0 )
    }

    /// Performs spehrical linear interpolation between two unit quaternions
    pub fn slerp( self, other : &Self, s : E ) -> Self
    {
      if s.is_zero() { return self; }
      if s.is_one() { return *other; }

      let mut q2 = *other;  


      let mut cos_half_theta = self.dot( other );

      if cos_half_theta < E::zero()
      {
        cos_half_theta = -cos_half_theta;
        q2[ 0 ] = -q2[ 0 ];
        q2[ 1 ] = -q2[ 1 ];
        q2[ 2 ] = -q2[ 2 ];
        q2[ 3 ] = -q2[ 3 ];
      }


      if cos_half_theta >= E::one() 
      {
        return self;
      }

      let sqr_sin_half_theta = E::one() - cos_half_theta * cos_half_theta;
      if sqr_sin_half_theta <= E::epsilon() 
      {
        return ( self * ( E::one() - s ) + *other * s ).normalize(); 
      }

      let sin_half_theta = sqr_sin_half_theta.sqrt();
      //let half_theta = cos_half_theta.acos();
      let half_theta = sin_half_theta.atan2( cos_half_theta );

      let ratio_a = ( ( E::one() - s ) * half_theta ).sin() / sin_half_theta;
      let ratio_b = ( s * half_theta ).sin() / sin_half_theta; 

      self * ratio_a + *other * ratio_b
    }

    // pub fn from_rotation_matrix( rot : F32x3x3 )
    // {

    // }
  }
}

crate::mod_interface!
{
  
}
