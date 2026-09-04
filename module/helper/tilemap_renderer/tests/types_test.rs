

//! Types tests.
//!
//! Covers:
//! - `Transform` identity state, translation matrix slots, scale diagonal, 90-degree rotation
//! - `ResourceId` type-safe equality and debug formatting
//! - `RenderConfig` default field values (width, height, antialias, background color, `max_depth`)

use tilemap_renderer::types::*;

/// Verifies that two `ResourceId<Image>` values with the same inner id compare
/// equal, and that a different id compares unequal — type-safe id equality.
#[ test ]
fn resource_id_type_safety()
{
  let id_a : ResourceId< asset::Image > = ResourceId::new( 5 );
  let id_b : ResourceId< asset::Image > = ResourceId::new( 5 );
  let id_c : ResourceId< asset::Image > = ResourceId::new( 7 );
  assert_eq!( id_a, id_b );
  assert_ne!( id_a, id_c );
  assert_eq!( id_a.inner(), 5 );
}

/// Verifies that `ResourceId` formats as `"ResourceId(N)"` so that
/// debug output is readable in test failure messages.
#[ test ]
fn resource_id_debug()
{
  let id : ResourceId< asset::Sprite > = ResourceId::new( 42 );
  assert_eq!( format!( "{id:?}" ), "ResourceId(42)" );
}

/// Verifies that `Transform::default()` is the identity transform:
/// zero position, zero rotation, unit scale, zero skew, zero depth.
#[ test ]
fn transform_default_is_identity()
{
  let transform = Transform::default();
  assert!( transform.position[ 0 ].abs() < 1e-6 );
  assert!( transform.position[ 1 ].abs() < 1e-6 );
  assert!( transform.rotation.abs() < 1e-6 );
  assert!( ( transform.scale[ 0 ] - 1.0 ).abs() < 1e-6 );
  assert!( ( transform.scale[ 1 ] - 1.0 ).abs() < 1e-6 );
  assert!( transform.skew[ 0 ].abs() < 1e-6 );
  assert!( transform.skew[ 1 ].abs() < 1e-6 );
  assert!( transform.depth.abs() < 1e-6 );
}

/// Verifies that the identity `Transform` produces a 3×3 identity matrix
/// from `to_mat3()`, element-by-element within float tolerance.
#[ test ]
fn to_mat3_identity()
{
  let transform = Transform::default();
  let mat = transform.to_mat3();
  let expected =
  [
    1.0, 0.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 0.0, 1.0,
  ];
  for ( actual, expected_val ) in mat.iter().zip( expected.iter() )
  {
    assert!( ( actual - expected_val ).abs() < 1e-6, "expected {expected_val} got {actual}" );
  }
}

/// Verifies that a translation-only `Transform` places the translation
/// vector in the correct matrix slots (column-major: indices 6 and 7).
#[ test ]
fn to_mat3_translation()
{
  let transform = Transform { position : [ 10.0, 20.0 ], ..Default::default() };
  let mat = transform.to_mat3();
  assert!( ( mat[ 6 ] - 10.0 ).abs() < 1e-6 );
  assert!( ( mat[ 7 ] - 20.0 ).abs() < 1e-6 );
  assert!( ( mat[ 0 ] - 1.0 ).abs() < 1e-6 );
  assert!( ( mat[ 4 ] - 1.0 ).abs() < 1e-6 );
}

/// Verifies that a scale-only `Transform` places the scale factors on
/// the matrix diagonal and zeroes the off-diagonal elements.
#[ test ]
fn to_mat3_scale()
{
  let transform = Transform { scale : [ 2.0, 3.0 ], ..Default::default() };
  let mat = transform.to_mat3();
  assert!( ( mat[ 0 ] - 2.0 ).abs() < 1e-6 );
  assert!( ( mat[ 4 ] - 3.0 ).abs() < 1e-6 );
  assert!( ( mat[ 1 ] ).abs() < 1e-6 );
  assert!( ( mat[ 3 ] ).abs() < 1e-6 );
}

/// Verifies that a 90-degree rotation produces the expected sine/cosine
/// values in the rotation slots of the matrix (indices 0, 1, 3, 4).
#[ test ]
fn to_mat3_rotation_90()
{
  let transform = Transform { rotation : core::f32::consts::FRAC_PI_2, ..Default::default() };
  let mat = transform.to_mat3();
  assert!( mat[ 0 ].abs() < 1e-6, "m00={}", mat[ 0 ] );
  assert!( ( mat[ 1 ] - 1.0 ).abs() < 1e-6, "m10={}", mat[ 1 ] );
  assert!( ( mat[ 3 ] + 1.0 ).abs() < 1e-6, "m01={}", mat[ 3 ] );
  assert!( mat[ 4 ].abs() < 1e-6, "m11={}", mat[ 4 ] );
}

/// Verifies the Y-up CCW invariant: a 90° CCW rotation maps world direction (1, 0) to (0, 1).
/// In column-major layout this means m[1] = +sin = +1 and m[3] = -sin = -1.
/// This is the contract every adapter must preserve.
#[ test ]
fn to_mat3_ccw_positive_rotation()
{
  let t = Transform { rotation : core::f32::consts::FRAC_PI_2, ..Default::default() };
  let m = t.to_mat3();
  assert!( ( m[ 1 ] - 1.0 ).abs() < 1e-6, "sin(π/2) should be +1 for CCW, got {}", m[ 1 ] );
  assert!( ( m[ 3 ] + 1.0 ).abs() < 1e-6, "-sin(π/2) should be -1 for CCW, got {}", m[ 3 ] );
}

// test_kind: bug_reproducer(BUG-239)
/// ## Root Cause
/// `Transform::to_mat3()` computed the x-basis column (`m00`, `m10`) as
/// `(cos_r + sin_r*sky, sin_r - cos_r*sky)`, the opposite sign on the `sky`
/// (`skew[1].tan()`) term from the real SVG `skewY(a)` matrix
/// (`x'=x, y'=y+x*tan(a)`) that `Transform::skew`'s own doc comment and
/// `SvgBackend::transform_to_svg_local` (`src/adapters/svg.rs`, which passes
/// `skew[1]` straight into a real SVG `skewY()` op with no sign flip) both
/// define `skew[1]` against. Isolated with rotation=0, scale=1, `skew=[0, π/4]`
/// (only Y-skew, removing all rotation/scale coupling): the pre-fix matrix
/// mapped the unit x-basis point `(1,0)` to `(1,-1)`, while real SVG
/// `skewY(45°)` applied to the same point gives `(1,+1)` — confirmed
/// numerically before writing this fix. `skew[0]` (`skx`, "skewX") had no
/// such error — isolating it the same way already matched real SVG `skewX`.
///
/// ## Why Not Caught
/// No test in `types_test.rs` ever set `skew` to a non-default value (every
/// existing `to_mat3_*` test relies on `..Default::default()`, which zeroes
/// `skew`), and grepping the entire workspace found no caller anywhere that
/// sets `Transform::skew` to anything but `[0.0, 0.0]` either — this path was
/// unexercised by both tests and every real caller.
///
/// ## Fix Applied
/// Flipped the `sky` operators in `m00`/`m10` (`src/types.rs`): `cos_r +
/// sin_r*sky` → `cos_r - sin_r*sky`, `sin_r - cos_r*sky` → `sin_r +
/// cos_r*sky`. `skx`'s (`m01`/`m11`) formula is unchanged.
///
/// ## Prevention
/// A hand-derived combined transform matrix needs each input checked against
/// an independent single-axis case (isolate one field, zero the rest) before
/// trusting the combined formula — a wrong sign on one field can hide
/// indefinitely behind passing tests that only ever exercise the other
/// fields, especially when no real caller exercises the field either.
///
/// ## Pitfall
/// `skew[0]` and `skew[1]` are NOT symmetric in a skewX/skewY matrix (skewX
/// shifts x by y's amount; skewY shifts y by x's amount) — the field that
/// looks like it "should" mirror its sibling by symmetry can legitimately
/// need a different sign, so don't assume a fix to one field's sign implies
/// the same fix for the other; verify each independently.
#[ test ]
fn to_mat3_skew_y_matches_svg_skew_y_convention()
{
  // rotation=0, scale=1, skew=[0, pi/4] -- isolates skew[1] alone, no
  // rotation/scale coupling to disambiguate.
  let t = Transform { skew : [ 0.0, core::f32::consts::FRAC_PI_4 ], ..Default::default() };
  let m = t.to_mat3();

  // Applying the matrix to unit x-basis point (1,0) reads out as column 0 = (m[0], m[1]).
  // Real SVG skewY(45 deg) on (1,0): x'=x=1, y'=y+x*tan(45deg)=0+1=1.
  assert!( ( m[ 0 ] - 1.0 ).abs() < 1e-5, "x must be unaffected by pure Y-skew, got {}", m[ 0 ] );
  assert!
  (
    ( m[ 1 ] - 1.0 ).abs() < 1e-5,
    "skew[1]=pi/4 must match SVG skewY(45deg) applied to (1,0), i.e. y=+1; got {} \
     (pre-fix code produced -1, the mirrored/wrong-sign result)",
    m[ 1 ]
  );
}

/// Regression guard for `skew[0]` ("skewX"), which was already correct before
/// the BUG-239 fix — proves the fix (scoped to `skew[1]` only) did not
/// disturb it. Isolates `skew[0]` the same way the sibling test isolates
/// `skew[1]`: rotation=0, scale=1, only `skew[0]` nonzero.
#[ test ]
fn to_mat3_skew_x_matches_svg_skew_x_convention()
{
  let t = Transform { skew : [ core::f32::consts::FRAC_PI_4, 0.0 ], ..Default::default() };
  let m = t.to_mat3();

  // Applying the matrix to unit y-basis point (0,1) reads out as column 1 = (m[3], m[4]).
  // Real SVG skewX(45 deg) on (0,1): x'=x+y*tan(45deg)=0+1=1, y'=y=1.
  assert!
  (
    ( m[ 3 ] - 1.0 ).abs() < 1e-5,
    "skew[0]=pi/4 must match SVG skewX(45deg) applied to (0,1), i.e. x=+1; got {}",
    m[ 3 ]
  );
  assert!( ( m[ 4 ] - 1.0 ).abs() < 1e-5, "y must be unaffected by pure X-skew, got {}", m[ 4 ] );
}

/// Verifies that `RenderConfig::default()` produces the expected
/// width (800), height (600), antialias mode, and background color.
#[ test ]
fn render_config_default()
{
  let config = RenderConfig::default();
  assert_eq!( config.width, 800 );
  assert_eq!( config.height, 600 );
  assert_eq!( config.antialias, Antialias::Default );
  assert!( config.background[ 0 ].abs() < 1e-6 );
  assert!( config.background[ 1 ].abs() < 1e-6 );
  assert!( config.background[ 2 ].abs() < 1e-6 );
  assert!( ( config.background[ 3 ] - 1.0 ).abs() < 1e-6 );
  assert!( ( config.max_depth - 1.0 ).abs() < 1e-6 );
}
