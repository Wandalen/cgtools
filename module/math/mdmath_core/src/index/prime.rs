use super::{ AsIx2, Ix2, AsIx3, Ix3 };

impl AsIx2 for Ix2
{
  #[ inline( always ) ]
  fn as_ix2( self ) -> Ix2
  {
    return self
  }
}

impl AsIx3 for Ix3
{
  #[ inline( always ) ]
  fn as_ix3( self ) -> Ix3
  {
    return self
  }
}
