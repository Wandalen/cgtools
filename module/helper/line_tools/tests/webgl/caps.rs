//! Regression coverage for BUG-236: `Cap::Round( 0 ).geometry()` divided by a zero `segments`
//! count, pushing a NaN vertex into the returned cap mesh instead of erroring or degenerating
//! gracefully.

use line_tools::Cap;

/// ## Root Cause
/// `round_cap_geometry`'s vertex loop computes `i as f32 / segments as f32` to angularly space
/// the semicircle's rim vertices -- for `segments == 0` this divides by zero, and `f32` division
/// never panics on a zero divisor (IEEE 754: `0.0 / 0.0` is `NaN`), so a NaN vertex was pushed
/// straight into the returned geometry with no error and no diagnostic.
///
/// ## Why Not Caught
/// No existing test constructed `Cap::Round` with `0` segments; `Cap::Round`'s own doc comment
/// only says "the usize parameter specifies the number of segments", stating no minimum.
///
/// ## Fix Applied
/// `round_cap_geometry` now floors its `segments` argument with `.max( 1 )` before it's used as
/// a division's divisor, mirroring `Tween::new`/`Step::new`'s established `.max( .. )` guard for
/// the identical defect shape (BUG-142/BUG-233).
///
/// ## Prevention
/// This test constructs `Cap::Round( 0 )` through the crate's real public entry point
/// (`Cap::geometry`) and asserts every returned vertex component is finite.
///
/// ## Pitfall
/// `f32`/`f64` division never panics on a zero divisor -- it silently returns `NaN` or `±inf` --
/// so any entry-point parameter later used as a division's divisor needs its own explicit floor;
/// the presence of a sibling guard elsewhere in the codebase (`Tween::new`) does not protect a
/// differently-shaped call site with the same defect.
// test_kind: bug_reproducer(BUG-236)
#[ test ]
fn round_cap_geometry_with_zero_segments_does_not_produce_nan_bug_236()
{
  let ( vertices, indices, len ) = Cap::Round( 0 ).geometry();

  assert!( vertices.iter().all( | v | v.is_finite() ), "expected every vertex component to be finite for Cap::Round( 0 ), got {vertices:?}" );
  assert!( len > 0, "expected at least the center vertex to be present, got len={len}" );
  assert!( !indices.is_empty(), "expected at least one triangle to be generated for a degenerate 0-segment round cap, got {indices:?}" );
}

// Confirms the floor doesn't perturb an ordinary, already-valid segment count.
#[ test ]
fn round_cap_geometry_with_ordinary_segments_is_unaffected_by_the_floor()
{
  let ( vertices, indices, len ) = Cap::Round( 8 ).geometry();

  assert!( vertices.iter().all( | v | v.is_finite() ), "expected every vertex component to be finite for Cap::Round( 8 ), got {vertices:?}" );
  assert_eq!( len, 10, "Cap::Round( 8 ) should produce 1 center + 9 rim vertices (0..=8 inclusive)" );
  assert_eq!( indices.len() / 3, 8, "Cap::Round( 8 ) should produce 8 triangles" );
}
