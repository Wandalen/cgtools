//! Integration tests for `primitive_generation::text::ufo::Font`'s `max_size`
//! bounding-box union.
//!
//! Covers BUG-216: the union loops in `Font::from_glyphs` and `Font::new` used
//! `Vector`'s `<`/`>` operators (lexicographic `Ord`/`PartialOrd`, comparing the
//! x component first) instead of its component-wise `.min()`/`.max()` methods.

#[ cfg( test ) ]
mod tests
{
  use primitive_generation::text::ufo::{ Font, Glyph };

  /// Builds minimal `.glif` bytes for a `move`/`line`/`line` triangle whose raw
  /// bounding box is exactly `x = [0, width]`, `y = [0, height]` -- independent
  /// width/height control, with no curve/off-curve points to keep this test
  /// orthogonal to BUG-215's per-point `typ` fix.
  fn glif_triangle_bytes( width : f64, height : f64 ) -> Vec< u8 >
  {
    format!
    (
      r#"<?xml version="1.0" encoding="UTF-8"?>
<glyph name="test" format="2">
  <outline>
    <contour>
      <point x="0" y="0" type="move"/>
      <point x="{width}" y="0" type="line"/>
      <point x="{half}" y="{height}" type="line"/>
    </contour>
  </outline>
</glyph>"#,
      width = width,
      half = width / 2.0,
      height = height
    )
    .into_bytes()
  }

  // test_kind: bug_reproducer(BUG-216)
  /// ## Root Cause
  /// `Font::from_glyphs`'s (and `Font::new`'s identical) union loop compared
  /// glyph bounding boxes via `if min > glyph.bounding_box.min { min = ...; }`
  /// / the symmetric `<` for `max`. `Vector`'s `PartialOrd`/`Ord` impls
  /// (`ndarray_cg::vector::general`) delegate straight to `[E; N]`'s
  /// lexicographic array comparison -- decided entirely by the x component,
  /// only falling through to y/z to break an x-tie -- not a component-wise
  /// per-axis min/max. Whenever one glyph's bbox has a more extreme x in both
  /// directions than another's, that glyph's *entire* min/max vector wins the
  /// comparison wholesale, discarding the other glyph's y-extent even where it
  /// was more extreme. Confirmed against `Vector::min`/`Vector::max`
  /// (`ndarray_cg::vector::arithmetics`), the correct component-wise methods
  /// this exact dependency's own `BoundingBox::compute`/`compute2d` already
  /// uses for the identical kind of union.
  ///
  /// Reproducer: glyph `t` (tall/narrow) has centered bbox
  /// `min = (-1, -5, 0)`, `max = (1, 5, 0)`; glyph `w` (wide/short) has
  /// `min = (-3, -1, 0)`, `max = (3, 1, 0)`. `w`'s x-extent dominates `t`'s in
  /// both directions, so lexicographic comparison lets `w`'s vector win
  /// wholesale regardless of iteration order, producing `min.y = -1` /
  /// `max.y = 1` instead of the true union's `min.y = -5` / `max.y = 5`.
  ///
  /// ## Why Not Caught
  /// `Font::max_size` had no public accessor at all before this fix -- nothing
  /// in the crate's public API surface could observe its value, so the union
  /// arithmetic was untested (`text_to_mesh`'s only read of it is gated behind
  /// `glyph.body`, which nothing in the public API populates for a
  /// `Font::from_glyphs`-built font).
  ///
  /// ## Fix Applied
  /// Changed both union loops (`Font::from_glyphs`, `Font::new`) from
  /// `if min > glyph.bounding_box.min { min = ...; }` to
  /// `min = min.min( glyph.bounding_box.min );` (and the `max` symmetric),
  /// using `Vector`'s component-wise methods instead of its ordering operator.
  /// Also added `Font::max_size()`, a minimal read-only accessor mirroring the
  /// existing `Glyph::contours()` precedent, to make the union independently
  /// testable.
  ///
  /// ## Prevention
  /// `Vector` intentionally supports two unrelated orderings: a total,
  /// lexicographic one (via `<`/`>`/`Ord`, useful for e.g. canonical sort
  /// keys) and a component-wise one (via `.min()`/`.max()`, useful for
  /// geometry). Reaching for the operator instead of the method silently
  /// selects the wrong one for AABB math, and both compile and typecheck
  /// identically -- there is no error to catch this at the call site.
  ///
  /// ## Pitfall
  /// The bug is invisible whenever the bboxes being unioned happen to already
  /// agree on which one has the more extreme x *and* y in the same direction
  /// (the common case for same-script glyphs of similar proportions) -- it
  /// only surfaces when one glyph's x-extent and another's y-extent are the
  /// two that matter, exactly the tall-vs-wide shape this test constructs.
  #[ test ]
  fn from_glyphs_unions_bounding_boxes_component_wise_not_lexicographically()
  {
    let font = Font::from_glyphs
    (
      [
        ( 't', Glyph::from_glif( &glif_triangle_bytes( 2.0, 10.0 ), 't' ).expect( "tall glyph" ) ),
        ( 'w', Glyph::from_glif( &glif_triangle_bytes( 6.0, 2.0 ), 'w' ).expect( "wide glyph" ) ),
      ]
    );

    let bbox = font.max_size();

    assert!
    (
      ( bbox.min.y() - ( -5.0 ) ).abs() < 1e-5,
      "expected union min.y = -5 (the tall glyph's own extent); got {} -- \
       consistent with the wide glyph's x-dominant vector winning the \
       lexicographic comparison wholesale and discarding the tall glyph's y",
      bbox.min.y()
    );
    assert!
    (
      ( bbox.max.y() - 5.0 ).abs() < 1e-5,
      "expected union max.y = 5 (the tall glyph's own extent); got {} -- \
       consistent with the wide glyph's x-dominant vector winning the \
       lexicographic comparison wholesale and discarding the tall glyph's y",
      bbox.max.y()
    );
    assert!
    (
      ( bbox.min.x() - ( -3.0 ) ).abs() < 1e-5,
      "expected union min.x = -3 (the wide glyph's own extent); got {}",
      bbox.min.x()
    );
    assert!
    (
      ( bbox.max.x() - 3.0 ).abs() < 1e-5,
      "expected union max.x = 3 (the wide glyph's own extent); got {}",
      bbox.max.x()
    );
  }
}
