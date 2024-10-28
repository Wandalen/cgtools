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
