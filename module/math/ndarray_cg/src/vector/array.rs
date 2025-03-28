mod private
{
  use crate::*;

  impl< E : MatEl, const N : usize > From< [ E; N ] > for Vector< E, N >
  {
    fn from( value: [ E; N ] ) -> Self 
    {
      Vector( value )
    }
  }

  impl< E : MatEl, const N : usize > From< Vector< E, N > > for [ E; N ]
  {
    fn from( value: Vector< E, N > ) -> Self 
    {
      value.0
    }
  }

  impl< E, const LEN : usize > Vector< E, LEN >  
  where
    E : MatEl
  {
    pub fn to_array( &self ) -> [ E; LEN ]
    {
      self.0
    }
  }
  
}

crate::mod_interface!
{
  
}

