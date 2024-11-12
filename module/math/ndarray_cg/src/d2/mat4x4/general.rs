use ndarray::Dimension;

use crate::*;

fn minor< E : MatEl + nd::NdFloat >
( 
  from : &Mat4< E >, 
  to : &mut Mat3< E >, 
  i : usize, 
  j : usize 
)
{
  for( id, ( _, v ) ) in from
  .iter_indexed_msfirst()
  .filter( 
    | ( id, _ ) |
    { 
      let ( r, c ) = id.into_pattern();
      r != i && c != j
    } 
  ).enumerate()
  {
    to.raw_slice_mut()[ id ] = *v;
  }
}

fn cofactor< E : MatEl + nd::NdFloat >
( 
  from : &Mat4< E >, 
  to : &mut Mat3< E >, 
  i : usize, 
  j : usize 
) -> E
{
  let k = E::from( ( -1i32 ).pow( ( i + j ) as u32 ) ).unwrap();
  minor( from, to, i, j );
  k * to.determinant()
}

impl< E > Mat4< E > 
where E : MatEl + nd::NdFloat
{
  pub fn to_array( &self ) -> [ E; 16 ]
  {
    self.raw_slice().try_into().unwrap()
  }

  pub fn determinant( &self ) -> E
  {
    let _a11 = *self.scalar_ref( Ix2( 0, 0 ) );
    let _a12 = *self.scalar_ref( Ix2( 0, 1 ) );
    let _a13 = *self.scalar_ref( Ix2( 0, 2 ) );
    let _a14 = *self.scalar_ref( Ix2( 0, 3 ) );

    let mut m = Mat3::default();

    minor( self, &mut m, 0, 0 );
    let _det11 = m.determinant();
    minor( self, &mut m, 0, 1 );
    let _det12 = m.determinant();
    minor( self, &mut m, 0, 2 );
    let _det13 = m.determinant();
    minor( self, &mut m, 0, 3 );
    let _det14 = m.determinant();

    _a11 * _det11 - _a12 * _det12 + _a13 * _det13 - _a14 * _det14
  }

  pub fn inverse( &self ) -> Option< Self >
  {
    let det = self.determinant();

    if det == E::zero() { return None; }

    let mut cfm = Mat3::default();
    let mut cf = | i, j |
    {
      cofactor( self, &mut cfm, i, j )
    };

    let adj = Self::from_column_major
    ([
      cf( 0, 0 ), cf( 0, 1 ), cf( 0, 2 ), cf( 0, 3 ),
      cf( 1, 0 ), cf( 1, 1 ), cf( 1, 2 ), cf( 1, 3 ),
      cf( 2, 0 ), cf( 2, 1 ), cf( 2, 2 ), cf( 2, 3 ),
      cf( 3, 0 ), cf( 3, 1 ), cf( 3, 2 ), cf( 3, 3 ),
    ]);

    Some( adj / det )
  }
}