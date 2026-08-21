use super::*;
use the_module::{ F32x3 };
use approx::assert_abs_diff_eq;

#[ test ]
fn test_rotation_disabled_prevents_rotation()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 1.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };
  controls.rotation.enabled = false;

  controls.rotate( [ 50.0, 50.0 ] );

  let exp_eye = F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );
}

#[ test ]
fn test_zoom_disabled_prevents_zoom()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 1.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };
  controls.zoom.enabled = false;

  controls.zoom( 50.0 );

  let exp_eye = F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );
}

#[ test ]
fn test_pan_disabled_prevents_pan()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 1.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };
  controls.pan.enabled = false;

  controls.pan( [ 50.0, 50.0 ] );

  let exp_eye = F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );
}

#[ test ]
fn test_rotation_longitude()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 1.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };
  controls.rotation.speed = 1.0;

  controls.rotate( [ core::f32::consts::FRAC_PI_2, 0.0 ] );

  let rot_mat = the_module::math::mat3x3::from_angle_y( core::f32::consts::FRAC_PI_2 );

  let exp_eye = rot_mat * F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );

  controls.rotate( [ -core::f32::consts::FRAC_PI_2, 0.0 ] );

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
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 2.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 1.0, 0.0, 0.0 ), ..Default::default() };
  controls.rotation.speed = 1.0;

  controls.rotate( [ core::f32::consts::FRAC_PI_2, 0.0 ] );

  let rot_mat = the_module::math::mat3x3::from_angle_y( core::f32::consts::FRAC_PI_2 );

  let exp_eye = rot_mat * F32x3::new( 1.0, 0.0, 0.0 ) + F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 1.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );

  controls.rotate( [ -core::f32::consts::FRAC_PI_2, 0.0 ] );

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
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 1.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };
  controls.rotation.speed = 1.0;

  controls.rotate( [ 0.0, core::f32::consts::FRAC_PI_4 ] );

  let rot_mat = the_module::math::mat3x3::from_angle_z( core::f32::consts::FRAC_PI_4 );

  let exp_eye = rot_mat * F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = rot_mat * F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );

  controls.rotate( [ 0.0, -core::f32::consts::FRAC_PI_4 ] );

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
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 2.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 1.0, 0.0, 0.0 ), ..Default::default() };
  controls.rotation.speed = 1.0;

  controls.rotate( [ 0.0, core::f32::consts::FRAC_PI_4 ] );

  let rot_mat = the_module::math::mat3x3::from_angle_z( core::f32::consts::FRAC_PI_4 );

  let exp_eye = rot_mat * F32x3::new( 1.0, 0.0, 0.0 ) +  F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = rot_mat * F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 1.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );

  controls.rotate( [ 0.0, -core::f32::consts::FRAC_PI_4 ] );

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
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 1.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };
  controls.rotation.longitude_range_set( 90.0 );
  controls.rotation.speed = 1.0;

  // Counter-clockwise
  controls.rotate( [ core::f32::consts::PI, 0.0 ] );

  let rot_mat = the_module::math::mat3x3::from_angle_y( 90.0f32.to_radians() );

  let exp_eye = rot_mat * F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );

  // Clockwise
  controls.eye = F32x3::new( 1.0, 0.0, 0.0 );
  controls.up = F32x3::new( 0.0, 1.0, 0.0 );
  controls.center = F32x3::new( 0.0, 0.0, 0.0 );

  controls.rotate( [ -core::f32::consts::PI, 0.0 ] );

  let rot_mat = the_module::math::mat3x3::from_angle_y( -90.0f32.to_radians() );

  let exp_eye = rot_mat * F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );
}

#[ test ]
fn test_latitude_range_clamps_correctly()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 1.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };
  controls.rotation.latitude_range_set( 45.0 );
  controls.rotation.speed = 1.0;

  // Counter-clockwise
  controls.rotate( [ 0.0, core::f32::consts::PI * 0.5 ] );

  let rot_mat = the_module::math::mat3x3::from_angle_z( 45.0f32.to_radians() );

  let exp_eye = rot_mat * F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = rot_mat * F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );

  // Clockwise
  controls.eye = F32x3::new( 1.0, 0.0, 0.0 );
  controls.up = F32x3::new( 0.0, 1.0, 0.0 );
  controls.center = F32x3::new( 0.0, 0.0, 0.0 );

  controls.rotate( [ 0.0, -core::f32::consts::PI * 0.5 ] );

  let rot_mat = the_module::math::mat3x3::from_angle_z( -45.0f32.to_radians() );

  let exp_eye = rot_mat * F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = rot_mat * F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );
}

#[ test ]
fn test_zoom_min_distance_enforced()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 1.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };
  controls.zoom.min_distance_set( 0.2 );
  controls.zoom.speed = 1.0;

  controls.zoom( -9.0 );

  let exp_eye = 0.2 * F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );
}

#[ test ]
fn test_zoom_max_distance_enforced()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 1.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };
  controls.zoom.max_distance_set( 2.0 );
  controls.zoom.speed = 1.0;

  controls.zoom( 0.6 );

  let exp_eye = 2.0 * F32x3::new( 1.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_up, controls.up );
  assert_abs_diff_eq!( exp_center, controls.center );
}


#[ test ]
fn test_zoom_invalid_bounds()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 1.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };
  controls.zoom.speed = 1.0;

  controls.zoom.max_distance_set( 0.5 );
  controls.zoom.min_distance_set( 2.0 );

  controls.zoom( -4.0 );

  let exp_eye = 0.5 * F32x3::new( 1.0, 0.0, 0.0 );
  assert_abs_diff_eq!( exp_eye, controls.eye );

  controls.eye = F32x3::new( 1.0, 0.0, 0.0 );
  controls.zoom.min_distance_set( -2.0 );

  controls.zoom( -4.0 );

  let exp_eye = 0.2 * F32x3::new( 1.0, 0.0, 0.0 );
  assert_abs_diff_eq!( exp_eye, controls.eye );
}

#[ test ]
fn test_zoom_with_non_origin_center()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 5.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 4.0, 0.0, 0.0 ), ..Default::default() };
  controls.zoom.speed = 1.0;

  controls.zoom.max_distance_set( 2.0 );
  controls.zoom.min_distance_set( 0.5 );

  controls.zoom( -4.0 );

  let exp_eye = 0.5 * F32x3::new( 1.0, 0.0, 0.0 ) + F32x3::new( 4.0, 0.0, 0.0 );
  assert_abs_diff_eq!( exp_eye, controls.eye );

  controls.eye = F32x3::new( 5.0, 0.0, 0.0 );

  controls.zoom( 0.8 );

  let exp_eye = 2.0 * F32x3::new( 1.0, 0.0, 0.0 ) + F32x3::new( 4.0, 0.0, 0.0 );
  assert_abs_diff_eq!( exp_eye, controls.eye );
}

// -- pan tests --

#[ test ]
fn test_pan_preserves_eye_to_center_vector()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 1.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };

  let eye_to_center = controls.center - controls.eye;

  controls.pan( [ 100.0, 75.0 ] );

  assert_abs_diff_eq!( eye_to_center, controls.center - controls.eye );
}

#[ test ]
fn test_pan_zero_delta_no_change()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 1.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };

  controls.pan( [ 0.0, 0.0 ] );

  let exp_eye = F32x3::new( 1.0, 0.0, 0.0 );
  let exp_center = F32x3::new( 0.0, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_center, controls.center );
  assert_abs_diff_eq!( exp_up, controls.up );
}

#[ test ]
fn test_pan_does_not_change_up_vector()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 1.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };

  controls.pan( [ 100.0, 75.0 ] );

  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );
  assert_abs_diff_eq!( exp_up, controls.up );
}

// test_kind: bug_reproducer(BUG-004)
/// ## Root Cause
/// Mouse delta used inconsistent sign convention: `prev - new` for X, `new - prev`
/// for Y. The `pan()` method expects standard `new - prev` (positive = right/down).
///
/// ## Why Not Caught
/// No pan tests existed. Only rotation/zoom were tested.
///
/// ## Fix Applied
/// Standardized delta to `new - prev` for both axes. Moved X negation to
/// rotation-only code path.
///
/// ## Prevention
/// Comprehensive pan tests now cover all movement directions and orientations.
///
/// ## Pitfall
/// Screen-space deltas must match method coordinate expectations. Document
/// expected delta sign convention in method docs.
#[ test ]
fn test_pan_horizontal()
{
  // Camera looking from +X toward origin.
  // dir_norm = [-1,0,0], camera right: dir_norm.cross(up) = [0,0,-1]
  // Horizontal pan moves opposite to right: offset = -[0,0,-1]*dx*pan_scale = [0,0,dx]*pan_scale
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 1.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };

  let dx = 100.0_f32;
  let dist = ( controls.center - controls.eye ).mag();
  let pan_scale = 2.0 * ( controls.fov / 2.0 ).tan() * dist / controls.window_size.y();

  controls.pan( [ dx, 0.0 ] );

  let exp_eye = F32x3::new( 1.0, 0.0, dx * pan_scale );
  let exp_center = F32x3::new( 0.0, 0.0, dx * pan_scale );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_center, controls.center );
  assert_abs_diff_eq!( exp_up, controls.up );
}

#[ test ]
fn test_pan_vertical()
{
  // Camera looking from +X toward origin.
  // View-plane up vector y = [0,1,0].
  // Vertical pan: offset = y*dy*pan_scale = [0,dy,0]*pan_scale
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 1.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };

  let dy = 100.0_f32;
  let dist = ( controls.center - controls.eye ).mag();
  let pan_scale = 2.0 * ( controls.fov / 2.0 ).tan() * dist / controls.window_size.y();

  controls.pan( [ 0.0, dy ] );

  let exp_eye = F32x3::new( 1.0, dy * pan_scale, 0.0 );
  let exp_center = F32x3::new( 0.0, dy * pan_scale, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_center, controls.center );
  assert_abs_diff_eq!( exp_up, controls.up );
}

#[ test ]
fn test_pan_with_non_origin_center()
{
  // Identical relative geometry to the origin-center case; the whole
  // camera frame is simply translated by (1,0,0).
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 2.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 1.0, 0.0, 0.0 ), ..Default::default() };

  let dy = 100.0_f32;
  let dist = ( controls.center - controls.eye ).mag();
  let pan_scale = 2.0 * ( controls.fov / 2.0 ).tan() * dist / controls.window_size.y();

  controls.pan( [ 0.0, dy ] );

  let exp_eye = F32x3::new( 2.0, dy * pan_scale, 0.0 );
  let exp_center = F32x3::new( 1.0, dy * pan_scale, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_center, controls.center );
  assert_abs_diff_eq!( exp_up, controls.up );
}

#[ test ]
fn test_pan_scales_with_distance()
{
  // The world-space offset grows linearly with camera distance,
  // so panning from twice the distance doubles the movement.
  let mut controls_near = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 1.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };
  controls_near.pan( [ 0.0, 100.0 ] );
  let offset_near = controls_near.eye.y();

  let mut controls_far = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 2.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };
  controls_far.pan( [ 0.0, 100.0 ] );
  let offset_far = controls_far.eye.y();

  assert_abs_diff_eq!( 2.0 * offset_near, offset_far );
}

#[ test ]
fn test_pan_horizontal_direction_depends_on_camera_orientation()
{
  // Camera looking from +Z: right vector is +X, so horizontal pan moves in -X.
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 0.0, 0.0, 1.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };

  let dx = 100.0_f32;
  let dist = ( controls.center - controls.eye ).mag();
  let pan_scale = 2.0 * ( controls.fov / 2.0 ).tan() * dist / controls.window_size.y();

  controls.pan( [ dx, 0.0 ] );

  // dir_norm=[0,0,-1], right x=[1,0,0], offset = -x*dx*pan_scale = [-dx*pan_scale, 0, 0]
  let exp_eye = F32x3::new( -dx * pan_scale, 0.0, 1.0 );
  let exp_center = F32x3::new( -dx * pan_scale, 0.0, 0.0 );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );

  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_center, controls.center );
  assert_abs_diff_eq!( exp_up, controls.up );
}

#[ test ]
fn test_pan_negative_horizontal_is_opposite()
{
  // Same camera as `test_pan_horizontal`, but with a negative dx (pan left).
  // A negative delta must move the camera the exact opposite way: where +dx
  // gave eye/center.z = +dx*pan_scale, -dx must give -dx*pan_scale. Guards the delta sign
  // convention for negative inputs, which the positive-only tests miss.
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 1.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };

  let dx = -100.0_f32;
  let dist = ( controls.center - controls.eye ).mag();
  let pan_scale = 2.0 * ( controls.fov / 2.0 ).tan() * dist / controls.window_size.y();

  controls.pan( [ dx, 0.0 ] );

  // dx is negative, so dx*pan_scale is negative — displacement opposite to the +dx case.
  let exp_eye = F32x3::new( 1.0, 0.0, dx * pan_scale );
  let exp_center = F32x3::new( 0.0, 0.0, dx * pan_scale );
  let exp_up = F32x3::new( 0.0, 1.0, 0.0 );

  assert!( controls.eye.z() < 0.0, "negative dx must move camera to -z, got {}", controls.eye.z() );
  assert_abs_diff_eq!( exp_eye, controls.eye );
  assert_abs_diff_eq!( exp_center, controls.center );
  assert_abs_diff_eq!( exp_up, controls.up );
}

// test_kind: bug_reproducer(BUG-125)
/// ## Root Cause
/// `CameraOrbitControls::update` treated its `delta_time` parameter as milliseconds inside the
/// `/10.0` and `/1000.0` smoothing-decay formulas, but the parameter's own doc contract (and
/// every real caller, traced to the `t / 1000.0` rAF-timestamp conversion in
/// `examples/minwebgl/skeletal_animation`) supplies seconds — scaling both `decay_percentage`
/// and `current_rotation_angle` down by exactly 1000x.
/// ## Why Not Caught
/// No existing test exercised `update()` with `movement_smoothing_enabled = true` at all — the
/// buggy branch is dead code under the type's own `Default` (`movement_smoothing_enabled:
/// false`), so a plain `cargo test` run never touches it.
/// ## Fix Applied
/// `update` now converts `delta_time` to milliseconds once (`delta_time_ms = delta_time *
/// 1000.0`) before applying the existing `/10.0` and `/1000.0` formulas, unchanged.
/// ## Prevention
/// This test drives `rotate()` then `update()` with smoothing enabled and a realistic 60fps
/// `delta_time` (seconds), and asserts the camera's swept rotation angle matches the
/// milliseconds-correct formula rather than the 1000x-too-small buggy one.
/// ## Pitfall
/// A doc comment naming a time unit ("every 10 milliseconds") is not proof the formula beneath
/// it actually receives that unit — verify the two independently.
#[ test ]
fn test_update_applies_smoothed_rotation_at_correct_time_scale()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 1.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };
  controls.rotation.movement_smoothing_enabled = true;
  controls.rotation.speed = 1.0;

  // Accumulate angular speed without applying it yet (smoothing defers application to update()).
  controls.rotate( [ 1.0, 0.0 ] );
  let eye_before = controls.eye;

  // A realistic 60fps frame delta, in seconds, per this function's own documented contract.
  controls.update( 0.016 );

  // Pure-longitude rotation (screen_d.y == 0) preserves eye.y() and rotates only x/z — measure
  // the swept angle via atan2 so the assertion doesn't depend on from_angle_y's sign convention.
  let angle_before = eye_before.z().atan2( eye_before.x() );
  let angle_after = controls.eye.z().atan2( controls.eye.x() );
  let swept = ( angle_after - angle_before ).abs();

  // Fixed formula: current_angular_speed(1.0) * delta_time_ms(16.0) / 1000.0 == 0.016 rad.
  // The buggy formula would have produced 0.016 / 1000.0 == 0.000016 rad instead — 1000x smaller.
  assert_abs_diff_eq!( 0.016_f32, swept, epsilon = 1e-4 );
}

// test_kind: bug_reproducer(BUG-126)
/// ## Root Cause
/// `CameraOrbitControls::zoom`'s zoom-out branch computed `k = 1.0 - delta_y.abs()` with no
/// lower bound — a single event whose `|delta_y|` (post `/speed`) reaches 1.0 drives `k` to
/// exactly 0.0 (division by zero, `eye_new` becomes non-finite), and beyond 1.0 drives `k`
/// negative (dividing by a negative number flips the camera through the `center` pivot to the
/// opposite side, the geometric opposite of "zoom out").
/// ## Why Not Caught
/// Every existing zoom-out test kept `|delta_y| < speed` (max tested: `delta_y=0.8` against
/// `speed=1.0`), never reaching the `k <= 0` boundary; `zoom.max_distance`/`min_distance` both
/// default to `None`, so no downstream clamp masks the corruption in default configuration.
/// Reachable via real input: a fast pinch gesture's raw `screen_x`/`screen_y` pixel-distance
/// delta, or a high-precision mouse wheel's `DOM_DELTA_PIXEL` event, both plausibly reach the
/// default `zoom.speed` of 1000.0 in a single event.
/// ## Fix Applied
/// The zoom-out branch's divisor is now `( 1.0 - delta_y.abs() ).max( f32::EPSILON )`, matching
/// the zoom-in branch's already-safe-by-construction `1.0 + delta_y.abs()` in spirit — a floor
/// that only changes behavior in the previously-broken `|delta_y| >= 1.0` region, leaving every
/// already-correct case (including all pre-existing passing tests) bit-for-bit unchanged.
/// ## Prevention
/// This test drives `delta_y` to exactly the `k == 0.0` boundary and past it into `k < 0.0`,
/// asserting the resulting eye position stays finite and on the original side of the pivot.
/// ## Pitfall
/// A divisor derived as `1.0 - x.abs()` is only safe while `x` is known to stay inside the unit
/// interval — an external, unbounded input (screen pixels, wheel events) can never be assumed to
/// satisfy that on its own; clamp at the boundary, don't trust the caller.
#[ test ]
fn test_zoom_out_extreme_delta_does_not_corrupt_or_flip_eye()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 1.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };
  controls.zoom.speed = 1.0;

  // delta_y == 1.0 drives the raw divisor `1.0 - delta_y.abs()` to exactly 0.0 (division by zero).
  controls.zoom( 1.0 );
  assert!( controls.eye.x().is_finite(), "zoom-out to the divisor's zero point must not produce a non-finite eye position, got {}", controls.eye.x() );
  assert!( controls.eye.x() > 0.0, "zoom-out must move the camera further along its original direction, not flip through the pivot, got {}", controls.eye.x() );

  controls.eye = F32x3::new( 1.0, 0.0, 0.0 );

  // delta_y == 1.5 drives the raw divisor negative (-0.5), flipping eye_new's sign.
  controls.zoom( 1.5 );
  assert!( controls.eye.x().is_finite(), "zoom-out past the divisor's zero point must not produce a non-finite eye position, got {}", controls.eye.x() );
  assert!( controls.eye.x() > 0.0, "zoom-out must never flip the camera through the pivot to the opposite side, got {}", controls.eye.x() );
}

// test_kind: bug_reproducer(BUG-427)
/// ## Root Cause
/// `update()`'s smoothed-rotation branch applied `rotation_apply()` ( and decayed
/// `current_angular_speed` ) purely on `self.rotation.movement_smoothing_enabled`, with no
/// check that `self.rotation.enabled` was also true -- unlike `rotate()`, which returns
/// immediately when `!self.rotation.enabled`. A caller that accumulated angular speed via
/// smoothing while rotation was enabled, then disabled rotation, would still see `update()`
/// keep applying the stale accumulated speed ( and keep decaying it ) on every subsequent
/// call, instead of the camera simply stopping.
/// ## Why Not Caught
/// Every existing `update()` test ( e.g. `test_update_applies_smoothed_rotation_at_correct_time_scale`,
/// above ) leaves `rotation.enabled` at its default `true` for the whole test, so none of them
/// exercise `update()` after disabling rotation mid-sequence; the existing disabled-rotation
/// coverage ( `test_rotation_disabled_prevents_rotation`, top of this file ) only calls
/// `rotate()`, never `update()`.
/// ## Fix Applied
/// Guarded `update()`'s smoothing branch with
/// `self.rotation.enabled && self.rotation.movement_smoothing_enabled`, mirroring `rotate()`'s
/// own early-return guard on the same `enabled` flag.
/// ## Prevention
/// RED state (empirically confirmed): reverting the guard back to
/// `if self.rotation.movement_smoothing_enabled` alone ( dropping the `self.rotation.enabled
/// &&` ) and re-running this test genuinely fails the eye/up "unchanged" assertions below --
/// verified via a temporary probe before this fix was finalized.
/// ## Pitfall
/// When smoothing state is shared between two entry points ( `rotate()` accumulates it,
/// `update()` applies it ), an `enabled` guard added to one is not automatically in effect on
/// the other -- grep every reader of the shared state for the same guard, not just the one
/// under review when the guard was first added.
#[ test ]
fn test_update_rotation_disabled_prevents_smoothed_rotation()
{
  let mut controls = the_module::controls::camera_orbit_controls::CameraOrbitControls { eye: F32x3::new( 1.0, 0.0, 0.0 ), up: F32x3::new( 0.0, 1.0, 0.0 ), center: F32x3::new( 0.0, 0.0, 0.0 ), ..Default::default() };
  controls.rotation.movement_smoothing_enabled = true;
  controls.rotation.speed = 1.0;

  // Accumulate angular speed while rotation is still enabled -- smoothing defers actually
  // applying it to update().
  controls.rotate( [ 1.0, 0.0 ] );

  // Disable rotation *after* speed has already accumulated -- update() must now no-op
  // entirely, not merely skip future accumulation.
  controls.rotation.enabled = false;

  let eye_before = controls.eye;
  let up_before = controls.up;
  let center_before = controls.center;

  controls.update( 0.016 );

  assert_abs_diff_eq!( eye_before, controls.eye );
  assert_abs_diff_eq!( up_before, controls.up );
  assert_abs_diff_eq!( center_before, controls.center );
}
