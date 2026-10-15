use crate::{MatNum, Indexable, Ix2, ScalarMut, IndexingRef, nd, VectorIterMut, VectorIter, Mul, Mat, mat, IndexingMut, Vector, Zero};

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
/// Panics if `a`'s row count does not equal `OUT` or its column count does not equal `IN`.
#[ inline ]
// Fix(BUG-121): split the single `const ROWS : usize` generic into `IN`/`OUT`, and changed
// the dimension check from `adim[ 1 ] == ROWS` to `adim[ 0 ] == OUT && adim[ 1 ] == IN`.
// Root cause: a matrix-vector product's input length (matrix column count) and output
// length (matrix row count) are independent quantities for a non-square matrix, but both
// `R : VectorIterMut< E, ROWS >` (output) and `B : VectorIter< E, ROWS >` (input) reused the
// SAME const generic — every existing caller only ever instantiates square matrices, where
// input length == output length == ROWS coincidentally, so this went unnoticed. For a real
// `Mat<M,N>` with `M != N`, the old signature forced the output vector to have the same
// length as the input vector (matrix columns), silently dropping any output rows beyond
// that length instead of producing the correct `M`-length result.
// Pitfall: two conceptually independent lengths (here: input dimension vs. output dimension
// of a linear map) that happen to coincide for every currently-existing caller (square
// matrices) can be safely, silently unified into one const generic — until a non-square
// instantiation is attempted, which the type system cannot catch because nothing in the
// signature says the two lengths must differ or must match; check the *general* shape
// (M x N, not just the N x N callers that exist today), not just currently-reachable cases.
pub fn mat_vec_mul< E, A, B, R, const IN : usize, const OUT : usize >( r : &mut R, a : &A, b : &B )
where
  E : MatNum,
  R : VectorIterMut< E, OUT >,
  A : Indexable< Index = Ix2 > + IndexingRef< Scalar = E >,
  B : VectorIter< E, IN >,
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
      adim[ 0 ] == OUT && adim[ 1 ] == IN,
      "Incompatible dimensions for matrix-vector multiplication : a : {adim:?}, b : {IN:?}, r : {OUT:?}"
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
  Mat< ROWS, COLS2, E, Descriptor > : Indexable< Index = Ix2 > + IndexingMut< Scalar = E > + ScalarMut< Scalar = E >,
{
  type Output = Mat< ROWS, COLS2, E, Descriptor >;

  /// # Overflow
  /// For integer `E` the inner-product accumulation is not overflow-checked: it
  /// panics in debug / wraps in release once a product or partial sum leaves
  /// `E`'s range.
  #[ inline ]
  fn mul( self, rhs : Mat< COLS, COLS2, E, Descriptor > ) -> Self::Output
  {
    let mut result = Self::Output::zero();
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
  Mat< ROWS, COLS2, E, Descriptor > : Indexable< Index = Ix2 > + IndexingMut< Scalar = E > + ScalarMut< Scalar = E >,
{
  type Output = Mat< ROWS, COLS2, E, Descriptor >;

  /// # Overflow
  /// For integer `E` the inner-product accumulation is not overflow-checked: it
  /// panics in debug / wraps in release once a product or partial sum leaves
  /// `E`'s range.
  #[ inline ]
  fn mul( self, rhs : &Mat< COLS, COLS2, E, Descriptor > ) -> Self::Output
  {
    let mut result = Self::Output::zero();
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
  // Fix(BUG-121): changed `Output` from `Vector< E, COLS >` to `Vector< E, ROWS >`.
  // Root cause: a matrix-vector product's result has one component per matrix ROW, not per
  // column — `Vector<E,COLS>` was only accidentally right for square (ROWS==COLS) matrices,
  // this crate's only current instantiations. See `mat_vec_mul`'s own fix comment for the
  // full mechanism (this Output type is what caused `mat_vec_mul`'s IN/OUT to both bind to
  // COLS instead of the correct COLS/ROWS pair).
  // Pitfall: see `mat_vec_mul`'s comment above.
  type Output = Vector< E, ROWS >;

  /// # Overflow
  /// For integer `E` the inner-product accumulation is not overflow-checked: it
  /// panics in debug / wraps in release once a product or partial sum leaves
  /// `E`'s range.
  #[ inline ]
  fn mul( self, rhs : Vector< E, COLS > ) -> Self::Output
  {
    // Not migrated to `.zero()` (task 391): `Self::Output` here is `Vector<E,ROWS>`, not `Mat`
    // -- `Vector` has no `Zero` impl (out of scope for task 391), so `.default()` stays as the
    // zero-seed for this matrix-vector product.
    let mut result = Self::Output::default();
    mat_vec_mul( &mut result, &self, &rhs );
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
  // Fix(BUG-121): changed `Output` from `Vector< E, COLS >` to `Vector< E, ROWS >` — see the
  // owned `Mat * Vector` impl above for the full `Fix(BUG-121)`/`Root cause`/`Pitfall` comment.
  type Output = Vector< E, ROWS >;

  /// # Overflow
  /// For integer `E` the inner-product accumulation is not overflow-checked: it
  /// panics in debug / wraps in release once a product or partial sum leaves
  /// `E`'s range.
  #[ inline ]
  fn mul( self, rhs : &Vector< E, COLS > ) -> Self::Output
  {
    // Not migrated to `.zero()` -- same reason as the owned `Mat * Vector` impl above
    // (`Self::Output` is `Vector<E,ROWS>`, which has no `Zero` impl).
    let mut result = Self::Output::default();
    mat_vec_mul( &mut result, self, rhs );
    result
  }
}

// impl_operator!( mat::DescriptorOrderRowMajor, Mul, mul );
// impl_operator!( mat::DescriptorOrderColumnMajor, Mul, mul );
