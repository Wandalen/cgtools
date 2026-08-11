//! Integration tests for `primitive_generation::curve_to_geometry`.
//!
//! Covers the TASK-018 precondition-gap fix: a curve containing a
//! zero-length segment (two consecutive points -- including the implicit
//! closing segment back to the first point -- that coincide) must be
//! rejected up front instead of silently producing `NaN`-filled geometry.

#[ cfg( test ) ]
mod tests
{
  use primitive_generation::curve_to_geometry;

  /// ## Root Cause
  /// `curve_to_geometry` builds every stroke segment -- including the
  /// implicit closing segment from the curve's last point back to its first
  /// point -- via `direction = ( end_point - start_point ).normalize()`,
  /// with no check that `end_point != start_point`. `F32x2::normalize()`
  /// (routed through `mdmath_core::vector::arithmetics::normalize`) divides
  /// each component by the vector's magnitude with no zero-length guard, so
  /// a zero-length segment computes `0.0 / 0.0`, which is `NaN` per
  /// IEEE-754. A single-point curve has no other segment than that
  /// self-closing one, so it is the most direct way to trigger it.
  ///
  /// ## Why Not Caught
  /// No prior test exercised `curve_to_geometry` with fewer than 2 points.
  /// Every existing caller (glyph outline stroking in `text/ufo.rs`) always
  /// supplied curves with visually distinct consecutive points, so the
  /// missing precondition was never exercised even though nothing in the
  /// function guarded against it.
  ///
  /// ## Fix Applied
  /// Added an upfront check in `curve_to_geometry` (`src/primitive.rs`)
  /// that rejects the curve -- returning `None`, the same failure channel
  /// the function already uses for an empty curve -- when any two
  /// consecutive points, including the wrap-around pair between the last
  /// and first point, are identical.
  ///
  /// ## Prevention
  /// Any function that normalizes a difference vector must validate the two
  /// inputs actually differ before calling `.normalize()`, since the vector
  /// math layer (`mdmath_core`/`ndarray_cg`) provides no such guard itself.
  ///
  /// ## Pitfall
  /// `Option`-returning geometry functions can silently turn a missing
  /// precondition into a `NaN`-filled `Some(..)` result instead of `None`.
  /// Degenerate/zero-length geometric inputs must be checked explicitly --
  /// never assume downstream math will fail loudly.
  #[ test ]
  fn curve_to_geometry_rejects_single_point_curve()
  {
    let curve = [ [ 1.0_f32, 2.0_f32 ] ];
    let result = curve_to_geometry( &curve, 1.0 );
    assert!
    (
      result.is_none(),
      "a single-point curve has no non-degenerate segment (its only segment \
      closes onto itself) and must be rejected, not produce NaN geometry"
    );
  }

  /// Narrower variant of the defect covered by
  /// `curve_to_geometry_rejects_single_point_curve`: interior points differ,
  /// but the curve is already explicitly closed (`first == last`), so only
  /// the implicit closing segment is degenerate. See that test's doc
  /// comment for the full Root Cause / Fix Applied / Prevention writeup.
  #[ test ]
  fn curve_to_geometry_rejects_explicitly_closed_curve_with_duplicate_endpoint()
  {
    let curve =
    [
      [ 0.0_f32, 0.0_f32 ],
      [ 1.0_f32, 0.0_f32 ],
      [ 1.0_f32, 1.0_f32 ],
      [ 0.0_f32, 0.0_f32 ],
    ];
    let result = curve_to_geometry( &curve, 1.0 );
    assert!
    (
      result.is_none(),
      "the closing segment collapses to zero length when the first and last \
      points coincide and must be rejected"
    );
  }

  /// Regression guard: a curve with no coincident consecutive points (in
  /// either direction, including the wrap-around) must still be accepted
  /// and must still produce finite geometry -- the new precondition check
  /// must not reject legitimate, non-degenerate input.
  #[ test ]
  fn curve_to_geometry_accepts_non_degenerate_curve_and_produces_finite_positions()
  {
    let curve =
    [
      [ 0.0_f32, 0.0_f32 ],
      [ 1.0_f32, 0.0_f32 ],
      [ 1.0_f32, 1.0_f32 ],
    ];
    let result = curve_to_geometry( &curve, 0.1 );
    let primitive = result.expect( "a valid 3-point curve must still produce geometry" );
    let attributes = primitive.attributes.expect( "geometry must have attributes" );
    let positions = attributes.borrow().positions.clone();
    assert!( !positions.is_empty(), "expected at least one rectangle segment" );
    for [ x, y, z ] in positions
    {
      assert!
      (
        x.is_finite() && y.is_finite() && z.is_finite(),
        "position must be finite, got [{x}, {y}, {z}]"
      );
    }
  }
}
