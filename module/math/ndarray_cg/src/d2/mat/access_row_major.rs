use crate::*;

impl< E, const ROWS : usize, const COLS : usize > IndexingRef
for Mat< ROWS, COLS, E, mat::DescriptorOrderRowMajor >
where
  E : MatEl,
  E : nd::NdFloat,
{

  #[ inline( always ) ]
  fn lane_iter( &self, varying_dim : usize, lane : usize )
  -> impl Iterator< Item = &Self::Scalar >
  {
    match varying_dim
    {
      0 => // Iterate over a row
      {
        if ROWS == 0
        {
          // Return an empty iterator
          self
          .raw_slice()
          .iter()
          .skip( 0 )
          .step_by( 1 )
          .take( 0 )
        }
        else
        {
          debug_assert!( lane < ROWS, "lane:{lane} | ROWS:{ROWS}" );
          self
          .raw_slice()
          .iter()
          .skip( lane * COLS )
          .step_by( 1 )
          .take( COLS )
        }
      },
      1 => // Iterate over a column
      {
        if COLS == 0
        {
          // Return an empty iterator
          self
          .raw_slice()
          .iter()
          .skip( 0 )
          .step_by( 1 )
          .take( 0 )
        }
        else
        {
          debug_assert!( lane < COLS );
          self
          .raw_slice()
          .iter()
          .skip( lane )
          .step_by( COLS )
          .take( ROWS )
        }
      },
      _ => panic!( "Invalid dimension: {}", varying_dim ),
    }

  }

  #[ inline( always ) ]
  fn lane_indexed_iter( &self, varying_dim : usize, lane : usize )
  -> impl Iterator< Item = ( < Self as Indexable >::Index, &Self::Scalar ) >
  {
    self.lane_iter( varying_dim, lane ).enumerate().map( move | ( i, value ) |
    {
      match varying_dim
      {
        0 => ( Ix2( lane, i ), value ), // Row
        1 => ( Ix2( i, lane ), value ), // Column
        _ => panic!( "Invalid dimension: {}", varying_dim ),
      }
    })
  }

  #[ inline ]
  fn iter_unstable( &self )
  -> impl Iterator< Item = &Self::Scalar >
  {
    self.raw_slice().iter()
  }

  #[ inline ]
  fn iter_indexed_unstable( &self )
  -> impl Iterator< Item = ( < Self as Indexable >::Index, &Self::Scalar ) >
  {
    self.raw_slice().iter().enumerate().map( | ( i, value ) |
    {
      let row = i / COLS;
      let col = i % COLS;
      ( Ix2( row, col ), value )
    })
  }

  #[ inline ]
  fn iter_lsfirst( &self )
  -> impl Iterator< Item = &Self::Scalar >
  {
    self.raw_slice().iter()
  }

  #[ inline ]
  fn iter_indexed_lsfirst( &self )
  -> impl Iterator< Item = ( < Self as Indexable >::Index, &Self::Scalar ) >
  {
    self.raw_slice().iter().enumerate().map( | ( i, value ) |
    {
      let row = i / COLS;
      let col = i % COLS;
      ( Ix2( row, col ), value )
    })
  }

  #[ inline ]
  fn iter_msfirst( &self ) -> impl Iterator< Item = &Self::Scalar >
  {
    ( 0..COLS ).flat_map( move | col |
    {
      self.raw_slice()
      .iter()
      .skip( col )
      .step_by( COLS )
      .take( ROWS )
    })
  }

  #[ inline ]
  fn iter_indexed_msfirst( &self ) -> impl Iterator< Item = ( < Self as Indexable >::Index, &Self::Scalar ) >
  {
    ( 0..COLS ).flat_map( move | col |
    {
      self.raw_slice()
      .iter()
      .enumerate()
      .filter( move | ( i, _ ) | i % COLS == col )
      .map( move | ( i, value ) | ( Ix2( i / COLS, col ), value ) )
    })
  }

}

impl< E, const ROWS : usize, const COLS : usize > IndexingMut
for Mat< ROWS, COLS, E, mat::DescriptorOrderRowMajor >
where
  E : MatEl,
  E : nd::NdFloat,
{
  fn lane_iter_mut( &mut self, varying_dim : usize, lane : usize ) -> impl Iterator< Item = &mut Self::Scalar >
  {
    match varying_dim
    {
      0 =>
      {
        if ROWS == 0
        {
          self.raw_slice_mut().iter_mut().skip( 0 ).step_by( 1 ).take( 0 )
        }
        else
        {
          debug_assert!( lane < ROWS, "lane:{lane} | ROWS:{ROWS}" );
          self.raw_slice_mut().iter_mut().skip( lane * COLS ).step_by( 1 ).take( COLS )
        }
      }
      1 =>
      {
        if COLS == 0
        {
          self.raw_slice_mut().iter_mut().skip( 0 ).step_by( 1 ).take( 0 )
        }
        else
        {
          debug_assert!( lane < COLS );
          self.raw_slice_mut().iter_mut().skip( lane ).step_by( COLS ).take( ROWS )
        }
      }
      _ => panic!( "Invalid dimension: {}", varying_dim ),
    }
  }

  fn lane_iter_indexed_mut( &mut self, varying_dim : usize, lane : usize ) -> impl Iterator< Item = ( <Self as Indexable>::Index, &mut Self::Scalar ) >
  {
    self.lane_iter_mut( varying_dim, lane ).enumerate().map( move | ( i, value ) |
    {
      match varying_dim
      {
        0 => ( ndarray::Ix2( lane, i ), value ), // Row
        1 => ( ndarray::Ix2( i, lane ), value ), // Column
        _ => panic!( "Invalid dimension: {}", varying_dim ),
      }
    })
  }

  fn iter_unstable_mut( &mut self ) -> impl Iterator< Item = &mut Self::Scalar >
  {
    self.raw_slice_mut().iter_mut()
  }

  fn iter_indexed_unstable_mut( &mut self ) -> impl Iterator< Item = ( <Self as Indexable>::Index, &mut Self::Scalar ) >
  {
    self.iter_unstable_mut().enumerate().map( | ( i, value ) |
    {
      let row = i / COLS;
      let col = i % COLS;
      ( ndarray::Ix2( row, col ), value )
    })
  }

  fn iter_lsfirst_mut( &mut self ) -> impl Iterator< Item = &mut Self::Scalar >
  {
    self.raw_slice_mut().iter_mut()
  }

  fn iter_indexed_lsfirst_mut( &mut self ) -> impl Iterator< Item = ( <Self as Indexable>::Index, &mut Self::Scalar ) >
  {
    self.iter_lsfirst_mut().enumerate().map( | ( i, value ) |
    {
      let row = i / COLS;
      let col = i % COLS;
      ( ndarray::Ix2( row, col ), value )
    })
  }

  fn iter_msfirst_mut( &mut self ) -> impl Iterator< Item = &mut Self::Scalar >
  {
    let ptr = self.raw_slice_mut().as_mut_ptr();
    ( 0..COLS ).flat_map( move | col |
    {
      ( 0..ROWS ).map( move | row |
      {
        // SAFETY: ptr is ROWS * COLS in length, and row * COLS + col will always be less than COLS * ROWS,
        #[ allow( unsafe_code ) ]
        unsafe { &mut *ptr.add( row * COLS + col ) }
      })
    })
  }

  fn iter_indexed_msfirst_mut( &mut self ) -> impl Iterator< Item = ( < Self as Indexable >::Index, &mut Self::Scalar ) >
  {
    let ptr = self.raw_slice_mut().as_mut_ptr();
    ( 0..COLS ).flat_map( move | col |
    {
      ( 0..ROWS ).map( move | row |
      {
        // SAFETY: ptr is ROWS * COLS in length, and for a row major matrix, scalar_offset
        // will return an 1-d offset for a matrix [ ROWS, COLS ], which will be less than ROWS * COLS,
        #[ allow( unsafe_code ) ]
        let value = unsafe { &mut *ptr.add( Self::scalar_offset( Ix2( row, col ) ) ) };
        ( Ix2( row, col ), value )
      })
    })
  }
}

impl< E, const ROWS : usize, const COLS : usize > ConstLayout
for Mat< ROWS, COLS, E, mat::DescriptorOrderRowMajor >
where
  E : MatEl,
{
  #[ inline( always ) ]
  fn scalar_offset( index : < Self as Indexable >::Index ) -> usize
  {
    use mdmath_core::plain::DimOffset;
    [ ROWS, COLS ].offset( &index )
    // if <Descriptor as mat::Descriptor>::IS_ROW_MAJOR { [ ROWS, COLS ] } else { [ COLS, ROWS ] }.offset( &index )
  }
}

impl< E, const ROWS : usize, const COLS : usize > RawSliceMut
for Mat< ROWS, COLS, E, mat::DescriptorOrderRowMajor >
where
  E : MatEl,
  Self : Collection< Scalar = E >,
{
  #[ inline( always ) ]
  fn raw_slice_mut( &mut self ) -> &mut [ Self::Scalar ]
  {
    // SAFETY: This is safe because the memory layout of [ [ E ; COLS ] ; ROWS ]
    // is contiguous and can be reinterpreted as a flat slice of E.
    #[ allow( unsafe_code ) ]
    unsafe { std::slice::from_raw_parts_mut( self.as_mut_ptr() as *mut Self::Scalar, ROWS * COLS ) }
  }

  #[ inline( always ) ]
  fn raw_set_slice( &mut self, scalars : &[ Self::Scalar ] )
  {
    self.raw_slice_mut().copy_from_slice( scalars );
  }

  #[ inline( always ) ]
  fn raw_set< const N : usize >( mut self, scalars : [ Self::Scalar ; N ] ) -> Self
  {
    debug_assert_eq!( scalars.len(), ROWS*COLS, "Size should be equal" );
    self.raw_slice_mut().copy_from_slice( &scalars );
    self
  }

  #[ inline( always ) ]
  fn with_row_major( mut self, scalars : &[ Self::Scalar ] ) -> Self {
      self.raw_set_slice( scalars );
      self
  }

  fn with_column_major( mut self, scalars : &[ Self::Scalar ] ) -> Self {
    debug_assert_eq!( scalars.len(), ROWS*COLS, "Size should be equal" );
    
    let ptr = scalars.as_ptr();
    let scalars : Vec< Self::Scalar > = 
    ( 0..ROWS ).flat_map( move | row |
    {
      ( 0..COLS ).map( move | col |
      {
        // SAFETY: Thanks to the check above, ptr is ROWS * COLS in length, 
        // so col * ROWS + row will always be less than ROWS * COLS,
        #[ allow( unsafe_code ) ]
        unsafe { *ptr.add( col * ROWS + row ) }
      })
    })
    .collect();
    
    self.raw_set_slice( scalars.as_ref() );
    self
  }
}
