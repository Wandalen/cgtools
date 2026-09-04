//! This module contains the private implementation details for the `Quat` (quaternion) struct.
//! It includes methods for creation, manipulation, and conversion of quaternions, which are
//! then selectively exposed through the public interface.
mod private
{
  use crate::{ mat::DescriptorOrderColumnMajor, MatEl, NdFloat, Quat, VectorIter, Vector, Into, Mat3 };

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
    ///
    /// # Panics
    /// Panics if `axis`'s iterator yields fewer than 3 elements.
    #[ inline ]
    pub fn from_axis_angle< T >( axis : T, angle : E ) -> Self
    where
      T : VectorIter< E, 3 >
    {
        // Fix(BUG-120): changed `angle.sin_cos()` to `(angle / two).sin_cos()`.
        // Root cause: the axis-angle-to-quaternion formula requires the HALF angle
        // (`q = (axis * sin(angle/2), cos(angle/2))`) — this function used the full angle
        // directly, unlike its sibling constructors `from_angle_x`/`from_angle_y`/
        // `from_angle_z` (all three correctly halve via `let two = E::one() + E::one(); (x /
        // two).sin_cos()`), so a caller requesting a rotation of `angle` radians about a given
        // axis actually got a rotation of `2 * angle` radians instead.
        // Pitfall: sibling constructors that should share an invariant (here: "always
        // half-angle the input") can drift independently when each is implemented as its own
        // free-standing function instead of being built on one shared half-angle helper —
        // cross-check new constructors against already-correct siblings for the same class of
        // input, not just against the formula in isolation.
        let two = E::one() + E::one();
        let ( s, c ) = ( angle / two ).sin_cos();

        let mut iter = axis.vector_iter();
        let x = *iter.next().unwrap() * s;
        let y = *iter.next().unwrap() * s;
        let z = *iter.next().unwrap() * s;
        Self( Vector::< E, 4 >::from( [ x, y, z, c ] ) )
    }

    /// Normalizes the quaternion to have a magnitude of 1.
    #[ inline ]
    #[ must_use ]
    pub fn normalize( self ) -> Self
    {
      Self( self.0.normalize() )
    }

    /// Converts the quaternion's components into a 4-element array `[x, y, z, w]`.
    #[ inline ]
    pub fn to_array( &self ) -> [ E; 4 ]
    {
      self.0.into()
    }

    /// Computes the conjugate of the quaternion, inverting its vector part.
    #[ inline ]
    #[ must_use ]
    pub fn conjugate( mut self ) -> Self
    {
      self.0[ 0 ] = -self.0[ 0 ];
      self.0[ 1 ] = -self.0[ 1 ];
      self.0[ 2 ] = -self.0[ 2 ];
      self
    }

    /// Calculates the squared magnitude (length) of the quaternion.
    #[ inline ]
    pub fn mag2( &self ) -> E
    {
      self.0.mag2()
    }

    /// Calculates the magnitude (length) of the quaternion.
    #[ inline ]
    pub fn mag( &self ) -> E
    {
      self.0.mag()
    }

    /// Computes the dot product of this quaternion with another.
    #[ inline ]
    pub fn dot( &self, other : &Self ) -> E
    {
      self.0.dot( &other.0 )
    }

    /// Multiplies this quaternion by another quaternion (`self * other`).
    #[ inline ]
    #[ must_use ]
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
    #[ inline ]
    pub fn multiply_mut( &mut self, other : &Self )
    {
      *self = self.multiply( other );
    }

    /// Multiplies another quaternion by this one (`other * self`).
    #[ inline ]
    #[ must_use ]
    pub fn premultiply( &self, other : &Self ) -> Self
    {
      other.multiply( self )
    }

    /// Multiplies another quaternion by this one in-place.
    #[ inline ]
    pub fn premultiply_mut( &mut self, other : &Self )
    {
      *self = self.premultiply( other );
    }

    /// Divides this quaternion by another (equivalent to `self * other.invert()`).
    // UX/DX: renamed from the misspelled `devide` to `divide` (and `device_mut` to
    // `divide_mut` below) -- plain authoring typo, never caught since it compiled and worked
    // correctly under the misspelled name.
    #[ inline ]
    #[ must_use ]
    pub fn divide( &self, other : &Self ) -> Self
    {
      *self * other.invert()
    }

    /// Divides this quaternion by another in-place.
    #[ inline ]
    pub fn divide_mut( &mut self, other : &Self )
    {
      *self = self.divide( other );
    }

    /// Performs spherical linear interpolation (slerp) between two unit quaternions.
    ///
    /// # Arguments
    /// * `other` - The target quaternion to interpolate towards.
    /// * `s` - The interpolation factor, a value between 0.0 and 1.0.
    #[ inline ]
    #[ must_use ]
    pub fn slerp( self, other : &Self, s : E ) -> Self
    {
      if s.is_zero() { return self; }
      if s.is_one() { return *other; }

      let mut q2 = *other;


      let mut cos_half_theta = self.dot( other );

      // Fix(BUG-194): `q2` is the hemisphere-corrected copy of `other` -- when `cos_half_theta`
      // is negative, `self` and `other` are more than 90 degrees apart as 4D vectors even though
      // they represent rotations less than 180 degrees apart ( `q` and `-q` encode the identical
      // rotation ). Both branches below previously kept blending against the original, un-flipped
      // `*other` instead of this corrected `q2` -- pairing the short-path angle ( derived from
      // the now-positive `cos_half_theta` ) with the long-path quaternion value produced a
      // non-unit-length result rotated the wrong way whenever the two inputs started in opposite
      // hemispheres. Every use of `*other` below is replaced with `q2`.
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
        return ( self * ( E::one() - s ) + q2 * s ).normalize();
      }

      let sin_half_theta = sqr_sin_half_theta.sqrt();
      let half_theta = sin_half_theta.atan2( cos_half_theta );

      let ratio_a = ( ( E::one() - s ) * half_theta ).sin() / sin_half_theta;
      let ratio_b = ( s * half_theta ).sin() / sin_half_theta;

      self * ratio_a + q2 * ratio_b
    }

    /// Performs spherical linear interpolation (slerp) in-place.
    #[ inline ]
    pub fn slerp_mut( &mut self, other : &Self, s : E )
    {
      *self = self.slerp( other, s );
    }

    /// Inverts the quaternion, producing its multiplicative inverse. Reduces to the conjugate
    /// for a unit-length quaternion ( `mag2() == 1` ), and is the general formula otherwise.
    // BUG-298 task/bug/298_quat_invert_wrong_for_non_unit_quaternions.md -- was
    // unconditionally `self.conjugate()`, wrong for any non-unit-length quaternion.
    // Fix(BUG-298): was `self.conjugate()`, correct only when `self` is unit-length.
    // Root cause: the general quaternion inverse is `conjugate(q) / mag2(q)`; the unit-only
    // shortcut was applied unconditionally, so `divide`/`Div`/`DivAssign` ( all routed through
    // `invert` ) silently scaled their result by the divisor's squared magnitude instead of
    // producing a true quotient whenever the divisor was not already unit-length.
    // Pitfall: a documented precondition ( "unit-length" ) on a function whose signature accepts
    // any value of the type gives callers no way to know they've violated it.
    #[ inline ]
    #[ must_use ]
    pub fn invert( &self ) -> Self
    {
      self.conjugate() / self.mag2()
    }

    /// Converts the quaternion into a column-major 3x3 rotation matrix.
    #[ inline ]
    pub fn to_matrix( &self ) -> Mat3< E, DescriptorOrderColumnMajor >
    {
      Mat3::< E, DescriptorOrderColumnMajor >::from_quat( *self )
    }

    /// Creates a quaternion representing a rotation around the X-axis.
    ///
    /// # Arguments
    /// * `x` - The rotation angle in radians.
    #[ inline ]
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
    #[ inline ]
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
    #[ inline ]
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
    ///
    /// # Panics
    /// Panics if `angles`'s iterator yields fewer than 3 elements.
    #[ inline ]
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
    ///
    /// # Panics
    /// Panics if `E::from( 1e-6 )` fails, i.e. if `E` cannot represent that
    /// literal (not expected for the standard float types this is used with).
    #[ inline ]
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

      // Fix(BUG-272): corrected the sign of the cross term in all three trig formulas (`w*y -
      // z*x` -> `w*y + z*x` for pitch; `w*x + y*z` -> `w*x - y*z` for roll; `w*z + x*y` -> `w*z
      // - x*y` for yaw), corrected the gimbal-lock branch's yaw denominator (`y*y + z*z` ->
      // `x*x + z*z`), and parenthesized the gimbal-lock branch's doubled numerator (`two * ( x *
      // y + w * z ).atan2( .. )` -> `( two * ( x * y + w * z ) ).atan2( .. )`) -- method-call
      // precedence bound `.atan2` tighter than the leading `two *`, so the whole `atan2` result
      // was doubled after the call instead of its first argument being doubled before it.
      // Root cause: the matrix-to-Euler-angle extraction formulas were transcribed with the
      // wrong sign on each cross term (and the wrong pair of squared components in the gimbal
      // lock denominator), so the function only happened to look correct for single-axis
      // rotations (where the mismatched cross term is a product against an always-zero
      // component) or very small angles (where the loose pre-existing test epsilon of 1e-1
      // hid the error) -- any genuine multi-axis rotation returned wrong roll/pitch/yaw. The
      // gimbal-lock branch carried a second, independent defect on top of that: `atan2( 2*n, d
      // )` (double the numerator, then take the angle) and `2 * atan2( n, d )` (take the angle,
      // then double it) are different functions whenever `n != 0`, but the unparenthesized `two
      // * ( .. ).atan2( .. )` computed the latter -- so even a case with the correct sign/
      // denominator still reported the wrong collapsed angle at true gimbal lock unless roll
      // and yaw were both zero (the only case where the two placements coincide).
      // Pitfall: a matrix/quaternion-to-Euler decomposition formula with several structurally
      // similar terms (here: `w*y +/- z*x`, `w*x +/- y*z`, `w*z +/- x*y`) needs each sign
      // verified independently against a derivation or ground truth (matrix product, or the
      // crate's own verified-correct forward conversion) -- copying the "shape" of a sibling
      // term without re-deriving its specific sign lets a single transcription slip silently
      // propagate across every term that shares the pattern, and single-axis/small-angle test
      // inputs cannot detect it because the buggy cross term evaluates to (near) zero either
      // way. Separately, `scalar * expr.method( .. )` silently binds the method call tighter
      // than the leading multiplication -- a formula that needs the multiplication applied
      // *before* the call (doubling a numerator, not a result) must parenthesize the multiplied
      // expression explicitly, and a test exercising only the degenerate zero-numerator case
      // cannot tell the two placements apart.
      // Pitch ( Y )
      let sinp = two * ( w * y + z * x );
      let sinp = sinp.max( - one ).min( one );
      let pitch = sinp.asin();

      // Gimbal lock handling
      if ( sinp.abs() - one ).abs() < eps
      {
        // Collapse roll into yaw
        let yaw = ( two * ( x * y + w * z ) ).atan2( one - two * ( x * x + z * z ) );
        return [ E::zero(), pitch, wrap_pi( yaw ) ].into();
      }

      // Roll ( X )
      let mut roll = ( two * ( w * x - y * z ) ).atan2( one - two * ( x * x + y * y ) );

      // Yaw ( Z )
      let mut yaw = ( two * ( w * z - x * y ) ).atan2( one - two * ( y * y + z * z ) );

      roll = wrap_pi( roll );
      yaw  = wrap_pi( yaw );

      [ roll, pitch, yaw ].into()
    }
  }
}

crate::mod_interface!
{

}
