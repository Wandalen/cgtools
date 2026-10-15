//! Regression coverage for BUG-491: `Join::Round/Miter/Bevel` constructed with a `0`
//! `column_precision` silently produced *empty* geometry (zero vertices/uvs) instead of either
//! erroring or degenerating to a minimum valid shape.
//!
//! An earlier hypothesis (matching the bug class already fixed via BUG-236/BUG-237's
//! `.max( 1 )` guards) assumed this produced NaN vertex data directly. Empirical probing
//! (`cargo nextest run -E 'test(probe_zero_precision_combinations)'` against the pre-fix code,
//! see BUG-491's own report for the raw output) disproved that: every vertex-buffer-populating
//! loop in all three functions is bounded by the *exclusive* range `0..column_precision`, which
//! is empty when `column_precision == 0` -- so the NaN that *is* genuinely computed internally
//! (`k as f32 / column_precision as f32` with `column_precision == 0`, unrescued by any
//! `.max( .. )`) is written into a scratch row buffer that the empty read range then never
//! reads from. The result is silently empty output, not NaN output.

use line_tools::Join;

/// ## Root Cause
/// All three functions compute `column_list` entries via `k as f32 / column_precision as f32`
/// with no floor on `column_precision` -- for `column_precision == 0` this is `0.0 / 0.0`,
/// which is NaN (IEEE 754 division never panics on a zero divisor). Every loop that reads
/// `column_list`/`vertex_row_list` back out into the returned `verticies`/`uvs` buffers is
/// bounded by the exclusive range `0..column_precision`, though -- which is empty when
/// `column_precision == 0` -- so the NaN value never actually reaches the returned geometry;
/// the function instead returns completely empty `Vec`s. `row_precision == 0` alone does not
/// NaN either: `rm`'s `1.0 - ( i as f32 / row_precision as f32 )` is rescued by the pre-existing
/// `.max( center_offset )` call (`f32::max` returns the non-NaN argument when one side is NaN),
/// leaving a valid, if maximally thin, single-row shape.
/// ## Why Not Caught
/// No existing test constructed any `Join` variant with a `0` precision component; the `Join`
/// variants' own doc comments describe the two `usize` fields only as "level of triangualtion
/// in the horizontal and vertical directions", stating no minimum. The masking described above
/// is also fragile, not intentional: nothing documents or tests that the exclusive read-range
/// is relied on to suppress the internal NaN, so a superficially reasonable future change (e.g.
/// widening a `0..column_precision` read loop to `0..=column_precision` to "include the last
/// segment", not realizing it currently doubles as an accidental NaN guard) would silently
/// reintroduce genuine NaN into the returned geometry.
/// ## Fix Applied
/// `row_precision`/`column_precision` are now floored with `.max( 1 )` in all three functions,
/// mirroring `caps.rs::round_cap_geometry` (BUG-236) and `helpers.rs::circle_geometry`
/// (BUG-237)'s established convention for this exact parameter shape. This removes reliance on
/// the accidental exclusive-range masking and makes a `0`-precision join degenerate to a valid
/// minimum (1-segment) shape instead of silently vanishing.
/// ## Prevention
/// These tests construct each `Join` variant through the crate's real public entry point
/// (`Join::geometry`) with `column_precision == 0` and assert the returned geometry is
/// non-empty and fully finite -- failing loudly if either the empty-output defect or a
/// reintroduced NaN ever returns.
/// ## Pitfall
/// An exclusion range that happens to prevent a downstream NaN from reaching callers is not a
/// substitute for flooring the value at its source: it silently changes the failure mode from
/// "NaN" to "empty output" rather than fixing anything, and depends on every future edit to
/// every read-loop in the function never touching that exact range shape.
// test_kind: bug_reproducer(BUG-491)
#[ test ]
fn round_geometry_with_zero_column_precision_does_not_produce_empty_geometry_bug_491()
{
  let ( vertices, _indices, uvs, len ) = Join::Round( 8, 0 ).geometry();

  assert!( len > 0, "expected Join::Round( 8, 0 ) to degenerate to a minimum valid shape, got empty geometry (len={len})" );
  assert!( !vertices.is_empty(), "expected non-empty vertex data for Join::Round( 8, 0 )" );
  assert!( vertices.iter().all( | v | v.is_finite() ), "expected every vertex component to be finite for Join::Round( 8, 0 ), got {vertices:?}" );
  assert!( uvs.iter().all( | v | v.is_finite() ), "expected every uv component to be finite for Join::Round( 8, 0 ), got {uvs:?}" );
}

// test_kind: bug_reproducer(BUG-491)
#[ test ]
fn miter_geometry_with_zero_column_precision_does_not_produce_empty_geometry_bug_491()
{
  let ( vertices, _indices, uvs, len ) = Join::Miter( 8, 0 ).geometry();

  assert!( len > 0, "expected Join::Miter( 8, 0 ) to degenerate to a minimum valid shape, got empty geometry (len={len})" );
  assert!( !vertices.is_empty(), "expected non-empty vertex data for Join::Miter( 8, 0 )" );
  assert!( vertices.iter().all( | v | v.is_finite() ), "expected every vertex component to be finite for Join::Miter( 8, 0 ), got {vertices:?}" );
  assert!( uvs.iter().all( | v | v.is_finite() ), "expected every uv component to be finite for Join::Miter( 8, 0 ), got {uvs:?}" );
}

// test_kind: bug_reproducer(BUG-491)
#[ test ]
fn bevel_geometry_with_zero_column_precision_does_not_produce_empty_geometry_bug_491()
{
  let ( vertices, _indices, uvs, len ) = Join::Bevel( 8, 0 ).geometry();

  assert!( len > 0, "expected Join::Bevel( 8, 0 ) to degenerate to a minimum valid shape, got empty geometry (len={len})" );
  assert!( !vertices.is_empty(), "expected non-empty vertex data for Join::Bevel( 8, 0 )" );
  assert!( vertices.iter().all( | v | v.is_finite() ), "expected every vertex component to be finite for Join::Bevel( 8, 0 ), got {vertices:?}" );
  assert!( uvs.iter().all( | v | v.is_finite() ), "expected every uv component to be finite for Join::Bevel( 8, 0 ), got {uvs:?}" );
}

/// `row_precision == 0` alone was already finite pre-fix (rescued by the existing
/// `.max( center_offset )` on `rm`) -- this confirms the `.max( 1 )` floor doesn't regress that
/// case, and that the fully-degenerate `(0, 0)` combination is finite and non-empty post-fix
/// across all three join kinds.
#[ test ]
fn join_geometry_with_zero_row_precision_or_fully_degenerate_precision_is_finite_and_non_empty()
{
  for join in
  [
    Join::Round( 0, 8 ), Join::Round( 0, 0 ),
    Join::Miter( 0, 8 ), Join::Miter( 0, 0 ),
    Join::Bevel( 0, 8 ), Join::Bevel( 0, 0 ),
  ]
  {
    let ( vertices, _indices, uvs, len ) = join.geometry();

    assert!( len > 0, "expected {join:?} to degenerate to a minimum valid shape, got empty geometry (len={len})" );
    assert!( vertices.iter().all( | v | v.is_finite() ), "expected every vertex component to be finite for {join:?}, got {vertices:?}" );
    assert!( uvs.iter().all( | v | v.is_finite() ), "expected every uv component to be finite for {join:?}, got {uvs:?}" );
  }
}

/// Confirms the `.max( 1 )` floor doesn't perturb already-valid, ordinary precision values.
#[ test ]
fn join_geometry_with_ordinary_precision_is_unaffected_by_the_floor()
{
  for join in [ Join::Round( 16, 8 ), Join::Miter( 16, 8 ), Join::Bevel( 16, 8 ) ]
  {
    let ( vertices, _indices, uvs, len ) = join.geometry();

    assert!( len > 0, "expected {join:?} to produce non-empty geometry" );
    assert!( vertices.iter().all( | v | v.is_finite() ), "expected every vertex component to be finite for {join:?}" );
    assert!( uvs.iter().all( | v | v.is_finite() ), "expected every uv component to be finite for {join:?}" );
  }
}
