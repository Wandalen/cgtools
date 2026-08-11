use ndarray::Dimension;

use crate::{IndexingRef, Mat, mat, MatEl, Indexable, Ix2, IndexingMut, RawSliceMut, ConstLayout, Collection};

impl< E, const ROWS : usize, const COLS : usize > IndexingRef
for Mat< ROWS, COLS, E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl,
{

  // Fix(TASK-014): changed `debug_assert!` to `assert!` for the row-branch and
  // column-branch lane-bound checks below so they run unconditionally instead of only in
  // debug builds.
  // Root cause: in a release build these checks were skipped, so an out-of-range `lane`
  // reached `.skip(..).step_by(..).take(..)` on the underlying slice, which never panics
  // on an out-of-range count — it silently returns a truncated or empty iterator,
  // producing wrong (or no) data instead of a loud failure.
  // Pitfall: `Iterator::skip`/`step_by`/`take` never panic on out-of-range arguments, so a
  // debug-only bound check in front of them is the only thing standing between bad input
  // and silently wrong output.
  #[ inline( always ) ]
  fn lane_iter( &self, varying_dim : usize, lane : usize )
  -> impl Iterator< Item = &Self::Scalar >
  {
    match varying_dim
    {
      0 => // Iterate over a row
      {
        let ( skip, step, take ) = if COLS == 0
        {
          // Return an empty iterator
          ( 0, 1, 0 )
        }
        else
        {
          assert!( lane < ROWS );
          ( lane, ROWS, COLS )
        };
        self
        .raw_slice()
        .iter()
        .skip( skip )
        .step_by( step )
        .take( take )
      },
      1 => // Iterate over a column
      {
        let ( skip, take ) = if ROWS == 0
        {
          // Return an empty iterator
          ( 0, 0 )
        }
        else
        {
          assert!( lane < COLS, "lane:{lane} | COLS:{COLS}" );
          ( lane * ROWS, ROWS )
        };
        self
        .raw_slice()
        .iter()
        .skip( skip )
        .step_by( 1 )
        .take( take )
      },
      _ => panic!( "Invalid dimension: {varying_dim}" ),
    }

  }

  #[ inline( always ) ]
  fn lane_indexed_iter( &self, varying_dim : usize, lane : usize ) -> impl Iterator< Item = ( <Self as Indexable>::Index, &Self::Scalar ) >
  {
    self.lane_iter( varying_dim, lane ).enumerate().map( move | ( i, value ) |
    {
      match varying_dim
      {
        0 => ( Ix2( lane, i ), value ), // Row
        1 => ( Ix2( i, lane ), value ), // Column
        _ => panic!( "Invalid dimension: {varying_dim}" ),
      }
    })
  }

  #[ inline ]
  fn iter_unstable( &self ) -> impl Iterator< Item = &Self::Scalar >
  {
    self.raw_slice().iter()
  }

  #[ inline ]
  fn iter_indexed_unstable( &self ) -> impl Iterator< Item = ( <Self as Indexable>::Index, &Self::Scalar ) >
  {
    self.iter_unstable().enumerate().map( | ( i, value ) |
    {
      let row = i % ROWS;
      let col = i / ROWS;
      ( Ix2( row, col ), value )
    })
  }

  #[ inline ]
  fn iter_lsfirst( &self ) -> impl Iterator< Item = &Self::Scalar >
  {
    ( 0..ROWS ).flat_map( move | row |
    {
      self.raw_slice()
      .iter()
      .skip( row )
      .step_by( ROWS )
      .take( COLS )
    })
  }

  #[ inline ]
  fn iter_indexed_lsfirst( &self ) -> impl Iterator< Item = ( <Self as Indexable>::Index, &Self::Scalar ) >
  {
    self.iter_lsfirst().enumerate().map( | ( i, value ) |
    {
      let row = i / COLS;
      let col = i % COLS;
      ( Ix2( row, col ), value )
    })
  }

  #[ inline ]
  fn iter_msfirst( &self ) -> impl Iterator< Item = &Self::Scalar >
  {
    self.raw_slice().iter()
  }

  #[ inline ]
  fn iter_indexed_msfirst( &self ) -> impl Iterator< Item = ( <Self as Indexable>::Index, &Self::Scalar ) >
  {
    self.iter_msfirst().enumerate().map( | ( i, value ) |
    {
      let row = i % ROWS;
      let col = i / ROWS;
      ( Ix2( row, col ), value )
    })
  }
}

impl< E, const ROWS : usize, const COLS : usize > IndexingMut
for Mat< ROWS, COLS, E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl,
{

  // Fix(TASK-014): changed `debug_assert!` to `assert!` for the row-branch and
  // column-branch lane-bound checks below so they run unconditionally instead of only in
  // debug builds.
  // Root cause: in a release build these checks were skipped, so an out-of-range `lane`
  // reached `.skip(..).step_by(..).take(..)` on the underlying slice, which never panics
  // on an out-of-range count — it silently returns a truncated or empty iterator,
  // producing wrong (or no) data instead of a loud failure.
  // Pitfall: `Iterator::skip`/`step_by`/`take` never panic on out-of-range arguments, so a
  // debug-only bound check in front of them is the only thing standing between bad input
  // and silently wrong output.
  #[ inline ]
  fn lane_iter_mut( &mut self, varying_dim : usize, lane : usize ) -> impl Iterator< Item = &mut Self::Scalar >
  {
    match varying_dim
    {
      // Iterate over a row
      0 =>
      {
        let ( skip, step, take ) = if COLS == 0
        {
          // Return an empty iterator
          ( 0, 1, 0 )
        }
        else
        {
          assert!( lane < ROWS );
          ( lane, ROWS, COLS )
        };
        self
        .raw_slice_mut()
        .iter_mut()
        .skip( skip )
        .step_by( step )
        .take( take )
      },
      // Iterate over a column
      1 =>
      {
        let ( skip, take ) = if ROWS == 0
        {
          // Return an empty iterator
          ( 0, 0 )
        }
        else
        {
          assert!( lane < COLS, "lane:{lane} | COLS:{COLS}" );
          ( lane * ROWS, ROWS )
        };
        self
        .raw_slice_mut()
        .iter_mut()
        .skip( skip )
        .step_by( 1 )
        .take( take )
      },
      _ => panic!( "Invalid dimension: {varying_dim}" ),
    }
  }

  #[ inline ]
  fn lane_iter_indexed_mut( &mut self, varying_dim : usize, lane : usize ) -> impl Iterator< Item = ( <Self as Indexable>::Index, &mut Self::Scalar ) >
  {
    self.lane_iter_mut( varying_dim, lane ).enumerate().map( move | ( i, value ) |
    {
      match varying_dim
      {
        0 => ( Ix2( lane, i ), value ), // Row
        1 => ( Ix2( i, lane ), value ), // Column
        _ => panic!( "Invalid dimension: {varying_dim}" ),
      }
    })
  }

  #[ inline ]
  fn iter_unstable_mut( &mut self ) -> impl Iterator< Item = &mut Self::Scalar >
  {
    self.raw_slice_mut().iter_mut()
  }

  #[ inline ]
  fn iter_indexed_unstable_mut( &mut self ) -> impl Iterator< Item = ( <Self as Indexable>::Index, &mut Self::Scalar ) >
  {
    self.iter_unstable_mut().enumerate().map( | ( i, value ) |
    {
      let row = i % ROWS;
      let col = i / ROWS;
      ( Ix2( row, col ), value )
    })
  }

  #[ inline ]
  fn iter_lsfirst_mut( &mut self ) -> impl Iterator< Item = &mut Self::Scalar >
  {
    let ptr = self.raw_slice_mut().as_mut_ptr();
    ( 0..ROWS ).flat_map( move | row |
    {
      ( 0..COLS).map( move | col |
      {
        // SAFETY: ptr is ROWS * COLS in length, and col * ROWS + row will always be less than COLS * ROWS,
        #[ allow( unsafe_code ) ]
        unsafe { &mut *ptr.add( col * ROWS + row ) }
      })
    })
  }

  #[ inline ]
  fn iter_indexed_lsfirst_mut( &mut self ) -> impl Iterator< Item = ( <Self as Indexable>::Index, &mut Self::Scalar ) >
  {
    self.iter_lsfirst_mut().enumerate().map( | ( i, value ) |
    {
      let row = i / COLS;
      let col = i % COLS;
      ( Ix2( row, col ), value )
    })
  }

  #[ inline ]
  fn iter_msfirst_mut( &mut self ) -> impl Iterator< Item = &mut Self::Scalar >
  {
    self.raw_slice_mut().iter_mut()
  }

  #[ inline ]
  fn iter_indexed_msfirst_mut( &mut self ) -> impl Iterator< Item = ( < Self as Indexable >::Index, &mut Self::Scalar ) >
  {
    self.iter_msfirst_mut().enumerate().map( | ( i, value ) | 
    {
      let row = i % ROWS;
      let col = i / ROWS;
      ( Ix2( row, col ), value )
    })
  }
}

impl< E, const ROWS : usize, const COLS : usize > ConstLayout
for Mat< ROWS, COLS, E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl,
{
  #[ inline( always ) ]
  fn scalar_offset( index : <Self as Indexable>::Index ) -> usize
  {
    use mdmath_core::plain::DimOffset;
    let ( row, col ) = index.into_pattern();
    [ COLS, ROWS ].offset( &Ix2( col, row ) )
  }
}

impl< E, const ROWS : usize, const COLS : usize,  > RawSliceMut
for Mat< ROWS, COLS, E, mat::DescriptorOrderColumnMajor >
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
    unsafe { std::slice::from_raw_parts_mut( self.as_mut_ptr(), ROWS * COLS ) }
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
  // Fix(TASK-014): changed `debug_assert_eq!` to `assert_eq!` so this size check runs
  // unconditionally instead of only in debug builds.
  // Root cause: the `unsafe` block below reads `ROWS*COLS` elements out of `scalars` via
  // raw pointer arithmetic (`ptr.add(row*COLS+col)`), relying on `scalars.len() ==
  // ROWS*COLS` as its safety invariant. In a release build the check was skipped, so a
  // shorter `scalars` slice caused an out-of-bounds read through the raw pointer —
  // undefined behavior, not just wrong data.
  // Pitfall: `debug_assert!` must never be the sole guard of an `unsafe` block's safety
  // invariant — once `debug_assertions` is off, the invariant goes unchecked and the
  // `unsafe` code's soundness proof no longer holds.
  fn with_row_major( mut self, scalars : &[ Self::Scalar ] ) -> Self {
    assert_eq!( scalars.len(), ROWS*COLS, "Size should be equal" );

    let ptr = scalars.as_ptr();
    let scalars : Vec< Self::Scalar > = 
    ( 0..COLS ).flat_map( move | col |
    {
      ( 0..ROWS ).map( move | row |
      {
        // SAFETY: Thanks to the check above, ptr is ROWS * COLS in length, 
        // so col * ROWS + row will always be less than ROWS * COLS,
        #[ allow( unsafe_code ) ]
        unsafe { *ptr.add( row * COLS + col ) }
      })
    })
    .collect();
    
    self.raw_set_slice( scalars.as_ref() );
    self
  }

  #[ inline ]
  fn with_column_major( mut self, scalars : &[ Self::Scalar ] ) -> Self {
    self.raw_set_slice( scalars );
    self
  }
}
