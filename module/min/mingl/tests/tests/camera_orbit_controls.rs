use super::*;
use the_module::{ F32x3 };
use approx::assert_abs_diff_eq;

#[ test ]
fn test_rotation_disabled_prevents_rotation()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls::default();
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

#[ test ]
fn test_zoom_disabled_prevents_zoom()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls::default();
  controls.eye = F32x3::new( 1.0, 0.0, 0.0 );
  controls.up = F32x3::new( 0.0, 1.0, 0.0 );
  controls.center = F32x3::new( 0.0, 0.0, 0.0 );
  controls.zoom.enabled = false;

  controls.zoom( 50.0 );

  let exp_eye =  F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );
}

#[ test ]
fn test_pan_disabled_prevents_pan()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls::default();
  controls.eye = F32x3::new( 1.0, 0.0, 0.0 );
  controls.up = F32x3::new( 0.0, 1.0, 0.0 );
  controls.center = F32x3::new( 0.0, 0.0, 0.0 );
  controls.pan.enabled = false;

  controls.pan( [ 50.0, 50.0 ] );

  let exp_eye =  F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );
}

#[ test ]
fn test_rotation_longitude()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls::default();
  controls.eye = F32x3::new( 1.0, 0.0, 0.0 );
  controls.up = F32x3::new( 0.0, 1.0, 0.0 );
  controls.center = F32x3::new( 0.0, 0.0, 0.0 );
  controls.rotation.speed = 1.0;

  controls.rotate( [ std::f32::consts::FRAC_PI_2, 0.0 ] );

  let rot_mat = the_module::math::mat3x3::from_angle_y( std::f32::consts::FRAC_PI_2 );

  let exp_eye =  rot_mat * F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );

  controls.rotate( [ -std::f32::consts::FRAC_PI_2, 0.0 ] );

  let exp_eye = F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );
}

#[ test ]
fn test_rotation_longitude_with_non_origin_center()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls::default();
  controls.eye = F32x3::new( 2.0, 0.0, 0.0 );
  controls.up = F32x3::new( 0.0, 1.0, 0.0 );
  controls.center = F32x3::new( 1.0, 0.0, 0.0 );
  controls.rotation.speed = 1.0;

  controls.rotate( [ std::f32::consts::FRAC_PI_2, 0.0 ] );

  let rot_mat = the_module::math::mat3x3::from_angle_y( std::f32::consts::FRAC_PI_2 );

  let exp_eye =  rot_mat * F32x3::new( 1.0, 0.0, 0.0 ) + F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 1.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );

  controls.rotate( [ -std::f32::consts::FRAC_PI_2, 0.0 ] );

  let exp_eye = F32x3::new( 2.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 1.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );
}

#[ test ]
fn test_rotation_latitude()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls::default();
  controls.eye = F32x3::new( 1.0, 0.0, 0.0 );
  controls.up = F32x3::new( 0.0, 1.0, 0.0 );
  controls.center = F32x3::new( 0.0, 0.0, 0.0 );
  controls.rotation.speed = 1.0;

  controls.rotate( [ 0.0, std::f32::consts::FRAC_PI_4 ] );

  let rot_mat = the_module::math::mat3x3::from_angle_z( std::f32::consts::FRAC_PI_4 );

  let exp_eye =  rot_mat * F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = rot_mat * F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );

  controls.rotate( [ 0.0, -std::f32::consts::FRAC_PI_4 ] );

  let exp_eye = F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );
}

#[ test ]
fn test_rotation_latitude_with_non_origin_center()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls::default();
  controls.eye = F32x3::new( 2.0, 0.0, 0.0 );
  controls.up = F32x3::new( 0.0, 1.0, 0.0 );
  controls.center = F32x3::new( 1.0, 0.0, 0.0 );
  controls.rotation.speed = 1.0;

  controls.rotate( [ 0.0, std::f32::consts::FRAC_PI_4 ] );

  let rot_mat = the_module::math::mat3x3::from_angle_z( std::f32::consts::FRAC_PI_4 );

  let exp_eye =  rot_mat * F32x3::new( 1.0, 0.0, 0.0 ) +  F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = rot_mat * F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 1.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );

  controls.rotate( [ 0.0, -std::f32::consts::FRAC_PI_4 ] );

  let exp_eye = F32x3::new( 2.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 1.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );
}

#[ test ]
fn test_longitude_range_clamps_correctly()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls::default();
  controls.eye = F32x3::new( 1.0, 0.0, 0.0 );
  controls.up = F32x3::new( 0.0, 1.0, 0.0 );
  controls.center = F32x3::new( 0.0, 0.0, 0.0 );
  controls.rotation.longitude_range_set( 90.0 );
  controls.rotation.speed = 1.0;

  // Counter-clockwise
  controls.rotate( [ std::f32::consts::PI, 0.0 ] );

  let rot_mat = the_module::math::mat3x3::from_angle_y( 90.0f32.to_radians() );

  let exp_eye =  rot_mat * F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );

  // Clockwise
  controls.eye = F32x3::new( 1.0, 0.0, 0.0 );
  controls.up = F32x3::new( 0.0, 1.0, 0.0 );
  controls.center = F32x3::new( 0.0, 0.0, 0.0 );

  controls.rotate( [ -std::f32::consts::PI, 0.0 ] );

  let rot_mat = the_module::math::mat3x3::from_angle_y( -90.0f32.to_radians() );

  let exp_eye =  rot_mat * F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );
}

#[ test ]
fn test_latitude_range_clamps_correctly()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls::default();
  controls.eye = F32x3::new( 1.0, 0.0, 0.0 );
  controls.up = F32x3::new( 0.0, 1.0, 0.0 );
  controls.center = F32x3::new( 0.0, 0.0, 0.0 );
  controls.rotation.latitude_range_set( 45.0 );
  controls.rotation.speed = 1.0;

  // Counter-clockwise
  controls.rotate( [ 0.0, std::f32::consts::PI * 0.5 ] );

  let rot_mat = the_module::math::mat3x3::from_angle_z( 45.0f32.to_radians() );

  let exp_eye =  rot_mat * F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = rot_mat * F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );

  // Clockwise
  controls.eye = F32x3::new( 1.0, 0.0, 0.0 );
  controls.up = F32x3::new( 0.0, 1.0, 0.0 );
  controls.center = F32x3::new( 0.0, 0.0, 0.0 );

  controls.rotate( [ 0.0, -std::f32::consts::PI * 0.5 ] );

  let rot_mat = the_module::math::mat3x3::from_angle_z( -45.0f32.to_radians() );

  let exp_eye =  rot_mat * F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = rot_mat * F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );
}

#[ test ]
fn test_zoom_min_distance_enforced()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls::default();
  controls.eye = F32x3::new( 1.0, 0.0, 0.0 );
  controls.up = F32x3::new( 0.0, 1.0, 0.0 );
  controls.center = F32x3::new( 0.0, 0.0, 0.0 );
  controls.zoom.min_distance_set( 0.2 );
  controls.zoom.speed = 1.0;

  controls.zoom( -9.0 );

  let exp_eye =  0.2 * F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );
}

#[ test ]
fn test_zoom_max_distance_enforced()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls::default();
  controls.eye = F32x3::new( 1.0, 0.0, 0.0 );
  controls.up = F32x3::new( 0.0, 1.0, 0.0 );
  controls.center = F32x3::new( 0.0, 0.0, 0.0 );
  controls.zoom.max_distance_set( 2.0 );
  controls.zoom.speed = 1.0;

  controls.zoom( 0.6 );

  let exp_eye =  2.0 * F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );
}


#[ test ]
fn test_zoom_invalid_bounds()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls::default();
  controls.eye = F32x3::new( 1.0, 0.0, 0.0 );
  controls.up = F32x3::new( 0.0, 1.0, 0.0 );
  controls.center = F32x3::new( 0.0, 0.0, 0.0 );
  controls.zoom.speed = 1.0;

  controls.zoom.max_distance_set( 0.5 );
  controls.zoom.min_distance_set( 2.0 );

  controls.zoom( -4.0 );

  let exp_eye =  0.5 * F32x3::new( 1.0, 0.0, 0.0 );
  assert_abs_diff_eq!( exp_eye, controls.eye );

  controls.eye = F32x3::new( 1.0, 0.0, 0.0 );
  controls.zoom.min_distance_set( -2.0 );

  controls.zoom( -4.0 );

  let exp_eye =  0.2 * F32x3::new( 1.0, 0.0, 0.0 );
  assert_abs_diff_eq!( exp_eye, controls.eye );
}

#[ test ]
fn test_zoom_with_non_origin_center()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls::default();
  controls.eye = F32x3::new( 5.0, 0.0, 0.0 );
  controls.up = F32x3::new( 0.0, 1.0, 0.0 );
  controls.center = F32x3::new( 4.0, 0.0, 0.0 );
  controls.zoom.speed = 1.0;

  controls.zoom.max_distance_set( 2.0 );
  controls.zoom.min_distance_set( 0.5  );

  controls.zoom( -4.0 );

  let exp_eye =  0.5 * F32x3::new( 1.0, 0.0, 0.0 ) + F32x3::new( 4.0, 0.0, 0.0 );
  assert_abs_diff_eq!( exp_eye, controls.eye );

  controls.eye = F32x3::new( 5.0, 0.0, 0.0 );

  controls.zoom( 0.8 );

  let exp_eye =  2.0 * F32x3::new( 1.0, 0.0, 0.0 ) + F32x3::new( 4.0, 0.0, 0.0 );
  assert_abs_diff_eq!( exp_eye, controls.eye );
}


