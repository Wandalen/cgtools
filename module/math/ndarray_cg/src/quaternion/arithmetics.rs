//! This module contains the private implementation details for the `Quat` (quaternion) struct.
//! It includes methods for creation, manipulation, and conversion of quaternions, which are
//! then selectively exposed through the public interface.
mod private
{
  use crate::{ mat::DescriptorOrderColumnMajor, * };

  #[ inline ]
  fn wrap_pi< E : MatEl + NdFloat >( a : E ) -> E
  {
    a.sin().atan2( a.cos() )
  }

  impl< E > Quat< E >
  where E : MatEl + NdFloat
  {
    /// Creates a quaternion from a normalized axis and an angle in radians.
    ///
    /// # Arguments
    /// * `axis` - The normalized 3D vector representing the axis of rotation.
    /// * `angle` - The angle of rotation in radians.
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

    /// Normalizes the quaternion to have a magnitude of 1.
    pub fn normalize( self ) -> Self
    {
      Self( self.0.normalize() )
    }

    /// Converts the quaternion's components into a 4-element array `[x, y, z, w]`.
    pub fn to_array( &self ) -> [ E; 4 ]
    {
      self.0.into()
    }

    /// Computes the conjugate of the quaternion, inverting its vector part.
    pub fn conjugate( mut self ) -> Self
    {
      self.0[ 0 ] = -self.0[ 0 ];
      self.0[ 1 ] = -self.0[ 1 ];
      self.0[ 2 ] = -self.0[ 2 ];
      self
    }

    /// Calculates the squared magnitude (length) of the quaternion.
    pub fn mag2( &self ) -> E
    {
      self.0.mag2()
    }

    /// Calculates the magnitude (length) of the quaternion.
    pub fn mag( &self ) -> E
    {
      self.0.mag()
    }

    /// Computes the dot product of this quaternion with another.
    pub fn dot( &self, other : &Self ) -> E
    {
      self.0.dot( &other.0 )
    }

    /// Multiplies this quaternion by another quaternion (`self * other`).
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

    /// Multiplies this quaternion by another in-place.
    pub fn multiply_mut( &mut self, other : &Self )
    {
      *self = self.multiply( other );
    }

    /// Multiplies another quaternion by this one (`other * self`).
    pub fn premultiply( &self, other : &Self ) -> Self
    {
      other.multiply( self )
    }

    /// Multiplies another quaternion by this one in-place.
    pub fn premultiply_mut( &mut self, other : &Self )
    {
      *self = self.premultiply( other );
    }

    /// Divides this quaternion by another (equivalent to `self * other.invert()`).
    pub fn devide( &self, other : &Self ) -> Self
    {
      *self * other.invert()
    }

    /// Divides this quaternion by another in-place.
    pub fn device_mut( &mut self, other : &Self )
    {
      *self = self.devide( other );
    }

    /// Performs spherical linear interpolation (slerp) between two unit quaternions.
    ///
    /// # Arguments
    /// * `other` - The target quaternion to interpolate towards.
    /// * `s` - The interpolation factor, a value between 0.0 and 1.0.
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

    /// Performs spherical linear interpolation (slerp) in-place.
    pub fn slerp_mut( &mut self, other : &Self, s : E )
    {
      *self = self.slerp( other, s );
    }

    /// Inverts the unit-length quaternion, which is equivalent to its conjugate.
    pub fn invert( &self ) -> Self
    {
      self.conjugate()
    }

    /// Converts the quaternion into a column-major 3x3 rotation matrix.
    pub fn to_matrix( &self ) -> Mat3< E, DescriptorOrderColumnMajor >
    {
      Mat3::< E, DescriptorOrderColumnMajor >::from_quat( *self )
    }

    /// Creates a quaternion representing a rotation around the X-axis.
    ///
    /// # Arguments
    /// * `x` - The rotation angle in radians.
    pub fn from_angle_x( x : E ) -> Self
    {
      let two = E::one() + E::one();
      let ( s, c ) = ( x / two ).sin_cos();
      Self::from( [ s, E::zero(), E::zero(), c ] )
    }

    /// Creates a quaternion representing a rotation around the Y-axis.
    ///
    /// # Arguments
    /// * `y` - The rotation angle in radians.
    pub fn from_angle_y( y : E ) -> Self
    {
      let two = E::one() + E::one();
      let ( s, c ) = ( y / two ).sin_cos();
      Self::from( [ E::zero(), s, E::zero(), c ] )
    }

    /// Creates a quaternion representing a rotation around the Z-axis.
    ///
    /// # Arguments
    /// * `z` - The rotation angle in radians.
    pub fn from_angle_z( z : E ) -> Self
    {
      let two = E::one() + E::one();
      let ( s, c ) = ( z / two ).sin_cos();
      Self::from( [ E::zero(), E::zero(), s, c ] )
    }

    /// Creates a quaternion from Euler angles in XYZ order.
    ///
    /// # Arguments
    /// * `angles` - A 3D vector containing the rotation angles (in radians) for the X, Y, and Z axes.
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

    /// Converts a quaternion to Euler angles in XYZ order (radians)
    pub fn to_euler_xyz( &self ) -> Vector< E, 3 >
    {
      let q = self.normalize();

      let x = q.x();
      let y = q.y();
      let z = q.z();
      let w = q.w();

      let two = E::one() + E::one();
      let one = E::one();
      let eps = E::from( 1e-6 ).unwrap();

      // Pitch ( Y )
      let sinp = two * ( w * y - z * x );
      let sinp = sinp.max( - one ).min( one );
      let pitch = sinp.asin();

      // Gimbal lock handling
      if ( sinp.abs() - one ).abs() < eps
      {
        // Collapse roll into yaw
        let yaw = two * ( x * y + w * z ).atan2( one - two * ( y * y + z * z ) );
        return [ E::zero(), pitch, wrap_pi( yaw ) ].into();
      }

      // Roll ( X )
      let mut roll = ( two * ( w * x + y * z ) ).atan2( one - two * ( x * x + y * y ) );

      // Yaw ( Z )
      let mut yaw = ( two * ( w * z + x * y ) ).atan2( one - two * ( y * y + z * z ) );

      roll = wrap_pi( roll );
      yaw  = wrap_pi( yaw );

      [ roll, pitch, yaw ].into()
    }
  }
}

crate::mod_interface!
{

}
