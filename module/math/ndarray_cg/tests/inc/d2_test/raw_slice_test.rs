use super::*;

fn test_raw_slice_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 2, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::
  {
    Mat,
    RawSlice,
    RawSliceMut,
  };
  let mat = Mat::< 2, 2, f32, D >::default().raw_set( [ 1.0, 2.0, 3.0, 4.0 ] );
  let slice = mat.raw_slice();
  let exp = &[ 1.0, 2.0, 3.0, 4.0 ];
  assert_eq!( slice, exp, "Raw slice mismatch. Expected {:?}, got {:?}", exp, slice );
}

#[ test ]
fn test_raw_slice_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_raw_slice_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_raw_slice_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_raw_slice_generic::< DescriptorOrderColumnMajor >();
}

fn test_raw_slice_mut_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 2, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::
  {
    Mat,
    RawSlice,
    RawSliceMut,
  };
  let mut mat = Mat::< 2, 2, f32, D >::default();
  {
    let slice_mut = mat.raw_slice_mut();
    slice_mut.copy_from_slice( &[ 5.0, 6.0, 7.0, 8.0 ] );
  }
  let exp = &[ 5.0, 6.0, 7.0, 8.0 ];
  assert_eq!( mat.raw_slice(), exp, "Raw slice mutable modification failed. Expected {:?}, got {:?}", exp, mat.raw_slice() );
}

#[ test ]
fn test_raw_slice_mut_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_raw_slice_mut_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_raw_slice_mut_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_raw_slice_mut_generic::< DescriptorOrderColumnMajor >();
}

fn test_raw_set_slice_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 2, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::
  {
    Mat,
    RawSlice,
    RawSliceMut,
  };
  let mut mat = Mat::< 2, 2, f32, D >::default();
  mat.raw_set_slice( &[ 9.0, 10.0, 11.0, 12.0 ] );
  let exp = &[ 9.0, 10.0, 11.0, 12.0 ];
  assert_eq!( mat.raw_slice(), exp, "Raw set slice failed. Expected {:?}, got {:?}", exp, mat.raw_slice() );
}

#[ test ]
fn test_raw_set_slice_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_raw_set_slice_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_raw_set_slice_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_raw_set_slice_generic::< DescriptorOrderColumnMajor >();
}

fn test_raw_set_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 2, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::
  {
    Mat,
    RawSlice,
    RawSliceMut,
  };
  let mat = Mat::< 2, 2, f32, D >::default().raw_set( [ 13.0, 14.0, 15.0, 16.0 ] );
  let exp = &[ 13.0, 14.0, 15.0, 16.0 ];
  assert_eq!( mat.raw_slice(), exp, "Raw set failed. Expected {:?}, got {:?}", exp, mat.raw_slice() );
}

#[ test ]
fn test_raw_set_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_raw_set_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_raw_set_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_raw_set_generic::< DescriptorOrderColumnMajor >();
}

/// ## Root Cause
/// `RawSliceMut::with_column_major` had two implementations. For
/// `DescriptorOrderRowMajor` (`access_row_major.rs`), it used `debug_assert_eq!` to check
/// `scalars.len() == ROWS*COLS` immediately before an `unsafe` block that reads
/// `ROWS*COLS` elements out of `scalars` via raw pointer arithmetic
/// (`ptr.add( col * ROWS + row )`) — in a release build the check was skipped, so a
/// shorter `scalars` slice caused an out-of-bounds read through the raw pointer
/// (undefined behavior, not just wrong data). For `DescriptorOrderColumnMajor`, the same
/// method instead delegates to `raw_set_slice`/`copy_from_slice`, which already panics
/// unconditionally on a length mismatch in every build profile, so that path was never
/// actually unsound.
///
/// ## Why Not Caught
/// No test called `with_column_major`/`set_column_major` with a mis-sized slice for
/// either descriptor.
///
/// ## Fix Applied
/// TASK-014 changed the `debug_assert_eq!` in `access_row_major.rs`'s
/// `with_column_major` to `assert_eq!` so the size check that guards the `unsafe` block
/// runs in every build profile. The `DescriptorOrderColumnMajor` path needed no source
/// change (already sound via `copy_from_slice`); this test pins that down too so a future
/// refactor cannot silently reintroduce a debug-only guard there.
///
/// ## Prevention
/// Running the row-major case under a release profile would have failed before the fix
/// (no panic; undefined behavior from the out-of-bounds raw pointer read) and passes
/// after. The column-major case passes in every profile, before and after.
///
/// ## Pitfall
/// `debug_assert!` must never be the sole guard of an `unsafe` block's safety invariant —
/// once `debug_assertions` is off, the invariant goes unchecked and the `unsafe` code's
/// soundness proof no longer holds.
fn test_set_column_major_size_mismatch_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 2, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::Mat;

  let mat = Mat::< 2, 2, f32, D >::default();
  // 2x2 matrix needs 4 scalars; only 3 are supplied.
  let _mat = mat.set_column_major( &[ 1.0, 2.0, 3.0 ] );
}

#[ test ]
#[ should_panic ]
fn test_set_column_major_size_mismatch_row_major()
{
  // Exercises `with_column_major` for `DescriptorOrderRowMajor`, whose implementation
  // performs raw pointer arithmetic guarded by this size check (see access_row_major.rs).
  use the_module::mat::DescriptorOrderRowMajor;
  test_set_column_major_size_mismatch_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
#[ should_panic ]
fn test_set_column_major_size_mismatch_column_major()
{
  // For `DescriptorOrderColumnMajor`, `with_column_major` delegates to `raw_set_slice`,
  // whose `copy_from_slice` call already panics unconditionally on a length mismatch —
  // this confirms that already-safe path stays safe.
  use the_module::mat::DescriptorOrderColumnMajor;
  test_set_column_major_size_mismatch_generic::< DescriptorOrderColumnMajor >();
}

/// ## Root Cause
/// `RawSliceMut::with_row_major` had two implementations, mirroring `with_column_major`
/// above but with the roles of the two descriptors swapped. For
/// `DescriptorOrderColumnMajor` (`access_column_major.rs`), it used `debug_assert_eq!`
/// immediately before an `unsafe` block that reads `ROWS*COLS` elements out of `scalars`
/// via raw pointer arithmetic — unchecked, hence unsound, in release builds. For
/// `DescriptorOrderRowMajor`, the same method delegates to `raw_set_slice`/
/// `copy_from_slice`, which already panics unconditionally in every build profile.
///
/// ## Why Not Caught
/// No test called `with_row_major`/`set_row_major` with a mis-sized slice for either
/// descriptor.
///
/// ## Fix Applied
/// TASK-014 changed the `debug_assert_eq!` in `access_column_major.rs`'s `with_row_major`
/// to `assert_eq!`. The `DescriptorOrderRowMajor` path needed no source change; this test
/// pins that down too.
///
/// ## Prevention
/// Running the column-major case under a release profile would have failed before the fix
/// (undefined behavior from the out-of-bounds raw pointer read) and passes after. The
/// row-major case passes in every profile, before and after.
///
/// ## Pitfall
/// `debug_assert!` must never be the sole guard of an `unsafe` block's safety invariant.
fn test_set_row_major_size_mismatch_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 2, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::Mat;

  let mat = Mat::< 2, 2, f32, D >::default();
  // 2x2 matrix needs 4 scalars; only 3 are supplied.
  let _mat = mat.set_row_major( &[ 1.0, 2.0, 3.0 ] );
}

#[ test ]
#[ should_panic ]
fn test_set_row_major_size_mismatch_row_major()
{
  // For `DescriptorOrderRowMajor`, `with_row_major` delegates to `raw_set_slice`, whose
  // `copy_from_slice` call already panics unconditionally on a length mismatch — this
  // confirms that already-safe path stays safe.
  use the_module::mat::DescriptorOrderRowMajor;
  test_set_row_major_size_mismatch_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
#[ should_panic ]
fn test_set_row_major_size_mismatch_column_major()
{
  // Exercises `with_row_major` for `DescriptorOrderColumnMajor`, whose implementation
  // performs raw pointer arithmetic guarded by this size check (see access_column_major.rs).
  use the_module::mat::DescriptorOrderColumnMajor;
  test_set_row_major_size_mismatch_generic::< DescriptorOrderColumnMajor >();
}
