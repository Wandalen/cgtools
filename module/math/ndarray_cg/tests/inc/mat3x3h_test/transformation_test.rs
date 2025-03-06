use ndarray_cg::*;
use mat3x3h::{rot, scale, translation};

#[ test ]
fn test_translation()
{
  let vec = Vector( [ 0.0_f32, 0.0, 0.0, 1.0 ] );
  let translation = translation( [ 1.0_f32, 2.0, 3.0 ] );
  let res = translation * vec;

  assert_eq!( res.x(), 1.0 );
  assert_eq!( res.y(), 2.0 );
  assert_eq!( res.z(), 3.0 );
}

#[ test ]
fn test_rotation()
{
  let x = Vector( [ 1.0_f32, 0.0, 0.0, 1.0 ] );
  let y = Vector( [ 0.0_f32, 1.0, 0.0, 1.0 ] );
  let z = Vector( [ 0.0_f32, 0.0, 1.0, 1.0 ] );
  
  let angle = std::f32::consts::FRAC_PI_2;
  
  let rotation_x = rot( angle, 0.0, 0.0 );
  let rotation_y = rot( 0.0, angle, 0.0 );
  let rotation_z = rot( 0.0, 0.0, angle );
  
  let rotated_x = rotation_z * x;
  let rotated_y = rotation_x * y;
  let rotated_z = rotation_y * z;

  assert_eq!( rotated_x.y(), 1.0 );
  assert_eq!( rotated_y.z(), 1.0 );
  assert_eq!( rotated_z.x(), 1.0 );
}

#[ test ]
fn test_scale()
{
  let vec = Vector( [ 1.0_f32, 1.0, 1.0, 1.0 ] );
  let scale = scale( [ 0.1, 0.2, 0.3 ] );
  let scale_vec = scale * vec;

  assert_eq!( scale_vec.x(), 0.1 );
  assert_eq!( scale_vec.y(), 0.2 );
  assert_eq!( scale_vec.z(), 0.3 );
}

