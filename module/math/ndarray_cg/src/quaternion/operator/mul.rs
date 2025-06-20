mod private
{
  use crate::*;

  // Quat * Quat
  impl< E > Mul for Quat< E >
  where
    E : MatEl + nd::NdFloat
  {
    type Output = Self;

    fn mul( self, rhs : Self ) -> Self::Output
    {
      let q1x = self.x();
      let q1y = self.y();
      let q1z = self.z();
      let q1w = self.w();

      let q2x = rhs.x();
      let q2y = rhs.y();
      let q2z = rhs.z();
      let q2w = rhs.w(); 

      let x = q1x * q2w + q1y * q2z - q1z * q2y + q1w * q2x;
      let y = -q1x * q2z + q1y * q2w + q1z * q2x + q1w * q2y;
      let z = q1x * q2y - q1y * q2x + q1z * q2w + q1w * q2z;
      let w = -q1x * q2x - q1y * q2y - q1z * q2z + q1w * q2w;

      Self( Vector::< E, 4 >::from( [ x, y, z, w ] ) )
    }
  }

  // Quat * Scalar
  impl< E > Mul< E > for Quat< E >
  where
    E : MatEl + nd::NdFloat
  {
    type Output = Self;

    fn mul( self, rhs : E ) -> Self::Output
    {
      Self( self.0 * rhs )
    }
  }

  // Quat *= Quat
  impl< E > MulAssign for Quat< E >
  where
    E : MatEl + nd::NdFloat
  {
    fn mul_assign( &mut self, rhs : Quat< E > )
    {
      //*self = rhs * *self;
      *self = *self * rhs;
    }
  }

  // Quat *= Scalar
  impl< E > MulAssign< E > for Quat< E >
  where
    E : MatEl + nd::NdFloat
  {
    fn mul_assign( &mut self, rhs : E )
    {
      ( *self ).0 = ( *self ).0 * rhs;
    }
  }
}

crate::mod_interface!
{

}