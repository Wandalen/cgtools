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
      self.multiply( &rhs )
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