//! Integration tests for `primitive_generation::contours_to_fill_geometry`.
//!
//! Covers the TASK-018 doc-contradicting silent failure fix: the function's
//! doc comment promises `Returns None ... if the triangulation process
//! fails`, but the triangulation failure path silently skipped the failed
//! body and kept going instead of honoring that contract.

#[ cfg( test ) ]
mod tests
{
  use primitive_generation::contours_to_fill_geometry;

  /// Builds a single closed contour of `point_count` points, roughly on a
  /// circle, then corrupts one coordinate with a non-finite value. Needs at
  /// least 40 points (80 flat f64 coordinates) so `earcutr::earcut`'s
  /// internal `usehash` z-order path stays enabled -- that path is the one
  /// that actually casts each coordinate through `zorder()`, which is where
  /// the non-finite value makes the triangulation call return `Err`.
  /// Verified directly against `earcutr` 0.5.0 (the workspace-pinned
  /// version) before writing this test: with fewer than 40 points, `usehash`
  /// is forced off and the same corrupted contour instead comes back `Ok`
  /// (see the fix's Why Not Caught section below for why that matters).
  // `point_count` is always a small handful of test points (never anywhere
  // near 2^24), so the usize -> f32 cast below cannot lose precision in
  // practice.
  #[ allow( clippy::cast_precision_loss ) ]
  fn contour_with_non_finite_coordinate( point_count : usize, bad_coord : f32 ) -> Vec< [ f32; 2 ] >
  {
    let mut contour = Vec::with_capacity( point_count );
    for i in 0..point_count
    {
      let t = i as f32 * 0.1;
      contour.push( [ t.cos() * 10.0, t.sin() * 10.0 ] );
    }
    contour[ 10 ] = [ bad_coord, contour[ 10 ][ 1 ] ];
    contour
  }

  /// ## Root Cause
  /// `contours_to_fill_geometry`'s doc comment promises `Returns ... None
  /// if the input contours is empty or if the triangulation process fails`.
  /// The triangulation call, however, was written as:
  /// `let Ok( body_indices ) = earcutr::earcut( .. ) else { continue };`
  /// inside the per-body loop -- so a triangulation failure for one body
  /// was silently swallowed (`continue` to the next body) instead of
  /// propagating as the documented `None`. When the failing body is the
  /// only body, the function still returns `Some( PrimitiveData )`, just
  /// with empty `positions`/`indices`, directly contradicting its own
  /// documented contract.
  ///
  /// ## Why Not Caught
  /// No existing test exercised a contour that makes `earcutr::earcut`
  /// itself return `Err`. `earcutr` 0.5.0 only reaches its error path
  /// (`Coord::zorder`'s cast-to-integer, used by the z-order hashing
  /// optimization) when the contour has >= 40 points; smaller degenerate
  /// contours (a single point, two coincident points, all-collinear points)
  /// were confirmed (via direct probing of `earcutr::earcut`) to still
  /// return `Ok` with an empty triangle list rather than `Err`, so a
  /// small/simple test contour would not have exercised this path at all.
  ///
  /// ## Fix Applied
  /// Changed the triangulation failure branch in
  /// `contours_to_fill_geometry` (`src/primitive.rs`) from `continue` to
  /// `return None`, so a triangulation failure now honors the documented
  /// contract instead of silently producing partial/empty geometry wrapped
  /// in a misleading `Some`.
  ///
  /// ## Prevention
  /// When a doc comment states an explicit failure contract ("Returns None
  /// if X fails"), every code path that detects that failure must actually
  /// return the documented failure value -- `continue`/skip inside a loop
  /// is an easy way to accidentally downgrade a hard failure into a silent
  /// partial success.
  ///
  /// ## Pitfall
  /// A `let-else` failure branch inside a `for` loop reads as "handle this
  /// item's failure and move on," which is correct for per-item recoverable
  /// errors but wrong when the function's own contract says the whole
  /// operation must fail -- always check the doc comment's stated contract
  /// before choosing `continue` over `return`.
  #[ test ]
  fn contours_to_fill_geometry_returns_none_when_triangulation_fails()
  {
    let contour = contour_with_non_finite_coordinate( 45, f32::NAN );
    let result = contours_to_fill_geometry( &[ contour ] );
    assert!
    (
      result.is_none(),
      "doc comment promises None when triangulation fails; got Some(..) instead"
    );
  }

  /// Regression guard: a well-formed contour (no non-finite coordinates)
  /// must still triangulate successfully and produce non-empty geometry --
  /// the fix must not turn every call into `None`.
  #[ test ]
  fn contours_to_fill_geometry_accepts_well_formed_contour()
  {
    let contour = vec!
    [
      [ 0.0_f32, 0.0_f32 ],
      [ 4.0_f32, 0.0_f32 ],
      [ 4.0_f32, 4.0_f32 ],
      [ 0.0_f32, 4.0_f32 ],
    ];
    let result = contours_to_fill_geometry( &[ contour ] );
    let primitive = result.expect( "a valid square contour must still produce geometry" );
    let attributes = primitive.attributes.expect( "geometry must have attributes" );
    assert!( !attributes.borrow().indices.is_empty(), "expected at least one triangle" );
  }
}
