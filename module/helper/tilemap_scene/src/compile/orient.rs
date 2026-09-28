//! Discrete orientation of a `VertexCorners` dual triangle for
//! `orient_to_grid` mode.
//!
//! The regular hex grid's dual triangles occur in six 60°-orientations;
//! [`dual_orientation_index`] picks which pre-baked frame a triangle draws
//! from its corner geometry, so no sprite is rotated at render time.

mod private
{
  use crate::pipeline::TilingStrategy;
  use crate::source::TriBlendPattern;

  /// `{rot}` frames of an oriented edge / corner tile: the six 60° steps.
  pub const ORIENTED_FRAMES : u8 = 6;
  /// `{rot}` frames of a 3-fold-symmetric (solid) tile: ▲/▽ parity only.
  pub const SYMMETRIC_FRAMES : u8 = 2;

  /// Number of `{rot}` frames `orient_to_grid` can select for `pattern`,
  /// i.e. how many the asset pass must allocate so every index
  /// [`dual_orientation_index`] returns resolves.
  ///
  /// A solid pattern ([`TriBlendPattern::self_id`] is `Some`) only matches a
  /// triangle whose corners are all one id, which that function reduces to
  /// parity; every other pattern (including `( "*", "*", "*" )`) can match
  /// an oriented edge or corner and needs all six.
  #[ inline ]
  #[ must_use ]
  pub fn orient_frame_count( pattern : &TriBlendPattern ) -> u8
  {
    if pattern.self_id().is_some() { SYMMETRIC_FRAMES } else { ORIENTED_FRAMES }
  }

  /// Discrete dual-grid orientation index for a triangle, in `orient_to_grid`
  /// mode. The regular hex grid's dual triangles occur in six 60°-orientations,
  /// each pre-baked as its own frame; this picks which one to draw.
  ///
  /// We align the *distinguishing* corner to its baked reference axis, then round
  /// the residual to the nearest 60° step. The baker lays the sorted corner slots
  /// at 60°/180°/300° (slot k at 60°+120°·k) and bakes orientation `o` by rotating
  /// the shape; crucially the export's PNG save flips vertically, so the baker's
  /// CCW `u_rot` reads as CLOCKWISE in world. A frame's reference corner therefore
  /// points at `base − 60°·o` in world space, index `round((base − bearing)/60°)`:
  ///   • corner tile (1 present)          → align the lone PRESENT corner,  base 60°,  6 frames
  ///   • edge tile   (2 present, 1 absent) → align the single ABSENT corner, base 300°, 6 frames
  ///   • full tile   (3 present)           → base 60°, 2 frames (▲/▽ parity only)
  ///
  /// "Present" means *this object's own id* (`self_id`, taken from its `(X,X,X)`
  /// full pattern), NOT a lexicographic property of the canonical triple. That
  /// distinction matters once a triangle holds two DIFFERENT non-void ids — e.g.
  /// two adjacent players' regions: for `region_1`'s edge tile the corners are
  /// `(region_1, region_1, region_0)`, and `"region_0" < "region_1"` sorts the
  /// foreign id FIRST, so the old canonical-order test misread the edge as a
  /// corner and pointed the petals at the neighbour's centre. Counting matches of
  /// `self_id` instead is exactly what the matched pattern meant by self vs.
  /// wildcard, so terrain (`self_id = "hexagon"`, absent = `"void"`) is unchanged
  /// while cross-region boundaries orient correctly. When `self_id` is `None`
  /// (object has no `(X,X,X)` pattern) we fall back to the canonical-order rule.
  ///
  /// NOTE: still assumes at most two distinct ids per triangle drive one object's
  /// shape (present vs. not-present). A genuine three-id chiral junction's ▲/▽
  /// mirror pair is out of scope (would need a parity-keyed reflected frame).
  #[ must_use ]
  pub fn dual_orientation_index
  (
    raw : &[ String; 3 ],
    canonical : &[ String; 3 ],
    self_id : Option< &str >,
    corner_px : &[ ( f32, f32 ); 3 ],
    wx : f32,
    wy : f32,
    tiling : TilingStrategy,
  ) -> u8
  {
    use core::f32::consts::{ FRAC_PI_3, FRAC_PI_6 };
    // The six dual-triangle corner bearings sit at multiples of 60° for a
    // flat-top grid but are rotated 30° on a pointy-top grid (the hex itself is
    // rotated 30°). Without compensating, every pointy bearing lands exactly on
    // a `round()` half-step boundary, so adjacent orientations collapse onto the
    // same index and a lone hex yields fewer than six distinct frames. Rotate
    // the orientation reference by the same 30° so the residuals are integral
    // again. (The absolute base — which frame is orientation 0 — is calibrated
    // visually per atlas; this only restores the 60° step alignment.)
    let base_offset = match tiling
    {
      TilingStrategy::HexPointyTop => FRAC_PI_6,
      _ => 0.0,
    };
    let ( base, period, dist_idx ) = if let Some( sid ) = self_id
    {
      // Classify by how many corners are THIS object's own id ("present").
      let present = [ raw[ 0 ] == sid, raw[ 1 ] == sid, raw[ 2 ] == sid ];
      match present.iter().filter( | p | **p ).count()
      {
        // edge: the lone NOT-present corner is the distinguishing (void) one.
        2 =>
        {
          let idx = present.iter().position( | p | !*p ).unwrap_or( 0 );
          ( FRAC_PI_3 * 5.0, ORIENTED_FRAMES, idx )
        }
        // corner: the lone PRESENT corner is the distinguishing one.
        1 =>
        {
          let idx = present.iter().position( | p | *p ).unwrap_or( 0 );
          ( FRAC_PI_3, ORIENTED_FRAMES, idx )
        }
        // full (3) — or the degenerate 0 — are 3-fold symmetric: parity only.
        _ => ( FRAC_PI_3, SYMMETRIC_FRAMES, 0 ),
      }
    }
    else
    {
      // Legacy fallback: derive the distinguishing corner from canonical order
      // (valid when the absent id sorts after the present id, e.g. literal void).
      let ( unique, base, period ) =
        if canonical[ 0 ] == canonical[ 2 ]      { ( None,                  FRAC_PI_3,       SYMMETRIC_FRAMES ) }
        else if canonical[ 0 ] == canonical[ 1 ] { ( Some( &canonical[ 2 ] ), FRAC_PI_3 * 5.0, ORIENTED_FRAMES ) }
        else                                     { ( Some( &canonical[ 0 ] ), FRAC_PI_3,       ORIENTED_FRAMES ) };
      let dist_idx = unique
        .and_then( | v | raw.iter().position( | c | c == v ) )
        .unwrap_or( 0 );
      ( base, period, dist_idx )
    };
    let ( cx, cy ) = corner_px[ dist_idx ];
    let bearing = ( cy - wy ).atan2( cx - wx );
    // `base − bearing` (not `bearing − base`): the baked frames advance
    // clockwise in world because the atlas export flips the PNG vertically.
    let steps = ( ( base + base_offset - bearing ) / FRAC_PI_3 ).round() as i32;
    steps.rem_euclid( i32::from( period ) ) as u8
  }

}

mod_interface::mod_interface!
{
  own use dual_orientation_index;
  own use orient_frame_count;
  own use ORIENTED_FRAMES;
  own use SYMMETRIC_FRAMES;
}
