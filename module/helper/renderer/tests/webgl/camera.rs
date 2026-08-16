use super::*;
use the_module::Camera;

/// A well-formed parameter set — mirrors this crate's own readme.md Quick Start example.
fn valid_args() -> ( math::F32x3, math::F32x3, math::F32x3, f32, f32, f32, f32 )
{
  (
    math::F32x3::from_array( [ 0.0, 1.0, 3.0 ] ),
    math::F32x3::from_array( [ 0.0, 1.0, 0.0 ] ),
    math::F32x3::from_array( [ 0.0, 0.0, 0.0 ] ),
    16.0 / 9.0,
    70.0f32.to_radians(),
    0.1,
    1000.0
  )
}

#[ test ]
fn accepts_valid_parameters_and_produces_a_finite_matrix()
{
  let ( eye, up, look_at, aspect_ratio, fov, near, far ) = valid_args();
  let result = Camera::new( eye, up, look_at, aspect_ratio, fov, near, far );

  assert!( result.is_ok(), "well-formed camera parameters must construct successfully — got {:?}", result.err() );

  let m = result.unwrap().projection_matrix_get().to_array();
  assert!( m.iter().all( | v | v.is_finite() ), "a valid camera's projection matrix must be fully finite — got {m:?}" );
}

/// ## Root Cause
/// `Camera::new` ( `src/webgl/camera.rs` ) fed `aspect_ratio`/`fov`/`near`/`far` straight into
/// `perspective_rh_gl` with no validation. For `near == 0.0` ( or `far == 0.0` ), the resulting
/// matrix's determinant works out to exactly `0.0` — construction itself never panicked, but the
/// very next `.inverse()` call ( `Renderer::skybox_draw`, `src/webgl/renderer.rs:641` ) returns
/// `None` per its own documented "if the determinant is zero" contract, and the `.unwrap()`
/// immediately after it panics — several call frames away from the actual root cause.
/// ## Why Not Caught
/// `Camera::new` had zero test coverage of any kind prior to this bug — no existing test called
/// it at all, degenerate or otherwise.
/// ## Fix Applied
/// `Camera::new` now returns `Result< Self, gl::WebglError >` and rejects a non-finite or
/// `<= 0.0` `near` ( along with the sibling `aspect_ratio`/`fov`/`far` guards below ) before
/// calling `perspective_rh_gl`, failing loudly and immediately at the actual construction site.
/// ## Prevention
/// Any constructor feeding caller-supplied scalars into a division/tangent-based formula must
/// validate the formula's mathematical domain before calling it, not trust the caller.
/// ## Pitfall
/// `near`/`far` are typically small literal constants at call sites, so this class of bug is
/// easy to dismiss as "can't happen" — but nothing stopped a caller from swapping the two
/// arguments or passing a runtime-computed `0.0`, and the resulting panic message
/// ( `Option::unwrap()` on `None` ) points at an unrelated skybox draw call, not at `Camera::new`.
#[ test ]
fn rejects_zero_near()
{
  let ( eye, up, look_at, aspect_ratio, fov, _, far ) = valid_args();
  let result = Camera::new( eye, up, look_at, aspect_ratio, fov, 0.0, far );

  assert!( result.is_err(), "near == 0.0 must be rejected — it used to produce a zero-determinant matrix that panicked downstream on .inverse().unwrap()" );
}

#[ test ]
fn rejects_zero_far()
{
  let ( eye, up, look_at, aspect_ratio, fov, near, _ ) = valid_args();
  let result = Camera::new( eye, up, look_at, aspect_ratio, fov, near, 0.0 );

  assert!( result.is_err(), "far == 0.0 must be rejected — it used to produce a zero-determinant matrix that panicked downstream on .inverse().unwrap()" );
}

#[ test ]
fn rejects_zero_aspect_ratio()
{
  let ( eye, up, look_at, _, fov, near, far ) = valid_args();
  let result = Camera::new( eye, up, look_at, 0.0, fov, near, far );

  assert!( result.is_err(), "aspect_ratio == 0.0 must be rejected, not silently baked into an Inf-poisoned matrix" );
}

#[ test ]
fn rejects_negative_aspect_ratio()
{
  let ( eye, up, look_at, _, fov, near, far ) = valid_args();
  let result = Camera::new( eye, up, look_at, -1.0, fov, near, far );

  assert!( result.is_err(), "negative aspect_ratio must be rejected" );
}

#[ test ]
fn rejects_near_equal_far()
{
  let ( eye, up, look_at, aspect_ratio, fov, near, _ ) = valid_args();
  let result = Camera::new( eye, up, look_at, aspect_ratio, fov, near, near );

  assert!( result.is_err(), "near == far must be rejected — it collapses the depth range to Inf" );
}

#[ test ]
fn rejects_near_greater_than_far()
{
  let ( eye, up, look_at, aspect_ratio, fov, near, far ) = valid_args();
  let result = Camera::new( eye, up, look_at, aspect_ratio, fov, far, near );

  assert!( result.is_err(), "near > far must be rejected — it silently swaps the depth mapping instead of erroring" );
}

#[ test ]
fn rejects_zero_fov()
{
  let ( eye, up, look_at, aspect_ratio, _, near, far ) = valid_args();
  let result = Camera::new( eye, up, look_at, aspect_ratio, 0.0, near, far );

  assert!( result.is_err(), "fov == 0.0 must be rejected — tan(0/2) is 0, making f = 1/0 = Inf" );
}

#[ test ]
fn rejects_fov_at_or_beyond_pi()
{
  let ( eye, up, look_at, aspect_ratio, _, near, far ) = valid_args();
  let result = Camera::new( eye, up, look_at, aspect_ratio, std::f32::consts::PI, near, far );

  assert!( result.is_err(), "fov >= PI must be rejected — tan(PI/2) is undefined ( +-Inf )" );
}

#[ test ]
fn rejects_non_finite_parameters()
{
  let ( eye, up, look_at, aspect_ratio, fov, near, far ) = valid_args();

  assert!( Camera::new( eye, up, look_at, f32::NAN, fov, near, far ).is_err(), "NaN aspect_ratio must be rejected" );
  assert!( Camera::new( eye, up, look_at, aspect_ratio, f32::INFINITY, near, far ).is_err(), "infinite fov must be rejected" );
  assert!( Camera::new( eye, up, look_at, aspect_ratio, fov, f32::NAN, far ).is_err(), "NaN near must be rejected" );
  assert!( Camera::new( eye, up, look_at, aspect_ratio, fov, near, f32::NAN ).is_err(), "NaN far must be rejected" );
}
