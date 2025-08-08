use crate::*;

impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > Mat< ROWS, COLS, E, Descriptor >
where
  E : MatEl,
  // < Self as Collection >::Scalar : Copy,
  Self : RawSlice,
{

  /// Returns a reference to the underlying data as a flat array.
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
  /// Returns a mutable reference to the underlying data as a flat array.
  ///
  /// # Returns
  /// - A mutable slice of the scalar data.
  #[ inline( always ) ]
  pub fn raw_slice_mut( &mut self ) -> &mut [ < Self as Collection >::Scalar ]
  {
    < Self as RawSliceMut >::raw_slice_mut( self )
  }

  /// Sets the underlying data from a slice of scalars.
  ///
  /// # Arguments
  /// - `scalars`: A slice of scalars to set the data.
  #[ inline( always ) ]
  pub fn raw_set_slice( &mut self, scalars : &[ < Self as Collection >::Scalar ] )
  {
    < Self as RawSliceMut >::raw_set_slice( self, scalars )
  }

  /// Sets the underlying data from an array of scalars.
  ///
  /// # Arguments
  /// - `scalars`: An array of scalars to set the data.
  #[ inline( always ) ]
  pub fn raw_set< const N : usize >( self, scalars : [ < Self as Collection >::Scalar ; N ] ) -> Self
  {
    < Self as RawSliceMut >::raw_set( self, scalars )
  }

  /// Sets the underlying data from an array of scalars, assuming the input to be in row major order.
  /// The resulting data will conform to the type of matrix used - row major or column major.
  ///
  /// # Arguments
  /// - `scalars`: An array of scalars to set the data.
  #[ inline( always ) ]
  pub fn set< const N : usize >( self, scalars : [ < Self as Collection >::Scalar ; N ] ) -> Self
  {
    < Self as RawSliceMut >::set( self, scalars )
  }

  /// Sets the underlying data from an array of scalars, assuming the input to be in row major order.
  /// The resulting data will conform to the type of matrix used - row major or column major.
  ///
  /// # Arguments
  /// - `scalars`: An array of scalars to set the data.
  #[ inline( always ) ]
  pub fn with_row_major( self, scalars : &[ < Self as Collection >::Scalar ] ) -> Self
  {
    < Self as RawSliceMut >::with_row_major( self, scalars )
  }

  /// Sets the underlying data from an array of scalars, assuming the input to be in column major order.
  /// The resulting data will conform to the type of matrix used - row major or column major.
  ///
  /// /// # Arguments
  /// - `scalars`: An array of scalars to set the data.
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
  /// Iterate over scalars of a single 1D lane.
  ///
  /// # Parameters
  /// - `varying_dim`: The dimension that varies (0 for rows, 1 for columns).
  /// - `lane`: The index of the lane (row or column) to iterate over.
  ///
  /// # Returns
  /// - An iterator over references to the scalars in the specified lane.
  #[ inline( always ) ]
  pub fn lane_iter( &self, varying_dim : usize, lane : usize )
  -> impl Iterator< Item = &< Self as Collection >::Scalar >
  {
    < Self as IndexingRef >::lane_iter( self, varying_dim, lane )
  }

  /// Iterate over scalars of a single 1D lane with indices.
  ///
  /// # Parameters
  /// - `varying_dim`: The dimension that varies (0 for rows, 1 for columns).
  /// - `lane`: The index of the lane (row or column) to iterate over.
  ///
  /// # Returns
  /// - An iterator over tuples of indices and references to the scalars in the specified lane.
  #[ inline( always ) ]
  pub fn lane_indexed_iter( &self, varying_dim : usize, lane : usize )
  -> impl Iterator< Item = ( < Self as Indexable >::Index, &< Self as Collection >::Scalar ) >
  {
    < Self as IndexingRef >::lane_indexed_iter( self, varying_dim, lane )
  }

  /// Iterate over all scalars. Order of iteration is not specified.
  ///
  /// # Returns
  /// - An iterator over references to all scalars in the matrix.
  #[ inline( always ) ]
  pub fn iter_unstable( &self )
  -> impl Iterator< Item = &< Self as Collection >::Scalar >
  {
    < Self as IndexingRef >::iter_unstable( self )
  }

  /// Iterate over all scalars with indices. Order of iteration is not specified.
  ///
  /// # Returns
  /// - An iterator over tuples of indices and references to all scalars in the matrix.
  #[ inline( always ) ]
  pub fn iter_indexed_unstable( &self )
  -> impl Iterator< Item = ( < Self as Indexable >::Index, &< Self as Collection >::Scalar ) >
  {
    < Self as IndexingRef >::iter_indexed_unstable( self )
  }

  /// Iterate over all scalars in least significant dimension order.
  ///
  /// # Returns
  /// - An iterator over references to all scalars in the matrix.
  #[ inline( always ) ]
  pub fn iter_lsfirst( &self )
  -> impl Iterator< Item = &< Self as Collection >::Scalar >
  {
    < Self as IndexingRef >::iter_lsfirst( self )
  }

  /// Iterate over all scalars with indices in least significant dimension order.
  ///
  /// # Returns
  /// - An iterator over tuples of indices and references to all scalars in the matrix.
  #[ inline( always ) ]
  pub fn iter_indexed_lsfirst( &self )
  -> impl Iterator< Item = ( < Self as Indexable >::Index, &< Self as Collection >::Scalar ) >
  {
    < Self as IndexingRef >::iter_indexed_lsfirst( self )
  }

  /// Iterate over all scalars in most significant dimension order.
  ///
  /// # Returns
  /// - An iterator over references to all scalars in the matrix.
  #[ inline( always ) ]
  pub fn iter_msfirst( &self )
  -> impl Iterator< Item = &< Self as Collection >::Scalar >
  {
    < Self as IndexingRef >::iter_msfirst( self )
  }

  /// Iterate over all scalars with indices in most significant dimension order.
  ///
  /// # Returns
  /// - An iterator over tuples of indices and references to all scalars in the matrix.
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
  /// Iterate over scalars of a single 1D lane with mutable access.
  ///
  /// # Parameters
  /// - `varying_dim`: The dimension that varies (0 for rows, 1 for columns).
  /// - `lane`: The index of the lane (row or column) to iterate over.
  ///
  /// # Returns
  /// - An iterator over mutable references to the scalars in the specified lane.
  #[ inline( always ) ]
  pub fn lane_iter_mut( &mut self, varying_dim : usize, lane : usize ) -> impl Iterator< Item = &mut < Self as Collection>::Scalar >
  {
    < Self as IndexingMut>::lane_iter_mut( self, varying_dim, lane )
  }

  /// Iterate over scalars of a single 1D lane with indices and mutable access.
  ///
  /// # Parameters
  /// - `varying_dim`: The dimension that varies (0 for rows, 1 for columns).
  /// - `lane`: The index of the lane (row or column) to iterate over.
  ///
  /// # Returns
  /// - An iterator over tuples of indices and mutable references to the scalars in the specified lane.
  #[ inline( always ) ]
  pub fn lane_iter_indexed_mut( &mut self, varying_dim : usize, lane : usize ) -> impl Iterator< Item = ( < Self as Indexable>::Index, &mut < Self as Collection>::Scalar ) >
  {
    < Self as IndexingMut>::lane_iter_indexed_mut( self, varying_dim, lane )
  }

  /// Iterate over all scalars with mutable access. Order of iteration is not specified.
  ///
  /// # Returns
  /// - An iterator over mutable references to all scalars in the matrix.
  #[ inline( always ) ]
  pub fn iter_unstable_mut( &mut self ) -> impl Iterator< Item = &mut < Self as Collection>::Scalar >
  {
    < Self as IndexingMut>::iter_unstable_mut( self )
  }

  /// Iterate over all scalars with indices and mutable access. Order of iteration is not specified.
  ///
  /// # Returns
  /// - An iterator over tuples of indices and mutable references to all scalars in the matrix.
  #[ inline( always ) ]
  pub fn iter_indexed_unstable_mut( &mut self ) -> impl Iterator< Item = ( < Self as Indexable>::Index, &mut < Self as Collection>::Scalar ) >
  {
    < Self as IndexingMut>::iter_indexed_unstable_mut( self )
  }

  /// Iterate over all scalars in least significant dimension order with mutable access.
  ///
  /// # Returns
  /// - An iterator over mutable references to all scalars in the matrix.
  #[ inline( always ) ]
  pub fn iter_lsfirst_mut( &mut self ) -> impl Iterator< Item = &mut < Self as Collection>::Scalar >
  {
    < Self as IndexingMut >::iter_lsfirst_mut( self )
  }

  /// Iterate over all scalars with indices in least significant dimension order with mutable access.
  ///
  /// # Returns
  /// - An iterator over tuples of indices and mutable references to all scalars in the matrix.
  #[ inline( always ) ]
  pub fn iter_indexed_lsfirst_mut( &mut self ) -> impl Iterator< Item = ( < Self as Indexable>::Index, &mut < Self as Collection>::Scalar ) >
  {
    < Self as IndexingMut >::iter_indexed_lsfirst_mut( self )
  }

  /// Iterate over all scalars in most significant dimension order with mutable access.
  ///
  /// # Returns
  /// - An iterator over mutable references to all scalars in the matrix.
  #[ inline( always ) ]
  pub fn iter_msfirst_mut( &mut self ) -> impl Iterator< Item = &mut < Self as Collection>::Scalar >
  {
    < Self as IndexingMut >::iter_msfirst_mut( self )
  }

  /// Iterate over all scalars with indices in most significant dimension order with mutable access.
  ///
  /// # Returns
  /// - An iterator over tuples of indices and mutable references to all scalars in the matrix.
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
  /// Get a reference to a scalar at a specified index.
  ///
  /// # Parameters
  /// - `index`: The index of the scalar to access.
  ///
  /// # Returns
  /// - A mutable reference to the scalar at the specified index.
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
  /// Get a mutable reference to a scalar at a specified index.
  ///
  /// # Parameters
  /// - `index`: The index of the scalar to access.
  ///
  /// # Returns
  /// - A mutable reference to the scalar at the specified index.
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
  /// Get offset for a specified scalar from beginning of underlying buffer.
  #[ inline( always ) ]
  pub fn scalar_offset( index : < Self as Indexable >::Index ) -> usize
  {
    < Self as ConstLayout >::scalar_offset( index )
  }
}
