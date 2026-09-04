//! Regression coverage for BUG-178: `apply_white_balance` in
//! `src/webgl/shaders/post_processing/color_grading.frag` applied `tint` with an inverted sign,
//! swapping the documented magenta/green directions.
//!
//! GLSL ES 3.00 has no native/offline execution path in this crate (see
//! `shader_validation_tests.rs`'s own scope note: naga's `glsl-in` front end parses desktop
//! GLSL, not the ES profile these `.frag` files use), so `apply_white_balance` below is a
//! line-for-line Rust port of the fixed shader function, kept deliberately close to the GLSL
//! source so the mapping stays auditable.

/// Port of the fixed `apply_white_balance` (`vec3 t = vec3(1.0); t.r += 0.2*temperature +
/// 0.1*tint; t.b -= 0.2*temperature - 0.1*tint; return color * t;`).
fn apply_white_balance( color : [ f32; 3 ], temperature : f32, tint : f32 ) -> [ f32; 3 ]
{
  let mut t = [ 1.0_f32, 1.0, 1.0 ];
  t[ 0 ] += 0.2 * temperature + 0.1 * tint;
  t[ 2 ] -= 0.2 * temperature - 0.1 * tint;
  [ color[ 0 ] * t[ 0 ], color[ 1 ] * t[ 1 ], color[ 2 ] * t[ 2 ] ]
}

/// ## Root Cause
/// `t.r += 0.2*temperature - 0.1*tint; t.b -= 0.2*temperature + 0.1*tint;` applied `tint`'s
/// contribution with opposing sign on the red and blue channels -- the same pattern used for
/// `temperature`, which is *supposed* to oppose (warm raises red, lowers blue). But magenta
/// requires red AND blue to move together (both boosted), and green requires both suppressed
/// together, so a positive ("magenta") tint value was actually shifting the image toward green,
/// and a negative ("green") tint value toward magenta -- exactly backwards from the doc comment
/// on `ColorGradingParams::tint`.
/// ## Why Not Caught
/// No test exercised `apply_white_balance`'s tint direction prior to this bug; the existing
/// `color_grading_tests.rs` only covers `ColorGradingParams`'s `Default`/`Clone` derives, not any
/// shader math, and GLSL ES has no CPU-side execution path in this crate to catch it otherwise.
/// ## Fix Applied
/// Changed the tint term's sign to match on both channels (`+= ... + 0.1*tint`, `-= ... -
/// 0.1*tint`), so positive tint boosts red and blue together (magenta) and negative tint
/// suppresses both together (green), while leaving `temperature`'s independent, intentionally
/// opposing sign relationship unchanged.
/// ## Prevention
/// This test asserts the documented direction directly: positive tint must increase both red and
/// blue relative to neutral (magenta), negative tint must decrease both (green), and neither
/// moves green itself -- plus a companion test confirming `temperature`'s own (correct,
/// unchanged) warm/cool behavior wasn't altered by the tint fix.
/// ## Pitfall
/// Two parameters sharing a channel-adjustment formula can each need a *different* sign
/// relationship between the channels they touch -- copying one parameter's sign pattern onto a
/// second parameter without checking that parameter's own intended channel relationship
/// independently is exactly how this bug was introduced.
// test_kind: bug_reproducer(BUG-178)
#[ test ]
fn positive_tint_shifts_toward_magenta_not_green()
{
  let neutral = [ 0.5, 0.5, 0.5 ];
  let result = apply_white_balance( neutral, 0.0, 1.0 );

  assert!( result[ 0 ] > neutral[ 0 ], "positive ( magenta ) tint should increase red, got {result:?}" );
  assert!( result[ 2 ] > neutral[ 2 ], "positive ( magenta ) tint should increase blue, got {result:?}" );
  assert!
  (
    ( result[ 1 ] - neutral[ 1 ] ).abs() < 1e-6,
    "tint must not directly affect green, got {result:?}"
  );
}

#[ test ]
fn negative_tint_shifts_toward_green_not_magenta()
{
  let neutral = [ 0.5, 0.5, 0.5 ];
  let result = apply_white_balance( neutral, 0.0, -1.0 );

  assert!( result[ 0 ] < neutral[ 0 ], "negative ( green ) tint should decrease red, got {result:?}" );
  assert!( result[ 2 ] < neutral[ 2 ], "negative ( green ) tint should decrease blue, got {result:?}" );
}

#[ test ]
fn temperature_direction_unaffected_by_the_tint_fix()
{
  let neutral = [ 0.5, 0.5, 0.5 ];

  let warm = apply_white_balance( neutral, 1.0, 0.0 );
  assert!
  (
    warm[ 0 ] > neutral[ 0 ] && warm[ 2 ] < neutral[ 2 ],
    "warm ( positive ) temperature should raise red and lower blue, got {warm:?}"
  );

  let cool = apply_white_balance( neutral, -1.0, 0.0 );
  assert!
  (
    cool[ 0 ] < neutral[ 0 ] && cool[ 2 ] > neutral[ 2 ],
    "cool ( negative ) temperature should lower red and raise blue, got {cool:?}"
  );
}
