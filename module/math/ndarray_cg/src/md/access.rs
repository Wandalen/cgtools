mod private
{
  use crate::*;

  /// Provides immutable access to the underlying data of a collection as a flat slice.
  pub trait RawSlice : Collection
  {
    /// Returns a reference to the underlying data as a flat array.
    fn raw_slice( &self ) -> &[ Self::Scalar ];
  }

  /// Trait for accessing and modifying the underlying data of a collection as a mutable slice.
  pub trait RawSliceMut : Collection
  {
    /// Returns a mutable reference to the underlying data as a flat array.
    ///
    /// # Returns
    /// - A mutable slice of the scalar data.
    fn raw_slice_mut( &mut self ) -> &mut [ Self::Scalar ];

    /// Sets the underlying data from a slice of scalars.
    ///
    /// # Arguments
    /// - `scalars`: A slice of scalars to set the data.
    ///
    /// # Returns
    /// - The modified collection with the new scalar data.
    fn raw_set_slice( &mut self, scalars : &[ Self::Scalar ] );

    /// Sets the underlying data from an array of scalars.
    ///
    /// # Arguments
    /// - `scalars`: An array of scalars to set the data.
    ///
    /// # Returns
    /// - The modified collection with the new scalar data.
    fn raw_set< const N : usize >( self, scalars : [ Self::Scalar; N ] ) -> Self;

    /// The same as `from_major_row`.
    fn set< const N : usize >( self, scalars : [ Self::Scalar; N ] ) -> Self
    where Self : Sized
    {
      self.with_row_major( &scalars )
    }

    /// Sets the underlying data from an array of scalars, assuming the input to be in row major order.
    /// The resulting data will conform to the type of matrix used - row major or column major.
    ///
    /// # Arguments
    /// - `scalars`: An array of scalars to set the data.
    ///
    /// # Returns
    /// - The modified collection with the new scalar data.
    ///
    /// Example:
    /// If `scalars = [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ]`,
    /// then internally it will be placed in memory like this:
    /// For Row major matrix: `[ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ]`
    /// For Column major matrix: `[ 1.0, 4.0, 2.0, 5.0, 3.0, 6.0 ]`
    fn with_row_major( self, scalars : &[ Self::Scalar ] ) -> Self;

    /// Sets the underlying data from an array of scalars, assuming the input to be in column major order.
    /// The resulting data will conform to the type of matrix used - row major or column major.
    ///
    /// /// # Arguments
    /// - `scalars`: An array of scalars to set the data.
    ///
    /// # Returns
    /// - The modified collection with the new scalar data.
    ///
    /// Example:
    /// If `scalars = [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ]`,
    /// then internally it will be placed in memory like this:
    /// For Row major matrix: `[ 1.0, 3.0, 5.0, 2.0, 4.0, 6.0 ]`
    /// For Column major matrix: `[ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ]`
    fn with_column_major( self, scalars : &[ Self::Scalar ] ) -> Self;
  }

  /// Trait for indexing and iterating over matrix elements.
  pub trait IndexingRef : Collection + Indexable
  {
    /// Iterate over scalars of a single 1D lane.
    ///
    /// # Parameters
    /// - `varying_dim`: The dimension that varies (0 for rows, 1 for columns).
    /// - `lane`: The index of the lane (row or column) to iterate over.
    ///
    /// # Returns
    /// - An iterator over references to the scalars in the specified lane.
    fn lane_iter( &self, varying_dim : usize, lane : usize ) -> impl Iterator< Item = &Self::Scalar >;

    /// Iterate over scalars of a single 1D lane with indices.
    ///
    /// # Parameters
    /// - `varying_dim`: The dimension that varies (0 for rows, 1 for columns).
    /// - `lane`: The index of the lane (row or column) to iterate over.
    ///
    /// # Returns
    /// - An iterator over tuples of indices and references to the scalars in the specified lane.
    fn lane_indexed_iter( &self, varying_dim : usize, lane : usize ) -> impl Iterator< Item = ( < Self as Indexable >::Index, &Self::Scalar ) >;

    /// Iterate over all scalars. Order of iteration is not specified.
    ///
    /// # Returns
    /// - An iterator over references to all scalars in the matrix.
    fn iter_unstable( &self ) -> impl Iterator< Item = &Self::Scalar >;

    /// Iterate over all scalars with indices. Order of iteration is not specified.
    ///
    /// # Returns
    /// - An iterator over tuples of indices and references to all scalars in the matrix.
    fn iter_indexed_unstable( &self ) -> impl Iterator< Item = ( < Self as Indexable >::Index, &Self::Scalar ) >;

    /// Iterate over all scalars in least significant dimension order.
    ///
    /// # Returns
    /// - An iterator over references to all scalars in the matrix.
    fn iter_lsfirst( &self ) -> impl Iterator< Item = &Self::Scalar >;

    /// Iterate over all scalars with indices in least significant dimension order.
    ///
    /// # Returns
    /// - An iterator over tuples of indices and references to all scalars in the matrix.
    fn iter_indexed_lsfirst( &self ) -> impl Iterator< Item = ( < Self as Indexable >::Index, &Self::Scalar ) >;

    /// Iterate over all scalars in most significant dimension order.
    ///
    /// # Returns
    /// - An iterator over references to all scalars in the matrix.
    fn iter_msfirst( &self ) -> impl Iterator< Item = &Self::Scalar >;

    /// Iterate over all scalars with indices in most significant dimension order.
    ///
    /// # Returns
    /// - An iterator over tuples of indices and references to all scalars in the matrix.
    fn iter_indexed_msfirst( &self ) -> impl Iterator< Item = ( < Self as Indexable >::Index, &Self::Scalar ) >;

  }

  /// Trait for indexing and iterating over matrix elements with mutable access.
  pub trait IndexingMut : IndexingRef
  {
    /// Iterate over scalars of a single 1D lane with mutable access.
    ///
    /// # Parameters
    /// - `varying_dim`: The dimension that varies (0 for rows, 1 for columns).
    /// - `lane`: The index of the lane (row or column) to iterate over.
    ///
    /// # Returns
    /// - An iterator over mutable references to the scalars in the specified lane.
    fn lane_iter_mut( &mut self, varying_dim : usize, lane : usize ) -> impl Iterator< Item = &mut Self::Scalar >;

    /// Iterate over scalars of a single 1D lane with indices and mutable access.
    ///
    /// # Parameters
    /// - `varying_dim`: The dimension that varies (0 for rows, 1 for columns).
    /// - `lane`: The index of the lane (row or column) to iterate over.
    ///
    /// # Returns
    /// - An iterator over tuples of indices and mutable references to the scalars in the specified lane.
    fn lane_iter_indexed_mut( &mut self, varying_dim : usize, lane : usize ) -> impl Iterator< Item = ( <Self as Indexable>::Index, &mut Self::Scalar ) >;

    /// Iterate over all scalars with mutable access. Order of iteration is not specified.
    ///
    /// # Returns
    /// - An iterator over mutable references to all scalars in the matrix.
    fn iter_unstable_mut( &mut self ) -> impl Iterator< Item = &mut Self::Scalar >;

    /// Iterate over all scalars with indices and mutable access. Order of iteration is not specified.
    ///
    /// # Returns
    /// - An iterator over tuples of indices and mutable references to all scalars in the matrix.
    fn iter_indexed_unstable_mut( &mut self ) -> impl Iterator< Item = ( <Self as Indexable>::Index, &mut Self::Scalar ) >;

    /// Iterate over all scalars in least significant dimension order with mutable access.
    ///
    /// # Returns
    /// - An iterator over mutable references to all scalars in the matrix.
    fn iter_lsfirst_mut( &mut self ) -> impl Iterator< Item = &mut Self::Scalar >;

    /// Iterate over all scalars with indices in least significant dimension order with mutable access.
    ///
    /// # Returns
    /// - An iterator over tuples of indices and mutable references to all scalars in the matrix.
    fn iter_indexed_lsfirst_mut( &mut self ) -> impl Iterator< Item = ( <Self as Indexable>::Index, &mut Self::Scalar ) >;

    /// Iterate over all scalars in most significant dimension order with mutable access.
    ///
    /// # Returns
    /// - An iterator over mutable references to all scalars in the matrix.
    fn iter_msfirst_mut( &mut self ) -> impl Iterator< Item = &mut Self::Scalar >;

    /// Iterate over all scalars with indices in most significant dimension order with mutable access.
    ///
    /// # Returns
    /// - An iterator over tuples of indices and mutable references to all scalars in the matrix.
    fn iter_indexed_msfirst_mut( &mut self ) -> impl Iterator< Item = ( <Self as Indexable>::Index, &mut Self::Scalar ) >;
  }

  /// Trait for setting values in a matrix.
  pub trait ScalarRef : Collection + Indexable
  {

    /// Get a reference to a scalar at a specified index.
    ///
    /// # Parameters
    /// - `index`: The index of the scalar to access.
    ///
    /// # Returns
    /// - A mutable reference to the scalar at the specified index.
    fn scalar_ref( &self, index : < Self as Indexable >::Index ) -> &Self::Scalar;

  }

  /// Trait for setting values in a matrix.
  pub trait ScalarMut : ScalarRef
  {
    /// Get a mutable reference to a scalar at a specified index.
    ///
    /// # Parameters
    /// - `index`: The index of the scalar to access.
    ///
    /// # Returns
    /// - A mutable reference to the scalar at the specified index.
    fn scalar_mut( &mut self, index : < Self as Indexable >::Index ) -> &mut Self::Scalar;
  }

  /// Trait for setting values in a matrix.
  pub trait ConstLayout : Indexable
  {

    /// Get offset for a specified scalar from beginning of underlying buffer.
    fn scalar_offset( index : < Self as Indexable >::Index ) -> usize;

  }

}

crate::mod_interface!
{

  exposed use
  {
    RawSlice,
    RawSliceMut,
    IndexingRef,
    IndexingMut,
    ScalarRef,
    ScalarMut,
    ConstLayout
  };

}
