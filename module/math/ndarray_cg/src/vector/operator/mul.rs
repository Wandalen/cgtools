mod private
{
  use mdmath_core::vector::mul_scalar;
  use mdmath_core::vector::mul;

  use crate::*;

  // Vector * Matrix
  impl< E, const ROWS : usize, const COLS : usize, Descriptor > MulAssign< Mat< ROWS, COLS, E, Descriptor > >
  for  Vector< E, COLS >
  where
    Descriptor : mat::Descriptor,
    E : MatEl + nd::NdFloat,
    Mat< ROWS, COLS, E, Descriptor > : Indexable< Index = Ix2 > + IndexingRef< Scalar = E >,
  {
    fn mul_assign( &mut self, rhs : Mat< ROWS, COLS, E, Descriptor > )
    {
      *self = rhs * *self;
    }
  }

  // Vector * Vector
  impl< E, const LEN : usize > Mul for Vector< E, LEN >
  where
    E : MatEl + nd::NdFloat
  {
    type Output = Self;

    fn mul( self, rhs : Self ) -> Self::Output
    {
      mul( &self, &rhs )
    }
  }

  // Vector * Scalar
  impl< E, const LEN : usize > Mul< E > for Vector< E, LEN >
  where
    E : MatEl + nd::NdFloat
  {
    type Output = Self;

    fn mul( self, rhs : E ) -> Self::Output
    {
      mul_scalar( &self, rhs )
    }
  }

  // Vector *= Scalar
  impl< E, const LEN : usize > MulAssign< E > for Vector< E, LEN >
  where
    E : MatEl + nd::NdFloat
  {
    fn mul_assign( &mut self, rhs : E )
    {
      *self = *self * rhs;
    }
  }

  // Scalar * Vector
  impl< const LEN : usize > Mul< Vector< f32, LEN > > for f32
  {
    type Output = Vector< f32, LEN >;

    fn mul( self, rhs : Vector< f32, LEN > ) -> Self::Output
    {
      mul_scalar( &rhs, self )
    }
  }

  // Scalar * Vector
  impl< const LEN : usize > Mul< Vector< f64, LEN > > for f64
  {
    type Output = Vector< f64, LEN >;

    fn mul( self, rhs : Vector< f64, LEN > ) -> Self::Output
    {
      mul_scalar( &rhs, self )
    }
  }
}

crate::mod_interface!
{

}