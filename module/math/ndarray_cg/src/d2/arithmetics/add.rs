use crate::*;

/// Adds two matrices.
#[ inline ]
pub fn add< E, A, B, R >( r : &mut R, a : &A, b : &B )
where
  E : MatEl,
  E : nd::NdFloat,
  R : Indexable< Index = Ix2 > + IndexingMut< Scalar = E >,
  A : Indexable< Index = Ix2 > + IndexingRef< Scalar = E >,
  B : Indexable< Index = Ix2 > + IndexingRef< Scalar = E >,
{
  #[ cfg( debug_assertions ) ]
  {
    let rdim = r.dim();
    let adim = a.dim();
    let bdim = b.dim();

    // Check if dimensions are compatible for addition
    if adim != bdim || rdim != adim
    {
      panic!
      (
        "Incompatible dimensions for matrix addition: a: {:?}, b: {:?}, r: {:?}",
        adim, bdim, rdim
      );
    }
  }

  // println!( "a: {:?}, b: {:?}, r: {:?}", adim, bdim, rdim );
  for ( ( r_val, a_val ), b_val ) in r.iter_lsfirst_mut().zip( a.iter_lsfirst() ).zip( b.iter_lsfirst() )
  {
    *r_val = *a_val + *b_val;
  }
}

// This macro is workaround and undesirable.
// This macro is required because of Rut compiler compilation issue.
// ```
// overflow evaluating the requirement `d2::mat::private::Mat<_, _, _, _>: derive_tools::Add`
// ``

// impl_operator!( mat::DescriptorOrderRowMajor, Add, add );
// impl_operator!( mat::DescriptorOrderColumnMajor, Add, add );

impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > Add
for Mat< ROWS, COLS, E, Descriptor >
where
  E : MatEl,
  E : nd::NdFloat,
  Descriptor : mat::Descriptor,
  Mat< ROWS, COLS, E, Descriptor > : Indexable< Index = Ix2 > + IndexingMut< Scalar = E >,
{
  type Output = Self;

  #[ inline ]
  fn add( self, rhs : Self ) -> Self::Output
  {
    let mut result = Self::Output::default();
    add( &mut result, &self, &rhs );
    result
  }

}

impl< E, const ROWS : usize, const COLS : usize, Descriptor > Add< &Mat< ROWS, COLS, E, Descriptor > >
for &Mat< ROWS, COLS, E, Descriptor >
where
  Descriptor : mat::Descriptor,
  E : MatEl,
  E : nd::NdFloat,
  // Self : IndexingRef,
  Mat< ROWS, COLS, E, Descriptor > : Indexable< Index = Ix2 > + IndexingMut< Scalar = E >,
{
  type Output = Mat< ROWS, COLS, E, Descriptor >;

  fn add( self, rhs : &Mat< ROWS, COLS, E, Descriptor > ) -> Self::Output
  {
    let mut result = Self::Output::default();
    add( &mut result, self, rhs );
    result
  }
}
