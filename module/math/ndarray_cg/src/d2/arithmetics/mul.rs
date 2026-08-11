use crate::{MatNum, Indexable, Ix2, ScalarMut, IndexingRef, nd, VectorIterMut, VectorIter, Mul, Mat, mat, IndexingMut, Vector};

/// Multiplies two matrices.
///
/// # Overflow
/// For integer `E` the inner-product accumulation is not overflow-checked: it
/// panics in debug / wraps in release once a product or partial sum leaves
/// `E`'s range.
///
/// # Panics
/// Panics if the inner dimensions of `a`/`b` or the shape of `r` are
/// incompatible with matrix multiplication.
#[ inline ]
pub fn mul< E, A, B, R >( r : &mut R, a : &A, b : &B )
where
  E : MatNum,
  R : Indexable< Index = Ix2 > + ScalarMut< Scalar = E >,
  A : Indexable< Index = Ix2 > + IndexingRef< Scalar = E >,
  B : Indexable< Index = Ix2 > + IndexingRef< Scalar = E >,
{
  let adim = a.dim();
  let bdim = b.dim();

  // Fix(TASK-014): removed `#[ cfg( debug_assertions ) ]` so this dimension check runs
  // unconditionally instead of only in debug builds.
  // Root cause: the check was gated to debug builds, so a release build skipped it and
  // `a.lane_iter(..).zip(b.lane_iter(..))` below silently truncated to the shorter lane on
  // an incompatible inner dimension, producing a wrong dot product instead of failing.
  // Pitfall: gating a correctness-critical dimension check behind `debug_assertions` makes
  // release builds trade a loud failure for silently wrong numeric output.
  {
    let rdim = r.dim();

    // Check if dimensions are compatible for multiplication
    assert!
    (
      adim[ 1 ] == bdim[ 0 ] && rdim[ 0 ] == adim[ 0 ] && rdim[ 1 ] == bdim[ 1 ],
      "Incompatible dimensions for matrix multiplication : a : {adim:?}, b : {bdim:?}, r : {rdim:?}"
    );
  }

  for row in 0..adim[ 0 ]
  {
    for col in 0..bdim[ 1 ]
    {
      *r.scalar_mut( nd::Ix2( row, col ) ) = a.lane_iter( 0, row )
      .zip( b.lane_iter( 1, col ) )
      .map( | ( a_val, b_val ) | *a_val * *b_val )
      .fold( E::zero(), | sum, val | sum + val );
    }
  }
}

/// Multiplies vector by a matrix.
///
/// # Overflow
/// For integer `E` the inner-product accumulation is not overflow-checked: it
/// panics in debug / wraps in release once a product or partial sum leaves
/// `E`'s range.
///
/// # Panics
/// Panics if `a`'s column count does not equal `ROWS`.
#[ inline ]
pub fn mul_mat_vec< E, A, B, R, const ROWS : usize >( r : &mut R, a : &A, b : &B )
where
  E : MatNum,
  R : VectorIterMut< E, ROWS >,
  A : Indexable< Index = Ix2 > + IndexingRef< Scalar = E >,
  B : VectorIter< E, ROWS >,
{
  // Fix(TASK-014): removed `#[ cfg( debug_assertions ) ]` so this dimension check runs
  // unconditionally instead of only in debug builds.
  // Root cause: the check was gated to debug builds, so a release build skipped it and
  // `a.lane_iter(..).zip(b.vector_iter())` below silently truncated to the shorter of the
  // two on a mismatched shape, producing a wrong dot product instead of failing. This
  // free function is only reachable directly (bypassing the `Mul<Vector<COLS>>` operator
  // impls, which always pass a matching `COLS`), so the check was its only guard.
  // Pitfall: gating a correctness-critical dimension check behind `debug_assertions` makes
  // release builds trade a loud failure for silently wrong numeric output.
  {
    let adim = a.dim();

    // Check if dimensions are compatible for multiplication
    assert!
    (
      adim[ 1 ] == ROWS,
      "Incompatible dimensions for matrix-vector multiplication : a : {adim:?}, b : {ROWS:?}, r : {ROWS:?}"
    );
  }

  for ( row, e ) in r.vector_iter_mut().enumerate()
  {
    *e = a.lane_iter( 0, row )
    .zip( b.vector_iter() )
    .map( | ( a_val, b_val ) | *a_val * *b_val )
    .fold( E::zero(), | sum, val | sum + val );
  }
}

impl< E, const ROWS : usize, const COLS : usize, const COLS2 : usize, Descriptor > Mul< Mat< COLS, COLS2, E, Descriptor > >
for Mat< ROWS, COLS, E, Descriptor >
where
  Descriptor : mat::Descriptor,
  E : MatNum,
  Mat< ROWS, COLS, E, Descriptor > : Indexable< Index = Ix2 > + IndexingMut< Scalar = E >,
  Mat< COLS, COLS2, E, Descriptor > : Indexable< Index = Ix2 > + IndexingRef< Scalar = E >,
  Mat< ROWS, COLS2, E, Descriptor > : Indexable< Index = Ix2 > + ScalarMut< Scalar = E >,
{
  type Output = Mat< ROWS, COLS2, E, Descriptor >;

  /// # Overflow
  /// For integer `E` the inner-product accumulation is not overflow-checked: it
  /// panics in debug / wraps in release once a product or partial sum leaves
  /// `E`'s range.
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
  E : MatNum,
  Mat< ROWS, COLS, E, Descriptor > : Indexable< Index = Ix2 > + IndexingMut< Scalar = E >,
  Mat< COLS, COLS2, E, Descriptor > : Indexable< Index = Ix2 > + IndexingRef< Scalar = E >,
  Mat< ROWS, COLS2, E, Descriptor > : Indexable< Index = Ix2 > + ScalarMut< Scalar = E >,
{
  type Output = Mat< ROWS, COLS2, E, Descriptor >;

  /// # Overflow
  /// For integer `E` the inner-product accumulation is not overflow-checked: it
  /// panics in debug / wraps in release once a product or partial sum leaves
  /// `E`'s range.
  #[ inline ]
  fn mul( self, rhs : &Mat< COLS, COLS2, E, Descriptor > ) -> Self::Output
  {
    let mut result = Self::Output::default();
    mul( &mut result, self, rhs );
    result
  }
}

//
// Vector
//

impl< E, const ROWS : usize, const COLS : usize, Descriptor > Mul< Vector< E, COLS > >
for Mat< ROWS, COLS, E, Descriptor >
where
  Descriptor : mat::Descriptor,
  E : MatNum,
  Mat< ROWS, COLS, E, Descriptor > : Indexable< Index = Ix2 > + IndexingRef< Scalar = E >,
{
  type Output = Vector< E, COLS >;

  /// # Overflow
  /// For integer `E` the inner-product accumulation is not overflow-checked: it
  /// panics in debug / wraps in release once a product or partial sum leaves
  /// `E`'s range.
  #[ inline ]
  fn mul( self, rhs : Vector< E, COLS > ) -> Self::Output
  {
    let mut result = Self::Output::default();
    mul_mat_vec( &mut result, &self, &rhs );
    result
  }
}

impl< E, const ROWS : usize, const COLS : usize, Descriptor > Mul< &Vector< E, COLS > >
for &Mat< ROWS, COLS, E, Descriptor >
where
  Descriptor : mat::Descriptor,
  E : MatNum,
  Mat< ROWS, COLS, E, Descriptor > : Indexable< Index = Ix2 > + IndexingRef< Scalar = E >,
{
  type Output = Vector< E, COLS >;

  /// # Overflow
  /// For integer `E` the inner-product accumulation is not overflow-checked: it
  /// panics in debug / wraps in release once a product or partial sum leaves
  /// `E`'s range.
  #[ inline ]
  fn mul( self, rhs : &Vector< E, COLS > ) -> Self::Output
  {
    let mut result = Self::Output::default();
    mul_mat_vec( &mut result, self, rhs );
    result
  }
}

// impl_operator!( mat::DescriptorOrderRowMajor, Mul, mul );
// impl_operator!( mat::DescriptorOrderColumnMajor, Mul, mul );
