mod private
{
  use crate:: {mat::DescriptorOrderColumnMajor, * };

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

    pub fn multiply( &self, other : &Self ) -> Self
    {
      let q1x = self.x();
      let q1y = self.y();
      let q1z = self.z();
      let q1w = self.w();

      let q2x = other.x();
      let q2y = other.y();
      let q2z = other.z();
      let q2w = other.w(); 

      let x = q1x * q2w + q1y * q2z - q1z * q2y + q1w * q2x;
      let y = -q1x * q2z + q1y * q2w + q1z * q2x + q1w * q2y;
      let z = q1x * q2y - q1y * q2x + q1z * q2w + q1w * q2z;
      let w = -q1x * q2x - q1y * q2y - q1z * q2z + q1w * q2w;

      Self( Vector::< E, 4 >::from( [ x, y, z, w ] ) )
    }

    pub fn multiply_mut( &mut self, other : &Self )
    {
      *self = self.multiply( other );
    }

    pub fn premultiply( &self, other : &Self ) -> Self
    {
      other.multiply( self )
    }

    pub fn premultiply_mut( &mut self, other : &Self )
    {
      *self = self.premultiply( other );
    }

    pub fn devide( &self, other : &Self ) -> Self
    {
      *self * other.invert()
    }

    pub fn device_mut( &mut self, other : &Self )
    {
      *self = self.devide( other );
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
      let half_theta = sin_half_theta.atan2( cos_half_theta );

      let ratio_a = ( ( E::one() - s ) * half_theta ).sin() / sin_half_theta;
      let ratio_b = ( s * half_theta ).sin() / sin_half_theta; 

      self * ratio_a + *other * ratio_b
    }

    pub fn slerp_mut( &mut self, other : &Self, s : E )
    {
      *self = self.slerp( other, s );
    }

    /// Inverts the unit length quaternion
    pub fn invert( &self ) -> Self
    {
      self.conjugate()
    }

    /// Transform the quaterion into a column major 3x3 rotation matrix
    pub fn to_matrix( &self ) -> Mat3< E, DescriptorOrderColumnMajor >
    {
      Mat3::< E, DescriptorOrderColumnMajor >::from_quat( *self )
    }

    /// Creates a quaternion from rotation around the X axis in radians
    pub fn from_angle_x( x : E ) -> Self
    {
      let two = E::one() + E::one();
      let ( s, c ) = ( x / two ).sin_cos();
      Self::from( [ s, E::zero(), E::zero(), c ] )
    }

    /// Creates a quaternion from rotation around the Y axis in radians
    pub fn from_angle_y( y : E ) -> Self
    {
      let two = E::one() + E::one();
      let ( s, c ) = ( y / two ).sin_cos();
      Self::from( [ E::zero(), s, E::zero(), c ] )
    }

    /// Creates a quaternion from rotation around the Z axis in radians
    pub fn from_angle_z( z : E ) -> Self
    {
      let two = E::one() + E::one();
      let ( s, c ) = ( z / two ).sin_cos();
      Self::from( [ E::zero(), E::zero(), s, c ] )
    }

    pub fn from_euler_xyz< T : VectorIter< E, 3 > >( angles : T ) -> Self
    {
      let mut iter = angles.vector_iter();
      let x = *iter.next().unwrap();
      let y = *iter.next().unwrap();
      let z = *iter.next().unwrap();

      let two = E::one() + E::one();
      let ( s1, c1 ) = ( x / two ).sin_cos();
      let ( s2, c2 ) = ( y / two ).sin_cos();
      let ( s3, c3 ) = ( z / two ).sin_cos();

      let mut q = Self::default();
      q[ 0 ] = s1 * c2 * c3 + c1 * s2 * s3;
      q[ 1 ] = c1 * s2 * c3 - s1 * c2 * s3;
      q[ 2 ] = c1 * c2 * s3 + s1 * s2 * c3;
      q[ 3 ] = c1 * c2 * c3 - s1 * s2 * s3;

      q
    }
  }
}

crate::mod_interface!
{
  
}
