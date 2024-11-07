mod private
{
  use mdmath_core::vector::mul_scalar;

use crate::*;

  impl< E, const ROWS : usize, const COLS : usize, Descriptor > MulAssign< Mat< ROWS, COLS, E, Descriptor > >
  for  Vector< E, COLS >
  where
    Descriptor : mat::Descriptor,
    E : MatEl + nd::NdFloat,
    Mat< ROWS, COLS, E, Descriptor > : Indexable< Index = Ix2 > + IndexingRef< Scalar = E >,
  {
    fn mul_assign( &mut self, rhs: Mat< ROWS, COLS, E, Descriptor > ) {
      *self = rhs * *self;
    }
  }

  impl< E, const LEN : usize > Mul< E > for Vector< E, LEN >
  where
    E : MatEl + nd::NdFloat
  {
    type Output = Self;

    fn mul( self, rhs: E ) -> Self::Output {
      mul_scalar( &self, rhs )
    }
  }

  impl< E, const LEN : usize > MulAssign< E > for Vector< E, LEN >
  where
    E : MatEl + nd::NdFloat
  {
    fn mul_assign( &mut self, rhs: E ) {
      *self = *self * rhs;
    }
  }
}

crate::mod_interface!
{
  
}