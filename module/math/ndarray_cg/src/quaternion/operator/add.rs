mod private
{
  use crate::*;

  // Quat + Quat
  impl< E > Add for Quat< E >
  where
  E : MatEl + NdFloat
  {
    type Output = Self;

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

    fn add( self, rhs : Self ) -> Self::Output 
    {
      Quat::< E >( self.0 + rhs.0 )
    }
  }

  impl< E > AddAssign for Quat< E >
  where
  E : MatEl + NdFloat
  {
    fn add_assign( &mut self, rhs : Self )
    {
        ( *self ).0 = ( *self ).0 + rhs.0;
    }
  }

  impl< E > AddAssign< E > for Quat< E >
  where
  E : MatEl + NdFloat
  {
    fn add_assign( &mut self, rhs : E )
    {
        ( *self ).0 = ( *self ).0 + rhs;
    }
  }
}

crate::mod_interface!
{

}