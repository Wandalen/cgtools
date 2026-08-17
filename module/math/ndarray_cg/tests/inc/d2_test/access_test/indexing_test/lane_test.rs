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
  // `data[ [ 1, 2 ] ]` only retrieves an element stored verbatim from the `array!` literal —
  // no arithmetic occurs, so the result is bit-identical to the literal `6.`.
  #[ expect( clippy::float_cmp, reason = "assertions check exact expected values; no arithmetic drift is possible and epsilon comparison would weaken them" ) ]
  { assert_eq!( 6., data[ [ 1, 2 ] ] ); }
}

fn test_valid_row_iteration_1x2_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 1, 2, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
{
  use the_module::{ Mat, RawSliceMut };
  // 1x2 matrix
  let mat = Mat::< 1, 2, f32, D >::default().set( [ 1.0, 2.0 ] );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 0 ).copied().collect(); // Convert references to values
  let exp = vec![ 1.0, 2.0 ];
  assert_eq!( row_iter, exp, "Expected {exp:?}, got {row_iter:?}" );
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
  use the_module::{ Mat, RawSliceMut };
  // 1x2 matrix
  let mat = Mat::< 1, 2, f32, D >::default().set( [ 1.0, 2.0 ] );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 0 ).copied().collect();
  let exp = vec![ 1.0 ];
  assert_eq!( col_iter, exp, "Expected {exp:?}, got {col_iter:?}" );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 1 ).copied().collect();
  let exp = vec![ 2.0 ];
  assert_eq!( col_iter, exp, "Expected {exp:?}, got {col_iter:?}" );
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
  use the_module::{ Mat, RawSliceMut };
  // 2x1 matrix
  let mat = Mat::< 2, 1, f32, D >::default().set( [ 1.0, 2.0 ] );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 0 ).copied().collect();
  let exp = vec![ 1.0 ];
  assert_eq!( row_iter, exp, "Expected {exp:?}, got {row_iter:?}" );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 1 ).copied().collect();
  let exp = vec![ 2.0 ];
  assert_eq!( row_iter, exp, "Expected {exp:?}, got {row_iter:?}" );
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
  use the_module::{ Mat, RawSliceMut };
  // 2x1 matrix
  let mat = Mat::< 2, 1, f32, D >::default().set( [ 1.0, 2.0 ] );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 0 ).copied().collect();
  let exp = vec![ 1.0, 2.0 ];
  assert_eq!( col_iter, exp, "Expected {exp:?}, got {col_iter:?}" );
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
  use the_module::{ Mat, RawSliceMut };
  // 0x0 matrix
  let mat = Mat::< 0, 0, f32, D >::default();
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 0 ).copied().collect();
  let exp : Vec< f32 > = vec![];
  assert_eq!( row_iter, exp, "Expected {exp:?}, got {row_iter:?}" );
  // 1x1 matrix
  let mat = Mat::< 1, 1, f32, D >::default().set( [ 1.0 ] );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 0 ).copied().collect();
  let exp = vec![ 1.0 ];
  assert_eq!( row_iter, exp, "Expected {exp:?}, got {row_iter:?}" );
  // 2x2 matrix
  let mat = Mat::< 2, 2, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0 ] );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 0 ).copied().collect();
  let exp = vec![ 1.0, 2.0 ];
  assert_eq!( row_iter, exp, "Expected {exp:?}, got {row_iter:?}" );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 1 ).copied().collect();
  let exp = vec![ 3.0, 4.0 ];
  assert_eq!( row_iter, exp, "Expected {exp:?}, got {row_iter:?}" );
  // 3x3 matrix
  let mat = Mat::< 3, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ] );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 0 ).copied().collect();
  let exp = vec![ 1.0, 2.0, 3.0 ];
  assert_eq!( row_iter, exp, "Expected {exp:?}, got {row_iter:?}" );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 1 ).copied().collect();
  let exp = vec![ 4.0, 5.0, 6.0 ];
  assert_eq!( row_iter, exp, "Expected {exp:?}, got {row_iter:?}" );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 2 ).copied().collect();
  let exp = vec![ 7.0, 8.0, 9.0 ];
  assert_eq!( row_iter, exp, "Expected {exp:?}, got {row_iter:?}" );
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
  use the_module::{ Mat, RawSliceMut };
  // 0x0 matrix
  let mat = Mat::< 0, 0, f32, D >::default();
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 0 ).copied().collect();
  let exp : Vec< f32 > = vec![];
  assert_eq!( col_iter, exp, "Expected {exp:?}, got {col_iter:?}" );
  // 1x1 matrix
  let mat = Mat::< 1, 1, f32, D >::default().set( [ 1.0 ] );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 0 ).copied().collect();
  let exp = vec![ 1.0 ];
  assert_eq!( col_iter, exp, "Expected {exp:?}, got {col_iter:?}" );
  // 2x2 matrix
  let mat = Mat::< 2, 2, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0 ] );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 0 ).copied().collect();
  let exp = vec![ 1.0, 3.0 ];
  assert_eq!( col_iter, exp, "Expected {exp:?}, got {col_iter:?}" );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 1 ).copied().collect();
  let exp = vec![ 2.0, 4.0 ];
  assert_eq!( col_iter, exp, "Expected {exp:?}, got {col_iter:?}" );
  // 3x3 matrix
  let mat = Mat::< 3, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ] );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 0 ).copied().collect();
  let exp = vec![ 1.0, 4.0, 7.0 ];
  assert_eq!( col_iter, exp, "Expected {exp:?}, got {col_iter:?}" );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 1 ).copied().collect();
  let exp = vec![ 2.0, 5.0, 8.0 ];
  assert_eq!( col_iter, exp, "Expected {exp:?}, got {col_iter:?}" );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 2 ).copied().collect();
  let exp = vec![ 3.0, 6.0, 9.0 ];
  assert_eq!( col_iter, exp, "Expected {exp:?}, got {col_iter:?}" );
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

/// ## Root Cause
/// `lane_iter`'s degenerate/empty-iterator guard tested the wrong dimension per branch in
/// `access_column_major.rs`: the row branch (`varying_dim == 0`, indexed by `lane < ROWS`)
/// checked `COLS == 0` instead of `ROWS == 0`, and the column branch (`varying_dim == 1`,
/// indexed by `lane < COLS`) checked `ROWS == 0` instead of `COLS == 0`. For an asymmetric
/// zero-size matrix (`ROWS != COLS`, exactly one of them `0`), the guard took the `else`
/// branch and ran `assert!( lane < ROWS )` / `assert!( lane < COLS, .. )` against a zero
/// bound with `lane == 0`, panicking instead of returning an empty iterator.
///
/// ## Why Not Caught
/// `test_valid_row_iteration_generic`/`test_valid_column_iteration_generic` above only cover
/// the *symmetric* `0x0` degenerate case, where `ROWS == COLS == 0` makes the buggy and the
/// correct guard condition equivalent (`COLS == 0` and `ROWS == 0` agree when both are `0`),
/// masking the swap entirely. No existing test used an asymmetric zero-size matrix (`0xN` or
/// `Nx0` with `N > 0`).
///
/// ## Fix Applied
/// BUG-271 swapped the guard conditions in both `lane_iter` and `lane_iter_mut` in
/// `access_column_major.rs`: the row branch now checks `ROWS == 0` and the column branch now
/// checks `COLS == 0`, matching `access_row_major.rs`'s already-correct per-branch guards.
///
/// ## Prevention
/// This test exercises `Mat<0,3,..>`/`Mat<3,0,..>` -- asymmetric zero-size matrices -- for
/// both descriptors, so a future guard-dimension swap on either branch fails loudly instead
/// of hiding behind the symmetric `0x0` case.
///
/// ## Pitfall
/// A degenerate-size guard pair (`if DIM_A == 0 {..} else { assert!( lane < DIM_A ) }`) must
/// be checked against the *specific* dimension the `else` branch's assertion actually bounds
/// -- testing only the symmetric `ROWS == COLS == 0` case can never distinguish a correct
/// guard from one where both branches' conditions were swapped with each other.
// test_kind: bug_reproducer(BUG-271)
fn test_lane_iter_asymmetric_zero_size_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 0, 3, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 3, 0, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
{
  use the_module::Mat;

  // 0 rows, 3 columns: iterating "row 0" (varying_dim == 0) must be empty, not panic --
  // there are no rows to index.
  let mat_zero_rows = Mat::< 0, 3, f32, D >::default();
  let row_iter : Vec< f32 > = mat_zero_rows.lane_iter( 0, 0 ).copied().collect();
  let exp : Vec< f32 > = vec![];
  assert_eq!( row_iter, exp, "Expected {exp:?}, got {row_iter:?}" );

  // 3 rows, 0 columns: iterating "column 0" (varying_dim == 1) must be empty, not panic --
  // there are no columns to index.
  let mat_zero_cols = Mat::< 3, 0, f32, D >::default();
  let col_iter : Vec< f32 > = mat_zero_cols.lane_iter( 1, 0 ).copied().collect();
  let exp : Vec< f32 > = vec![];
  assert_eq!( col_iter, exp, "Expected {exp:?}, got {col_iter:?}" );
}

#[ test ]
fn test_lane_iter_asymmetric_zero_size_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_lane_iter_asymmetric_zero_size_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_lane_iter_asymmetric_zero_size_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_lane_iter_asymmetric_zero_size_generic::< DescriptorOrderColumnMajor >();
}

/// ## Root Cause
/// `lane_iter_mut` (`IndexingMut`) duplicates `lane_iter`'s (`IndexingRef`) degenerate-size
/// branching logic in `access_column_major.rs` and carried the identical guard-dimension
/// swap -- see `test_lane_iter_asymmetric_zero_size_generic` above for the full root cause.
///
/// ## Why Not Caught
/// No existing `lane_iter_mut` test used an asymmetric zero-size matrix (`0xN`/`Nx0` with
/// `N > 0`); `test_lane_iter_mut_generic` only covers a `3x3` matrix.
///
/// ## Fix Applied
/// BUG-271 swapped the guard conditions in `lane_iter_mut` the same way as `lane_iter`: the
/// row branch now checks `ROWS == 0` and the column branch now checks `COLS == 0`.
///
/// ## Prevention
/// This test exercises `Mat<0,3,..>`/`Mat<3,0,..>` through `lane_iter_mut` for both
/// descriptors, so a future guard-dimension swap on either branch fails loudly.
///
/// ## Pitfall
/// A `_mut` sibling that duplicates a checked accessor's branching logic (rather than
/// delegating to it) duplicates its defects too -- fixing and testing the immutable accessor
/// alone leaves the mutable sibling's own copy of the same bug completely uncovered.
// test_kind: bug_reproducer(BUG-271)
fn test_lane_iter_mut_asymmetric_zero_size_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 0, 3, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingMut< Scalar = f32 >,
  the_module::Mat< 3, 0, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingMut< Scalar = f32 >,
{
  use the_module::Mat;

  // 0 rows, 3 columns: mutably iterating "row 0" (varying_dim == 0) must be empty, not panic.
  let mut mat_zero_rows = Mat::< 0, 3, f32, D >::default();
  let row_count = mat_zero_rows.lane_iter_mut( 0, 0 ).count();
  assert_eq!( row_count, 0, "Expected an empty mutable row iterator, got {row_count} elements" );

  // 3 rows, 0 columns: mutably iterating "column 0" (varying_dim == 1) must be empty, not panic.
  let mut mat_zero_cols = Mat::< 3, 0, f32, D >::default();
  let col_count = mat_zero_cols.lane_iter_mut( 1, 0 ).count();
  assert_eq!( col_count, 0, "Expected an empty mutable column iterator, got {col_count} elements" );
}

#[ test ]
fn test_lane_iter_mut_asymmetric_zero_size_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_lane_iter_mut_asymmetric_zero_size_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_lane_iter_mut_asymmetric_zero_size_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_lane_iter_mut_asymmetric_zero_size_generic::< DescriptorOrderColumnMajor >();
}

fn test_invalid_dimension_generic<D: the_module::mat::Descriptor + std::panic::RefUnwindSafe>()
where
  the_module::Mat<2, 2, f32, D>: Default + the_module::RawSliceMut<Scalar = f32> + the_module::IndexingRef<Scalar = f32>,
{
  use std::panic;
  use the_module::{ Mat, RawSliceMut };

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
  use the_module::{ Mat, RawSliceMut };

  let mat = Mat::<2, 2, f32, D>::default().set([ 1.0, 2.0, 3.0, 4.0 ]);
  let _collected: Vec<_> = mat.lane_iter( 0, usize::MAX ).collect();
}

#[test]
#[should_panic( expected = "lane:" )]
fn test_negative_lane_index_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_negative_lane_index_generic::<DescriptorOrderRowMajor>();
}

#[test]
#[should_panic( expected = "assertion failed: lane < ROWS" )]
fn test_negative_lane_index_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_negative_lane_index_generic::<DescriptorOrderColumnMajor>();
}

fn test_out_of_bounds_lane_index_generic<D: the_module::mat::Descriptor>()
where
  the_module::Mat<2, 2, f32, D>: Default + the_module::RawSliceMut<Scalar = f32> + the_module::IndexingRef<Scalar = f32>,
{
  use the_module::{ Mat, RawSliceMut };

  let mat = Mat::<2, 2, f32, D>::default().set([ 1.0, 2.0, 3.0, 4.0 ]);
  let collected: Vec<_> = mat.lane_iter( 0, 2 ).collect();
  println!( "{collected:?}" );
}

#[test]
#[should_panic( expected = "lane:" )]
fn test_out_of_bounds_lane_index_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_out_of_bounds_lane_index_generic::<DescriptorOrderRowMajor>();
}

#[test]
#[should_panic( expected = "assertion failed: lane < ROWS" )]
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
  use the_module::{ Mat, RawSliceMut };

  let mat = Mat::<2, 2, f32, D>::default().set([ 1.0, 2.0, 3.0, 4.0 ]);
  let collected: Vec<_> = mat.lane_iter( 1, 2 ).collect();
  println!( "{collected:?}" );
}

#[test]
#[should_panic( expected = "assertion failed: lane < COLS" )]
fn test_out_of_bounds_column_lane_index_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_out_of_bounds_column_lane_index_generic::<DescriptorOrderRowMajor>();
}

#[test]
#[should_panic( expected = "lane:" )]
fn test_out_of_bounds_column_lane_index_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_out_of_bounds_column_lane_index_generic::<DescriptorOrderColumnMajor>();
}

fn test_lane_iter_mut_generic<D: the_module::mat::Descriptor>()
where
  the_module::Mat<3, 3, f32, D>: Default + the_module::RawSliceMut<Scalar = f32> + the_module::IndexingMut<Scalar = f32>,
{
  use the_module::{ Mat, RawSliceMut };

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
  use the_module::{ Mat, RawSliceMut };

  let mut mat = Mat::<2, 2, f32, D>::default().set([ 1.0, 2.0, 3.0, 4.0 ]);
  let collected: Vec<_> = mat.lane_iter_mut( 0, 2 ).collect();
  println!( "{collected:?}" );
}

#[test]
#[should_panic( expected = "lane:" )]
fn test_lane_iter_mut_out_of_bounds_row_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_lane_iter_mut_out_of_bounds_row_generic::<DescriptorOrderRowMajor>();
}

#[test]
#[should_panic( expected = "assertion failed: lane < ROWS" )]
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
  use the_module::{ Mat, RawSliceMut };

  let mut mat = Mat::<2, 2, f32, D>::default().set([ 1.0, 2.0, 3.0, 4.0 ]);
  let collected: Vec<_> = mat.lane_iter_mut( 1, 2 ).collect();
  println!( "{collected:?}" );
}

#[test]
#[should_panic( expected = "assertion failed: lane < COLS" )]
fn test_lane_iter_mut_out_of_bounds_column_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_lane_iter_mut_out_of_bounds_column_generic::<DescriptorOrderRowMajor>();
}

#[test]
#[should_panic( expected = "lane:" )]
fn test_lane_iter_mut_out_of_bounds_column_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_lane_iter_mut_out_of_bounds_column_generic::<DescriptorOrderColumnMajor>();
}
