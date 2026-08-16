use super::*;
use the_module::SpotLight;
use the_module::shadow::Light;

/// A well-formed `SpotLight` with a given `outer_cone_angle` (radians), otherwise fixed.
fn spot_with_outer_cone_angle( outer_cone_angle : f32 ) -> SpotLight
{
  SpotLight
  {
    position : math::F32x3::from_array( [ 0.0, 5.0, 0.0 ] ),
    direction : math::F32x3::from_array( [ 0.0, -1.0, 0.0 ] ),
    color : math::F32x3::from_array( [ 1.0, 1.0, 1.0 ] ),
    strength : 1.0,
    range : 10.0,
    inner_cone_angle : 0.1,
    outer_cone_angle,
    use_light_map : false,
  }
}

/// ## Root Cause
/// `From< SpotLight > for Light` ( `src/webgl/shadow.rs` ) computed `light_size` as
/// `( ( radius / max_radius ).min( 1.0 ) * 1.7 ).min( 0.01 )` — the trailing `.min( 0.01 )` acts
/// as a *ceiling*, not the intended floor. Since the preceding scaled term is `>= 0.01` for every
/// `outer_cone_angle` above roughly 0.4 degrees, `.min` always picked the constant `0.01`
/// regardless of cone angle — every spot light baked identically soft shadows.
/// ## Why Not Caught
/// `From< SpotLight > for Light` had zero test coverage prior to this bug — nothing exercised
/// `Light::size()` for more than one `outer_cone_angle`, so the constant-output behavior was
/// never compared against a second data point.
/// ## Fix Applied
/// Changed the trailing clamp from `.min( 0.01 )` to `.max( 0.01 )`, turning it back into a
/// lower-bound floor (avoiding a degenerate near-zero size at a near-zero cone angle) while
/// letting the angle-dependent scaling term actually reach the caller.
/// ## Prevention
/// This test constructs two `SpotLight`s differing only in `outer_cone_angle` and asserts the
/// resulting shadow `Light::size()` actually differs — the pre-fix code returned the same `0.01`
/// for both.
/// ## Pitfall
/// A `.min( FLOOR )`/`.max( FLOOR )` mixup still compiles and still returns an in-range value —
/// there is no type error or panic to reveal it. Only a test comparing two different inputs'
/// outputs can catch a clamp direction silently discarding the computation that precedes it.
#[ test ]
fn wide_cone_produces_a_larger_light_size_than_narrow_cone()
{
  let narrow = Light::from( spot_with_outer_cone_angle( 5.0_f32.to_radians() ) );
  let wide = Light::from( spot_with_outer_cone_angle( 80.0_f32.to_radians() ) );

  assert!
  (
    wide.size() > narrow.size(),
    "a wider spot cone must produce a larger (softer) light size than a narrow one — narrow: {}, wide: {}",
    narrow.size(), wide.size()
  );
}

#[ test ]
fn near_zero_cone_angle_floors_at_a_sane_minimum_size()
{
  let light = Light::from( spot_with_outer_cone_angle( 0.001 ) );

  assert!
  (
    ( light.size() - 0.01 ).abs() < 1e-6,
    "a near-zero cone angle must floor at the 0.01 minimum light size, got {}", light.size()
  );
}

#[ test ]
fn wide_cone_light_size_is_well_above_the_floor()
{
  let light = Light::from( spot_with_outer_cone_angle( 80.0_f32.to_radians() ) );

  assert!
  (
    light.size() > 0.1,
    "a wide spot cone's light size must be well above the 0.01 floor — got {}", light.size()
  );
}
