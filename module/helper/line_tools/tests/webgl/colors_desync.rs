//! Regression coverage for BUG-492: `d3::Line`'s `colors` `VecDeque` is fully independent from
//! `points`/`distances` -- every `point_*`/`points_*` add/remove method (from
//! `impl_basic_line!`) keeps `distances` in lockstep, but none of them touch `colors`, despite
//! doc comments describing `colors` entries as belonging "to a point with the same index".

use line_tools::d3::Line;
use line_tools::d3::line::colors_length_consistency_check;

/// Confirms the desync described above is genuinely reachable through the crate's public API,
/// not merely a hypothetical -- `point_remove_front()` shrinks `points` without touching
/// `colors`, so their public, externally-observable lengths (`points_get()`/`colors_get()`)
/// diverge after a single call.
///
/// ## Root Cause
/// See `colors_length_consistency_check`'s own doc comment in `src/d3/line.rs`.
/// ## Why Not Caught
/// No existing test constructed a `Line`, added matching points and colors, and then removed
/// only one side -- `points`/`distances`' own lockstep invariant is well covered, but nothing
/// exercised `colors` against it.
/// ## Fix Applied
/// See `colors_length_consistency_check`'s own doc comment and its new call site in
/// `mesh_update` (guarded behind `colors_changed && vertex_color_use`, immediately before the
/// colors buffer upload).
/// ## Prevention
/// This test demonstrates the desync condition itself is real and public-API-reachable, since
/// `mesh_update`'s own guard cannot be exercised here (it requires a live
/// `WebGl2RenderingContext`, which this crate has no test infrastructure to construct
/// natively -- see the sibling test below for the guard's own logic, tested directly).
/// ## Pitfall
/// A doc comment asserting index-correspondence between two independently-mutable collections
/// is not an enforced invariant -- only a check at the point of consumption (or, more robustly,
/// folding both into one structure) actually prevents the desync from being observable.
// test_kind: bug_reproducer(BUG-492)
#[ test ]
fn point_remove_front_without_matching_color_remove_desyncs_colors_and_points_bug_492()
{
  let mut line = Line::default();

  line.point_add_back( &[ 0.0_f32, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0_f32, 0.0, 0.0 ] );
  line.point_add_back( &[ 2.0_f32, 0.0, 0.0 ] );

  line.color_add_back( [ 1.0_f32, 0.0, 0.0 ] );
  line.color_add_back( [ 0.0_f32, 1.0, 0.0 ] );
  line.color_add_back( [ 0.0_f32, 0.0, 1.0 ] );

  assert_eq!( line.points_get().len(), 3, "test setup: expected 3 points before the desyncing call" );
  assert_eq!( line.colors_get().len(), 3, "test setup: expected 3 colors before the desyncing call" );

  line.point_remove_front();

  assert_eq!( line.points_get().len(), 2, "point_remove_front() should have removed exactly one point" );
  assert_eq!
  (
    line.colors_get().len(), 3,
    "color_remove_front() was never called, so colors should still hold all 3 entries -- \
    the resulting length mismatch (2 vs 3) is exactly BUG-492's desync"
  );
  assert_ne!
  (
    line.points_get().len(), line.colors_get().len(),
    "points/colors have desynced after point_remove_front() with no matching color_remove_front()"
  );
}

/// Direct test of the new length-consistency guard itself (`colors_length_consistency_check`),
/// exercised without a live GL context since it's a pure function over the two lengths.
#[ test ]
fn colors_length_consistency_check_rejects_mismatched_lengths_and_accepts_matched_ones()
{
  assert!
  (
    colors_length_consistency_check( 3, 3 ).is_ok(),
    "matched lengths should pass the consistency check"
  );
  assert!
  (
    colors_length_consistency_check( 0, 0 ).is_ok(),
    "matched (empty) lengths should pass the consistency check"
  );
  assert!
  (
    colors_length_consistency_check( 3, 2 ).is_err(),
    "colors.len() > points.len() should fail the consistency check"
  );
  assert!
  (
    colors_length_consistency_check( 2, 3 ).is_err(),
    "colors.len() < points.len() (BUG-492's exact shape, from point_remove_front() alone) should fail the consistency check"
  );
}
