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
