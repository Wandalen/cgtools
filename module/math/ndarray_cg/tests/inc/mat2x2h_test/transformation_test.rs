use super::*;
use std::f32::consts::PI;

#[ test ]
fn test_rot()
{
  use the_module::{ RawSlice, mat::DescriptorOrderRowMajor };
  let angle_radians = PI / 4.0;
  let cos_theta = angle_radians.cos();
  let sin_theta = angle_radians.sin();
  let exp : [ f32; 9 ] =
  [
    cos_theta, -sin_theta, 0.0,
    sin_theta, cos_theta, 0.0,
    0.0, 0.0, 1.0,
  ];
  let got = the_module::mat2x2h::rot::< _, DescriptorOrderRowMajor >( angle_radians );
  assert_eq!( got.raw_slice(), exp );
}

#[ test ]
fn test_translate()
{
  use the_module::{ RawSliceMut, mat::DescriptorOrderRowMajor };
  let tx = 2.0;
  let ty = 3.0;
  let exp = the_module::Mat::< 3, 3, _, DescriptorOrderRowMajor >::default().raw_set
  ([
    1.0, 0.0, tx,
    0.0, 1.0, ty,
    0.0, 0.0, 1.0,
  ]);
  let got = the_module::mat2x2h::translate::< _, _, DescriptorOrderRowMajor >( [ tx, ty ] );
  assert_eq!( got, exp );
  let got = the_module::mat2x2h::translate::< _, _, DescriptorOrderRowMajor >( &[ tx, ty ] );
  assert_eq!( got, exp );
}

#[ test ]
fn test_scale()
{
  use the_module::{ RawSliceMut, mat::DescriptorOrderRowMajor };
  let sx = 2.0;
  let sy = 3.0;
  let exp = the_module::Mat::< 3, 3, _, DescriptorOrderRowMajor >::default().raw_set
  ([
    sx, 0.0, 0.0,
    0.0, sy, 0.0,
    0.0, 0.0, 1.0,
  ]);
  let got = the_module::mat2x2h::scale::< _, _, DescriptorOrderRowMajor >( [ sx, sy ] );
  assert_eq!( got, exp );
  let got = the_module::mat2x2h::scale::< _, _, DescriptorOrderRowMajor >( &[ sx, sy ] );
  assert_eq!( got, exp );
}

#[ test ]
fn test_shear()
{
  use the_module::{ RawSliceMut, mat::DescriptorOrderRowMajor };
  let shx = 1.0;
  let shy = 0.5;
  let exp = the_module::Mat::< 3, 3, _, DescriptorOrderRowMajor >::default().raw_set
  ([
    1.0, shx, 0.0,
    shy, 1.0, 0.0,
    0.0, 0.0, 1.0,
  ]);
  let got = the_module::mat2x2h::shear::< _, _, DescriptorOrderRowMajor >( [ shx, shy ] );
  assert_eq!( got, exp );
  let got = the_module::mat2x2h::shear::< _, _, DescriptorOrderRowMajor >( &[ shx, shy ] );
  assert_eq!( got, exp );
}

#[ test ]
fn test_reflect_x()
{
  use the_module::{ RawSliceMut, mat::DescriptorOrderRowMajor };
  let exp = the_module::Mat::< 3, 3, _, DescriptorOrderRowMajor >::default().raw_set
  ([
    1.0, 0.0, 0.0,
    0.0, -1.0, 0.0,
    0.0, 0.0, 1.0,
  ]);
  let got = the_module::mat2x2h::reflect_x::< _, DescriptorOrderRowMajor >();
  assert_eq!( got, exp );
}

#[ test ]
fn test_reflect_y()
{
  use the_module::{ RawSliceMut, mat::DescriptorOrderRowMajor };
  let exp = the_module::Mat::< 3, 3, _, DescriptorOrderRowMajor >::default().raw_set
  ([
    -1.0, 0.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 0.0, 1.0,
  ]);
  let got = the_module::mat2x2h::reflect_y::< _, DescriptorOrderRowMajor >();
  assert_eq!( got, exp );
}

#[ test ]
fn test_rot_around_point()
{
  use the_module::{ RawSliceMut, mat::DescriptorOrderRowMajor };
  let angle = PI / 4.0;
  let px = 1.0;
  let py = 1.0;
  let cos_theta = angle.cos();
  let sin_theta = angle.sin();
  let exp = the_module::Mat::< 3, 3, _, DescriptorOrderRowMajor >::default().raw_set
  ([
    cos_theta, -sin_theta, px - cos_theta * px + sin_theta * py,
    sin_theta, cos_theta, py - sin_theta * px - cos_theta * py,
    0.0, 0.0, 1.0,
  ]);
  let got = the_module::mat2x2h::rot_around_point::< _, _, DescriptorOrderRowMajor >( angle, [ px, py ] );
  assert_eq!( got, exp );
  let got = the_module::mat2x2h::rot_around_point::< _, _, DescriptorOrderRowMajor >( angle, &[ px, py ] );
  assert_eq!( got, exp );
}

#[ test ]
fn test_scale_relative_to_point()
{
  use the_module::{ RawSliceMut, mat::DescriptorOrderRowMajor };
  let sx = 2.0;
  let sy = 3.0;
  let px = 1.0;
  let py = 1.0;
  let exp = the_module::Mat::< 3, 3, _, DescriptorOrderRowMajor >::default().raw_set
  ([
    sx, 0.0, px - sx * px,
    0.0, sy, py - sy * py,
    0.0, 0.0, 1.0,
  ]);
  let got = the_module::mat2x2h::scale_relative_to_point::< _, _, _, DescriptorOrderRowMajor >( [ sx, sy ], [ px, py ] );
  assert_eq!( got, exp );
  let got = the_module::mat2x2h::scale_relative_to_point::< _, _, _, DescriptorOrderRowMajor >( &[ sx, sy ], &[ px, py ] );
  assert_eq!( got, exp );
}
