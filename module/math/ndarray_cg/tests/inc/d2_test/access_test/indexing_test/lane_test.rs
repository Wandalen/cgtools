use super::*;

#[ test ]
fn assumptions()
{
  use the_module::nd::array;

  let data = array!
  [
    [ 1., 2., 3. ],
    [ 3., 5., 6. ],
  ];

  println!( "shape : {:?}", data.shape() );
  println!( "strides : {:?}", data.strides() );
  println!( "row : {:?}", data.row( 0 ) );
  println!( "[ 1 ][ 2 ] : {:?}", data[ [ 1, 2 ] ] ); // 6

  assert_eq!( [ 2, 3 ], data.shape() );
  assert_eq!( [ 3, 1 ], data.strides() );
  assert_eq!( 6., data[ [ 1, 2 ] ] );
}

fn test_valid_row_iteration_1x2_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 1, 2, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
{
  use the_module::{ Mat, IndexingRef, RawSliceMut };
  // 1x2 matrix
  let mat = Mat::< 1, 2, f32, D >::default().set( [ 1.0, 2.0 ] );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 0 ).copied().collect(); // Convert references to values
  let exp = vec![ 1.0, 2.0 ];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
}

#[ test ]
fn test_valid_row_iteration_1x2_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_valid_row_iteration_1x2_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_valid_row_iteration_1x2_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_valid_row_iteration_1x2_generic::< DescriptorOrderColumnMajor >();
}

fn test_valid_column_iteration_1x2_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 1, 2, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
{
  use the_module::{ Mat, IndexingRef, RawSliceMut };
  // 1x2 matrix
  let mat = Mat::< 1, 2, f32, D >::default().set( [ 1.0, 2.0 ] );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 0 ).copied().collect();
  let exp = vec![ 1.0 ];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 1 ).copied().collect();
  let exp = vec![ 2.0 ];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
}

#[ test ]
fn test_valid_column_iteration_1x2_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_valid_column_iteration_1x2_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_valid_column_iteration_1x2_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_valid_column_iteration_1x2_generic::< DescriptorOrderColumnMajor >();
}

fn test_valid_row_iteration_2x1_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 1, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
{
  use the_module::{ Mat, IndexingRef, RawSliceMut };
  // 2x1 matrix
  let mat = Mat::< 2, 1, f32, D >::default().set( [ 1.0, 2.0 ] );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 0 ).copied().collect();
  let exp = vec![ 1.0 ];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 1 ).copied().collect();
  let exp = vec![ 2.0 ];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
}

#[ test ]
fn test_valid_row_iteration_2x1_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_valid_row_iteration_2x1_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_valid_row_iteration_2x1_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_valid_row_iteration_2x1_generic::< DescriptorOrderColumnMajor >();
}

fn test_valid_column_iteration_2x1_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 1, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
{
  use the_module::{ Mat, IndexingRef, RawSliceMut };
  // 2x1 matrix
  let mat = Mat::< 2, 1, f32, D >::default().set( [ 1.0, 2.0 ] );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 0 ).copied().collect();
  let exp = vec![ 1.0, 2.0 ];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
}

#[ test ]
fn test_valid_column_iteration_2x1_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_valid_column_iteration_2x1_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_valid_column_iteration_2x1_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_valid_column_iteration_2x1_generic::< DescriptorOrderColumnMajor >();
}

fn test_valid_row_iteration_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 0, 0, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 1, 1, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 2, 2, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 3, 3, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
{
  use the_module::{ Mat, IndexingRef, RawSliceMut };
  // 0x0 matrix
  let mat = Mat::< 0, 0, f32, D >::default();
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 0 ).copied().collect();
  let exp : Vec< f32 > = vec![];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
  // 1x1 matrix
  let mat = Mat::< 1, 1, f32, D >::default().set( [ 1.0 ] );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 0 ).copied().collect();
  let exp = vec![ 1.0 ];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
  // 2x2 matrix
  let mat = Mat::< 2, 2, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0 ] );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 0 ).copied().collect();
  let exp = vec![ 1.0, 2.0 ];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 1 ).copied().collect();
  let exp = vec![ 3.0, 4.0 ];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
  // 3x3 matrix
  let mat = Mat::< 3, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ] );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 0 ).copied().collect();
  let exp = vec![ 1.0, 2.0, 3.0 ];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 1 ).copied().collect();
  let exp = vec![ 4.0, 5.0, 6.0 ];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 2 ).copied().collect();
  let exp = vec![ 7.0, 8.0, 9.0 ];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
}

#[ test ]
fn test_valid_row_iteration_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_valid_row_iteration_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_valid_row_iteration_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_valid_row_iteration_generic::< DescriptorOrderColumnMajor >();
}

fn test_valid_column_iteration_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 0, 0, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 1, 1, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 2, 2, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 3, 3, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
{
  use the_module::{ Mat, IndexingRef, RawSliceMut };
  // 0x0 matrix
  let mat = Mat::< 0, 0, f32, D >::default();
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 0 ).copied().collect();
  let exp : Vec< f32 > = vec![];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
  // 1x1 matrix
  let mat = Mat::< 1, 1, f32, D >::default().set( [ 1.0 ] );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 0 ).copied().collect();
  let exp = vec![ 1.0 ];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
  // 2x2 matrix
  let mat = Mat::< 2, 2, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0 ] );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 0 ).copied().collect();
  let exp = vec![ 1.0, 3.0 ];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 1 ).copied().collect();
  let exp = vec![ 2.0, 4.0 ];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
  // 3x3 matrix
  let mat = Mat::< 3, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ] );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 0 ).copied().collect();
  let exp = vec![ 1.0, 4.0, 7.0 ];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 1 ).copied().collect();
  let exp = vec![ 2.0, 5.0, 8.0 ];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 2 ).copied().collect();
  let exp = vec![ 3.0, 6.0, 9.0 ];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
}

#[ test ]
fn test_valid_column_iteration_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_valid_column_iteration_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_valid_column_iteration_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_valid_column_iteration_generic::< DescriptorOrderColumnMajor >();
}

fn test_invalid_dimension_generic<D: the_module::mat::Descriptor + std::panic::RefUnwindSafe>()
where
  the_module::Mat<2, 2, f32, D>: Default + the_module::RawSliceMut<Scalar = f32> + the_module::IndexingRef<Scalar = f32>,
{
  use std::panic;
  use the_module::{ Mat, IndexingRef, RawSliceMut };

  let mat = Mat::<2, 2, f32, D>::default().set([ 1.0, 2.0, 3.0, 4.0 ]);
  let result = panic::catch_unwind( ||
  {
    let _ = mat.lane_iter( 2, 0 ).collect::<Vec<_>>();
  });

  assert!( result.is_err(), "Expected panic, but no panic occurred" );
}

#[test]
fn test_invalid_dimension_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_invalid_dimension_generic::<DescriptorOrderRowMajor>();
}

#[test]
fn test_invalid_dimension_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_invalid_dimension_generic::<DescriptorOrderColumnMajor>();
}

fn test_negative_lane_index_generic<D: the_module::mat::Descriptor>()
where
  the_module::Mat<2, 2, f32, D>: Default + the_module::RawSliceMut<Scalar = f32> + the_module::IndexingRef<Scalar = f32>,
{
  use the_module::{ Mat, IndexingRef, RawSliceMut };

  let mat = Mat::<2, 2, f32, D>::default().set([ 1.0, 2.0, 3.0, 4.0 ]);
  let _collected: Vec<_> = mat.lane_iter( 0, usize::MAX ).collect();
}

#[test]
#[should_panic]
fn test_negative_lane_index_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_negative_lane_index_generic::<DescriptorOrderRowMajor>();
}

#[test]
#[should_panic]
fn test_negative_lane_index_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_negative_lane_index_generic::<DescriptorOrderColumnMajor>();
}

fn test_out_of_bounds_lane_index_generic<D: the_module::mat::Descriptor>()
where
  the_module::Mat<2, 2, f32, D>: Default + the_module::RawSliceMut<Scalar = f32> + the_module::IndexingRef<Scalar = f32>,
{
  use the_module::{ Mat, IndexingRef, RawSliceMut };

  let mat = Mat::<2, 2, f32, D>::default().set([ 1.0, 2.0, 3.0, 4.0 ]);
  let _collected: Vec<_> = mat.lane_iter( 0, 2 ).collect();
  println!( "{_collected:?}" )
}

#[test]
#[should_panic]
fn test_out_of_bounds_lane_index_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_out_of_bounds_lane_index_generic::<DescriptorOrderRowMajor>();
}

#[test]
#[should_panic]
fn test_out_of_bounds_lane_index_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_out_of_bounds_lane_index_generic::<DescriptorOrderColumnMajor>();
}

/// ## Root Cause
/// `lane_iter`'s column-lane bound (`varying_dim == 1`) was checked only via
/// `debug_assert!( lane < COLS, .. )`. `test_out_of_bounds_lane_index_generic` above only
/// ever exercises the row-lane bound (`varying_dim == 0`), so the column-lane bound had no
/// out-of-bounds regression coverage at all.
///
/// ## Why Not Caught
/// No existing test called `lane_iter( 1, out_of_range_lane )`, so the column-lane
/// `debug_assert!` was never exercised by a bounds-violating input in either build profile.
///
/// ## Fix Applied
/// TASK-014 changed the underlying `debug_assert!` to `assert!` in
/// `access_row_major.rs`/`access_column_major.rs` so the check fires in every build
/// profile, not just debug. This test pins that behavior down for the column-lane branch.
///
/// ## Prevention
/// Running this test under a release profile (`debug_assertions` off) would have failed
/// before the fix (no panic; `.skip( lane * ROWS )` either silently returns an empty
/// iterator or the wrong lane's data) and passes after.
///
/// ## Pitfall
/// A bound check guarded by `debug_assert!` needs its own dedicated out-of-bounds test per
/// branch — coverage of a sibling branch (here, the row-lane check) does not exercise it.
fn test_out_of_bounds_column_lane_index_generic<D: the_module::mat::Descriptor>()
where
  the_module::Mat<2, 2, f32, D>: Default + the_module::RawSliceMut<Scalar = f32> + the_module::IndexingRef<Scalar = f32>,
{
  use the_module::{ Mat, IndexingRef, RawSliceMut };

  let mat = Mat::<2, 2, f32, D>::default().set([ 1.0, 2.0, 3.0, 4.0 ]);
  let _collected: Vec<_> = mat.lane_iter( 1, 2 ).collect();
  println!( "{_collected:?}" )
}

#[test]
#[should_panic]
fn test_out_of_bounds_column_lane_index_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_out_of_bounds_column_lane_index_generic::<DescriptorOrderRowMajor>();
}

#[test]
#[should_panic]
fn test_out_of_bounds_column_lane_index_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_out_of_bounds_column_lane_index_generic::<DescriptorOrderColumnMajor>();
}

fn test_lane_iter_mut_generic<D: the_module::mat::Descriptor>()
where
  the_module::Mat<3, 3, f32, D>: Default + the_module::RawSliceMut<Scalar = f32> + the_module::IndexingMut<Scalar = f32>,
{
  use the_module::{ Mat, RawSliceMut, IndexingMut };

  let mut mat = Mat::<3, 3, f32, D>::default().set([ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ]);

  // Modify the first row
  for value in mat.lane_iter_mut( 0, 0 )
  {
    *value *= 2.0;
  }

  let expected = Mat::<3, 3, f32, D>::default().set([ 2.0, 4.0, 6.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ]);
  assert_eq!( mat.raw_slice(), expected.raw_slice(), "Row modification failed" );
}

#[test]
fn test_lane_iter_mut_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_lane_iter_mut_generic::<DescriptorOrderRowMajor>();
}

#[test]
fn test_lane_iter_mut_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_lane_iter_mut_generic::<DescriptorOrderColumnMajor>();
}

/// ## Root Cause
/// `lane_iter_mut`'s row-lane bound (`varying_dim == 0`) was checked only via
/// `debug_assert!( lane < ROWS, .. )`, mirroring the immutable `lane_iter`'s row-lane check
/// but with no out-of-bounds regression coverage of its own.
///
/// ## Why Not Caught
/// Existing `lane_iter_mut` tests (`test_lane_iter_mut_generic`) only exercise valid lane
/// indices; no test called `lane_iter_mut` with an out-of-range lane.
///
/// ## Fix Applied
/// TASK-014 changed the underlying `debug_assert!` to `assert!` in
/// `access_row_major.rs`/`access_column_major.rs` so the check fires in every build
/// profile, not just debug.
///
/// ## Prevention
/// Running this test under a release profile would have failed before the fix (no panic;
/// `.skip( lane * COLS )` silently returns an empty or wrong-offset mutable iterator) and
/// passes after.
///
/// ## Pitfall
/// The `_mut` sibling of a checked accessor needs its own out-of-bounds test — a passing
/// immutable-accessor test says nothing about the mutable accessor's own debug-only guard.
fn test_lane_iter_mut_out_of_bounds_row_generic<D: the_module::mat::Descriptor>()
where
  the_module::Mat<2, 2, f32, D>: Default + the_module::RawSliceMut<Scalar = f32> + the_module::IndexingMut<Scalar = f32>,
{
  use the_module::{ Mat, IndexingMut, RawSliceMut };

  let mut mat = Mat::<2, 2, f32, D>::default().set([ 1.0, 2.0, 3.0, 4.0 ]);
  let _collected: Vec<_> = mat.lane_iter_mut( 0, 2 ).collect();
  println!( "{_collected:?}" )
}

#[test]
#[should_panic]
fn test_lane_iter_mut_out_of_bounds_row_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_lane_iter_mut_out_of_bounds_row_generic::<DescriptorOrderRowMajor>();
}

#[test]
#[should_panic]
fn test_lane_iter_mut_out_of_bounds_row_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_lane_iter_mut_out_of_bounds_row_generic::<DescriptorOrderColumnMajor>();
}

/// ## Root Cause
/// `lane_iter_mut`'s column-lane bound (`varying_dim == 1`) was checked only via
/// `debug_assert!( lane < COLS, .. )`, with no out-of-bounds regression coverage.
///
/// ## Why Not Caught
/// No test called `lane_iter_mut( 1, out_of_range_lane )`.
///
/// ## Fix Applied
/// TASK-014 changed the underlying `debug_assert!` to `assert!` so the check fires in
/// every build profile, not just debug.
///
/// ## Prevention
/// Running this test under a release profile would have failed before the fix and passes
/// after.
///
/// ## Pitfall
/// Each of the four lane-bound branches (`lane_iter`/`lane_iter_mut` x row/column) is an
/// independent debug-only guard and needs its own dedicated out-of-bounds test.
fn test_lane_iter_mut_out_of_bounds_column_generic<D: the_module::mat::Descriptor>()
where
  the_module::Mat<2, 2, f32, D>: Default + the_module::RawSliceMut<Scalar = f32> + the_module::IndexingMut<Scalar = f32>,
{
  use the_module::{ Mat, IndexingMut, RawSliceMut };

  let mut mat = Mat::<2, 2, f32, D>::default().set([ 1.0, 2.0, 3.0, 4.0 ]);
  let _collected: Vec<_> = mat.lane_iter_mut( 1, 2 ).collect();
  println!( "{_collected:?}" )
}

#[test]
#[should_panic]
fn test_lane_iter_mut_out_of_bounds_column_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_lane_iter_mut_out_of_bounds_column_generic::<DescriptorOrderRowMajor>();
}

#[test]
#[should_panic]
fn test_lane_iter_mut_out_of_bounds_column_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_lane_iter_mut_out_of_bounds_column_generic::<DescriptorOrderColumnMajor>();
}
