mod private
{
  use crate::*;

  impl< E : MatEl, const N : usize > From< [ E; N ] > for Vector< E, N >
  {
    fn from( value: [ E; N ] ) -> Self {
      Vector( value )
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

