use crate::*;

impl< E, Descriptor > Mat< 3, 3, E, Descriptor > 
where 
E : MatEl + nd::NdFloat,
Descriptor : mat::Descriptor,
Self : ScalarMut< Scalar = E, Index = Ix2 >,
Self : RawSliceMut< Scalar = E >,
Self : ConstLayout< Index = Ix2 >,
Self : IndexingMut< Scalar = E, Index = Ix2 >
{
  /// Construct a matrix from columns
  pub fn from_cols
  ( 
    x : Vector< E, 3 >,
    y : Vector< E, 3 >,
    z : Vector< E, 3 >
  ) -> Self
  {
    let x = x.vector_ref();
    let y = y.vector_ref();
    let z = z.vector_ref();

    Self::from_column_major
    ([
      x[ 0 ], x[ 1 ], x[ 2 ],
      y[ 0 ], y[ 1 ], y[ 2 ],
      z[ 0 ], z[ 1 ], z[ 2 ]
    ])
  }

  /// Computes the determinant of the matrix
  pub fn determinant( &self ) -> E
  {
    let a = *self.scalar_ref( Ix2( 0, 0 ) );
    let b = *self.scalar_ref( Ix2( 0, 1 ) );
    let c = *self.scalar_ref( Ix2( 0, 2 ) );

    let d = *self.scalar_ref( Ix2( 1, 0 ) );
    let e = *self.scalar_ref( Ix2( 1, 1 ) );
    let f = *self.scalar_ref( Ix2( 1, 2 ) );

    let g = *self.scalar_ref( Ix2( 2, 0 ) );
    let h = *self.scalar_ref( Ix2( 2, 1 ) );
    let i = *self.scalar_ref( Ix2( 2, 2 ) );

    ( a * e * i ) + 
    ( b * f * g ) + 
    ( c * d * h ) -
    ( c * e * g ) -
    ( b * d * i ) -
    ( a * f * h )
  }    

  /// Computes the inverse of the matrix.
  /// If the determinant is zero - return `None`
  pub fn inverse( &self ) -> Option< Self >
  {
    let det = self.determinant();

    if det == E::zero() { return None; }
    let x = Vector::< E, 3 >::try_from( &self.raw_slice()[ 0..3 ] ).unwrap();
    let y = Vector::< E, 3 >::try_from( &self.raw_slice()[ 3..6 ] ).unwrap();
    let z = Vector::< E, 3 >::try_from( &self.raw_slice()[ 6..9 ] ).unwrap();

    let a = y.cross( z );
    let b = z.cross( x );
    let c = x.cross( y );

    let res = Self::from_cols( a, b, c ) / det;
    Some( res.transpose() )
  }
}