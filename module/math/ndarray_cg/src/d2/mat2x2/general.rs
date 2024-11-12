use crate::*;

impl< E > Mat2< E > 
where 
E : MatEl + nd::NdFloat
{
  /// Returns the determinant of the matrix
  pub fn determinant( &self ) -> E
  {
    let a = *self.scalar_ref( Ix2( 0, 0 ) );
    let b = *self.scalar_ref( Ix2( 0, 1 ) );
    let c = *self.scalar_ref( Ix2( 1, 0 ) );
    let d = *self.scalar_ref( Ix2( 1, 1 ) );

    a * d - b * c
  }    

  pub fn inverse( &self ) -> Option< Self >
  {
    let det = self.determinant();

    if det == E::zero() { return None; }

    let a = *self.scalar_ref( Ix2( 0, 0 ) );
    let b = *self.scalar_ref( Ix2( 0, 1 ) );
    let c = *self.scalar_ref( Ix2( 1, 0 ) );
    let d = *self.scalar_ref( Ix2( 1, 1 ) );

    let inverse = Self::from_column_major
    (
      [ d, -c, -b, a ]
    );

    Some( inverse / det )
  }
}