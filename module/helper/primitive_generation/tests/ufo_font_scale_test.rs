//! Integration tests for `primitive_generation::text::ufo::glyph_rescale_factor`.
//!
//! Covers BUG-500: `Font::new` divided `scale / max_y` with no guard against
//! `max_y == 0.0`, producing `Infinity` and poisoning every glyph coordinate.

#[ cfg( test ) ]
mod tests
{
  use primitive_generation::text::ufo::glyph_rescale_factor;

  // test_kind: bug_reproducer(BUG-500)
  /// ## Root Cause
  /// `Font::new` computed `glyph.scale( scale / max_y )` where `max_y` is the
  /// tallest loaded glyph's height, seeded at `0.0` before the measuring loop.
  /// Whenever every loaded glyph is zero-height, or zero glyphs load at all
  /// (the seed then never gets raised), `max_y` stays `0.0` and the division
  /// produces `f32::INFINITY` -- Rust float division by zero does not panic,
  /// it silently returns `Infinity`.
  ///
  /// Reproducer: `glyph_rescale_factor( 250.0, 0.0 )` with the pre-fix
  /// unguarded `target_scale / max_y` expression evaluates to
  /// `250.0 / 0.0 == f32::INFINITY`.
  ///
  /// ## Why Not Caught
  /// The division lived inline inside the async, wasm-only, real-file-I/O
  /// `Font::new` -- not natively unit-testable at all (see the codebase's own
  /// established `Font::max_size()` / BUG-216 precedent for extracting a pure
  /// accessor to make otherwise GL/IO-bound logic independently testable).
  /// Nothing exercised the `max_y == 0.0` edge case because nothing could
  /// reach the arithmetic without also standing up real UFO font files.
  ///
  /// ## Fix Applied
  /// Extracted the division into a pure `glyph_rescale_factor( target_scale,
  /// max_y )` helper that floors `max_y` at `f32::EPSILON` via `.max(...)`
  /// before dividing, and wired `Font::new` to call it instead of the raw
  /// division. `glyph_rescale_factor` is independently testable with plain
  /// `f32` inputs -- no font loading, no WebGL context required.
  ///
  /// ## Prevention
  /// A "max of measured values" seeded at `0.0` is a safe default for the
  /// *measuring* loop itself, but that safety does not carry over to a
  /// *later* use of the result as a divisor -- the seed value must be
  /// re-examined at every place the measured max gets consumed, not just
  /// where it gets produced.
  ///
  /// ## Pitfall
  /// Float division by zero doesn't panic in Rust -- it silently returns
  /// `Infinity`/`NaN`, so a missing guard gives zero compile-time or runtime
  /// signal; the defect only surfaces later as corrupted (infinite/NaN)
  /// glyph geometry, far from its actual cause.
  #[ test ]
  fn zero_max_y_yields_finite_scale_not_infinity()
  {
    let factor = glyph_rescale_factor( 250.0, 0.0 );

    assert!
    (
      factor.is_finite(),
      "expected a finite rescale factor when max_y is 0.0, got {factor} \
       (pre-fix: 250.0 / 0.0 == Infinity, poisoning every glyph coordinate \
       it multiplies)"
    );
  }

  #[ test ]
  fn positive_max_y_divides_normally()
  {
    let factor = glyph_rescale_factor( 250.0, 50.0 );

    assert!
    (
      ( factor - 5.0 ).abs() < 1e-5,
      "expected 250.0 / 50.0 == 5.0 for a normal positive max_y, got {factor}"
    );
  }

  #[ test ]
  fn negative_max_y_also_yields_finite_scale()
  {
    // Guards against a hypothetical negative max_y (should not occur given the
    // caller's own max-tracking loop, but the guard is `.max( EPSILON )`, not
    // `.abs().max( EPSILON )`, so this is worth locking in explicitly.
    let factor = glyph_rescale_factor( 250.0, -10.0 );

    assert!
    (
      factor.is_finite(),
      "expected a finite rescale factor for a negative max_y, got {factor}"
    );
  }
}
