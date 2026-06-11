use crate::*;

/// Divide matrix by a scalar.
///
/// # Panics
/// For integer element types this panics if `a` is zero, in both debug and
/// release mode, via Rust's built-in division-by-zero check. For float element
/// types division by zero is not a panic — it yields `INFINITY` or `NAN`.
pub fn div_scalar< E, R >( r : &mut R, a : E )
where
  E : MatNum,
  R : Indexable< Index = Ix2 > + ScalarMut< Scalar = E >
{
  let rdim = r.dim();

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
  E : MatNum,
  Self : Indexable< Index = Ix2 > + ScalarMut< Scalar = E >
{
  type Output = Self;

  /// # Panics
  /// For integer `E` this panics if `rhs` is zero, in both debug and release
  /// mode. For float `E`, division by zero yields `INFINITY` or `NAN` instead.
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
  E : MatNum,
  Self : Indexable< Index = Ix2 > + ScalarMut< Scalar = E >
{
  /// # Panics
  /// For integer `E` this panics if `rhs` is zero, in both debug and release
  /// mode. For float `E`, division by zero yields `INFINITY` or `NAN` instead.
  #[ inline ]
  fn div_assign( &mut self, rhs : E ) {
    div_scalar( self, rhs );
  }
}
