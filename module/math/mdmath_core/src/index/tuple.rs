use super::*;

impl AsIx2 for ( Ix, Ix )
{
  #[ inline( always ) ]
  fn as_ix2( self ) -> Ix2
  {
    Ix2( self.0, self.1 )
  }
}

impl AsIx3 for ( Ix, Ix, Ix )
{
  #[ inline( always ) ]
  fn as_ix3( self ) -> Ix3
  {
    Ix3( self.0, self.1, self.2 )
  }
}
