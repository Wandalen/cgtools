use crate::*;

/// Devide matrix by a scalar.
pub fn div_scalar< E, R >( r : &mut R, a : E )
where
  E : nd::NdFloat,
  R : Indexable< Index = Ix2 > + ScalarMut< Scalar = E >
{
  let rdim = r.dim();

  #[ cfg( debug_assertions ) ]
  if a == E::zero()
  {
    panic!("Matrix division by zero");
  }

  for row in 0..rdim[ 0 ]
  {
    for col in 0..rdim[ 1 ]
    {
      *r.scalar_mut( nd::Ix2( row, col ) ) /= a;
    }
  }
}

impl< E, const ROWS : usize, const COLS : usize, Descriptor > Div< E >
for Mat< ROWS, COLS, E, Descriptor >
where
  Descriptor : mat::Descriptor,
  E : MatEl + nd::NdFloat,
  Self : Indexable< Index = Ix2 > + ScalarMut< Scalar = E >
{
  type Output = Self;

  #[ inline ]
  fn div( mut self, rhs : E ) -> Self::Output
  {
    div_scalar( &mut self, rhs );
    self
  }
}

impl< E, const ROWS : usize, const COLS : usize, Descriptor > DivAssign< E >
for Mat< ROWS, COLS, E, Descriptor >
where
  Descriptor : mat::Descriptor,
  E : MatEl + nd::NdFloat,
  Self : Indexable< Index = Ix2 > + ScalarMut< Scalar = E >
{
  #[ inline ]
  fn div_assign( &mut self, rhs: E ) {
    div_scalar( self, rhs );
  }
}