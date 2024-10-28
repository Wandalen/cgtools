use crate::*;

impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > RawSlice
for Mat< ROWS, COLS, E, Descriptor >
where
  E : MatEl,
{

  /// Returns a reference to the underlying data as a flat array.
  #[ inline( always ) ]
  fn raw_slice( &self ) -> &[ Self::Scalar ]
  {
    // SAFETY: This is safe because the memory layout of [ [ E ; COLS ] ; ROWS ]
    // is contiguous and can be reinterpreted as a flat slice of E.
    #[ allow( unsafe_code ) ]
    unsafe { std::slice::from_raw_parts( self.as_ptr() as *const Self::Scalar, ROWS * COLS ) }
  }

}

// impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > ZeroIdentity
// for Mat< ROWS, COLS, E, Descriptor >
// where
//   E : nd::NdFloat + Copy,
//   Self : IndexingMut< Scalar = E >,
//   // Self : Collection< Scalar = E >,
//   // Self : Add< Self, Output = Self >,
// {
//
//   #[ inline( always ) ]
//   fn zer() -> Self
//   {
//     Self::_fill( E::zero() )
//   }
//
//   #[ inline( always ) ]
//   fn is_zer( &self ) -> bool
//   {
//     < Self as IndexingRef >::iter_unstable( self ).all( | e | e.is_zero() )
//   }
//
//   #[ inline( always ) ]
//   fn zer_set( &mut self )
//   {
//     < Self as IndexingMut >::iter_unstable_mut( self ).for_each( | e | e.set_zero() )
//   }
//
// }

impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > ScalarRef
for Mat< ROWS, COLS, E, Descriptor >
where
  E : MatEl,
  E : nd::NdFloat,
  Self : ConstLayout,
{
  #[ inline( always ) ]
  fn scalar_ref( &self, index : < Self as Indexable >::Index ) -> &Self::Scalar
  {
    &self.raw_slice()[ Self::scalar_offset( index ) ]
  }
}

impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > ScalarMut
for Mat< ROWS, COLS, E, Descriptor >
where
  E : MatEl,
  E : nd::NdFloat,
  Self : ConstLayout + RawSliceMut,
{
  #[ inline( always ) ]
  fn scalar_mut( &mut self, index : < Self as Indexable >::Index ) -> &mut Self::Scalar
  {
    &mut self.raw_slice_mut()[ Self::scalar_offset( index ) ]
  }
}

impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > Mat< ROWS, COLS, E, Descriptor >
where
  E : MatEl,
  Self : RawSliceMut< Scalar = E >,
{
  /// Creates a matrix assuming the input to be in row major order
  pub fn from_row_major< const N : usize >( scalars: impl VectorRef< E, N > ) -> Self {
    debug_assert_eq!( N, ROWS*COLS, "Matrix size should be equal to the size of the input" );

    let result = Self::default();
    result.with_row_major( scalars.vector_ref() )
  }

  /// Creates a matrix assuming the input to be in column major order
  pub fn from_column_major< const N : usize >( scalars: impl VectorRef< E, N > ) -> Self {
    debug_assert_eq!( N, ROWS*COLS, "Matrix size should be equal to the size of the input" );

    let result = Self::default();
    result.with_column_major( scalars.vector_ref() )
  }
}