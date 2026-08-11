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
