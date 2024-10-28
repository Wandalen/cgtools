use super::*;

fn test_add_matrices_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 2, f32, D > : the_module::IndexingMut,
  the_module::Mat< 2, 2, f32, D > : the_module::RawSliceMut< Scalar = f32 >,
  the_module::Mat< 2, 2, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
{
  use the_module::
  {
    Mat,
    d2,
  };

  // Define matrices
  let mat_a = the_module::Mat::< 2, 2, f32, D >::default().set
  ([
    1.0, 2.0,
    3.0, 4.0,
  ]);
  let mat_b = Mat::< 2, 2, f32, D >::default().set
  ([
    5.0, 6.0,
    7.0, 8.0,
  ]);
  let mut mat_r = Mat::< 2, 2, f32, D >::default();

  // Perform addition
  d2::add( &mut mat_r, &mat_a, &mat_b );

  // Expected result
  let exp = Mat::< 2, 2, f32, D >::default().set
  ([
    6.0, 8.0,
    10.0, 12.0,
  ]);
  assert_eq!( mat_r.raw_slice(), exp.raw_slice(), "Expected {:?}, got {:?}", exp.raw_slice(), mat_r.raw_slice() );

  // Test operator overloading
  let mat_r = &mat_a + &mat_b;
  assert_eq!( mat_r.raw_slice(), exp.raw_slice(), "Expected {:?}, got {:?}", exp.raw_slice(), mat_r.raw_slice() );

  let mat_r = mat_a + mat_b;
  assert_eq!( mat_r.raw_slice(), exp.raw_slice(), "Expected {:?}, got {:?}", exp.raw_slice(), mat_r.raw_slice() );
}

// xxx

#[ test ]
fn test_add_matrices_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_add_matrices_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_add_matrices_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_add_matrices_generic::< DescriptorOrderColumnMajor >();
}

fn test_add_incompatible_dimensions_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 2, f32, D > : the_module::IndexingMut,
  the_module::Mat< 2, 2, f32, D > : the_module::RawSliceMut< Scalar = f32 >,
  the_module::Mat< 2, 2, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
  the_module::Mat< 2, 3, f32, D > : the_module::IndexingMut,
  the_module::Mat< 2, 3, f32, D > : the_module::RawSliceMut< Scalar = f32 >,
  the_module::Mat< 2, 3, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
{
  use the_module::
  {
    Mat,
    RawSliceMut,
    d2,
  };

  // Define incompatible matrices
  let mat_a = Mat::< 2, 3, f32, D >::default().set([ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ]);
  let mat_b = Mat::< 2, 2, f32, D >::default().set([ 7.0, 8.0, 9.0, 10.0 ]);
  let mut mat_r = Mat::< 2, 2, f32, D >::default();

  // Attempt addition, should panic
  d2::add( &mut mat_r, &mat_a, &mat_b );
}

#[ test ]
#[ should_panic ]
fn test_add_incompatible_dimensions_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_add_incompatible_dimensions_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
#[ should_panic ]
fn test_add_incompatible_dimensions_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_add_incompatible_dimensions_generic::< DescriptorOrderColumnMajor >();
}
