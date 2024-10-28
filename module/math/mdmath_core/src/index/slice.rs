use super::*;

impl AsIx2 for &[ Ix ]
{
  #[ inline( always ) ]
  fn as_ix2( self ) -> Ix2
  {
    assert!( self.len() == 2 );
    Ix2( self[ 0 ], self[ 1 ] )
  }
}

impl AsIx3 for &[ Ix ]
{
  #[ inline( always ) ]
  fn as_ix3( self ) -> Ix3
  {
    assert!( self.len() == 3 );
    Ix3( self[ 0 ], self[ 1 ], self[ 2 ] )
  }
}
