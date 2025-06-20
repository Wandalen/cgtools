mod private
{
  use crate::*;

  impl< E > Quat< E >
  where E : MatEl
  {
    pub fn x( &self ) -> E
    {
      self.0[ 0 ]
    }

    pub fn y( &self ) -> E
    {
      self.0[ 1 ]
    }

    pub fn z( &self ) -> E
    {
      self.0[ 2 ]
    }

    pub fn w( &self ) -> E
    {
      self.0[ 3 ]
    }
  }
}

crate::mod_interface!
{
  
}
