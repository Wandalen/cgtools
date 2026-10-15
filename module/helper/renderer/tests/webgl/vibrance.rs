//! Regression coverage for BUG-244: `adjust_vibrance` in
//! `src/webgl/shaders/post_processing/color_grading.frag` weighted its saturation push by a term
//! that GREW with a color's existing saturation, boosting already-vivid colors (including skin
//! tones) harder than dull ones -- exactly backwards from the file's own documented "Vibrance vs
//! Saturation" contract ("Smart saturation affecting less-saturated colors more").
//!
//! GLSL ES 3.00 has no native/offline execution path in this crate (see
//! `shader_validation_tests.rs`'s own scope note: naga's `glsl-in` front end parses desktop
//! GLSL, not the ES profile these `.frag` files use), so `adjust_vibrance` below is a
//! line-for-line Rust port of the fixed shader function, kept deliberately close to the GLSL
//! source so the mapping stays auditable, mirroring this crate's own `white_balance.rs` (BUG-178)
//! precedent for logic with no test-reachable execution path.

/// Port of the fixed `adjust_vibrance` (`float mx = max(...); float mn = min(...); float sat =
/// (mx - mn) / max(mx, 0.0001); float amt = (1.0 - sat) * (-vibrance * 3.0); return mix(color,
/// vec3(mx), amt);`).
fn adjust_vibrance( color : [ f32; 3 ], vibrance : f32 ) -> [ f32; 3 ]
{
  let mx = color[ 0 ].max( color[ 1 ] ).max( color[ 2 ] );
  let mn = color[ 0 ].min( color[ 1 ] ).min( color[ 2 ] );
  let sat = ( mx - mn ) / mx.max( 0.0001 );
  let amt = ( 1.0 - sat ) * ( -vibrance * 3.0 );
  [
    color[ 0 ] + amt * ( mx - color[ 0 ] ),
    color[ 1 ] + amt * ( mx - color[ 1 ] ),
    color[ 2 ] + amt * ( mx - color[ 2 ] ),
  ]
}

/// Raw ( unclamped ) saturation spread `mx - mn` -- used to measure relative boost strength
/// without the final shader's `clamp( color, 0.0, 1.0 )` masking how far a channel overshot.
fn spread( color : [ f32; 3 ] ) -> f32
{
  let mx = color[ 0 ].max( color[ 1 ] ).max( color[ 2 ] );
  let mn = color[ 0 ].min( color[ 1 ] ).min( color[ 2 ] );
  mx - mn
}

/// ## Root Cause
/// `amt = ( mx - average ) * ( -vibrance * 3.0 )` weighted the saturation push by `( mx -
/// average )`, a proxy that is 0 for a gray color and grows toward 2/3 for a fully saturated
/// primary -- i.e. it INCREASES with existing saturation. Since `amt`'s magnitude directly scales
/// the push toward/away from `vec3( mx )`, colors that were already more saturated received a
/// proportionally LARGER push than duller colors, the opposite of "affects less-saturated colors
/// more".
/// ## Why Not Caught
/// No test exercised `adjust_vibrance`'s relative boost strength across colors of differing
/// existing saturation prior to this bug; the existing `color_grading_tests.rs` only covers
/// `ColorGradingParams`'s `Default`/`Clone` derives, not any shader math, and GLSL ES has no
/// CPU-side execution path in this crate to catch it otherwise. The bug also produces no crash or
/// obviously-wrong image -- a stronger-than-intended push on already-vivid colors still looks
/// like "more vibrant," just not distributed the way the documented contract promises.
/// ## Fix Applied
/// Replaced the `( mx - average )` weight with the complement of a normalized HSV-style
/// saturation, `( 1.0 - ( mx - mn ) / mx )` -- 1 at zero existing saturation (maximal boost
/// headroom), 0 at full existing saturation (fully protected) -- while keeping the `-vibrance *
/// 3.0` sign/scale convention and the `mix( color, vec3( mx ), amt )` blend unchanged, so positive
/// vibrance still saturates and negative still desaturates, only the weighting direction changed.
/// ## Prevention
/// This test compares two colors with the same vibrance applied -- one with low existing
/// saturation, one with high -- and asserts the low-saturation color's raw channel spread grows by
/// a strictly larger relative factor than the high-saturation color's. The pre-fix formula fails
/// this: it grows the already-saturated color's spread by the larger relative factor instead.
/// ## Pitfall
/// A weight term meant to protect already-prominent values (here: already-saturated colors) must
/// be a DECREASING function of that value's own prominence, not an increasing one -- reusing a
/// proxy quantity (`mx - average`) that correlates with the thing you're trying to protect, in the
/// wrong direction, produces a formula that is internally consistent and plausible-looking while
/// doing the opposite of its documented intent.
// test_kind: bug_reproducer(BUG-244)
#[ test ]
fn low_saturation_color_gets_a_larger_relative_boost_than_high_saturation_color()
{
  let low_sat = [ 0.55, 0.50, 0.45 ]; // spread 0.10
  let high_sat = [ 0.90, 0.50, 0.10 ]; // spread 0.80

  let low_before = spread( low_sat );
  let high_before = spread( high_sat );

  let low_after = spread( adjust_vibrance( low_sat, 1.0 ) );
  let high_after = spread( adjust_vibrance( high_sat, 1.0 ) );

  let low_ratio = low_after / low_before;
  let high_ratio = high_after / high_before;

  assert!
  (
    low_ratio > high_ratio,
    "low-saturation color must gain relative spread ( {low_ratio} ) faster than the \
    already-saturated color ( {high_ratio} ) -- vibrance must affect less-saturated colors more"
  );
}

#[ test ]
fn fully_saturated_color_is_unaffected_by_positive_vibrance()
{
  // mn == 0 -> sat == 1 -> weight ( 1.0 - sat ) == 0 -> fully protected.
  let fully_saturated = [ 1.0, 0.0, 0.0 ];
  let result = adjust_vibrance( fully_saturated, 1.0 );

  for ( a, b ) in fully_saturated.iter().zip( result.iter() )
  {
    assert!( ( a - b ).abs() < 1e-5, "fully saturated color must be unaffected, got {result:?}" );
  }
}

#[ test ]
fn gray_color_is_unaffected_by_any_vibrance()
{
  let gray = [ 0.5, 0.5, 0.5 ];

  for vibrance in [ -1.0, -0.5, 0.5, 1.0 ]
  {
    let result = adjust_vibrance( gray, vibrance );
    for ( a, b ) in gray.iter().zip( result.iter() )
    {
      assert!
      (
        ( a - b ).abs() < 1e-5,
        "gray must stay unchanged at vibrance {vibrance}, got {result:?}"
      );
    }
  }
}

#[ test ]
fn positive_vibrance_increases_saturation_of_a_partially_saturated_color()
{
  let color = [ 0.7, 0.5, 0.3 ];
  let before = spread( color );
  let after = spread( adjust_vibrance( color, 1.0 ) );

  assert!( after > before, "positive vibrance must increase saturation spread, got {after} <= {before}" );
}

#[ test ]
fn negative_vibrance_decreases_saturation_of_a_partially_saturated_color()
{
  let color = [ 0.7, 0.5, 0.3 ];
  let before = spread( color );
  let after = spread( adjust_vibrance( color, -1.0 ) );

  assert!( after < before, "negative vibrance must decrease saturation spread, got {after} >= {before}" );
}
