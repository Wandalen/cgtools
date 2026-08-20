use ndarray_cg::approx::assert_abs_diff_eq;

use super::*;

#[ test ]
fn test_slerp()
{
  use the_module::
  {
    QuatF64,
  };

  let q1 = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ] ).normalize();
  let q2 = QuatF64::from( [ -5.0, 6.0, 1.0, 3.0 ] ).normalize();

  let exp = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ] ).normalize();
  assert_abs_diff_eq!( q1.slerp( &q2, 0.0 ), exp );

  let exp =  QuatF64::from( [ -5.0, 6.0, 1.0, 3.0 ] ).normalize();
  assert_abs_diff_eq!( q1.slerp( &q2, 1.0 ), exp );

  let exp = QuatF64::from( [ -0.071_897_658_162_071_14, 0.540_143_988_792_169_5, 0.468_246_330_630_098_35, 0.695_572_118_456_413_6 ] );
  assert_abs_diff_eq!( q1.slerp( &q2, 0.3 ), exp );

  let exp = QuatF64::from( [ -0.239_050_070_065_631_06, 0.626_821_944_003_501, 0.387_771_873_937_87, 0.632_125_215_681_197_8 ] );
  assert_abs_diff_eq!( q1.slerp( &q2, 0.5 ), exp );

  let exp = QuatF64::from( [ -0.533_221_914_304_581_1, 0.711_102_202_219_155_7, 0.177_880_287_914_574_53, 0.422_334_762_097_382_5 ] );
  assert_abs_diff_eq!( q1.slerp( &q2, 0.9 ), exp );


  let q1 = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ] ).normalize();
  let q2 = QuatF64::from( [ 0.9, 2.0, 3.0, 4.0 ] ).normalize();

  let exp = QuatF64::from( [ 0.180_803_295_752_926_92, 0.365_269_889_495_024_7, 0.547_904_834_242_537_1, 0.730_539_778_990_049_4 ] );

  assert_abs_diff_eq!( q1.slerp( &q2, 0.1 ), exp );

  let exp = QuatF64::from( [ 0.173_713_929_236_047_12, 0.365_744_110_809_234_75, 0.548_616_166_213_852, 0.731_488_221_618_469_5 ] );
  assert_abs_diff_eq!( q1.slerp( &q2, 0.5 ), exp );

  let exp = QuatF64::from( [ 0.165_727_635_773_993_4, 0.366_254_923_608_804_8, 0.549_382_385_413_207, 0.732_509_847_217_609_6 ] );
  assert_abs_diff_eq!( q1.slerp( &q2, 0.95 ), exp );

}

/// ## Root Cause
/// `Quat<E>: From<&[E]>` used `debug_assert!( value.len() > 4, .. )`, requiring a slice of
/// at least 5 elements even though the very next line converts it into a `[E; 4]` via
/// `value.try_into().unwrap()` — a slice of the intended, correct length (exactly 4) failed
/// this assertion in every debug build.
///
/// ## Why Not Caught
/// No caller anywhere in `src/` or `tests/` used the `&[E]` slice constructor before this
/// task — every existing quaternion test uses the `[E; 4]` array constructor
/// (`QuatF64::from( [ .. ] )`) instead, so the slice path was never exercised.
///
/// ## Fix Applied
/// TASK-014 removed the `debug_assert!` from `quaternion/from.rs`. The very next line,
/// `value.try_into().unwrap()`, already performs the equivalent length check
/// unconditionally (in every build profile, not just debug), so no replacement check is
/// needed.
///
/// ## Prevention
/// This test constructs a `Quat` from a valid 4-element slice and confirms the result
/// equals the array-constructed equivalent. Before the fix this failed in a normal debug
/// test run (the `> 4` condition rejects a length-4 slice); it passes after the fix.
///
/// ## Pitfall
/// A `debug_assert!` duplicating a check that another, always-on code path already
/// performs can silently drift out of sync with it (here: `> 4` vs the real `== 4`
/// requirement) without being noticed — release builds never evaluate the drifted
/// condition, and no test exercised the one input shape (correct length) that would have
/// exposed the drift in debug builds either.
#[ test ]
fn test_quat_from_slice_valid()
{
  use the_module::QuatF64;

  let from_slice = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ].as_slice() );
  let from_array = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ] );
  assert_eq!( from_slice, from_array, "Quat::from(&[E]) should match Quat::from([E;4]) for a valid 4-element slice" );
}

/// ## Root Cause
/// N/A for this specific test — added alongside `test_quat_from_slice_valid` to confirm
/// that removing the buggy `debug_assert!` (see above) does not remove loud-failure
/// coverage for a genuinely wrong-length slice.
///
/// ## Why Not Caught
/// N/A — this is new coverage, not a regression test for a previously-shipped bug.
///
/// ## Fix Applied
/// N/A — no source change is needed for this case: `value.try_into().unwrap()` already
/// panics unconditionally when `value.len() != 4`.
///
/// ## Prevention
/// This test pins down that a 3-element slice still panics after the `debug_assert!` was
/// removed, in every build profile.
///
/// ## Pitfall
/// Removing a redundant debug-only check must not silently remove the *only* check —
/// this test confirms the always-on `try_into().unwrap()` still guards the invariant.
#[ test ]
#[ should_panic( expected = "called `Result::unwrap()` on an `Err` value" ) ]
fn test_quat_from_slice_wrong_length()
{
  use the_module::QuatF64;

  let _ = QuatF64::from( [ 1.0, 2.0, 3.0 ].as_slice() );
}

/// ## Root Cause
/// `Quat::from(Mat3)`'s final array literal wrote the trace-derived `w` term (`n0`) into the
/// `x` slot instead of `w`'s own slot -- a cyclic shift of all four components, since this
/// crate stores quaternion components as `[x,y,z,w]` (confirmed by `from_angle_x`/`from_angle_y`/
/// `from_angle_z`).
///
/// ## Why Not Caught
/// No test constructed a `Mat3` and converted it to a `Quat` before this task.
///
/// ## Fix Applied
/// BUG-119 reordered the array literal from `[n0,n1,n2,n3]` to `[n1,n2,n3,n0]`, matching
/// each term to the storage slot its own algebraic identity corresponds to.
///
/// ## Prevention
/// This test hand-derives the expected quaternion for an exact, closed-form 90 degree
/// rotation about Z and asserts the full quaternion matches -- the pre-fix cyclic shift
/// fails this immediately (`x` and `w` both nonzero, `z` zero, instead of `z` and `w`
/// nonzero, `x` zero).
///
/// ## Pitfall
/// A derivation's intermediate-term computation order (trace term first, for algebraic
/// convenience) can silently diverge from the target type's storage order -- always map each
/// term back to its named component before assembling the final array.
#[ test ]
fn test_from_mat3_recovers_known_axis_angle_rotation()
{
  use the_module::{ Mat3, Quat, mat::DescriptorOrderColumnMajor };

  // Exactly 90 deg about Z: r11=cos90=0, r12=-sin90=-1, r21=sin90=1, r22=cos90=0, r33=1.
  let m = Mat3::< f64, DescriptorOrderColumnMajor >::from_column_major
  (
    [
      0.0, 1.0, 0.0,
      -1.0, 0.0, 0.0,
      0.0, 0.0, 1.0,
    ]
  );

  let got : Quat< f64 > = m.into();
  let exp = Quat::< f64 >::from( [ 0.0, 0.0, std::f64::consts::FRAC_1_SQRT_2, std::f64::consts::FRAC_1_SQRT_2 ] );
  assert_abs_diff_eq!( got, exp );
}

/// ## Root Cause
/// See `test_from_mat3_recovers_known_axis_angle_rotation` above (BUG-119).
///
/// ## Why Not Caught
/// No round-trip test existed comparing `Quat::from(Mat3::from_quat(q))` against the
/// original `q` -- this would have caught any non-identity-preserving defect in either
/// conversion direction.
///
/// ## Fix Applied
/// See BUG-119's fix in `src/quaternion/from.rs`.
///
/// ## Prevention
/// This test builds a `Mat3` from a generic, non-axis-aligned quaternion via the
/// already-correct `Mat3::from_quat`, converts it back via `Quat::from(Mat3)`, and asserts
/// the round trip recovers the original quaternion. A positive-`w` input quaternion is used
/// so the comparison doesn't need to account for the `q`/`-q` double-cover ambiguity.
///
/// ## Pitfall
/// A round-trip test against an independently-verified reverse conversion is a strong, cheap
/// regression guard for any bidirectional representation conversion.
fn test_from_mat3_round_trips_through_from_quat_generic< Descriptor >()
where
  Descriptor : the_module::mat::Descriptor,
  the_module::Mat3< f64, Descriptor > :
    the_module::RawSliceMut< Scalar = f64 > +
    the_module::ScalarMut< Scalar = f64, Index = the_module::Ix2 > +
    the_module::ConstLayout< Index = the_module::Ix2 > +
    the_module::IndexingMut< Scalar = f64, Index = the_module::Ix2 >
{
  use the_module::{ Mat3, Quat, QuatF64 };

  let q = QuatF64::from( [ -5.0, 4.0, 1.0, 10.0 ] ).normalize();
  let m = Mat3::< f64, Descriptor >::from_quat( q );
  let got : Quat< f64 > = m.into();

  assert_abs_diff_eq!( got, q );
}

#[ test ]
fn test_from_mat3_round_trips_through_from_quat_row_major()
{
  test_from_mat3_round_trips_through_from_quat_generic::< the_module::mat::DescriptorOrderRowMajor >();
}

#[ test ]
fn test_from_mat3_round_trips_through_from_quat_column_major()
{
  test_from_mat3_round_trips_through_from_quat_generic::< the_module::mat::DescriptorOrderColumnMajor >();
}

/// ## Root Cause
/// `Quat::slerp` computed a hemisphere-corrected copy `q2` of `other` whenever
/// `cos_half_theta` ( `self.dot(other)` ) was negative -- `self` and `other` are more than 90
/// degrees apart as 4D vectors even though `q` and `-q` represent the identical rotation, so a
/// short-path blend requires interpolating towards `-other`, not `other`. Both return branches
/// kept blending against the original, un-flipped `*other` instead of the corrected `q2` --
/// pairing the short-path angle ( derived from the now-positive `cos_half_theta` ) with the
/// long-path quaternion value produced a non-unit-length result rotated the wrong way.
///
/// ## Why Not Caught
/// The existing `test_slerp` above only exercises quaternion pairs with a strictly positive
/// dot product ( both hand-picked pairs happen to start under 90 degrees apart ), so the
/// `cos_half_theta < 0` branch -- and therefore `q2` -- was never exercised by any test.
///
/// ## Fix Applied
/// BUG-194 replaced every use of `*other` after the hemisphere-correction block with `q2`
/// (`src/quaternion/arithmetics.rs`'s `slerp`), so the short-path angle is now always paired
/// with the correspondingly-corrected quaternion value.
///
/// ## Prevention
/// `q1` is the identity rotation and `q2_long` is a 270 degree rotation about Z -- the physical
/// rotation this represents is equivalent to a -90 degree rotation about Z, so the correct
/// halfway ( `s = 0.5` ) point is a -45 degree rotation about Z, not the +135 degree point a
/// naive long-path blend would produce. This test asserts the exact expected quaternion
/// (hand-derived via the corrected algorithm, confirmed unit-length) and, separately, that the
/// result is unit-length at all -- the pre-fix defect produced a magnitude around 0.41 for this
/// exact input, not 1.0.
///
/// ## Pitfall
/// A hemisphere-correction block that computes a corrected value into a new binding but never
/// routes that binding into the function's actual return expressions is a silent no-op --
/// nothing type-checks or panics, the corrected value is simply discarded. Any test suite
/// exercising only same-hemisphere inputs cannot detect this, since the correction path is
/// never taken at all in that regime.
// test_kind: bug_reproducer(BUG-194)
#[ test ]
fn test_slerp_negative_dot_product_takes_short_path()
{
  use the_module::QuatF64;

  // Identity rotation.
  let q1 = QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] );
  // 270 degree rotation about Z -- physically equivalent to -90 degrees about Z.
  let half = 270.0_f64.to_radians() / 2.0;
  let q2_long = QuatF64::from( [ 0.0, 0.0, half.sin(), half.cos() ] );
  assert!( q1.dot( &q2_long ) < 0.0, "fixture must exercise the negative-dot-product branch" );

  let got = q1.slerp( &q2_long, 0.5 );

  // Halfway along the short path ( 0 deg -> -90 deg ) is -45 degrees about Z.
  let neg45_half = -45.0_f64.to_radians() / 2.0;
  let exp = QuatF64::from( [ 0.0, 0.0, neg45_half.sin(), neg45_half.cos() ] );
  assert_abs_diff_eq!( got, exp, epsilon = 1e-9 );

  let len_sq = got.dot( &got );
  assert!( ( len_sq - 1.0 ).abs() < 1e-9, "slerp result must be unit-length, got squared length {len_sq}" );
}
