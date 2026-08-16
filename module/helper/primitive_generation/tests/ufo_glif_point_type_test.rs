//! Integration tests for `primitive_generation::text::ufo::Glyph::from_glif`.
//!
//! Covers BUG-128: the `.glif` XML parser matched point-type attributes against
//! `b"typ"` instead of `b"type"`, so no point's declared type was ever recognized.

#[ cfg( test ) ]
mod tests
{
  use primitive_generation::text::ufo::Glyph;

  /// A single quadratic curve segment (`move` -> `offcurve` -> `curve`), the
  /// minimal shape that distinguishes correct point-type parsing from the bug:
  /// verified directly against `norad` 0.18.4's `Contour::to_kurbo` (which the
  /// crate calls internally) that this exact coordinate set flattens to 14
  /// points when types are honored, vs 3 (one per point, each its own
  /// disconnected `MoveTo`) when every point is misread as `PointType::Move`.
  const GLIF_WITH_CURVE : &[ u8 ] = br#"<?xml version="1.0" encoding="UTF-8"?>
<glyph name="test" format="2">
  <outline>
    <contour>
      <point x="0" y="0" type="move"/>
      <point x="50" y="100" type="offcurve"/>
      <point x="100" y="0" type="curve"/>
    </contour>
  </outline>
</glyph>"#;

  // test_kind: bug_reproducer(BUG-128)
  /// ## Root Cause
  /// `Glyph::from_glif`'s point-attribute loop matched
  /// `match attr.key.0 { ... b"typ" => { .. typ = t; } ... }` -- a one-letter
  /// typo of the UFO/glif spec's real attribute name, `type` (confirmed against
  /// `norad` 0.18.4's own glif parser, `src/glyph/parse.rs`, which reads exactly
  /// `b"type"`). `b"typ"` can never match a real `.glif` file's `type="..."`
  /// attribute, so the `typ` loop variable never left its `PointType::Move`
  /// default. Every point was parsed as `PointType::Move` regardless of its
  /// declared type, so `norad::Contour::to_kurbo` (per its own match on
  /// `pt.typ`) turned every non-first point into a fresh, disconnected
  /// `MoveTo` instead of the intended `LineTo`/`QuadTo`/`CurveTo` -- silently
  /// producing a degenerate few-point path instead of the real glyph outline.
  ///
  /// ## Why Not Caught
  /// No existing test exercised `Glyph::from_glif` at all (the crate's only
  /// public entry point into UFO loading, `fonts_load`, reads real `.ufo`
  /// directories via an async, browser-only file loader with no test fixture
  /// wired up) -- the byte-level `.glif` parser was entirely untested.
  ///
  /// ## Fix Applied
  /// Changed the match arm in `Glyph::from_glif` (`src/text/ufo.rs`) from
  /// `b"typ"` to `b"type"`.
  ///
  /// ## Prevention
  /// An unmatched byte-string arm in a `match` with a `_ => {}` catch-all fails
  /// silently -- it never panics or errors, it just never fires. Cross-check
  /// hardcoded attribute-name literals against the format spec or a reference
  /// parser (`norad`, in this case), not just internal self-consistency.
  ///
  /// ## Pitfall
  /// A parser that silently accepts and misclassifies every input (rather than
  /// erroring) gives no signal that a match arm is dead -- the bug is only
  /// visible in the shape of the *output*, not in any error path.
  #[ test ]
  fn from_glif_honors_the_declared_curve_point_type()
  {
    let glyph = Glyph::from_glif( GLIF_WITH_CURVE, 'a' ).expect( "well-formed glif must parse" );
    let point_count : usize = glyph.contours().iter().map( Vec::len ).sum();

    assert!
    (
      point_count > 3,
      "expected the curve to be flattened into many points (14, per direct \
       norad probe); got {point_count} -- consistent with every point being \
       misread as PointType::Move (3 disconnected single points)"
    );
  }
}
