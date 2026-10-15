mod private
{
  use crate::{Div, Quat, MatEl, NdFloat, DivAssign};

  impl< E > Div< E > for Quat< E >
  where
    E : MatEl + NdFloat
  {
    type Output = Self;

    #[ inline ]
    fn div( self, rhs : E ) -> Self::Output
    {
      Self( self.0 / rhs )
    }
  }

  impl< E > Div for Quat< E >
  where
    E : MatEl + NdFloat
  {
    type Output = Self;

    #[ inline ]
    fn div( self, rhs : Self ) -> Self::Output
    {
      self.divide( &rhs )
    }
  }

  impl< E > DivAssign< E > for Quat< E >
  where
    E : MatEl + NdFloat
  {
    #[ inline ]
    fn div_assign( &mut self, rhs : E )
    {
      self.0 = self.0 / rhs;
    }
  }

  impl< E > DivAssign for Quat< E >  
  where
    E : MatEl + NdFloat
  {
    #[ inline ]
    fn div_assign( &mut self, rhs : Self )
    {
      *self = *self / rhs;
    }
  }

}

crate::mod_interface!
{

}