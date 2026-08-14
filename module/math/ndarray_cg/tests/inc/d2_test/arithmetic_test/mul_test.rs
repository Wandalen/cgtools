use super::*;

fn test_multiply_matrices_generic< D : the_module::mat::Descriptor >()
where

  the_module::Mat< 1, 2, f32, D > : the_module::ScalarMut< Scalar = f32 >,
  the_module::Mat< 1, 3, f32, D > : the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 3, 2, f32, D > : the_module::IndexingRef< Scalar = f32 >,

  the_module::Mat< 1, 2, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
  the_module::Mat< 1, 3, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
  the_module::Mat< 3, 2, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,

  the_module::Mat< 1, 2, f32, D > : the_module::RawSliceMut< Scalar = f32 >,
  the_module::Mat< 1, 3, f32, D > : the_module::RawSliceMut< Scalar = f32 >,
  the_module::Mat< 3, 2, f32, D > : the_module::RawSliceMut< Scalar = f32 >,

  for< 'a > &'a the_module::Mat< 1, 3, f32, D > : core::ops::Mul< &'a the_module::Mat< 3, 2, f32, D >, Output = the_module::Mat< 1, 2, f32, D > >,
  the_module::Mat< 1, 3, f32, D > : core::ops::Mul< the_module::Mat< 3, 2, f32, D >, Output = the_module::Mat< 1, 2, f32, D > >,

{
  use the_module::
  {
    Mat,
    d2,
  };

  // Define matrices using row_major for consistent logical layout
  let mat_a = Mat::< 1, 3, f32, D >::default().row_major_set
  (&[
    1.0, 2.0, 3.0,
  ]);

  let mat_b = Mat::< 3, 2, f32, D >::default().row_major_set
  (&[
    7.0, 8.0,
    9.0, 10.0,
    11.0, 12.0,
  ]);

  let mut mat_r = Mat::< 1, 2, f32, D >::default();

  println!( "Before mul" );
  // Perform multiplication
  d2::mul( &mut mat_r, &mat_a, &mat_b );
  println!( "After mul" );

  // Expected result
  let exp = Mat::< 1, 2, f32, D >::default().row_major_set
  (&[
    58.0, 64.0,
  ]);
  assert_eq!( mat_r.raw_slice(), exp.raw_slice(), "Expected {:?}, got {:?}", exp.raw_slice(), mat_r.raw_slice() );

  let mat_r = &mat_a * &mat_b;
  assert_eq!( mat_r.raw_slice(), exp.raw_slice(), "Expected {:?}, got {:?}", exp.raw_slice(), mat_r.raw_slice() );
  let mat_r = mat_a * mat_b;
  assert_eq!( mat_r.raw_slice(), exp.raw_slice(), "Expected {:?}, got {:?}", exp.raw_slice(), mat_r.raw_slice() );

}

#[ test ]
fn test_multiply_matrices_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_multiply_matrices_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_multiply_matrices_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_multiply_matrices_generic::< DescriptorOrderColumnMajor >();
}

/// ## Root Cause
/// `d2::mul`'s dimension-compatibility check (`adim[1] != bdim[0] || ..`) lived inside
/// `#[ cfg( debug_assertions ) ]`, so a release build skipped it entirely. This test was
/// previously commented out (`qqq : implement try build test throwing error`) rather than
/// exercising the check as an always-on `should_panic` test.
///
/// ## Why Not Caught
/// The mismatched-dimension case was never compiled in, so neither profile's behavior was
/// under test.
///
/// ## Fix Applied
/// TASK-014 changed `mul()`'s `#[ cfg( debug_assertions ) ]` block to run unconditionally
/// (see `src/d2/arithmetics/mul.rs`). This test replaces the old commented-out attempt with
/// a working one, isolating the inner-dimension mismatch (`a`'s column count vs `b`'s row
/// count).
///
/// ## Prevention
/// Running this test under a release profile (`debug_assertions` off) would have failed
/// before the fix: no panic, and `a.lane_iter(..).zip(b.lane_iter(..))` silently truncates
/// to the shorter lane, producing a wrong dot product. It passes after the fix, in every
/// profile.
///
/// ## Pitfall
/// A dimension check gated to debug builds is invisible to `#[ should_panic ]` coverage
/// intent unless the test itself is written and actually compiled — a commented-out test
/// verifies nothing.
fn test_multiply_incompatible_dimensions_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 2, f32, D > : the_module::ScalarMut< Scalar = f32 >,
  the_module::Mat< 2, 3, f32, D > : the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 2, 2, f32, D > : the_module::IndexingRef< Scalar = f32 >,

  the_module::Mat< 2, 2, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
  the_module::Mat< 2, 3, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,

  the_module::Mat< 2, 2, f32, D > : the_module::RawSliceMut< Scalar = f32 >,
  the_module::Mat< 2, 3, f32, D > : the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::
  {
    Mat,
    d2,
  };

  // a is 2x3, b is 2x2: a's column count (3) does not match b's row count (2), so the
  // inner multiplication dimension is incompatible.
  let mat_a = Mat::< 2, 3, f32, D >::default().row_major_set( &[ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ] );
  let mat_b = Mat::< 2, 2, f32, D >::default().row_major_set( &[ 1.0, 2.0, 3.0, 4.0 ] );
  let mut mat_r = Mat::< 2, 2, f32, D >::default();

  // Attempt multiplication with incompatible inner dimensions, should panic
  d2::mul( &mut mat_r, &mat_a, &mat_b );
}

#[ test ]
#[ should_panic( expected = "Incompatible dimensions for matrix multiplication" ) ]
fn test_multiply_incompatible_dimensions_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_multiply_incompatible_dimensions_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
#[ should_panic( expected = "Incompatible dimensions for matrix multiplication" ) ]
fn test_multiply_incompatible_dimensions_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_multiply_incompatible_dimensions_generic::< DescriptorOrderColumnMajor >();
}

/// ## Root Cause
/// `d2::mat_vec_mul`'s dimension-compatibility check (`adim[1] != ROWS`) lived inside
/// `#[ cfg( debug_assertions ) ]`, so a release build skipped it entirely.
///
/// ## Why Not Caught
/// `mat_vec_mul` has no direct test anywhere in this suite; it is only reachable
/// indirectly through the `Mul<Vector<COLS>>` operator impls, which never exercise a
/// mismatched shape.
///
/// ## Fix Applied
/// TASK-014 changed `mat_vec_mul()`'s `#[ cfg( debug_assertions ) ]` block to run
/// unconditionally (see `src/d2/arithmetics/mul.rs`). This test calls the free function
/// directly with a matrix column count that does not match the vector length.
///
/// ## Prevention
/// Running this test under a release profile would have failed before the fix: no panic,
/// and `a.lane_iter(..).zip(b.vector_iter())` silently truncates to the shorter of the two,
/// producing a wrong dot product. It passes after the fix, in every profile.
///
/// ## Pitfall
/// A dimension check gated to debug builds gives no protection to code paths (like this
/// free function) that are only reachable directly, bypassing whatever validation an
/// operator-overload wrapper might otherwise imply.
fn test_multiply_vec_incompatible_dimensions_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 3, 3, f32, D > : the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 3, 3, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
  the_module::Mat< 3, 3, f32, D > : the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::
  {
    Mat,
    Vector,
    d2,
  };

  // a is 3x3 (3 columns), but b/r are length-2 vectors: a's column count (3) does not
  // match the vector length (2).
  let mat_a = Mat::< 3, 3, f32, D >::default().row_major_set( &[ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ] );
  let vec_b = Vector::< f32, 2 >::from_array( [ 1.0, 2.0 ] );
  let mut vec_r = Vector::< f32, 2 >::default();

  // Attempt matrix-vector multiplication with incompatible dimensions, should panic
  d2::mat_vec_mul( &mut vec_r, &mat_a, &vec_b );
}

#[ test ]
#[ should_panic( expected = "Incompatible dimensions for matrix-vector multiplication" ) ]
fn test_multiply_vec_incompatible_dimensions_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_multiply_vec_incompatible_dimensions_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
#[ should_panic( expected = "Incompatible dimensions for matrix-vector multiplication" ) ]
fn test_multiply_vec_incompatible_dimensions_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_multiply_vec_incompatible_dimensions_generic::< DescriptorOrderColumnMajor >();
}
