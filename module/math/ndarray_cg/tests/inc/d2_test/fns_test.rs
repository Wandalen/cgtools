use ndarray_cg::IndexingRef;

use super::*;

fn test_debug_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 0, 0, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 1, 1, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 2, 2, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 3, 3, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 2, 3, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
{
  use the_module::
  {
    Mat,
    RawSliceMut,
  };
  use std::fmt;

  // 0x0 Matrix
  let mat_0x0 = Mat::< 0, 0, f32, D >::default();
  let expected_debug_output_0x0 = format!( "Mat {{ order : {} | Coordinate : {} }}\n",
    if < D as the_module::mat::Descriptor >::IS_ROW_MAJOR { "row-major" } else { "column-major" },
    if < D as the_module::mat::Descriptor >::IS_ORDINARY { "ordinary" } else { "homogenous" }
  );
  let debug_output_0x0 = format!( "{:?}", mat_0x0 );
  assert_eq!( debug_output_0x0, expected_debug_output_0x0, "Debug output mismatch for 0x0 matrix" );

  // 1x1 Matrix
  let mat_1x1 = Mat::< 1, 1, f32, D >::from_row_major( [ 1.0 ] );
  let expected_debug_output_1x1 = format!( "Mat {{ order : {} | Coordinate : {} }}\n  [ 1.0 ],\n",
    if < D as the_module::mat::Descriptor >::IS_ROW_MAJOR { "row-major" } else { "column-major" },
    if < D as the_module::mat::Descriptor >::IS_ORDINARY { "ordinary" } else { "homogenous" }
  );
  let debug_output_1x1 = format!( "{:?}", mat_1x1 );
  assert_eq!( debug_output_1x1, expected_debug_output_1x1, "Debug output mismatch for 1x1 matrix" );

  // 2x2 Matrix
  let mat_2x2 = Mat::< 2, 2, f32, D >::from_row_major( [ 1.0, 2.0, 3.0, 4.0 ] );
  let expected_debug_output_2x2 = format!( "Mat {{ order : {} | Coordinate : {} }}\n  [ 1.0, 2.0 ],\n  [ 3.0, 4.0 ],\n",
    if < D as the_module::mat::Descriptor >::IS_ROW_MAJOR { "row-major" } else { "column-major" },
    if < D as the_module::mat::Descriptor >::IS_ORDINARY { "ordinary" } else { "homogenous" }
  );
  let debug_output_2x2 = format!( "{:?}", mat_2x2 );
  assert_eq!( debug_output_2x2, expected_debug_output_2x2, "Debug output mismatch for 2x2 matrix" );

  // 3x3 Matrix
  let mat_3x3 = Mat::< 3, 3, f32, D >::from_row_major( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ] );
  let expected_debug_output_3x3 = format!( "Mat {{ order : {} | Coordinate : {} }}\n  [ 1.0, 2.0, 3.0 ],\n  [ 4.0, 5.0, 6.0 ],\n  [ 7.0, 8.0, 9.0 ],\n",
    if < D as the_module::mat::Descriptor >::IS_ROW_MAJOR { "row-major" } else { "column-major" },
    if < D as the_module::mat::Descriptor >::IS_ORDINARY { "ordinary" } else { "homogenous" }
  );
  let debug_output_3x3 = format!( "{:?}", mat_3x3 );
  assert_eq!( debug_output_3x3, expected_debug_output_3x3, "Debug output mismatch for 3x3 matrix" );

  // 2x3 Matrix
  let mat_2x3 = Mat::< 2, 3, f32, D >::from_row_major( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ] );
  let expected_debug_output_2x3 = format!( "Mat {{ order : {} | Coordinate : {} }}\n  [ 1.0, 2.0, 3.0 ],\n  [ 4.0, 5.0, 6.0 ],\n",
    if < D as the_module::mat::Descriptor >::IS_ROW_MAJOR { "row-major" } else { "column-major" },
    if < D as the_module::mat::Descriptor >::IS_ORDINARY { "ordinary" } else { "homogenous" }
  );
  let debug_output_2x3 = format!( "{:?}", mat_2x3 );
  assert_eq!( debug_output_2x3, expected_debug_output_2x3, "Debug output mismatch for 2x3 matrix" );
}

#[ test ]
fn test_debug_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_debug_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_debug_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_debug_generic::< DescriptorOrderColumnMajor >();
}

fn test_transpose_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 3, f32, D > : Default + std::cmp::PartialEq,
  the_module::Mat< 3, 2, f32, D > : Default + std::cmp::PartialEq,

  the_module::Mat< 2, 3, f32, D > : the_module::IndexingMut< Scalar = f32 >,
  the_module::Mat< 3, 2, f32, D > : the_module::IndexingMut< Scalar = f32 >,

  the_module::Mat< 2, 3, f32, D > : the_module::RawSliceMut< Scalar = f32 >,
  the_module::Mat< 3, 2, f32, D > : the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::
  {
    Mat,
    RawSliceMut,
    IndexingRef,
  };

  let mat = Mat::< 2, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ] );
  let expected_transpose = Mat::< 3, 2, f32, D >::default().set( [ 1.0, 4.0, 2.0, 5.0, 3.0, 6.0 ] );
  let transposed_mat = mat.transpose();
  assert_eq!( transposed_mat, expected_transpose, "Transpose result mismatch" );
}

#[ test ]
fn test_transpose_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_transpose_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_transpose_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_transpose_generic::< DescriptorOrderColumnMajor >();
}

/// ## Root Cause
/// `Mat::from_row_major`'s size check (`N == ROWS*COLS`) lived in `debug_assert_eq!`, so
/// a release build skipped it and passed the mis-sized array straight into
/// `with_row_major`/`with_column_major` (whichever the matrix's descriptor selects) —
/// one of which performs unchecked raw pointer arithmetic downstream (see
/// `raw_slice_test.rs`'s `with_column_major`/`with_row_major` tests).
///
/// ## Why Not Caught
/// Every existing `from_row_major` call in this suite (e.g. `test_debug_generic` above)
/// passes a correctly-sized array; none exercised a size mismatch.
///
/// ## Fix Applied
/// TASK-014 changed `from_row_major`'s `debug_assert_eq!` to `assert_eq!` in
/// `access_common.rs` so the check runs, and fails loudly with a clear message, in every
/// build profile — before the caller-supplied data can reach either downstream
/// `with_row_major`/`with_column_major` implementation.
///
/// ## Prevention
/// Running this test under a release profile would have failed before the fix (no panic
/// at this boundary) and passes after, for both descriptors.
///
/// ## Pitfall
/// A debug-only size check at a public constructor can be the only thing standing between
/// caller input and an `unsafe` block several calls downstream.
fn test_from_row_major_size_mismatch_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 2, f32, D > : the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::Mat;

  // 2x2 matrix needs 4 scalars; only 3 are supplied.
  let _mat = Mat::< 2, 2, f32, D >::from_row_major( [ 1.0, 2.0, 3.0 ] );
}

#[ test ]
#[ should_panic ]
fn test_from_row_major_size_mismatch_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_from_row_major_size_mismatch_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
#[ should_panic ]
fn test_from_row_major_size_mismatch_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_from_row_major_size_mismatch_generic::< DescriptorOrderColumnMajor >();
}

/// ## Root Cause
/// `Mat::from_column_major`'s size check (`N == ROWS*COLS`) lived in `debug_assert_eq!`,
/// mirroring `from_row_major` above with the same release-mode gap.
///
/// ## Why Not Caught
/// No existing test exercised `from_column_major` with a mis-sized array.
///
/// ## Fix Applied
/// TASK-014 changed `from_column_major`'s `debug_assert_eq!` to `assert_eq!` in
/// `access_common.rs`.
///
/// ## Prevention
/// Running this test under a release profile would have failed before the fix and passes
/// after, for both descriptors.
///
/// ## Pitfall
/// A debug-only size check at a public constructor can be the only thing standing between
/// caller input and an `unsafe` block several calls downstream.
fn test_from_column_major_size_mismatch_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 2, f32, D > : the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::Mat;

  // 2x2 matrix needs 4 scalars; only 3 are supplied.
  let _mat = Mat::< 2, 2, f32, D >::from_column_major( [ 1.0, 2.0, 3.0 ] );
}

#[ test ]
#[ should_panic ]
fn test_from_column_major_size_mismatch_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_from_column_major_size_mismatch_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
#[ should_panic ]
fn test_from_column_major_size_mismatch_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_from_column_major_size_mismatch_generic::< DescriptorOrderColumnMajor >();
}
