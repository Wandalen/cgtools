use ndarray::Dimension;

use crate::*;

fn minor
< 
  E : MatEl + nd::NdFloat, 
  Descriptor : mat::Descriptor 
>
( 
  from : &Mat4< E, Descriptor >, 
  to : &mut Mat3< E, Descriptor >, 
  i : usize, 
  j : usize 
)
where 
Mat4< E, Descriptor > : RawSliceMut< Scalar = E > + IndexingRef< Scalar = E, Index = Ix2 >,
Mat3< E, Descriptor > : RawSliceMut< Scalar = E >
{
  for( id, ( _, v ) ) in from
  .iter_indexed_unstable()
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

fn cofactor
< 
  E : MatEl + nd::NdFloat, 
  Descriptor : mat::Descriptor 
>
( 
  from : &Mat4< E, Descriptor >, 
  to : &mut Mat3< E, Descriptor >,  
  i : usize, 
  j : usize 
) -> E
where 
Mat4< E, Descriptor > : 
  RawSliceMut< Scalar = E > + 
  IndexingRef< Scalar = E, Index = Ix2 >,
Mat3< E, Descriptor > : 
  RawSliceMut< Scalar = E > + 
  ScalarRef< Scalar = E, Index = Ix2 > + 
  ConstLayout< Index = Ix2 > + 
  IndexingMut< Scalar = E, Index = Ix2 >
{
  let k = E::from( ( -1i32 ).pow( ( i + j ) as u32 ) ).unwrap();
  minor( from, to, i, j );
  k * to.determinant()
}

impl< E, Descriptor > Mat< 4, 4, E, Descriptor > 
where 
E : MatEl + nd::NdFloat,
Descriptor : mat::Descriptor,
Self : ScalarMut< Scalar = E, Index = Ix2 > +
       RawSliceMut< Scalar = E > + 
       ConstLayout< Index = Ix2 > + 
       IndexingMut< Scalar = E, Index = Ix2 >
{
  /// Converts the matrix to an array
  pub fn to_array( &self ) -> [ E; 16 ]
  {
    self.raw_slice().try_into().unwrap()
  }


  /// Computes the determinant of the matrix
  pub fn determinant( &self ) -> E
  where 
    Mat< 3, 3, E, Descriptor > : 
      RawSliceMut< Scalar = E > +
      ScalarMut< Scalar = E, Index = Ix2 > + 
      ConstLayout< Index = Ix2 > + 
      IndexingMut< Scalar = E, Index = Ix2 >
  {
    let _a11 = *self.scalar_ref( Ix2( 0, 0 ) );
    let _a12 = *self.scalar_ref( Ix2( 0, 1 ) );
    let _a13 = *self.scalar_ref( Ix2( 0, 2 ) );
    let _a14 = *self.scalar_ref( Ix2( 0, 3 ) );

    let mut m = Mat3::< E, Descriptor >::default();

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

  /// Computes the inverse of the matrix.
  /// If the determinant is zero - return `None`
  pub fn inverse( &self ) -> Option< Self >
  where 
    Mat< 3, 3, E, Descriptor > : 
      RawSliceMut< Scalar = E > +
      ScalarMut< Scalar = E, Index = Ix2 > + 
      ConstLayout< Index = Ix2 > + 
      IndexingMut< Scalar = E, Index = Ix2 >
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