use super::*;

fn test_zero_matrices_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 2, f32, D > : the_module::IndexingMut,
  the_module::Mat< 2, 2, f32, D > : the_module::RawSliceMut< Scalar = f32 >,
  the_module::Mat< 2, 2, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
{
  use the_module::{ Mat, Zero };

  // `Mat::zero()` must equal the pre-existing `Default::default()` value -- regression proof
  // that migrating `add.rs`/`mul.rs` call sites from `.default()` to `.zero()` is
  // behavior-preserving.
  let zero = Mat::< 2, 2, f32, D >::zero();
  let default = Mat::< 2, 2, f32, D >::default();
  assert_eq!( zero.raw_slice(), default.raw_slice(), "Mat::zero() must equal Default::default()" );

  // `is_zero()` must be true for the zero matrix.
  assert!( zero.is_zero(), "the zero matrix must report is_zero() == true" );

  // `is_zero()` must be false for a genuinely non-zero matrix -- a test that only ever
  // exercises the zero case can't catch a buggy `is_zero` that always returns `true`.
  let non_zero = Mat::< 2, 2, f32, D >::default().set_raw
  ([
    1.0, 0.0,
    0.0, 0.0,
  ]);
  assert!( !non_zero.is_zero(), "a non-zero matrix must report is_zero() == false" );
}

#[ test ]
fn test_zero_matrices_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_zero_matrices_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_zero_matrices_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_zero_matrices_generic::< DescriptorOrderColumnMajor >();
}
