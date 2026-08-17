//! Integration tests for `primitive_generation::text::ufo::Glyph::from_glif`.
//!
//! Covers BUG-128: the `.glif` XML parser matched point-type attributes against
//! `b"typ"` instead of `b"type"`, so no point's declared type was ever recognized.
//!
//! Covers BUG-215: the `typ` accumulator was scoped per-contour instead of
//! per-point, so an untyped point (the spec-correct way to write an off-curve
//! control point) silently inherited the previous point's type.

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

  /// A point with no `type` attribute at all -- the normal, spec-correct way to
  /// encode an off-curve bezier control point in UFO/glif (confirmed against
  /// `norad` 0.18.4's own `parse_point`, which defaults to `PointType::OffCurve`
  /// absent the attribute). The first point's explicit `type="move"` primes the
  /// bug: a per-contour (rather than per-point) `typ` accumulator would carry
  /// `Move` over onto the untyped second point instead of defaulting fresh.
  const GLIF_WITH_OMITTED_TYPE_ATTRIBUTE : &[ u8 ] = br#"<?xml version="1.0" encoding="UTF-8"?>
<glyph name="test" format="2">
  <outline>
    <contour>
      <point x="0" y="0" type="move"/>
      <point x="50" y="100"/>
      <point x="100" y="0" type="curve"/>
    </contour>
  </outline>
</glyph>"#;

  // test_kind: bug_reproducer(BUG-215)
  /// ## Root Cause
  /// `Glyph::from_glif`'s `typ` accumulator was declared once per *contour*
  /// (`let mut typ = PointType::Move;` outside the point-parsing loop, reset
  /// only at `</contour>`), not once per *point*. A point that omits the
  /// `type` attribute -- the normal way to write an off-curve control point --
  /// left `typ` untouched by the attribute-parsing loop, so it silently
  /// inherited whatever type the *previous* point in the same contour had,
  /// instead of defaulting to `PointType::OffCurve` (confirmed against `norad`
  /// 0.18.4's own reference parser, `glyph/parse.rs::parse_point`, which
  /// declares its `typ` default fresh inside the per-point function itself).
  /// In this fixture, the untyped second point inherits `Move` from the first
  /// point instead of defaulting to `OffCurve`, so it never reaches the `offs`
  /// queue `norad::Contour::to_kurbo`'s `Curve` arm needs -- the third point's
  /// `Curve` type then finds an empty queue and `to_kurbo` returns
  /// `Err(BadPoint)`, which `from_glif` maps straight to `None`.
  ///
  /// ## Why Not Caught
  /// BUG-128's own regression test (`from_glif_honors_the_declared_curve_point_type`,
  /// above) only exercises points that all carry an explicit `type` attribute --
  /// it never constructs a contour where a later point omits `type` after an
  /// earlier point had one, so it could not observe the accumulator leaking
  /// across points.
  ///
  /// ## Fix Applied
  /// Moved the `typ` declaration inside the point-parsing match arm (fresh
  /// `let mut typ = PointType::OffCurve;` per point, matching `norad`'s own
  /// default) and removed the now-unnecessary per-contour reset at `</contour>`.
  ///
  /// ## Prevention
  /// A state-machine accumulator that must reset per-iteration needs its
  /// `let mut` declared *inside* the loop body at the right granularity --
  /// declaring it one level too high silently widens its lifetime to the next
  /// coarser loop level.
  ///
  /// ## Pitfall
  /// The leak is invisible in any fixture where every point happens to carry
  /// an explicit `type` attribute -- it only manifests when a point *omits*
  /// the attribute after a differently-typed point earlier in the same
  /// contour, which is the spec-correct, common case for off-curve points.
  #[ test ]
  fn from_glif_defaults_an_untyped_point_to_offcurve_not_the_previous_points_type()
  {
    let glyph = Glyph::from_glif( GLIF_WITH_OMITTED_TYPE_ATTRIBUTE, 'a' );

    assert!
    (
      glyph.is_some(),
      "an untyped point following a `type=\"move\"` point must default to \
       OffCurve and feed the following Curve point's control-point queue; \
       got None, consistent with the untyped point inheriting Move instead \
       and leaving norad::Contour::to_kurbo's Curve arm with an empty queue"
    );
  }
}
