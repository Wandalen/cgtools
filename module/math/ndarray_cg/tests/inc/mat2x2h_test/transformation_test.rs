use super::*;
use std::f32::consts::PI;

#[ test ]
fn test_rot()
{
  let angle_radians = PI / 4.0;
  let cos_theta = angle_radians.cos();
  let sin_theta = angle_radians.sin();
  let exp = the_module::F32x3x3::from_row_major
  ([
    cos_theta, -sin_theta, 0.0,
    sin_theta, cos_theta, 0.0,
    0.0, 0.0, 1.0,
  ]);
  let got = the_module::mat2x2h::rot( angle_radians );
  assert_eq!( got, exp );
}

#[ test ]
fn test_translate()
{
  let tx = 2.0;
  let ty = 3.0;
  let exp = the_module::F32x3x3::from_row_major
  ([
    1.0, 0.0, tx,
    0.0, 1.0, ty,
    0.0, 0.0, 1.0,
  ]);
  let got = the_module::mat2x2h::translate( [ tx, ty ] );
  assert_eq!( got, exp );
  let got = the_module::mat2x2h::translate( &[ tx, ty ] );
  assert_eq!( got, exp );
}

#[ test ]
fn test_scale()
{
  let sx = 2.0;
  let sy = 3.0;
  let exp = the_module::F32x3x3::from_row_major
  ([
    sx, 0.0, 0.0,
    0.0, sy, 0.0,
    0.0, 0.0, 1.0,
  ]);
  let got = the_module::mat2x2h::scale( [ sx, sy ] );
  assert_eq!( got, exp );
  let got = the_module::mat2x2h::scale( &[ sx, sy ] );
  assert_eq!( got, exp );
}

#[ test ]
fn test_shear()
{
  let shx = 1.0;
  let shy = 0.5;
  let exp = the_module::F32x3x3::from_row_major
  ([
    1.0, shx, 0.0,
    shy, 1.0, 0.0,
    0.0, 0.0, 1.0,
  ]);
  let got = the_module::mat2x2h::shear( [ shx, shy ] );
  assert_eq!( got, exp );
  let got = the_module::mat2x2h::shear( &[ shx, shy ] );
  assert_eq!( got, exp );
}

#[ test ]
fn test_reflect_x()
{
  let exp = the_module::F32x3x3::from_row_major
  ([
    1.0, 0.0, 0.0,
    0.0, -1.0, 0.0,
    0.0, 0.0, 1.0,
  ]);
  let got = the_module::mat2x2h::reflect_x();
  assert_eq!( got, exp );
}

#[ test ]
fn test_reflect_y()
{
  let exp = the_module::F32x3x3::from_row_major
  ([
    -1.0, 0.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 0.0, 1.0,
  ]);
  let got = the_module::mat2x2h::reflect_y();
  assert_eq!( got, exp );
}

#[ test ]
fn test_rot_around_point()
{
  let angle = PI / 4.0;
  let px = 1.0;
  let py = 1.0;
  let cos_theta = angle.cos();
  let sin_theta = angle.sin();
  let exp = the_module::F32x3x3::from_row_major
  ([
    cos_theta, -sin_theta, px - cos_theta * px + sin_theta * py,
    sin_theta, cos_theta, py - sin_theta * px - cos_theta * py,
    0.0, 0.0, 1.0,
  ]);
  let got = the_module::mat2x2h::rot_around_point( angle, [ px, py ] );
  assert_eq!( got, exp );
  let got = the_module::mat2x2h::rot_around_point( angle, &[ px, py ] );
  assert_eq!( got, exp );
}

#[ test ]
fn test_scale_relative_to_point()
{
  let sx = 2.0;
  let sy = 3.0;
  let px = 1.0;
  let py = 1.0;
  let exp = the_module::F32x3x3::from_row_major
  ([
    sx, 0.0, px - sx * px,
    0.0, sy, py - sy * py,
    0.0, 0.0, 1.0,
  ]);
  let got = the_module::mat2x2h::scale_relative_to_point( [ sx, sy ], [ px, py ] );
  assert_eq!( got, exp );
  let got = the_module::mat2x2h::scale_relative_to_point( &[ sx, sy ], &[ px, py ] );
  assert_eq!( got, exp );
}
