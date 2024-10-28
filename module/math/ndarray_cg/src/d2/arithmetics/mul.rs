use crate::*;

/// Multiplies two matrices.
pub fn mul< E, A, B, R >( r : &mut R, a : &A, b : &B )
where
  E : nd::NdFloat,
  R : Indexable< Index = Ix2 > + ScalarMut< Scalar = E >,
  A : Indexable< Index = Ix2 > + IndexingRef< Scalar = E >,
  B : Indexable< Index = Ix2 > + IndexingRef< Scalar = E >,
{
  let rdim = r.dim();
  let adim = a.dim();
  let bdim = b.dim();

  // Check if dimensions are compatible for multiplication
  #[ cfg( debug_assertions ) ]
  if adim[ 1 ] != bdim[ 0 ] || rdim[ 0 ] != adim[ 0 ] || rdim[ 1 ] != bdim[ 1 ]
  {
    panic!
    (
      "Incompatible dimensions for matrix multiplication : a : {:?}, b : {:?}, r : {:?}",
      adim, bdim, rdim
    );
  }

  // println!( "a : {:?}, b : {:?}, r : {:?}", adim, bdim, rdim );
  // println!( "a.lane( 0, 0 ) : {:?}", a.lane_iter( 0, 0 ).collect::< Vec< _ > >() );
  // println!( "b.lane( 1, 0 ) : {:?}", a.lane_iter( 1, 0 ).collect::< Vec< _ > >() );
  for row in 0..adim[ 0 ]
  {
    for col in 0..bdim[ 1 ]
    {
      println!( "{:?}", ( row, col ) );
      *r.scalar_mut( nd::Ix2( row, col ) ) = a.lane_iter( 0, row )
      .zip( b.lane_iter( 1, col ) )
      .map( | ( a_val, b_val ) | *a_val * *b_val )
      .fold( E::zero(), | sum, val | sum + val );
    }
  }

}

impl< E, const ROWS : usize, const COLS : usize, const COLS2 : usize, Descriptor > Mul< Mat< COLS, COLS2, E, Descriptor > >
for Mat< ROWS, COLS, E, Descriptor >
where
  Descriptor : mat::Descriptor,
  E : MatEl,
  E : nd::NdFloat,
  Mat< ROWS, COLS, E, Descriptor > : Indexable< Index = Ix2 > + IndexingMut< Scalar = E >,
  Mat< COLS, COLS2, E, Descriptor > : Indexable< Index = Ix2 > + IndexingRef< Scalar = E >,
  Mat< ROWS, COLS2, E, Descriptor > : Indexable< Index = Ix2 > + ScalarMut< Scalar = E >,
{
  type Output = Mat< ROWS, COLS2, E, Descriptor >;

  #[ inline ]
  fn mul( self, rhs : Mat< COLS, COLS2, E, Descriptor > ) -> Self::Output
  {
    let mut result = Self::Output::default();
    mul( &mut result, &self, &rhs );
    result
  }
}

impl< E, const ROWS : usize, const COLS : usize, const COLS2 : usize, Descriptor > Mul< &Mat< COLS, COLS2, E, Descriptor > >
for &Mat< ROWS, COLS, E, Descriptor >
where
  Descriptor : mat::Descriptor,
  E : MatEl,
  E : nd::NdFloat,
  Mat< ROWS, COLS, E, Descriptor > : Indexable< Index = Ix2 > + IndexingMut< Scalar = E >,
  Mat< COLS, COLS2, E, Descriptor > : Indexable< Index = Ix2 > + IndexingRef< Scalar = E >,
  Mat< ROWS, COLS2, E, Descriptor > : Indexable< Index = Ix2 > + ScalarMut< Scalar = E >,
{
  type Output = Mat< ROWS, COLS2, E, Descriptor >;

  #[ inline ]
  fn mul( self, rhs : &Mat< COLS, COLS2, E, Descriptor > ) -> Self::Output
  {
    let mut result = Self::Output::default();
    mul( &mut result, self, rhs );
    result
  }
}

// impl_operator!( mat::DescriptorOrderRowMajor, Mul, mul );
// impl_operator!( mat::DescriptorOrderColumnMajor, Mul, mul );
