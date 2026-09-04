mod private
{
  use mdmath_core::vector::scalar_mul;
  use mdmath_core::vector::mul;

  use crate::{MulAssign, Mat, Vector, mat, MatNum, Indexable, Ix2, IndexingRef, Mul};

  // Fix(BUG-121): narrowed the matrix's independent `ROWS`/`COLS` generics to a single `N`
  // (i.e. `Mat<N,N,..>`), matching the `Vector<E,N>` this impl mutates in place.
  // Root cause: `MulAssign` has no separate `Output` type -- `*self = rhs * *self` requires
  // the product's type to equal `Self` exactly. With independent `ROWS`/`COLS` generics this
  // impl only type-checked because `Mat<ROWS,COLS> * Vector<COLS>`'s `Output` was itself
  // wrongly pinned to `Vector<E,COLS>` (the very defect BUG-121 fixes elsewhere in this
  // crate, see `src/d2/arithmetics/mul.rs`); once that `Output` was corrected to the
  // mathematically-real `Vector<E,ROWS>`, `*self = rhs * *self` no longer matched `Self`
  // (`Vector<E,COLS>`) for any non-square instantiation, and the impl failed to compile.
  // Pitfall: `v *= M` is only dimensionally sound in the first place when `M` is square --
  // a non-square matrix produces a result vector of a different length than its input,
  // which `MulAssign`'s in-place contract cannot express. Never parameterize this kind of
  // impl over independent row/column generics; require one shared `N` up front.
  impl< E, const N : usize, Descriptor > MulAssign< Mat< N, N, E, Descriptor > >
  for  Vector< E, N >
  where
    Descriptor : mat::Descriptor,
    E : MatNum,
    Mat< N, N, E, Descriptor > : Indexable< Index = Ix2 > + IndexingRef< Scalar = E >,
  {
    #[ inline ]
    fn mul_assign( &mut self, rhs : Mat< N, N, E, Descriptor > )
    {
      *self = rhs * *self;
    }
  }

  // Vector * Vector
  impl< E, const LEN : usize > Mul for Vector< E, LEN >
  where
    E : MatNum
  {
    type Output = Self;

    /// # Overflow
    /// For integer `E` the element-wise multiplication is not overflow-checked:
    /// it panics in debug / wraps in release once a product leaves `E`'s range.
    #[ inline ]
    fn mul( self, rhs : Self ) -> Self::Output
    {
      mul( &self, &rhs )
    }
  }

  // Vector * Scalar
  impl< E, const LEN : usize > Mul< E > for Vector< E, LEN >
  where
    E : MatNum
  {
    type Output = Self;

    /// # Overflow
    /// For integer `E` the element-wise multiplication is not overflow-checked:
    /// it panics in debug / wraps in release once a product leaves `E`'s range.
    #[ inline ]
    fn mul( self, rhs : E ) -> Self::Output
    {
      scalar_mul( &self, rhs )
    }
  }

  // Vector *= Scalar
  impl< E, const LEN : usize > MulAssign< E > for Vector< E, LEN >
  where
    E : MatNum
  {
    /// # Overflow
    /// For integer `E` the element-wise multiplication is not overflow-checked:
    /// it panics in debug / wraps in release once a product leaves `E`'s range.
    #[ inline ]
    fn mul_assign( &mut self, rhs : E )
    {
      *self = *self * rhs;
    }
  }

  // Scalar * Vector
  impl< const LEN : usize > Mul< Vector< f32, LEN > > for f32
  {
    type Output = Vector< f32, LEN >;

    #[ inline ]
    fn mul( self, rhs : Vector< f32, LEN > ) -> Self::Output
    {
      scalar_mul( &rhs, self )
    }
  }

  // Scalar * Vector
  impl< const LEN : usize > Mul< Vector< f64, LEN > > for f64
  {
    type Output = Vector< f64, LEN >;

    #[ inline ]
    fn mul( self, rhs : Vector< f64, LEN > ) -> Self::Output
    {
      scalar_mul( &rhs, self )
    }
  }

  // Scalar * Vector
  impl< const LEN : usize > Mul< Vector< i32, LEN > > for i32
  {
    type Output = Vector< i32, LEN >;

    /// # Overflow
    /// For integer `E` the element-wise multiplication is not overflow-checked:
    /// it panics in debug / wraps in release once a product leaves `E`'s range.
    #[ inline ]
    fn mul( self, rhs : Vector< i32, LEN > ) -> Self::Output
    {
      scalar_mul( &rhs, self )
    }
  }

  // Scalar * Vector
  impl< const LEN : usize > Mul< Vector< i64, LEN > > for i64
  {
    type Output = Vector< i64, LEN >;

    /// # Overflow
    /// For integer `E` the element-wise multiplication is not overflow-checked:
    /// it panics in debug / wraps in release once a product leaves `E`'s range.
    #[ inline ]
    fn mul( self, rhs : Vector< i64, LEN > ) -> Self::Output
    {
      scalar_mul( &rhs, self )
    }
  }

  // Scalar * Vector
  impl< const LEN : usize > Mul< Vector< u32, LEN > > for u32
  {
    type Output = Vector< u32, LEN >;

    /// # Overflow
    /// For integer `E` the element-wise multiplication is not overflow-checked:
    /// it panics in debug / wraps in release once a product leaves `E`'s range.
    #[ inline ]
    fn mul( self, rhs : Vector< u32, LEN > ) -> Self::Output
    {
      scalar_mul( &rhs, self )
    }
  }

  // Scalar * Vector
  impl< const LEN : usize > Mul< Vector< u64, LEN > > for u64
  {
    type Output = Vector< u64, LEN >;

    /// # Overflow
    /// For integer `E` the element-wise multiplication is not overflow-checked:
    /// it panics in debug / wraps in release once a product leaves `E`'s range.
    #[ inline ]
    fn mul( self, rhs : Vector< u64, LEN > ) -> Self::Output
    {
      scalar_mul( &rhs, self )
    }
  }
}

crate::mod_interface!
{

}