use ndarray_cg::approx::{ assert_abs_diff_eq, assert_relative_eq };

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

  let exp = QuatF64::from( [ -0.07189765816207114, 0.5401439887921695, 0.46824633063009835, 0.6955721184564136 ] );
  assert_abs_diff_eq!( q1.slerp( &q2, 0.3 ), exp );

  let exp = QuatF64::from( [ -0.23905007006563106, 0.626821944003501, 0.38777187393787, 0.6321252156811978 ] );
  assert_abs_diff_eq!( q1.slerp( &q2, 0.5 ), exp );

  let exp = QuatF64::from( [ -0.5332219143045811, 0.7111022022191557, 0.17788028791457453, 0.4223347620973825 ] );
  assert_abs_diff_eq!( q1.slerp( &q2, 0.9 ), exp );


  let q1 = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ] ).normalize();
  let q2 = QuatF64::from( [ 0.9, 2.0, 3.0, 4.0 ] ).normalize();

  let exp = QuatF64::from( [ 0.18080329575292692, 0.3652698894950247, 0.5479048342425371, 0.7305397789900494 ] );

  assert_abs_diff_eq!( q1.slerp( &q2, 0.1 ), exp );

  let exp = QuatF64::from( [ 0.17371392923604712, 0.36574411080923475, 0.548616166213852, 0.7314882216184695 ] );
  assert_abs_diff_eq!( q1.slerp( &q2, 0.5 ), exp );

  let exp = QuatF64::from( [ 0.1657276357739934, 0.3662549236088048, 0.549382385413207, 0.7325098472176096 ] );
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
#[ should_panic ]
fn test_quat_from_slice_wrong_length()
{
  use the_module::QuatF64;

  let _ = QuatF64::from( [ 1.0, 2.0, 3.0 ].as_slice() );
}
