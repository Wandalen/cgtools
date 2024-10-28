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
    RawSlice,
    d2,
  };

  // Define matrices
  let mat_a = Mat::< 1, 3, f32, D >::default().set
  ([
    1.0, 2.0, 3.0,
  ]);

  let mat_b = Mat::< 3, 2, f32, D >::default().set
  ([
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
  let exp = Mat::< 1, 2, f32, D >::default().set
  ([
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

// xxx : uncomment
#[ test ]
fn test_multiply_matrices_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_multiply_matrices_generic::< DescriptorOrderColumnMajor >();
}

// // qqq : implement try build test throwing error
//
// fn test_multiply_incompatible_dimensions_generic< D : the_module::mat::Descriptor >()
// where
//
//   the_module::Mat< 1, 2, f32, D > : the_module::ScalarMut< Scalar = f32 >,
//   the_module::Mat< 1, 3, f32, D > : the_module::IndexingRef< Scalar = f32 >,
//   the_module::Mat< 3, 2, f32, D > : the_module::IndexingRef< Scalar = f32 >,
//
//   the_module::Mat< 1, 2, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
//   the_module::Mat< 1, 3, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
//   the_module::Mat< 3, 2, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
//
//   for< 'a > &'a the_module::Mat< 1, 3, f32, D > : core::ops::Mul< &'a the_module::Mat< 3, 2, f32, D >, Output = the_module::Mat< 1, 2, f32, D > >,
//   the_module::Mat< 1, 3, f32, D > : core::ops::Mul< the_module::Mat< 3, 2, f32, D >, Output = the_module::Mat< 1, 2, f32, D > >,
//
//   // the_module::Mat< 2, 3, f32, D > : the_module::Zero,
//   // the_module::Mat< 2, 2, f32, D > : the_module::Zero,
// {
//   use the_module::
//   {
//     Zero,
//     Mat,
//     RawSliceMut,
//     d2,
//   };
//
//   // Define incompatible matrices
//   let mat_a = Mat::< 2, 3, f32, D >::default().raw_set([ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ]);
//   let mat_b = Mat::< 2, 2, f32, D >::default().raw_set([ 7.0, 8.0, 9.0, 10.0 ]);
//   let mut mat_r = Mat::< 2, 2, f32, D >::default();
//
//   // Attempt multiplication, should panic
//   d2::mul( &mut mat_r, &mat_a, &mat_b );
// }
//
// #[ test ]
// #[ should_panic ]
// fn test_multiply_incompatible_dimensions_row_major()
// {
//   use the_module::mat::DescriptorOrderRowMajor;
//   test_multiply_incompatible_dimensions_generic::< DescriptorOrderRowMajor >();
// }
//
// #[ test ]
// #[ should_panic ]
// fn test_multiply_incompatible_dimensions_column_major()
// {
//   use the_module::mat::DescriptorOrderColumnMajor;
//   test_multiply_incompatible_dimensions_generic::< DescriptorOrderColumnMajor >();
// }
