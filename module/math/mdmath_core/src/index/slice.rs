use super::{ AsIx2, Ix, Ix2, AsIx3, Ix3 };

impl AsIx2 for &[ Ix ]
{
  #[ inline( always ) ]
  fn as_ix2( self ) -> Ix2
  {
    assert!( self.len() == 2, "Slice must have exactly 2 elements for Ix2 conversion" );
    #[allow(clippy::indexing_slicing)] // Safe due to length assertion above
    return Ix2( self[ 0 ], self[ 1 ] )
  }
}

impl AsIx3 for &[ Ix ]
{
  #[ inline( always ) ]
  fn as_ix3( self ) -> Ix3
  {
    assert!( self.len() == 3, "Slice must have exactly 3 elements for Ix3 conversion" );
    #[allow(clippy::indexing_slicing)] // Safe due to length assertion above
    return Ix3( self[ 0 ], self[ 1 ], self[ 2 ] )
  }
}
