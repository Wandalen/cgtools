//! Hex-neighbour iteration, bitmask computation, and per-tile priority lookup.
//!
//! The canonical direction ordering is SPEC §2.3: flat-top runs clockwise
//! from `N` at bit 0, pointy-top runs clockwise from `NE`. The bitmask
//! produced by [`compute_neighbor_bitmask`] uses those indices.

mod private
{
  use std::collections::HashMap;
  use crate::scene_model::anchor::EdgeDirection;
  use crate::scene_model::object::Object;
  use crate::scene_model::pipeline::TilingStrategy;
  use crate::scene_model::scene::Tile;
  use crate::scene_model::spec::RenderSpec;

  /// Reserved id meaning "off-map or unset". See SPEC §15.1.
  pub const VOID_ID : &str = "void";

  /// Flat-top neighbour axial offsets in SPEC §2.3 order.
  /// Index 0 = N, 1 = NE, 2 = SE, 3 = S, 4 = SW, 5 = NW.
  pub const FLAT_NEIGHBORS : [ ( i32, i32 ); 6 ] =
  [
    (  0, -1 ),
    (  1, -1 ),
    (  1,  0 ),
    (  0,  1 ),
    ( -1,  1 ),
    ( -1,  0 ),
  ];

  /// Pointy-top neighbour axial offsets in SPEC §2.3 order.
  /// Index 0 = NE, 1 = E, 2 = SE, 3 = SW, 4 = W, 5 = NW.
  pub const POINTY_NEIGHBORS : [ ( i32, i32 ); 6 ] =
  [
    (  1, -1 ),
    (  1,  0 ),
    (  0,  1 ),
    ( -1,  1 ),
    ( -1,  0 ),
    (  0, -1 ),
  ];

  /// Axial offset from the current hex to its neighbour at direction index
  /// `0..6` (SPEC §2.3 ordering).
  #[ inline ]
  #[ must_use ]
  pub fn neighbor_offset( tiling : TilingStrategy, dir_index : usize ) -> Option< ( i32, i32 ) >
  {
    let table : &[ ( i32, i32 ); 6 ] = match tiling
    {
      TilingStrategy::HexFlatTop => &FLAT_NEIGHBORS,
      TilingStrategy::HexPointyTop => &POINTY_NEIGHBORS,
      TilingStrategy::Square4 | TilingStrategy::Square8 => return None,
    };
    table.get( dir_index ).copied()
  }

  /// Axial offset for a named [`EdgeDirection`] under the given tiling.
  #[ inline ]
  #[ must_use ]
  pub fn neighbor_offset_by_dir( tiling : TilingStrategy, dir : EdgeDirection ) -> Option< ( i32, i32 ) >
  {
    let idx = dir_to_index( tiling, dir )?;
    neighbor_offset( tiling, idx )
  }

  /// Map an [`EdgeDirection`] to its bitmask index under the current tiling.
  /// Returns `None` for directions not defined by the tiling (e.g. `E`/`W`
  /// on flat-top or `N`/`S` on pointy-top).
  #[ must_use ]
  pub fn dir_to_index( tiling : TilingStrategy, dir : EdgeDirection ) -> Option< usize >
  {
    match tiling
    {
      TilingStrategy::HexFlatTop => match dir
      {
        EdgeDirection::N  => Some( 0 ),
        EdgeDirection::NE => Some( 1 ),
        EdgeDirection::SE => Some( 2 ),
        EdgeDirection::S  => Some( 3 ),
        EdgeDirection::SW => Some( 4 ),
        EdgeDirection::NW => Some( 5 ),
        EdgeDirection::E | EdgeDirection::W => None,
      },
      TilingStrategy::HexPointyTop => match dir
      {
        EdgeDirection::NE => Some( 0 ),
        EdgeDirection::E  => Some( 1 ),
        EdgeDirection::SE => Some( 2 ),
        EdgeDirection::SW => Some( 3 ),
        EdgeDirection::W  => Some( 4 ),
        EdgeDirection::NW => Some( 5 ),
        EdgeDirection::N | EdgeDirection::S => None,
      },
      TilingStrategy::Square4 | TilingStrategy::Square8 => None,
    }
  }

  /// Lower-case short name for a direction — used for `{dir}` substitution
  /// in `NeighborCondition.sprite_pattern`. Convention is lowercase because
  /// asset filenames in this codebase are lowercase.
  #[ inline ]
  #[ must_use ]
  pub fn dir_name( dir : EdgeDirection ) -> &'static str
  {
    match dir
    {
      EdgeDirection::N  => "n",
      EdgeDirection::NE => "ne",
      EdgeDirection::E  => "e",
      EdgeDirection::SE => "se",
      EdgeDirection::S  => "s",
      EdgeDirection::SW => "sw",
      EdgeDirection::W  => "w",
      EdgeDirection::NW => "nw",
    }
  }

  /// Build a `( q, r ) → &Tile` lookup map for the scene.
  ///
  /// Used by bitmask computation and condition evaluation to ask "what's
  /// at neighbour position" without linear scans per query.
  #[ must_use ]
  pub fn tile_lookup( tiles : &[ Tile ] ) -> HashMap< ( i32, i32 ), &Tile >
  {
    tiles.iter().map( | t | ( t.pos, t ) ).collect()
  }

  /// Compute a 6-bit neighbour bitmask for a tile at `pos`.
  ///
  /// Bit `i` is 1 iff the neighbour at SPEC §2.3 direction `i` contains an
  /// object whose id is in `connects_with`. Off-map / empty neighbours
  /// contribute bit 1 only when `"void"` is in `connects_with` (SPEC §15.1).
  #[ must_use ]
  pub fn compute_neighbor_bitmask
  (
    pos : ( i32, i32 ),
    connects_with : &[ String ],
    tiling : TilingStrategy,
    tile_lookup : &HashMap< ( i32, i32 ), &Tile >,
  ) -> u8
  {
    let table : &[ ( i32, i32 ); 6 ] = match tiling
    {
      TilingStrategy::HexFlatTop => &FLAT_NEIGHBORS,
      TilingStrategy::HexPointyTop => &POINTY_NEIGHBORS,
      TilingStrategy::Square4 | TilingStrategy::Square8 => return 0,
    };

    let mut mask : u8 = 0;
    for ( i, ( dq, dr ) ) in table.iter().enumerate()
    {
      let neighbour_pos = ( pos.0 + dq, pos.1 + dr );
      let connects = match tile_lookup.get( &neighbour_pos )
      {
        Some( t ) => t.objects.iter().any( | id | connects_with.iter().any( | c | c == id ) ),
        None => connects_with.iter().any( | c | c == VOID_ID ),
      };
      if connects
      {
        mask |= 1 << i;
      }
    }
    mask
  }

  /// Neighbour-facing view of a tile — just enough for condition evaluation.
  ///
  /// Constructed on-demand by `neighbor_state_at`; avoids coupling
  /// `evaluate_condition` to the whole [`Tile`] / [`Object`] graph.
  #[ derive( Debug, Clone ) ]
  pub struct NeighborState< 'a >
  {
    /// Object ids present on this neighbour cell. Empty for off-map.
    pub object_ids : &'a [ String ],
    /// Max priority across the neighbour's objects that declare one.
    pub max_priority : Option< i32 >,
  }

  /// Resolve the neighbour at `pos` into its condition-facing view.
  ///
  /// Off-map neighbours produce `NeighborState { object_ids: &[], max_priority: None }`.
  #[ must_use ]
  pub fn neighbor_state_at< 'a >
  (
    pos : ( i32, i32 ),
    tile_lookup : &'a HashMap< ( i32, i32 ), &'a Tile >,
    spec : &'a RenderSpec,
  ) -> NeighborState< 'a >
  {
    const EMPTY : &[ String ] = &[];
    match tile_lookup.get( &pos )
    {
      Some( t ) =>
      {
        let max_priority = tile_max_priority( t, spec );
        NeighborState { object_ids : &t.objects, max_priority }
      },
      None => NeighborState { object_ids : EMPTY, max_priority : None },
    }
  }

  /// Max `Object.priority` across the tile's object stack, ignoring `None`s.
  ///
  /// Used by `NeighborPriorityLower` comparisons. A tile whose objects all
  /// have no priority returns `None`.
  #[ must_use ]
  pub fn tile_max_priority( tile : &Tile, spec : &RenderSpec ) -> Option< i32 >
  {
    tile.objects.iter()
      .filter_map( | id | find_object( spec, id ) )
      .filter_map( | o | o.priority )
      .max()
  }

  /// First object on the tile whose `Object.priority` is `Some(_)` — used as
  /// the "terrain" id for vertex-corner resolution. Falls back to `None`
  /// when no such object exists.
  #[ must_use ]
  pub fn tile_terrain_id< 'a >
  (
    tile : &'a Tile,
    spec : &RenderSpec,
  ) -> Option< &'a str >
  {
    for object_id in &tile.objects
    {
      if let Some( obj ) = find_object( spec, object_id )
      {
        if obj.priority.is_some()
        {
          return Some( object_id.as_str() );
        }
      }
    }
    None
  }

  fn find_object< 'a >( spec : &'a RenderSpec, id : &str ) -> Option< &'a Object >
  {
    spec.objects.iter().find( | o | o.id == id )
  }
}

mod_interface::mod_interface!
{
  own use VOID_ID;
  own use FLAT_NEIGHBORS;
  own use POINTY_NEIGHBORS;
  own use neighbor_offset;
  own use neighbor_offset_by_dir;
  own use dir_to_index;
  own use dir_name;
  own use tile_lookup;
  own use compute_neighbor_bitmask;
  own use NeighborState;
  own use neighbor_state_at;
  own use tile_max_priority;
  own use tile_terrain_id;
}
