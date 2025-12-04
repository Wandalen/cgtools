use super::*;
use the_module::{ geometry::BoundingBox, F32x3 };
use approx::assert_abs_diff_eq;

#[ test ]
fn test_default()
{
  let bb = BoundingBox::default();

  assert_eq!( bb.min, F32x3::MAX );
  assert_eq!( bb.max, F32x3::MIN );
}

#[ test ]
fn test_apply_rotation()
{
  let mut bb = BoundingBox::default();
  bb.min = F32x3::splat( 0.0 );
  bb.max = F32x3::splat( 1.0 );

  let mat = mingl::math::mat3x3h::rot( 0.0, 90.0f32.to_radians(), 0.0 );

  bb.apply_transform_mut( mat );

  assert_abs_diff_eq!( bb.min, F32x3::new( 0.0, 0.0, -1.0 ) );
  assert_abs_diff_eq!( bb.max, F32x3::new( 1.0, 1.0, 0.0 ) );
}

#[ test ]
fn test_apply_scale()
{
  let mut bb = BoundingBox::default();
  bb.min = F32x3::splat( 0.0 );
  bb.max = F32x3::splat( 1.0 );

  let mat = mingl::math::mat3x3h::scale( [ 2.0, 5.0, 3.0 ] );

  bb.apply_transform_mut( mat );

  assert_abs_diff_eq!( bb.min, F32x3::new( 0.0, 0.0, 0.0 ) );
  assert_abs_diff_eq!( bb.max, F32x3::new( 2.0, 5.0, 3.0 ) );
}

#[ test ]
fn test_apply_translation()
{
  let mut bb = BoundingBox::default();
  bb.min = F32x3::splat( 0.0 );
  bb.max = F32x3::splat( 1.0 );

  let mat = mingl::math::mat3x3h::translation( [ 2.0, 5.0, 3.0 ] );

  bb.apply_transform_mut( mat );

  assert_abs_diff_eq!( bb.min, F32x3::new( 2.0, 5.0, 3.0 ) );
  assert_abs_diff_eq!( bb.max, F32x3::new( 3.0, 6.0, 4.0 ) );
}

#[ test ]
fn test_combine1()
{
  let mut bb1 = BoundingBox::new( [ 0.0, 0.0, 0.0 ], [ 1.0, 1.0, 1.0 ] );
  let bb2 = BoundingBox::new( [ 0.0, 0.0, 0.0 ], [ 5.0, 5.0, 5.0 ] );

  bb1.combine_mut( &bb2 );

  assert_abs_diff_eq!( bb1.min, F32x3::new( 0.0, 0.0, 0.0 ) );
  assert_abs_diff_eq!( bb1.max, F32x3::new( 5.0, 5.0, 5.0 ) );
}

#[ test ]
fn test_combine2()
{
  let mut bb1 = BoundingBox::new( [ 0.0, 0.0, 0.0 ], [ 1.0, 1.0, 1.0 ] );
  let bb2 = BoundingBox::new( [ -1.0, -1.0, -1.0 ], [ 0.0, 0.0, 0.0  ] );

  bb1.combine_mut( &bb2 );

  assert_abs_diff_eq!( bb1.min, F32x3::new( -1.0, -1.0, -1.0 ) );
  assert_abs_diff_eq!( bb1.max, F32x3::new( 1.0, 1.0, 1.0 ) );
}

#[ test ]
fn test_combine3()
{
  let mut bb1 = BoundingBox::new( [ 0.0, 0.0, 0.0 ], [ 1.0, 1.0, 1.0 ] );
  let bb2 = BoundingBox::new( [ 0.0, 0.0, -1.0 ], [ 1.0, 1.0, 0.0  ] );

  bb1.combine_mut( &bb2 );

  assert_abs_diff_eq!( bb1.min, F32x3::new( 0.0, 0.0, -1.0 ) );
  assert_abs_diff_eq!( bb1.max, F32x3::new( 1.0, 1.0, 1.0) );
}

#[ test ]
fn test_compute1()
{
  let points = 
  [
    0.0, 0.0, 0.0,
    1.0, 1.0, 1.0,
  ];

  let bb = BoundingBox::compute( &points );

  assert_abs_diff_eq!( bb.min, F32x3::new( 0.0, 0.0, 0.0 ) );
  assert_abs_diff_eq!( bb.max, F32x3::new( 1.0, 1.0, 1.0 ) );
}

fn test_compute2()
{
  let points = 
  [
    -1.0, -5.0, -2.0,
    0.0, 0.0, 0.0,
    1.0, 5.0, 3.0,
  ];

  let bb = BoundingBox::compute( &points );

  assert_abs_diff_eq!( bb.min, F32x3::new( -1.0, -5.0, -2.0 ) );
  assert_abs_diff_eq!( bb.max, F32x3::new( 1.0, 5.0, 3.0 ) );
}
