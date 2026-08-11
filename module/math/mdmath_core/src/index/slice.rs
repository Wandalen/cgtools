use super::{ AsIx2, Ix, Ix2, AsIx3, Ix3 };

impl AsIx2 for &[ Ix ]
{
  #[ inline( always ) ]
  fn as_ix2( self ) -> Ix2
  {
    match self
    {
      &[ a, b ] => Ix2( a, b ),
      _ => panic!( "Slice must have exactly 2 elements for Ix2 conversion" ),
    }
  }
}

impl AsIx3 for &[ Ix ]
{
  #[ inline( always ) ]
  fn as_ix3( self ) -> Ix3
  {
    match self
    {
      &[ a, b, c ] => Ix3( a, b, c ),
      _ => panic!( "Slice must have exactly 3 elements for Ix3 conversion" ),
    }
  }
}
