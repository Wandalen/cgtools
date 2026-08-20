mod private
{
  use crate::{Sub, Quat, MatEl, NdFloat, SubAssign};
  // use vector::arithmetics::inner_product::*;

  // Quat - Quat
  impl< E > Sub for Quat< E >
  where
    E : MatEl + NdFloat
  {
    type Output = Self;

    #[ inline ]
    fn sub( self, rhs : Self ) -> Self::Output
    {
      let v = self.0 - rhs.0;
      Self( v )
    }
  }

  // &Quat - &Quat
  impl< E > Sub for &Quat< E >
  where
    E : MatEl + NdFloat
  {
    type Output = Quat< E >;

    #[ inline ]
    fn sub( self, rhs : Self ) -> Self::Output
    {
      let v = self.0 - rhs.0;
      Quat::< E >( v )
    }
  }

  // Quat -= Quat
  impl< E > SubAssign for Quat< E >
  where
    E : MatEl + NdFloat
  {
    #[ inline ]
    fn sub_assign( &mut self, rhs : Self )
    {
      self.0 = self.0 - rhs.0;
    }
  }

  // Quat - scalar
  impl< E > Sub< E > for Quat< E >
  where
    E : MatEl + NdFloat
  {
    type Output = Self;

    #[ inline ]
    fn sub( self, rhs : E ) -> Self::Output
    {
      let v = self.0 - rhs;
      Self( v )
    }
  }
}

crate::mod_interface!
{

}