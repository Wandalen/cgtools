//! Integration tests for `primitive_generation::text::ufo::text_to_countour_mesh`.
//!
//! Covers BUG-129: pass 2 of the glyph-layout loop advanced by each glyph's
//! *full* slot width before placing it, instead of a half-width step on each
//! side of the placement -- over-advancing by one half slot-width per glyph.

#[ cfg( test ) ]
mod tests
{
  use primitive_generation::text::ufo::{ Font, Glyph };
  use primitive_generation::Transform;

  /// Builds minimal `.glif` bytes for a thin triangle spanning `x = [0, width]`,
  /// using only `move`/`line` point types (no curves) so the glyph's bounding
  /// box -- and therefore its layout width -- is exactly `width`, with no
  /// flattening-tolerance approximation to account for.
  fn glif_triangle_bytes( width : f64 ) -> Vec< u8 >
  {
    format!
    (
      r#"<?xml version="1.0" encoding="UTF-8"?>
<glyph name="test" format="2">
  <outline>
    <contour>
      <point x="0" y="0" type="move"/>
      <point x="{width}" y="0" type="line"/>
      <point x="{half}" y="1" type="line"/>
    </contour>
  </outline>
</glyph>"#,
      width = width,
      half = width / 2.0
    )
    .into_bytes()
  }

  // test_kind: bug_reproducer(BUG-129)
  /// ## Root Cause
  /// `text_to_countour_mesh` (and its duplicate, `text_to_mesh`) lay out glyphs
  /// in two passes: pass 1 subtracts each glyph's *half* slot-width to find the
  /// centered starting position; pass 2 was supposed to mirror this with a
  /// half-step, placement, half-step around each glyph, but instead advanced by
  /// the glyph's *full* slot-width in one step before placing it. This
  /// over-advances by exactly one half slot-width per glyph, compounding
  /// across the string: for 3 glyphs of raw widths `[2, 6, 4]` (scaled `x0.003`
  /// per the function's own hardcoded scale) starting centered at `x = 0`, the
  /// correct slot-midpoint positions are `[-0.015, -0.003, 0.012]`; the buggy
  /// code instead produced `[-0.012, 0.006, 0.018]`.
  ///
  /// ## Why Not Caught
  /// No existing test exercised the glyph-layout functions at all -- both are
  /// gated behind `font-processing` and their only public construction path
  /// (`fonts_load`) is an async, browser-only file loader with no test fixture
  /// wired up, so the pure layout arithmetic was never independently checked.
  ///
  /// ## Fix Applied
  /// Changed pass 2's advance in both `text_to_mesh` and `text_to_countour_mesh`
  /// (`src/text/ufo.rs`) from a single full-slot-width step before placement to
  /// a half-slot-width step, placement, then a second half-slot-width step --
  /// symmetric with pass 1's half-step subtraction.
  ///
  /// ## Prevention
  /// When a layout algorithm splits an advance across two passes (find start,
  /// then place-and-advance), the two passes' per-item step sizes must be
  /// derived from the same formula -- an asymmetric split (half here, full
  /// there) silently drifts every item after the first.
  ///
  /// ## Pitfall
  /// The drift is easy to miss by inspection: the *first* glyph's mismatch is
  /// small (bounded by that glyph's own half-width), so a visual smoke test on
  /// short strings can look "close enough" while still being systematically
  /// wrong and compounding on longer text.
  #[ test ]
  fn text_to_countour_mesh_centers_each_glyph_in_its_own_slot()
  {
    let font = Font::from_glyphs
    (
      [
        ( 'a', Glyph::from_glif( &glif_triangle_bytes( 2.0 ), 'a' ).expect( "glyph a" ) ),
        ( 'b', Glyph::from_glif( &glif_triangle_bytes( 6.0 ), 'b' ).expect( "glyph b" ) ),
        ( 'c', Glyph::from_glif( &glif_triangle_bytes( 4.0 ), 'c' ).expect( "glyph c" ) ),
      ]
    );

    let mesh = primitive_generation::text::ufo::text_to_countour_mesh
    (
      "abc",
      &font,
      &Transform::default(),
      0.1
    );

    assert_eq!( mesh.len(), 3, "one contour geometry per glyph" );

    let xs : Vec< f32 > = mesh.iter().map( | p | p.transform.translation.x() ).collect();
    let expected = [ -0.015_f32, -0.003_f32, 0.012_f32 ];

    for ( i, ( &actual, &want ) ) in xs.iter().zip( expected.iter() ).enumerate()
    {
      assert!
      (
        ( actual - want ).abs() < 1e-5,
        "glyph {i}: expected slot-centered x ~= {want}, got {actual} \
         (buggy full-step code would have produced [-0.012, 0.006, 0.018])"
      );
    }
  }
}
