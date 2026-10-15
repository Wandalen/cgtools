mod private
{
  use crate::{Add, Quat, MatEl, NdFloat, AddAssign};

  // Quat + Quat
  impl< E > Add for Quat< E >
  where
  E : MatEl + NdFloat
  {
    type Output = Self;

    #[ inline ]
    fn add( self, rhs : Self ) -> Self::Output
    {
      Self( self.0 + rhs.0 )
    }
  }

  // &Quat + &Quat
  impl< E > Add for &Quat< E >
  where
    E : MatEl + NdFloat
  {
    type Output = Quat< E >;

    #[ inline ]
    fn add( self, rhs : Self ) -> Self::Output
    {
      Quat::< E >( self.0 + rhs.0 )
    }
  }

  impl< E > AddAssign for Quat< E >
  where
  E : MatEl + NdFloat
  {
    #[ inline ]
    fn add_assign( &mut self, rhs : Self )
    {
        self.0 = self.0 + rhs.0;
    }
  }

  impl< E > AddAssign< E > for Quat< E >
  where
  E : MatEl + NdFloat
  {
    #[ inline ]
    fn add_assign( &mut self, rhs : E )
    {
        self.0 = self.0 + rhs;
    }
  }
}

crate::mod_interface!
{

}