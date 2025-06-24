mod private
{
  use crate::*;
  use std::ops::{ Index, IndexMut };

  impl< E > Index< usize > for Quat< E >
  where
    E : MatEl,
  {
    type Output = E;

    #[ inline ]
    fn index( &self, index : usize ) -> &Self::Output
    {
      &self.0[ index ]
    }
  }

  impl< E > IndexMut< usize > for Quat< E >
  where
    E : MatEl,
  {

    #[ inline ]
    fn index_mut( &mut self, index : usize ) -> &mut Self::Output
    {
      &mut self.0[ index ]
    }
  }
}

crate::mod_interface!
{
  /// Mul trait implementations
  layer mul;
  /// Sub trait implementations
  layer sub;
  /// Add trait implementations
  layer add;
  /// Div trait implementations
  layer div;
}
