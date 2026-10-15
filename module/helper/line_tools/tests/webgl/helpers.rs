//! Regression coverage for BUG-237: `circle_geometry( 0 )` divided by a zero `segments` count,
//! pushing a NaN vertex into the returned circle mesh instead of erroring or degenerating
//! gracefully. Same defect shape as BUG-236's `round_cap_geometry`, found in the same scouting
//! pass, in a different public function.

use line_tools::helpers::circle_geometry;

/// ## Root Cause
/// `circle_geometry`'s vertex loop computes `wedge as f32 / segments as f32` to angularly space
/// the circle's rim vertices -- for `segments == 0` this divides by zero, and `f32` division
/// never panics on a zero divisor (IEEE 754: `0.0 / 0.0` is `NaN`), so a NaN vertex was pushed
/// straight into the returned geometry with no error and no diagnostic.
///
/// ## Why Not Caught
/// No existing test constructed `circle_geometry` with `0` segments; the function has no doc
/// comment stating a minimum, and this crate has no other caller of `circle_geometry` at all
/// (confirmed via a workspace-wide grep) -- the defect was reachable only through direct,
/// unprivileged use of the crate's own public API, which no test or in-tree caller exercised.
///
/// ## Fix Applied
/// `circle_geometry` now floors its `segments` argument with `.max( 1 )` before it's used as a
/// division's divisor, mirroring BUG-236's identical fix for `round_cap_geometry` in the same
/// crate.
///
/// ## Prevention
/// This test calls `circle_geometry( 0 )` directly and asserts every returned vertex component
/// is finite.
///
/// ## Pitfall
/// `f32`/`f64` division never panics on a zero divisor -- it silently returns `NaN` or `±inf` --
/// so any entry-point parameter later used as a division's divisor needs its own explicit floor.
/// A public function with zero in-tree callers is still a live defect: it is part of the crate's
/// exported API surface and reachable by any external consumer with no privilege beyond an
/// ordinary function call.
// test_kind: bug_reproducer(BUG-237)
#[ test ]
fn circle_geometry_with_zero_segments_does_not_produce_nan_bug_237()
{
  let positions = circle_geometry( 0 );

  assert!( !positions.is_empty(), "expected at least one vertex to be present, got {positions:?}" );
  assert!
  (
    positions.iter().all( | p | p[ 0 ].is_finite() && p[ 1 ].is_finite() ),
    "expected every vertex to be finite for circle_geometry( 0 ), got {positions:?}"
  );
}

// Confirms the floor doesn't perturb an ordinary, already-valid segment count.
#[ test ]
fn circle_geometry_with_ordinary_segments_is_unaffected_by_the_floor()
{
  let positions = circle_geometry( 8 );

  assert_eq!( positions.len(), 9, "circle_geometry( 8 ) should produce 9 vertices (0..=8 inclusive)" );
  assert!
  (
    positions.iter().all( | p | p[ 0 ].is_finite() && p[ 1 ].is_finite() ),
    "expected every vertex to be finite for circle_geometry( 8 ), got {positions:?}"
  );
}

// `circle_left_half_geometry`/`circle_right_half_geometry` use an exclusive `0..segments` range,
// a structurally different shape that already degenerates safely (zero iterations, empty output)
// for `segments == 0` -- this test documents and locks in that existing safe behavior so a future
// refactor toward `circle_geometry`'s inclusive-range shape doesn't silently reintroduce BUG-237.
#[ test ]
fn circle_half_geometry_with_zero_segments_stays_empty_and_finite()
{
  let left = line_tools::helpers::circle_left_half_geometry( 0 );
  let right = line_tools::helpers::circle_right_half_geometry( 0 );

  assert!( left.is_empty(), "expected circle_left_half_geometry( 0 ) to stay empty (exclusive range), got {left:?}" );
  assert!( right.is_empty(), "expected circle_right_half_geometry( 0 ) to stay empty (exclusive range), got {right:?}" );
}
