use super::*;
use the_module::Camera;
use mingl::geometry::BoundingBox;

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

/// ## Root Cause
/// `projection_matrix_set` ( `src/webgl/camera.rs` ) assigned its `projection_matrix` argument
/// straight to `self.projection_matrix` with no validation at all — unlike `Camera::new`
/// ( BUG-174 ), which validates its scalar inputs before ever calling `perspective_rh_gl`. Any
/// caller that recomputes a projection matrix itself ( e.g. this crate's own `gltf_viewer`
/// example, on canvas resize ) and passes it to this setter bypassed BUG-174's protections
/// entirely, feeding a singular or Inf/NaN-poisoned matrix straight through to whatever
/// downstream `.inverse()` call ran next.
/// ## Why Not Caught
/// `projection_matrix_set` had zero test coverage of any kind prior to this bug — BUG-174's own
/// tests only ever exercised `Camera::new`, never this setter.
/// ## Fix Applied
/// `projection_matrix_set` now returns `Result< (), gl::WebglError >` and rejects a
/// non-finite-component or non-invertible ( singular ) matrix before assigning it to
/// `self.projection_matrix`.
/// ## Prevention
/// When a constructor's inputs are validated to protect an invariant on one of its fields, every
/// other entry point that can set that same field ( setters included ) must enforce the same
/// invariant independently — validating the constructor alone does not protect the field.
/// ## Pitfall
/// It's easy to validate only the path a bug was actually found through ( here, `Camera::new` )
/// and assume a sibling setter accepting the same field's type is safe by association — it isn't,
/// since the setter never routes through the constructor's own checks.
#[ test ]
fn projection_matrix_set_rejects_a_singular_matrix()
{
  let ( eye, up, look_at, aspect_ratio, fov, near, far ) = valid_args();
  let mut camera = Camera::new( eye, up, look_at, aspect_ratio, fov, near, far ).unwrap();

  let singular = math::F32x4x4::_fill( 0.0 );
  let result = camera.projection_matrix_set( singular );

  assert!( result.is_err(), "an all-zero ( singular, non-invertible ) projection matrix must be rejected" );
}

#[ test ]
fn projection_matrix_set_rejects_non_finite_components()
{
  let ( eye, up, look_at, aspect_ratio, fov, near, far ) = valid_args();
  let mut camera = Camera::new( eye, up, look_at, aspect_ratio, fov, near, far ).unwrap();

  let poisoned = math::F32x4x4::_fill( f32::NAN );
  let result = camera.projection_matrix_set( poisoned );

  assert!( result.is_err(), "a projection matrix with NaN components must be rejected" );
}

#[ test ]
fn projection_matrix_set_accepts_a_valid_matrix()
{
  let ( eye, up, look_at, aspect_ratio, fov, near, far ) = valid_args();
  let mut camera = Camera::new( eye, up, look_at, aspect_ratio, fov, near, far ).unwrap();
  let valid = camera.projection_matrix_get();

  let result = camera.projection_matrix_set( valid );

  assert!( result.is_ok(), "a well-formed, invertible projection matrix must be accepted — got {:?}", result.err() );
  // A bit-exact store-then-load round-trip, not a computed-value comparison prone to rounding --
  // `clippy::float_cmp` guards against the latter, which doesn't apply here.
  #[ expect( clippy::float_cmp, reason = "round-trip identity check: the setter must store the exact bits it was given, not an approximation" ) ]
  let round_trips = camera.projection_matrix_get().to_array() == valid.to_array();
  assert!( round_trips, "the setter must actually store the accepted matrix" );
}

#[ test ]
fn from_bounding_box_computes_eye_at_the_expected_distance_along_direction()
{
  let bounding_box = BoundingBox::new( [ -1.0, -1.0, -1.0 ], [ 1.0, 1.0, 1.0 ] );
  let direction = math::F32x3::from_array( [ 0.0, 0.0, 1.0 ] );
  let up = math::F32x3::from_array( [ 0.0, 1.0, 0.0 ] );

  let camera = Camera::from_bounding_box( &bounding_box, direction, up, 1.0, 90.0f32.to_radians(), 0.01 )
  .expect( "well-formed bounding box and camera parameters must construct successfully" );

  // radius = half-diagonal of a [-1,1]^3 box = sqrt(3); at fov=90deg/aspect=1 the limiting
  // half-angle is 45deg on both axes, so distance = radius / sin(45deg) = sqrt(6).
  let expected_distance = 6.0f32.sqrt();
  let eye = camera.eye_get();

  assert!
  (
    ( eye.z() - expected_distance ).abs() < 0.001,
    "expected eye.z() close to {expected_distance} ( sqrt(6) ), got {}", eye.z()
  );
  assert!
  (
    eye.x().abs() < 0.0001 && eye.y().abs() < 0.0001,
    "eye must stay on the z-axis for a box centered on the origin viewed along +z — got {eye:?}"
  );
}

#[ test ]
fn from_bounding_box_looks_at_the_box_center()
{
  let bounding_box = BoundingBox::new( [ 2.0, 4.0, 6.0 ], [ 4.0, 8.0, 10.0 ] );
  let direction = math::F32x3::from_array( [ 1.0, 1.0, 1.0 ] );
  let up = math::F32x3::from_array( [ 0.0, 1.0, 0.0 ] );

  let camera = Camera::from_bounding_box( &bounding_box, direction, up, 16.0 / 9.0, 70.0f32.to_radians(), 0.1 )
  .expect( "well-formed bounding box and camera parameters must construct successfully" );

  let center = camera.controls_get().borrow().center;
  let expected = bounding_box.center();

  assert!
  (
    ( center - expected ).mag() < 0.0001,
    "camera must look at the bounding box's own center — expected {expected:?}, got {center:?}"
  );
}

/// Transforms every corner of an off-center, non-axis-aligned bounding box through the
/// resulting camera's view and projection matrices and confirms each one lands within the
/// [-1, 1] NDC range on both axes — the actual "does the box fit in frustum" contract, not
/// just a re-check of the function's own internal arithmetic.
#[ test ]
fn from_bounding_box_frames_every_corner_within_the_view_frustum()
{
  let bounding_box = BoundingBox::new( [ -3.0, 0.0, 5.0 ], [ 7.0, 4.0, 9.0 ] );
  let direction = math::F32x3::from_array( [ -1.0, 2.0, 0.5 ] );
  let up = math::F32x3::from_array( [ 0.0, 1.0, 0.0 ] );

  let camera = Camera::from_bounding_box( &bounding_box, direction, up, 16.0 / 9.0, 70.0f32.to_radians(), 0.1 )
  .expect( "well-formed bounding box and camera parameters must construct successfully" );

  let view = camera.view_matrix_get();
  let projection = camera.projection_matrix_get();

  let ( min, max ) = ( bounding_box.min, bounding_box.max );
  let corners =
  [
    math::F32x3::from_array( [ min.x(), min.y(), min.z() ] ),
    math::F32x3::from_array( [ max.x(), min.y(), min.z() ] ),
    math::F32x3::from_array( [ min.x(), max.y(), min.z() ] ),
    math::F32x3::from_array( [ max.x(), max.y(), min.z() ] ),
    math::F32x3::from_array( [ min.x(), min.y(), max.z() ] ),
    math::F32x3::from_array( [ max.x(), min.y(), max.z() ] ),
    math::F32x3::from_array( [ min.x(), max.y(), max.z() ] ),
    math::F32x3::from_array( [ max.x(), max.y(), max.z() ] ),
  ];

  for corner in corners
  {
    let view_space = view * corner.to_homogenous();
    let clip = projection * view_space;
    let ndc_x = clip.x() / clip.w();
    let ndc_y = clip.y() / clip.w();

    assert!
    (
      ndc_x.abs() <= 1.0001 && ndc_y.abs() <= 1.0001,
      "every bounding box corner must land within the view frustum on x/y -- corner {corner:?} projected to ndc ({ndc_x}, {ndc_y})"
    );
  }
}

#[ test ]
fn from_bounding_box_accepts_a_degenerate_zero_radius_box()
{
  let bounding_box = BoundingBox::new( [ 5.0, 5.0, 5.0 ], [ 5.0, 5.0, 5.0 ] );
  let direction = math::F32x3::from_array( [ 0.0, 0.0, 1.0 ] );
  let up = math::F32x3::from_array( [ 0.0, 1.0, 0.0 ] );

  let result = Camera::from_bounding_box( &bounding_box, direction, up, 1.0, 70.0f32.to_radians(), 0.5 );

  assert!( result.is_ok(), "a zero-radius ( single-point ) box must not divide-by-zero or otherwise fail construction — got {:?}", result.err() );
}

#[ test ]
fn from_bounding_box_rejects_invalid_aspect_ratio()
{
  let bounding_box = BoundingBox::new( [ -1.0, -1.0, -1.0 ], [ 1.0, 1.0, 1.0 ] );
  let direction = math::F32x3::from_array( [ 0.0, 0.0, 1.0 ] );
  let up = math::F32x3::from_array( [ 0.0, 1.0, 0.0 ] );

  let result = Camera::from_bounding_box( &bounding_box, direction, up, 0.0, 70.0f32.to_radians(), 0.1 );

  assert!( result.is_err(), "from_bounding_box must propagate Camera::new's own aspect_ratio validation" );
}
