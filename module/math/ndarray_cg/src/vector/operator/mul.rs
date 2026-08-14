mod private
{
  use mdmath_core::vector::scalar_mul;
  use mdmath_core::vector::mul;

  use crate::{MulAssign, Mat, Vector, mat, MatNum, Indexable, Ix2, IndexingRef, Mul};

  // Vector * Matrix
  impl< E, const ROWS : usize, const COLS : usize, Descriptor > MulAssign< Mat< ROWS, COLS, E, Descriptor > >
  for  Vector< E, COLS >
  where
    Descriptor : mat::Descriptor,
    E : MatNum,
    Mat< ROWS, COLS, E, Descriptor > : Indexable< Index = Ix2 > + IndexingRef< Scalar = E >,
  {
    #[ inline ]
    fn mul_assign( &mut self, rhs : Mat< ROWS, COLS, E, Descriptor > )
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