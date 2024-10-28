use super::*;
use std::f32::consts::PI;

#[ test ]
fn test_rot()
{
  use the_module::
  {
    RawSlice,
    mat::DescriptorOrderRowMajor,
  };
  let angle_radians = PI / 4.0;
  let cos_theta = angle_radians.cos();
  let sin_theta = angle_radians.sin();
  let exp : [ f32; 4 ] =
  [
    cos_theta, -sin_theta,
    sin_theta, cos_theta,
  ];
  let got = the_module::mat2x2::rot::< _, DescriptorOrderRowMajor >( angle_radians );
  println!( "name: {} | size: {}", core::any::type_name_of_val( &got ), core::mem::size_of_val( &got ) );
  println!( "name: {} | size: {}", core::any::type_name_of_val( &exp ), core::mem::size_of_val( &exp ) );
  println!( "Rotation matrix for {} degrees:\n{:?}", angle_radians, got );
  assert_eq!( got.raw_slice(), exp );
}

#[ test ]
fn test_scale()
{
  use the_module::
  {
    RawSliceMut,
    mat::DescriptorOrderRowMajor,
  };
  let sx = 2.0;
  let sy = 3.0;
  let exp = the_module::Mat::< 2, 2, _, DescriptorOrderRowMajor >::default().raw_set
  ([
    sx, 0.0,
    0.0, sy,
  ]);
  let got = the_module::mat2x2::scale::< _, _, DescriptorOrderRowMajor >( [ sx, sy ] );
  assert_eq!( got, exp );
  let got = the_module::mat2x2::scale::< _, _, DescriptorOrderRowMajor >( &[ sx, sy ] );
  assert_eq!( got, exp );
}

#[ test ]
fn test_shear()
{
  use the_module::
  {
    RawSliceMut,
    mat::DescriptorOrderRowMajor,
  };

  let shx = 1.0;
  let shy = 0.5;
  let exp = the_module::Mat::< 2, 2, _, DescriptorOrderRowMajor >::default().raw_set
  ([
    1.0, shx,
    shy, 1.0,
  ]);
  let got = the_module::mat2x2::shear::< _, _, DescriptorOrderRowMajor >( [ shx, shy ] );
  assert_eq!( got, exp );
  let got = the_module::mat2x2::shear::< _, _, DescriptorOrderRowMajor >( &[ shx, shy ] );
  assert_eq!( got, exp );
}

#[ test ]
fn test_reflect_x()
{
  use the_module::
  {
    RawSliceMut,
    mat::DescriptorOrderRowMajor,
  };
  let exp = the_module::Mat::< 2, 2, _, DescriptorOrderRowMajor >::default().raw_set
  ([
    1.0, 0.0,
    0.0, -1.0,
  ]);
  let got = the_module::mat2x2::reflect_x::< _, DescriptorOrderRowMajor >();
  assert_eq!( got, exp );
}

#[ test ]
fn test_reflect_y()
{
  use the_module::
  {
    RawSliceMut,
    mat::DescriptorOrderRowMajor,
  };
  let exp = the_module::Mat::< 2, 2, _, DescriptorOrderRowMajor >::default().raw_set
  ([
    -1.0, 0.0,
    0.0, 1.0,
  ]);
  let got = the_module::mat2x2::reflect_y::< _, DescriptorOrderRowMajor >();
  assert_eq!( got, exp );
}
