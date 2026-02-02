use super::*;
use the_module::{ geometry::{ BoundingBox, BoundingSphere }, F32x3 };
use approx::assert_abs_diff_eq;

#[ test ]
fn test_default()
{
  let bs = BoundingSphere::default();

  assert_abs_diff_eq!( bs.center, F32x3::default() );
  assert_abs_diff_eq!( bs.radius, &0.0 );
}

#[ test ]
fn test_compute()
{
  let points = 
  [
    0.0, 0.0, 0.0,
    1.0, 1.0, 1.0,
  ];

  let bb = BoundingBox::compute( &points );
  let bs = BoundingSphere::compute( &points, &bb );

  assert_abs_diff_eq!( bs.center, F32x3::new( 0.5, 0.5, 0.5 ) );
  assert_abs_diff_eq!( bs.radius, &0.75f32.sqrt() );
}

