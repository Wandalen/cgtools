use super::*;
use the_module::{ F32x3 };
use approx::assert_abs_diff_eq;

#[ test ]
fn test_rotation_disabled_prevents_rotation()
{
  let mut controls = the_module::camera_orbit_controls::CameraOrbitControls::default();
  controls.eye = F32x3::new( 1.0, 0.0, 0.0 );
  controls.up = F32x3::new( 0.0, 1.0, 0.0 );
  controls.center = F32x3::new( 0.0, 0.0, 0.0 );
  controls.rotation.enabled = false;

  controls.rotate( [ 50.0, 50.0 ] );

  let exp_eye =  F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );
}