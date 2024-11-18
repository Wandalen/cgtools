use super::*;
use std::f32::consts::PI;

#[ test ]
fn test_rot()
{
  let angle_radians = PI / 4.0;
  let cos_theta = angle_radians.cos();
  let sin_theta = angle_radians.sin();
  let exp = the_module::F32x2x2::from_row_major
  ([
    cos_theta, -sin_theta,
    sin_theta, cos_theta,
  ]);
  let got = the_module::mat2x2::rot( angle_radians );
  println!( "name: {} | size: {}", core::any::type_name_of_val( &got ), core::mem::size_of_val( &got ) );
  println!( "name: {} | size: {}", core::any::type_name_of_val( &exp ), core::mem::size_of_val( &exp ) );
  println!( "Rotation matrix for {} degrees:\n{:?}", angle_radians, got );
  assert_eq!( got, exp );
}

#[ test ]
fn test_scale()
{
  let sx = 2.0;
  let sy = 3.0;
  let exp = the_module::F32x2x2::from_row_major
  ([
    sx, 0.0,
    0.0, sy,
  ]);
  let got = the_module::mat2x2::scale( [ sx, sy ] );
  assert_eq!( got, exp );
  let got = the_module::mat2x2::scale( &[ sx, sy ] );
  assert_eq!( got, exp );
}

#[ test ]
fn test_shear()
{
  let shx = 1.0;
  let shy = 0.5;
  let exp = the_module::F32x2x2::from_row_major
  ([
    1.0, shx,
    shy, 1.0,
  ]);
  let got = the_module::mat2x2::shear( [ shx, shy ] );
  assert_eq!( got, exp );
  let got = the_module::mat2x2::shear( &[ shx, shy ] );
  assert_eq!( got, exp );
}

#[ test ]
fn test_reflect_x()
{
  let exp = the_module::F32x2x2::from_row_major
  ([
    1.0, 0.0,
    0.0, -1.0,
  ]);
  let got = the_module::mat2x2::reflect_x();
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
  let exp = the_module::F32x2x2::from_row_major
  ([
    -1.0, 0.0,
    0.0, 1.0,
  ]);
  let got = the_module::mat2x2::reflect_y();
  assert_eq!( got, exp );
}
