use ndarray_cg::{ F64x3, approx::assert_abs_diff_eq };

use super::*;

#[ test ]
fn test_multiply()
{
  use the_module::
  {
    QuatF64,
  };

  let q1 = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ] );
  let q2 = QuatF64::from( [ -5.0, 1.0, 3.0, 10.0 ] );

  let exp = QuatF64::from( [ -13.0, 42.0, 31.0, 34.0 ] );
  assert_eq!( q2 * q1, exp, "Quaternion * Quaternion multiplication mismatch" );

  let exp = QuatF64::from( [ -7.0, 6.0, 53.0, 34.0 ] );
  assert_eq!( q1 * q2, exp, "Quaternion * Quaternion multiplication mismatch" );

  let exp =  QuatF64::from( [ 5.0, 10.0, 15.0, 20.0 ] );
  assert_eq!( q1 * 5.0, exp, "Quaternion * Scalar multiplication mismatch" );

  let mut q1 = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ] );
  let q2 = QuatF64::from( [ -5.0, 1.0, 3.0, 10.0 ] );

  let exp =  QuatF64::from( [ -7.0, 6.0, 53.0, 34.0 ] );

  q1 *= q2;
  assert_eq!( q1, exp, "Quaternion *= Quaternion multiplication mismatch" );
}

#[ test ]
fn test_divide()
{
  use the_module::
  {
    QuatF64,
  };

  let q1 = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ] ).normalize();
  let q2 = QuatF64::from( [ -5.0, 1.0, 3.0, 10.0 ] ).normalize();

  let exp = QuatF64::from( [ 0.424_264_068_711_928_5, 0.534_258_456_896_502_5, 0.109_994_388_184_574_05, 0.722_820_265_212_915_2 ] );
  assert_abs_diff_eq!( q1 / q2, exp, );

  let q1 = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ] ).normalize();
  let q2 = QuatF64::from( [ 0.9, 2.0, 3.0, 4.0 ] ).normalize();

  let exp = QuatF64::from( [ 0.013_375_757_175_498_215, 0.010_031_817_881_623_634, -0.006_687_878_587_749_038, 0.999_837_848_868_489 ] );
  assert_abs_diff_eq!( q1 / q2, exp, );
}

// BUG-298 task/bug/298_quat_invert_wrong_for_non_unit_quaternions.md -- reproducer
// for `Quat::invert()`'s wrong result on non-unit-length quaternions.
/// ## Root Cause
/// `Quat::invert()` unconditionally returned `self.conjugate()`, which is only the true
/// multiplicative inverse when the quaternion is unit-length ( magnitude 1 ). For a non-unit
/// quaternion `q`, the correct inverse is `conjugate(q) / mag2(q)`; `divide`/`Div`/`DivAssign`
/// all route through `invert()`, so dividing by any non-unit quaternion silently scaled the
/// result by the divisor's squared magnitude instead of producing a true quotient.
///
/// ## Why Not Caught
/// The only existing division test, `test_divide` above, normalizes both operands before
/// dividing -- for a unit quaternion `mag2() == 1`, so `conjugate()` and the true inverse
/// coincide and the bug is invisible. No test exercised division with a non-unit divisor or
/// checked the defining round-trip property `(a / b) * b == a`.
///
/// ## Fix Applied
/// BUG-298 changed `invert()` in `src/quaternion/arithmetics.rs` from `self.conjugate()` to
/// `self.conjugate() / self.mag2()`, which reduces to the prior behavior exactly when the
/// quaternion is already unit-length and is otherwise the correct general inverse.
///
/// ## Prevention
/// This test divides two deliberately non-unit quaternions and asserts the defining property of
/// division holds: `(a / b) * b == a`. The pre-fix formula fails this for any divisor whose
/// squared magnitude is not 1.
///
/// ## Pitfall
/// A function whose doc comment names a precondition ( "unit-length" ) but whose signature
/// accepts any value of the type provides no compile-time or run-time signal when that
/// precondition is violated -- every caller reachable through a general-purpose op like `Div`
/// silently inherits the narrower assumption. Prefer implementing the operation correctly for
/// the general case when the correct general formula is no more expensive than the
/// unit-only shortcut, rather than documenting a precondition callers have no way to check.
// test_kind: bug_reproducer(BUG-298)
#[ test ]
fn test_divide_non_unit_round_trip()
{
  use the_module::
  {
    QuatF64,
  };

  let q1 = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ] );
  let q2 = QuatF64::from( [ -5.0, 1.0, 3.0, 10.0 ] );

  let quotient = q1.divide( &q2 );
  let reconstructed = quotient * q2;
  assert_abs_diff_eq!( reconstructed, q1, epsilon = 1e-9 );

  let mut q1_mut = q1;
  q1_mut.divide_mut( &q2 );
  assert_abs_diff_eq!( q1_mut, quotient );
}

#[ test ]
fn test_from_angle_x()
{
  use the_module::
  {
    QuatF64,
  };

  let q = QuatF64::from_angle_x( 1.0 );
  let exp = QuatF64::from( [ 0.479_425_538_604_203, 0.0, 0.0, 0.877_582_561_890_372_8 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_angle_x( -1.0 );
  let exp = QuatF64::from( [ -0.479_425_538_604_203, 0.0, 0.0, 0.877_582_561_890_372_8 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_angle_x( 256.0 );
  let exp = QuatF64::from( [ 0.721_037_710_501_731_6, -0.0, 0.0, -0.692_895_821_920_165_1 ] );
  assert_abs_diff_eq!( q, exp );
}

#[ test ]
fn test_from_angle_y()
{
  use the_module::
  {
    QuatF64,
  };

  let q = QuatF64::from_angle_y( 1.0 );
  let exp = QuatF64::from( [ 0.0, 0.479_425_538_604_203, 0.0, 0.877_582_561_890_372_8 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_angle_y( -1.0 );
  let exp = QuatF64::from( [ 0.0, -0.479_425_538_604_203, 0.0, 0.877_582_561_890_372_8 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_angle_y( 256.0 );
  let exp = QuatF64::from( [ 0.0, 0.721_037_710_501_731_6, 0.0, -0.692_895_821_920_165_1 ] );
  assert_abs_diff_eq!( q, exp );
}

// BUG-311 task/bug/311_from_angle_y_called_with_raw_degrees_not_radians.md -- reproducer for
// `Quat::from_angle_y` being called with a raw degree literal instead of a radians value at 3
// sibling call sites in `examples/minwebgl/{curve,lottie,animation}_surface_rendering`.
/// ## Root Cause
/// `Quat::from_angle_y` takes its angle in radians (its own doc comment states this explicitly,
/// and its implementation applies the half-angle formula `(angle / two).sin_cos()`), but
/// `examples/minwebgl/curve_surface_rendering/src/main.rs`, `lottie_surface_rendering/src/main.rs`,
/// and `animation_surface_rendering/src/main.rs` -- a copy-pasted "clouds" mesh setup block --
/// each called `gl::Quat::from_angle_y( 90.0 )` intending a 90-degree rotation, passing the raw
/// degree value directly instead of `90.0_f32.to_radians()`.
/// ## Why Not Caught
/// `from_angle_y` cannot distinguish a degrees-shaped caller mistake from a genuine (small)
/// radians value -- `90.0` radians is a valid input, just not the intended one -- and none of
/// the 3 affected example crates has any test asserting the clouds mesh's actual orientation.
/// ## Fix Applied
/// Changed all 3 call sites from `gl::Quat::from_angle_y( 90.0 )` to
/// `gl::Quat::from_angle_y( 90.0_f32.to_radians() )`.
/// ## Prevention
/// This test asserts `QuatF64::from_angle_y` with a genuine `90_f64.to_radians()` input matches
/// the closed-form 90-degree-about-Y quaternion, and separately asserts the raw literal `90.0`
/// -- what all 3 call sites passed pre-fix -- does NOT produce that same quaternion.
/// ## Pitfall
/// A rotation constructor documented as taking radians gives no compile-time or run-time signal
/// when a caller passes a degrees-shaped value instead -- `to_radians()` must be applied
/// explicitly at every call site that starts from a human-readable degree constant.
// test_kind: bug_reproducer(BUG-311)
#[ test ]
fn test_from_angle_y_rejects_raw_degrees()
{
  use the_module::QuatF64;

  // A genuine 90-degree rotation about Y -- closed form mirrors BUG-272's own
  // `from_angle_y( -90 deg )` precedent below, mirrored to +90 deg.
  let correct = QuatF64::from_angle_y( 90.0_f64.to_radians() );
  let expected = QuatF64::from( [ 0.0, std::f64::consts::FRAC_1_SQRT_2, 0.0, std::f64::consts::FRAC_1_SQRT_2 ] );
  assert_abs_diff_eq!( correct, expected, epsilon = 1e-9 );

  // The raw literal `90.0` -- what all 3 example call sites passed pre-fix -- is ~5_157 degrees,
  // not 90, and must not match the genuine 90-degree rotation above.
  let buggy = QuatF64::from_angle_y( 90.0 );
  assert_ne!( buggy, expected );
}

#[ test ]
fn test_from_angle_z()
{
  use the_module::
  {
    QuatF64,
  };

  let q = QuatF64::from_angle_z( 1.0 );
  let exp = QuatF64::from( [ 0.0, 0.0, 0.479_425_538_604_203, 0.877_582_561_890_372_8 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_angle_z( -1.0 );
  let exp = QuatF64::from( [ 0.0, 0.0, -0.479_425_538_604_203, 0.877_582_561_890_372_8 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_angle_z( 256.0 );
  let exp = QuatF64::from( [ 0.0, 0.0, 0.721_037_710_501_731_6, -0.692_895_821_920_165_1 ] );
  assert_abs_diff_eq!( q, exp );
}

/// ## Root Cause
/// `from_axis_angle` used `angle.sin_cos()` directly instead of `(angle / 2.0).sin_cos()` --
/// the axis-angle-to-quaternion formula requires the HALF angle in the `sin`/`cos` terms
/// (`w = cos(angle/2)`, `xyz = axis * sin(angle/2)`), so the un-halved call produced a
/// quaternion representing twice the requested rotation.
///
/// ## Why Not Caught
/// No test compared `from_axis_angle` against an independently-correct sibling constructor
/// for the same rotation before this task -- the already-correct `from_angle_x`/`_y`/`_z`
/// (which do halve internally) were only ever tested in isolation.
///
/// ## Fix Applied
/// BUG-120 changed `angle.sin_cos()` to `(angle / two).sin_cos()` in
/// `src/quaternion/arithmetics.rs::from_axis_angle`.
///
/// ## Prevention
/// This test constructs the same rotation two independent ways -- `from_axis_angle` about a
/// standard basis axis, and the axis-specific `from_angle_x`/`_y`/`_z` -- and asserts they
/// agree. The pre-fix doubled angle fails this immediately for any non-zero angle.
///
/// ## Pitfall
/// A rotation constructor taking a plain (non-halved) angle parameter must apply the half-angle
/// conversion internally wherever the underlying representation's algebra requires it --
/// verify new rotation constructors against an independently-correct sibling, not just against
/// hand-derived numbers that can silently encode the same mistake.
#[ test ]
fn test_from_axis_angle_matches_axis_aligned_from_angle_x()
{
  use the_module::QuatF64;

  let got = QuatF64::from_axis_angle( F64x3::new( 1.0, 0.0, 0.0 ), 1.0 );
  let exp = QuatF64::from_angle_x( 1.0 );
  assert_abs_diff_eq!( got, exp );
}

/// ## Root Cause
/// See `test_from_axis_angle_matches_axis_aligned_from_angle_x` above (BUG-120).
///
/// ## Why Not Caught
/// See `test_from_axis_angle_matches_axis_aligned_from_angle_x` above (BUG-120).
///
/// ## Fix Applied
/// See `test_from_axis_angle_matches_axis_aligned_from_angle_x` above (BUG-120).
///
/// ## Prevention
/// Same mechanism as `test_from_axis_angle_matches_axis_aligned_from_angle_x`, for the Y axis.
///
/// ## Pitfall
/// See `test_from_axis_angle_matches_axis_aligned_from_angle_x` above (BUG-120).
#[ test ]
fn test_from_axis_angle_matches_axis_aligned_from_angle_y()
{
  use the_module::QuatF64;

  let got = QuatF64::from_axis_angle( F64x3::new( 0.0, 1.0, 0.0 ), 1.0 );
  let exp = QuatF64::from_angle_y( 1.0 );
  assert_abs_diff_eq!( got, exp );
}

/// ## Root Cause
/// See `test_from_axis_angle_matches_axis_aligned_from_angle_x` above (BUG-120).
///
/// ## Why Not Caught
/// See `test_from_axis_angle_matches_axis_aligned_from_angle_x` above (BUG-120).
///
/// ## Fix Applied
/// See `test_from_axis_angle_matches_axis_aligned_from_angle_x` above (BUG-120).
///
/// ## Prevention
/// Same mechanism as `test_from_axis_angle_matches_axis_aligned_from_angle_x`, for the Z axis.
///
/// ## Pitfall
/// See `test_from_axis_angle_matches_axis_aligned_from_angle_x` above (BUG-120).
#[ test ]
fn test_from_axis_angle_matches_axis_aligned_from_angle_z()
{
  use the_module::QuatF64;

  let got = QuatF64::from_axis_angle( F64x3::new( 0.0, 0.0, 1.0 ), 1.0 );
  let exp = QuatF64::from_angle_z( 1.0 );
  assert_abs_diff_eq!( got, exp );
}

#[ test ]
fn test_from_euler_xyz()
{
  use the_module::
  {
    QuatF64,
  };

  let q = QuatF64::from_euler_xyz( [ 1.0, 2.0, 3.0 ] );
  let exp = QuatF64::from( [ 0.754_933_801_264_452_5, -0.206_149_226_026_877_7, 0.501_509_096_403_722_1, -0.368_871_357_713_289_8 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_euler_xyz( [ 0.0, 0.0, 0.0 ] );
  let exp = QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_euler_xyz( [ -23.0, 123.0, 0.53 ] );
  let exp = QuatF64::from( [ 0.076_980_141_411_157_5, -0.507_448_993_073_131_5, -0.790_928_849_502_003_3, 0.333_168_324_248_900_8 ] );
  assert_abs_diff_eq!( q, exp );
}

#[test]
fn test_to_euler_xyz()
{
  use ndarray_cg::QuatF64;

  let test_cases =
  [
    [ 1.0_f64.to_radians(), 2.0_f64.to_radians(), 3.0_f64.to_radians() ],
    [ 0.0, 0.0, 0.0 ],
    [ 0.01, 0.01, 0.01 ],
    [ 0.0, 90.0_f64.to_radians(), 0.0 ],
  ];

  for input in test_cases
  {
    let q_in = QuatF64::from_euler_xyz( input ).normalize();

    let result = q_in.to_euler_xyz();

    assert_abs_diff_eq!
    (
      result,
      F64x3::from_array( input ),
      epsilon = 1e-1
    );
  }
}

// Note (BUG-272): case 4's raw quat literal was corrected from `[ 0.707, 0.0, 0.707, 0.0 ]` to
// `[ 0.0, -FRAC_1_SQRT_2, 0.0, FRAC_1_SQRT_2 ]`. The original literal has `w = 0`, i.e. it is a
// 180 degree rotation about axis `( 1, 0, 1 ) / sqrt( 2 )`, not `Ry( -90 deg )` -- it only
// "passed" against the pre-fix buggy formula because that formula's mismatched `sinp = 2 * ( w *
// y - z * x )` happened to evaluate to `-1.0` for this specific (wrong) quat, coincidentally
// landing on the claimed `-90 deg` within this test's loose `epsilon = 1e-1`. The corrected
// literal is the quaternion `from_angle_y( -90 deg )` actually produces (expressed via the named
// constant rather than a decimal literal, since the latter trips clippy's `approx_constant`).
#[ test ]
fn test_to_euler_xyz_from_raw_quat()
{
  use ndarray_cg::QuatF64;

  let test_cases =
  [
    ( [ 0.009, 0.017, 0.026, 0.999 ], [ 1.0_f64.to_radians(), 2.0_f64.to_radians(), 3.0_f64.to_radians() ] ),
    ( [ 0.0, 0.0, 0.0, 1.0 ], [ 0.0, 0.0, 0.0 ] ),
    ( [ 0.0, 0.0, 0.0, 1.0 ], [ 0.01_f64.to_radians(), 0.01_f64.to_radians(), 0.01_f64.to_radians() ] ),
    ( [ 0.0, -std::f64::consts::FRAC_1_SQRT_2, 0.0, std::f64::consts::FRAC_1_SQRT_2 ], [ 0.0, -90.0_f64.to_radians(), 0.0 ] ),
  ];

  for ( raw_quat, expected ) in test_cases
  {
    let q_in = QuatF64::from( raw_quat );

    let result = q_in.to_euler_xyz();

    println!("Expected: {:?}", [ expected[ 0 ].to_degrees(), expected[ 1 ].to_degrees(), expected[ 2 ].to_degrees() ]);

    assert_abs_diff_eq!
    (
      result,
      F64x3::from_array( expected ),
      epsilon = 1e-1
    );
  }
}

/// ## Root Cause
/// `to_euler_xyz` extracted pitch/roll/yaw via `asin`/`atan2` formulas whose cross terms had
/// the wrong sign ( `w*y - z*x` instead of `w*y + z*x` for pitch; `w*x + y*z` instead of
/// `w*x - y*z` for roll; `w*z + x*y` instead of `w*z - x*y` for yaw ), the gimbal-lock branch's
/// collapsed-yaw denominator used `y*y + z*z` instead of `x*x + z*z`, and that same branch's
/// leading `two *` bound to the *result* of `.atan2( .. )` instead of its numerator (`two * (
/// x * y + w * z ).atan2( .. )` computes `2 * atan2( n, d )`, not the required `atan2( 2*n, d
/// )` -- the two are different functions whenever `n != 0`).
///
/// ## Why Not Caught
/// The pre-existing `test_to_euler_xyz`/`test_to_euler_xyz_from_raw_quat` cases used only
/// small angles ( 1-3 degrees, where the mismatched cross term is numerically tiny ) or
/// single-axis rotations ( where the mismatched cross term multiplies an always-zero
/// component, and where the gimbal-lock case's roll and yaw are both zero -- the one condition
/// under which `2 * atan2( n, d )` and `atan2( 2*n, d )` coincide, since `n = 0` either way ),
/// both of which mask a wrong sign or misplaced doubling under those tests' loose `epsilon =
/// 1e-1` tolerance -- neither a genuine multi-axis rotation nor an exact gimbal-lock case with
/// nonzero roll and yaw was ever exercised at a tight tolerance.
///
/// ## Fix Applied
/// BUG-272 corrected the three cross-term signs, the gimbal-lock denominator, and the
/// gimbal-lock branch's numerator parenthesization in
/// `src/quaternion/arithmetics.rs::to_euler_xyz`.
///
/// ## Prevention
/// This test round-trips three genuinely exercising rotations -- one away from gimbal lock
/// ( 30 deg / 20 deg / 10 deg, checked by direct angle comparison since the decomposition is
/// unique there ), and one at each gimbal pole ( pitch = +90 deg and pitch = -90 deg ) with
/// nonzero roll and yaw on both sides -- through `from_euler_xyz` ( independently verified
/// correct against the crate's own Hamilton-product `multiply` convention ) and back through
/// `to_euler_xyz`, at a tight `epsilon = 1e-6`. Roll and yaw are individually ambiguous at
/// gimbal lock ( only their combination is determined by the rotation ), so each gimbal case
/// checks reported roll/pitch directly ( both are set by construction: roll = 0 exactly, pitch
/// = +/-90 deg ) and then re-composes the reported angles through `from_euler_xyz` again,
/// asserting the round-tripped quaternion represents the same rotation as the original ( `|dot|
/// ~= 1`, since unit quaternions `p` and `-p` both encode the same rotation ) rather than
/// hardcoding a specific collapsed-angle literal. The pre-fix formulas fail this immediately;
/// only the corrected signs, denominator, and parenthesization recover the original rotation.
///
/// ## Pitfall
/// A round-trip test using only small angles or single-axis rotations cannot distinguish a
/// correct Euler-angle extraction formula from one with flipped cross-term signs or a misplaced
/// doubling, because the erroneous term is numerically negligible, multiplies an always-zero
/// component, or is applied to a zero numerator in every such case -- always include at least
/// one genuinely multi-axis case with non-trivial angles, and gimbal-lock cases with nonzero
/// angles on both sides, at a tight tolerance. At gimbal lock specifically, comparing to a
/// hand-derived expected angle triple is itself fragile ( roll/yaw are non-unique, and
/// re-deriving the collapsed-angle formula by hand is exactly the kind of sign-sensitive
/// arithmetic prone to the same class of transcription slip as the bug under test ) -- asserting
/// round-trip rotation equivalence instead is both more robust and does not require hand-solving
/// the ambiguous decomposition.
// test_kind: bug_reproducer(BUG-272)
#[ test ]
fn test_to_euler_xyz_multi_axis_round_trip()
{
  use the_module::QuatF64;

  // Away from gimbal lock: roll = 30 deg, pitch = 20 deg, yaw = 10 deg. The decomposition is
  // unique here, so a direct angle comparison is valid.
  let input = [ 30.0_f64.to_radians(), 20.0_f64.to_radians(), 10.0_f64.to_radians() ];
  let q = QuatF64::from_euler_xyz( input );
  let result = q.to_euler_xyz();
  assert_abs_diff_eq!( result, F64x3::from_array( input ), epsilon = 1e-6 );

  // Exact gimbal lock ( pitch = 90 deg ) with nonzero roll and yaw: roll and yaw individually
  // become ambiguous ( only their combination is determined by the rotation ), so this checks
  // the unambiguous roll/pitch directly and then asserts the recomposed quaternion represents
  // the same rotation as the original, instead of hardcoding a specific collapsed-angle value.
  let gimbal_input = [ 30.0_f64.to_radians(), 90.0_f64.to_radians(), 20.0_f64.to_radians() ];
  let q_gimbal = QuatF64::from_euler_xyz( gimbal_input );
  let result_gimbal = q_gimbal.to_euler_xyz();
  assert_abs_diff_eq!( result_gimbal.x(), 0.0, epsilon = 1e-6 );
  assert_abs_diff_eq!( result_gimbal.y(), 90.0_f64.to_radians(), epsilon = 1e-6 );
  let q_gimbal_roundtrip = QuatF64::from_euler_xyz( result_gimbal );
  assert_abs_diff_eq!( q_gimbal_roundtrip.dot( &q_gimbal ).abs(), 1.0, epsilon = 1e-6 );

  // Exact gimbal lock at the opposite pole ( pitch = -90 deg ), also with nonzero roll and yaw
  // -- the same branch handles both signs, so this independently guards the collapsed-angle
  // numerator's parenthesization for the negative-pitch case too.
  let gimbal_input_neg = [ 30.0_f64.to_radians(), -90.0_f64.to_radians(), 20.0_f64.to_radians() ];
  let q_gimbal_neg = QuatF64::from_euler_xyz( gimbal_input_neg );
  let result_gimbal_neg = q_gimbal_neg.to_euler_xyz();
  assert_abs_diff_eq!( result_gimbal_neg.x(), 0.0, epsilon = 1e-6 );
  assert_abs_diff_eq!( result_gimbal_neg.y(), -90.0_f64.to_radians(), epsilon = 1e-6 );
  let q_gimbal_neg_roundtrip = QuatF64::from_euler_xyz( result_gimbal_neg );
  assert_abs_diff_eq!( q_gimbal_neg_roundtrip.dot( &q_gimbal_neg ).abs(), 1.0, epsilon = 1e-6 );
}
