//! Discrete orientation of a `VertexCorners` dual triangle for
//! `orient_to_grid` mode.
//!
//! The regular hex grid's dual triangles occur in six 60°-orientations;
//! [`dual_orientation_index`] picks which pre-baked frame a triangle draws
//! from its corner geometry, so no sprite is rotated at render time.

mod private
{
  use crate::compile::neighbors::VOID_ID;
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
  /// "Present" is decided against the layer's *self id* — the id of its solid
  /// `(X,X,X)` pattern ([`TriBlendPattern::self_id`]) — NOT by a lexicographic
  /// property of the canonical triple. That distinction matters once a
  /// triangle holds two DIFFERENT non-void ids — e.g. two adjacent players'
  /// regions: for `region_1`'s edge tile the corners are
  /// `(region_1, region_1, region_0)`, and `"region_0" < "region_1"` sorts the
  /// foreign id FIRST, so a canonical-order test misreads the edge as a corner
  /// and points the petals at the neighbour's centre. Counting matches of
  /// `self_id` treats a foreign id exactly like void.
  ///
  /// A layer with no solid pattern (`self_id` is `None`) has no own id to
  /// count, so every non-void corner is present. A sort-based fallback would
  /// be wrong for any id that sorts after `"void"`: `(water, water, void)`
  /// sorts to `(void, water, water)` and would read as a corner.
  ///
  /// NOTE: still assumes at most two distinct ids per triangle drive one object's
  /// shape (present vs. not-present). A genuine three-id chiral junction's ▲/▽
  /// mirror pair is out of scope (would need a parity-keyed reflected frame).
  #[ must_use ]
  pub fn dual_orientation_index
  (
    raw : &[ String; 3 ],
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
    // again. Frame 0's reference direction therefore sits 30° further round on
    // a pointy-top grid; format/005's atlas-baker contract states both tilings.
    let base_offset = match tiling
    {
      TilingStrategy::HexPointyTop => FRAC_PI_6,
      _ => 0.0,
    };
    let present : [ bool; 3 ] = match self_id
    {
      Some( sid ) => core::array::from_fn( | i | raw[ i ] == sid ),
      None => core::array::from_fn( | i | raw[ i ] != VOID_ID ),
    };
    let ( base, period, dist_idx ) = match present.iter().filter( | p | **p ).count()
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
