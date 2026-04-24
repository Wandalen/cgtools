//! Edge-anchored rendering helpers — canonicalisation, pixel midpoint +
//! rotation, and the 4-bit `EdgeConnectedBitmask` computation (SPEC §5.9).
//!
//! An edge is shared by two hexes. Authors may declare `EdgePosition { hex, dir }`
//! from either side; [`canonical_edge`] reduces both forms to the same
//! `( canonical_hex, dir )` key so every edge emits once per frame.

mod private
{
  use rustc_hash::FxHashMap as HashMap;
  use crate::anchor::EdgeDirection;
  use crate::compile::coords::{ hex_to_world_pixel_flat, hex_to_world_pixel_pointy };
  use crate::compile::neighbors::{ dir_to_index, neighbor_offset_by_dir, VOID_ID };
  use crate::pipeline::TilingStrategy;
  use crate::scene::{ EdgeInstance, EdgePosition };

  /// Canonical key for an edge.
  ///
  /// Two `EdgePosition`s refer to the same edge iff their canonical forms
  /// are equal. Picked by taking the neighbour pair with the lexicographically
  /// smaller `(q, r)` as the owning hex; the direction is then set so that
  /// the offset from that hex points at the other hex.
  pub type CanonicalEdge = ( ( i32, i32 ), EdgeDirection );

  /// Canonicalise an edge so that both sides reduce to the same key.
  ///
  /// Returns `None` when the direction is not valid for the tiling (e.g.
  /// `E` / `W` on flat-top hex).
  #[ must_use ]
  pub fn canonical_edge
  (
    at : EdgePosition,
    tiling : TilingStrategy,
  ) -> Option< CanonicalEdge >
  {
    let offset = neighbor_offset_by_dir( tiling, at.dir )?;
    let other_hex = ( at.hex.0 + offset.0, at.hex.1 + offset.1 );

    if at.hex <= other_hex
    {
      Some( ( at.hex, at.dir ) )
    }
    else
    {
      // Flip: the canonical hex is the other one, direction is the opposite.
      Some( ( other_hex, opposite_dir( at.dir ) ) )
    }
  }

  /// Opposite direction in the 6-neighbour ring.
  #[ inline ]
  #[ must_use ]
  pub fn opposite_dir( dir : EdgeDirection ) -> EdgeDirection
  {
    match dir
    {
      EdgeDirection::N  => EdgeDirection::S,
      EdgeDirection::S  => EdgeDirection::N,
      EdgeDirection::NE => EdgeDirection::SW,
      EdgeDirection::SW => EdgeDirection::NE,
      EdgeDirection::SE => EdgeDirection::NW,
      EdgeDirection::NW => EdgeDirection::SE,
      EdgeDirection::E  => EdgeDirection::W,
      EdgeDirection::W  => EdgeDirection::E,
    }
  }

  /// Pixel midpoint of an edge — average of its two adjacent hex centres.
  #[ must_use ]
  pub fn edge_world_pixel
  (
    canon : CanonicalEdge,
    tiling : TilingStrategy,
    grid_stride : ( u32, u32 ),
  ) -> Option< ( f32, f32 ) >
  {
    let ( hex_a, dir ) = canon;
    let offset = neighbor_offset_by_dir( tiling, dir )?;
    let hex_b = ( hex_a.0 + offset.0, hex_a.1 + offset.1 );
    let ca = hex_pixel( hex_a, tiling, grid_stride )?;
    let cb = hex_pixel( hex_b, tiling, grid_stride )?;
    Some( ( ( ca.0 + cb.0 ) * 0.5, ( ca.1 + cb.1 ) * 0.5 ) )
  }

  #[ inline ]
  fn hex_pixel( pos : ( i32, i32 ), tiling : TilingStrategy, stride : ( u32, u32 ) ) -> Option< ( f32, f32 ) >
  {
    match tiling
    {
      TilingStrategy::HexFlatTop   => Some( hex_to_world_pixel_flat( pos.0, pos.1, stride ) ),
      TilingStrategy::HexPointyTop => Some( hex_to_world_pixel_pointy( pos.0, pos.1, stride ) ),
      TilingStrategy::Square4 | TilingStrategy::Square8 => None,
    }
  }

  /// Rotation angle (radians, Y-up world) for a sprite drawn along an edge.
  ///
  /// Authors design the sprite in the canonical orientation (e.g. running
  /// horizontally); this angle rotates the quad so its long axis matches
  /// the direction perpendicular to the edge normal. Three pairs
  /// (N↔S, NE↔SW, SE↔NW) collapse to three unique orientations — each
  /// pair shares an angle modulo π.
  #[ must_use ]
  pub fn edge_rotation( dir : EdgeDirection, tiling : TilingStrategy ) -> f32
  {
    // Angles pick the edge's normal in world space (Y-up).
    // Flat-top: N faces up (angle 0), step CW = π/3.
    // Pointy-top: NE is CW from north, so start at π/6.
    use core::f32::consts::PI;
    match tiling
    {
      TilingStrategy::HexFlatTop => match dir
      {
        EdgeDirection::N  => 0.0,
        EdgeDirection::NE => PI / 3.0,
        EdgeDirection::SE => 2.0 * PI / 3.0,
        EdgeDirection::S  => PI,
        EdgeDirection::SW => 4.0 * PI / 3.0,
        EdgeDirection::NW => 5.0 * PI / 3.0,
        EdgeDirection::E | EdgeDirection::W => 0.0,
      },
      TilingStrategy::HexPointyTop => match dir
      {
        EdgeDirection::NE => PI / 6.0,
        EdgeDirection::E  => PI / 2.0,
        EdgeDirection::SE => 5.0 * PI / 6.0,
        EdgeDirection::SW => 7.0 * PI / 6.0,
        EdgeDirection::W  => 3.0 * PI / 2.0,
        EdgeDirection::NW => 11.0 * PI / 6.0,
        EdgeDirection::N | EdgeDirection::S => 0.0,
      },
      TilingStrategy::Square4 | TilingStrategy::Square8 => 0.0,
    }
  }

  /// Build a `CanonicalEdge -> &EdgeInstance` lookup for the current scene.
  /// The canonicalisation guarantees deduping when the user supplied both
  /// sides of the same edge — last write wins.
  #[ must_use ]
  pub fn edge_lookup< 'a >
  (
    edges : &'a [ EdgeInstance ],
    tiling : TilingStrategy,
  ) -> HashMap< CanonicalEdge, &'a EdgeInstance >
  {
    let mut map = HashMap::default();
    for e in edges
    {
      if let Some( canon ) = canonical_edge( e.at, tiling )
      {
        map.insert( canon, e );
      }
    }
    map
  }

  /// Compute the 4-bit `EdgeConnectedBitmask` for the given canonical edge.
  ///
  /// Bit layout (SPEC §5.9 `EdgeHex`):
  ///
  /// - `0x1` — CCW-neighbour edge at the edge's "start" vertex is connected.
  /// - `0x2` — CW-neighbour edge at the start vertex is connected.
  /// - `0x4` — CCW-neighbour edge at the "end" vertex is connected.
  /// - `0x8` — CW-neighbour edge at the end vertex is connected.
  ///
  /// "Connected" means: the neighbour edge has an `EdgeInstance` whose
  /// `object` id appears in `connects_with`. When the neighbour edge is
  /// off-map / has no instance, it counts as connected iff `"void"` is in
  /// `connects_with` (SPEC §15.1).
  ///
  /// Start / end vertex convention: walking the edge direction `dir` from
  /// the canonical hex toward the neighbour hex, "start" is the
  /// CCW-rotation (the vertex on the CCW side), "end" is the CW-rotation.
  #[ must_use ]
  pub fn compute_edge_connected_bitmask
  (
    canon : CanonicalEdge,
    connects_with : &[ String ],
    tiling : TilingStrategy,
    edge_lookup : &HashMap< CanonicalEdge, &EdgeInstance >,
  ) -> u8
  {
    let Some( idx ) = dir_to_index( tiling, canon.1 ) else { return 0 };
    // Six directions make a ring; ±1 mod 6 gives CCW/CW of the current dir.
    let ccw_idx = ( idx + 5 ) % 6;
    let cw_idx  = ( idx + 1 ) % 6;

    let Some( ccw_dir ) = index_to_dir( tiling, ccw_idx ) else { return 0 };
    let Some( cw_dir )  = index_to_dir( tiling, cw_idx )  else { return 0 };

    let hex_a = canon.0;
    let Some( offset_b ) = neighbor_offset_by_dir( tiling, canon.1 ) else { return 0 };
    let hex_b = ( hex_a.0 + offset_b.0, hex_a.1 + offset_b.1 );

    let mut mask : u8 = 0;

    // Start vertex (CCW side): touches hex_A, hex_B, and a third hex C_ccw.
    //   - CCW-neighbour edge at start vertex = edge between A and C_ccw
    //     = canonical form of (hex_A, ccw_dir).
    //   - CW-neighbour edge at start vertex = edge between B and C_ccw
    //     = canonical form of (hex_B, opposite(cw_dir)).
    if neighbour_connects( ( hex_a, ccw_dir ), tiling, connects_with, edge_lookup )
    {
      mask |= 0x1;
    }
    if neighbour_connects( ( hex_b, opposite_dir( cw_dir ) ), tiling, connects_with, edge_lookup )
    {
      mask |= 0x2;
    }

    // End vertex (CW side): touches hex_A, hex_B, and C_cw.
    //   - CCW-neighbour edge at end = edge between B and C_cw
    //     = canonical form of (hex_B, opposite(ccw_dir)).
    //   - CW-neighbour edge at end = edge between A and C_cw
    //     = canonical form of (hex_A, cw_dir).
    if neighbour_connects( ( hex_b, opposite_dir( ccw_dir ) ), tiling, connects_with, edge_lookup )
    {
      mask |= 0x4;
    }
    if neighbour_connects( ( hex_a, cw_dir ), tiling, connects_with, edge_lookup )
    {
      mask |= 0x8;
    }

    mask
  }

  fn neighbour_connects
  (
    edge : ( ( i32, i32 ), EdgeDirection ),
    tiling : TilingStrategy,
    connects_with : &[ String ],
    edge_lookup : &HashMap< CanonicalEdge, &EdgeInstance >,
  ) -> bool
  {
    let Some( canon ) = canonical_edge( EdgePosition { hex : edge.0, dir : edge.1 }, tiling )
    else { return false };

    match edge_lookup.get( &canon )
    {
      Some( inst ) => connects_with.iter().any( | c | c == &inst.object ),
      None         => connects_with.iter().any( | c | c == VOID_ID ),
    }
  }

  /// Map a ring index back to a direction under the current tiling — inverse
  /// of [`crate::compile::neighbors::dir_to_index`].
  #[ must_use ]
  pub fn index_to_dir( tiling : TilingStrategy, idx : usize ) -> Option< EdgeDirection >
  {
    match tiling
    {
      TilingStrategy::HexFlatTop => Some( match idx
      {
        0 => EdgeDirection::N,
        1 => EdgeDirection::NE,
        2 => EdgeDirection::SE,
        3 => EdgeDirection::S,
        4 => EdgeDirection::SW,
        5 => EdgeDirection::NW,
        _ => return None,
      }),
      TilingStrategy::HexPointyTop => Some( match idx
      {
        0 => EdgeDirection::NE,
        1 => EdgeDirection::E,
        2 => EdgeDirection::SE,
        3 => EdgeDirection::SW,
        4 => EdgeDirection::W,
        5 => EdgeDirection::NW,
        _ => return None,
      }),
      TilingStrategy::Square4 | TilingStrategy::Square8 => None,
    }
  }
}

#[ cfg( test ) ]
mod tests
{
  use super::private::*;
  use crate::anchor::EdgeDirection;
  use crate::pipeline::TilingStrategy;
  use crate::scene::EdgePosition;

  #[ test ]
  fn canonical_picks_smaller_hex()
  {
    let tiling = TilingStrategy::HexFlatTop;
    // Edge between (0,0) and its N neighbour (0,-1).
    let from_a = EdgePosition { hex : ( 0, 0 ), dir : EdgeDirection::N };
    let from_b = EdgePosition { hex : ( 0, -1 ), dir : EdgeDirection::S };
    let ca = canonical_edge( from_a, tiling ).unwrap();
    let cb = canonical_edge( from_b, tiling ).unwrap();
    assert_eq!( ca, cb, "both sides must canonicalise to the same key" );
    // (0,-1) is lexicographically smaller than (0,0) → canonical hex = (0,-1).
    assert_eq!( ca.0, ( 0, -1 ) );
  }

  #[ test ]
  fn edge_rotation_flat_top_table()
  {
    use core::f32::consts::PI;
    let t = TilingStrategy::HexFlatTop;
    assert!( ( edge_rotation( EdgeDirection::N,  t ) - 0.0 ).abs() < 1e-5 );
    assert!( ( edge_rotation( EdgeDirection::NE, t ) - PI / 3.0 ).abs() < 1e-5 );
    assert!( ( edge_rotation( EdgeDirection::S,  t ) - PI ).abs() < 1e-5 );
  }
}

mod_interface::mod_interface!
{
  exposed use CanonicalEdge;
  exposed use canonical_edge;
  exposed use opposite_dir;
  exposed use edge_world_pixel;
  exposed use edge_rotation;
  exposed use edge_lookup;
  exposed use compute_edge_connected_bitmask;
  exposed use index_to_dir;
}
