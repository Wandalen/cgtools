use crate::*;

impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > Mat< ROWS, COLS, E, Descriptor >
where
  E : MatEl,
  // < Self as Collection >::Scalar : Copy,
  Self : RawSlice,
{

  #[ inline( always ) ]
  pub fn raw_slice( &self ) -> &[ < Self as Collection >::Scalar ]
  {
    < Self as RawSlice >::raw_slice( self )
  }

}

impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > Mat< ROWS, COLS, E, Descriptor >
where
  E : MatEl,
  // < Self as Collection >::Scalar : Copy,
  Self : RawSliceMut,
{

  #[ inline( always ) ]
  pub fn raw_slice_mut( &mut self ) -> &mut [ < Self as Collection >::Scalar ]
  {
    < Self as RawSliceMut >::raw_slice_mut( self )
  }

  #[ inline( always ) ]
  pub fn raw_set_slice( &mut self, scalars : &[ < Self as Collection >::Scalar ] )
  {
    < Self as RawSliceMut >::raw_set_slice( self, scalars )
  }

  #[ inline( always ) ]
  pub fn raw_set< const N : usize >( self, scalars : [ < Self as Collection >::Scalar ; N ] ) -> Self
  {
    < Self as RawSliceMut >::raw_set( self, scalars )
  }

  #[ inline( always ) ]
  pub fn set< const N : usize >( self, scalars : [ < Self as Collection >::Scalar ; N ] ) -> Self
  {
    < Self as RawSliceMut >::set( self, scalars )
  }

  #[ inline( always ) ]
  pub fn with_row_major( self, scalars : &[ < Self as Collection >::Scalar ] ) -> Self
  {
    < Self as RawSliceMut >::with_row_major( self, scalars )
  }

  #[ inline( always ) ]
  pub fn with_column_major( self, scalars : &[ < Self as Collection >::Scalar ] ) -> Self
  {
    < Self as RawSliceMut >::with_column_major( self, scalars )
  }
}

// impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > Mat< ROWS, COLS, E, Descriptor >
// where
//   E : nd::NdFloat + Copy,
//   Self : Zero,
// {
//
//   #[ inline( always ) ]
//   pub fn zero() -> Self
//   {
//     < Self as Zero >::zero()
//   }
//
//   #[ inline( always ) ]
//   pub fn is_zero( &self ) -> bool
//   {
//     < Self as Zero >::is_zero( self )
//   }
//
//   #[ inline( always ) ]
//   pub fn set_zero( &mut self )
//   {
//     < Self as Zero >::set_zero( self )
//   }
//
// }

impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > Mat< ROWS, COLS, E, Descriptor >
where
  E : MatEl,
  E : nd::NdFloat,
  Self : IndexingRef,
{

  #[ inline( always ) ]
  pub fn lane_iter( &self, varying_dim : usize, lane : usize )
  -> impl Iterator< Item = &< Self as Collection >::Scalar >
  {
    < Self as IndexingRef >::lane_iter( self, varying_dim, lane )
  }

  #[ inline( always ) ]
  pub fn lane_indexed_iter( &self, varying_dim : usize, lane : usize )
  -> impl Iterator< Item = ( < Self as Indexable >::Index, &< Self as Collection >::Scalar ) >
  {
    < Self as IndexingRef >::lane_indexed_iter( self, varying_dim, lane )
  }

  #[ inline( always ) ]
  pub fn iter_unstable( &self )
  -> impl Iterator< Item = &< Self as Collection >::Scalar >
  {
    < Self as IndexingRef >::iter_unstable( self )
  }

  #[ inline( always ) ]
  pub fn iter_indexed_unstable( &self )
  -> impl Iterator< Item = ( < Self as Indexable >::Index, &< Self as Collection >::Scalar ) >
  {
    < Self as IndexingRef >::iter_indexed_unstable( self )
  }

  #[ inline( always ) ]
  pub fn iter_lsfirst( &self )
  -> impl Iterator< Item = &< Self as Collection >::Scalar >
  {
    < Self as IndexingRef >::iter_lsfirst( self )
  }

  #[ inline( always ) ]
  pub fn iter_indexed_lsfirst( &self )
  -> impl Iterator< Item = ( < Self as Indexable >::Index, &< Self as Collection >::Scalar ) >
  {
    < Self as IndexingRef >::iter_indexed_lsfirst( self )
  }

  #[ inline( always ) ]
  pub fn iter_msfirst( &self )
  -> impl Iterator< Item = &< Self as Collection >::Scalar >
  {
    < Self as IndexingRef >::iter_msfirst( self )
  }

  #[ inline( always ) ]
  pub fn iter_indexed_msfirst( &self )
  -> impl Iterator< Item = ( < Self as Indexable >::Index, &< Self as Collection >::Scalar ) >
  {
    < Self as IndexingRef >::iter_indexed_msfirst( self )
  }

}

impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > Mat< ROWS, COLS, E, Descriptor >
where
  E : MatEl,
  E : nd::NdFloat,
  Self : IndexingMut,
{
  #[ inline( always ) ]
  pub fn lane_iter_mut( &mut self, varying_dim : usize, lane : usize ) -> impl Iterator< Item = &mut < Self as Collection>::Scalar >
  {
    < Self as IndexingMut>::lane_iter_mut( self, varying_dim, lane )
  }

  #[ inline( always ) ]
  pub fn lane_iter_indexed_mut( &mut self, varying_dim : usize, lane : usize ) -> impl Iterator< Item = ( < Self as Indexable>::Index, &mut < Self as Collection>::Scalar ) >
  {
    < Self as IndexingMut>::lane_iter_indexed_mut( self, varying_dim, lane )
  }

  #[ inline( always ) ]
  pub fn iter_unstable_mut( &mut self ) -> impl Iterator< Item = &mut < Self as Collection>::Scalar >
  {
    < Self as IndexingMut>::iter_unstable_mut( self )
  }

  #[ inline( always ) ]
  pub fn iter_indexed_unstable_mut( &mut self ) -> impl Iterator< Item = ( < Self as Indexable>::Index, &mut < Self as Collection>::Scalar ) >
  {
    < Self as IndexingMut>::iter_indexed_unstable_mut( self )
  }

  #[ inline( always ) ]
  pub fn iter_lsfirst_mut( &mut self ) -> impl Iterator< Item = &mut < Self as Collection>::Scalar >
  {
    < Self as IndexingMut>::iter_lsfirst_mut( self )
  }

  #[ inline( always ) ]
  pub fn iter_indexed_lsfirst_mut( &mut self ) -> impl Iterator< Item = ( < Self as Indexable>::Index, &mut < Self as Collection>::Scalar ) >
  {
    < Self as IndexingMut>::iter_indexed_lsfirst_mut( self )
  }

  #[ inline( always ) ]
  pub fn iter_msfirst_mut( &mut self ) -> impl Iterator< Item = &mut < Self as Collection>::Scalar >
  {
    < Self as IndexingMut>::iter_msfirst_mut( self )
  }

  #[ inline( always ) ]
  pub fn iter_indexed_msfirst_mut( &mut self ) -> impl Iterator< Item = ( < Self as Indexable>::Index, &mut < Self as Collection>::Scalar ) >
  {
    < Self as IndexingMut >::iter_indexed_msfirst_mut( self )
  }
}

impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > Mat< ROWS, COLS, E, Descriptor >
where
  E : MatEl,
  E : nd::NdFloat,
  Self : ScalarRef,
{
  #[ inline( always ) ]
  pub fn scalar_ref( &self, index : < Self as Indexable >::Index ) -> &< Self as Collection >::Scalar
  {
    < Self as ScalarRef >::scalar_ref( self, index )
  }
}

impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > Mat< ROWS, COLS, E, Descriptor >
where
  E : MatEl,
  E : nd::NdFloat,
  Self : ScalarMut,
{
  #[ inline( always ) ]
  pub fn scalar_mut( &mut self, index : < Self as Indexable >::Index ) -> &mut < Self as Collection >::Scalar
  {
    < Self as ScalarMut >::scalar_mut( self, index )
  }
}

impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > Mat< ROWS, COLS, E, Descriptor >
where
  E : MatEl,
  Self : ConstLayout,
{
  #[ inline( always ) ]
  pub fn scalar_offset( index : < Self as Indexable >::Index ) -> usize
  {
    < Self as ConstLayout >::scalar_offset( index )
  }
}
