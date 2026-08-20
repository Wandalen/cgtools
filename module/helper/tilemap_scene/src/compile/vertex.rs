//! Dual-mesh triangle enumeration and pattern matching.
//!
//! Covers `SpriteSource::VertexCorners` — for every vertex of the hex grid
//! where three hexes meet, emit a sprite based on the sorted tuple of
//! corner terrain ids.
//!
//! Two halves:
//!
//! - [`triangles_enumerate`] walks `scene.tiles` and yields each unique
//!   triangle of the dual mesh exactly once (a triangle is shared by three
//!   hexes; dedup via `HashSet<TriCoord>`).
//! - [`canonicalize`] + [`matching_pattern_find`] implement the lexicographic
//!   sort + wildcard specificity matching from SPEC §5.6 / §9.

mod private
{
  use rustc_hash::{ FxHashMap as HashMap, FxHashSet as HashSet };
  use tiles_tools::coordinates::ToDual;
  use tiles_tools::coordinates::hexagonal::{ Axial, Coordinate as HexCoordinate, Flat, Pointy };
  use tiles_tools::coordinates::triangular::{ Coordinate as TriCoordinate, FlatSided, FlatTopped };
  use crate::compile::neighbors::{ VOID_ID, tile_corner_id };
  use crate::pipeline::TilingStrategy;
  use crate::snapshot::Tile;
  use crate::source::TriBlendPattern;
  use crate::spec::RenderSpec;

  /// A triangle of the dual mesh together with its three hex corners.
  ///
  /// `corners` are in the order [`tiles_tools`] returns them from
  /// `TriCoord::dual()` — used for deriving the pixel centre of the
  /// triangle as the average of the three hex pixel centres.
  #[ derive( Debug, Clone ) ]
  pub struct TriangleContext
  {
    /// The three corner hex positions in axial `( q, r )` coordinates.
    pub corners : [ ( i32, i32 ); 3 ],
  }

  /// Enumerate every unique dual-mesh triangle touched by at least one tile
  /// in the scene. Each triangle is yielded once even though three hexes
  /// share it.
  #[ must_use ]
  pub fn triangles_enumerate( tiles : &[ Tile ], tiling : TilingStrategy ) -> Vec< TriangleContext >
  {
    match tiling
    {
      TilingStrategy::HexFlatTop =>
        enumerate::< Flat, FlatSided >( tiles ),
      TilingStrategy::HexPointyTop =>
        enumerate::< Pointy, FlatTopped >( tiles ),
      TilingStrategy::Square4 | TilingStrategy::Square8 =>
        Vec::new(),   // unsupported — caller should already have rejected earlier
    }
  }

  fn enumerate< HO, TO >( tiles : &[ Tile ] ) -> Vec< TriangleContext >
  where
    HexCoordinate< Axial, HO > : ToDual< TriCoordinate< TO > >,
    TriCoordinate< TO > : ToDual< HexCoordinate< Axial, HO > > + core::hash::Hash + Eq + Clone,
  {
    let mut seen : HashSet< TriCoordinate< TO > > = HashSet::default();
    let mut out = Vec::new();

    for tile in tiles
    {
      let hex : HexCoordinate< Axial, HO > = HexCoordinate::< Axial, HO >::new( tile.pos.0, tile.pos.1 );
      for tri in hex.dual()
      {
        if !seen.insert( tri ) { continue; }
        let hex_corners = tri.dual();
        let mut corners = [ ( 0_i32, 0_i32 ); 3 ];
        for ( i, corner ) in hex_corners.iter().enumerate().take( 3 )
        {
          corners[ i ] = ( corner.q, corner.r );
        }
        out.push( TriangleContext { corners } );
      }
    }
    out
  }

  /// Resolve the three corner ids of a triangle against the scene, using
  /// [`tile_corner_id`] for each corner. `source` selects the corner channel
  /// (`None` = the cell's terrain id; `Some(layer)` = the cell's object in that
  /// draw layer). Corners outside the scene resolve to [`VOID_ID`].
  #[ must_use ]
  // `tile_lookup` is always this crate's `FxHashMap` alias (every caller builds
  // it via `tile_lookup()` in `neighbors.rs`); there is no existing or planned
  // caller passing a different hasher, so generalizing over `BuildHasher` would
  // add API surface for no current need.
  #[ allow( clippy::implicit_hasher, reason = "tile_lookup is always this crate's FxHashMap alias; every caller builds it via tile_lookup() in neighbors.rs, so generalizing over BuildHasher would add API surface for no current need" ) ]
  pub fn corners_resolve
  (
    tri : &TriangleContext,
    tile_lookup : &HashMap< ( i32, i32 ), &Tile >,
    spec : &RenderSpec,
    source : Option< &str >,
  ) -> [ String; 3 ]
  {
    tri.corners.map( | pos |
    {
      match tile_lookup.get( &pos )
      {
        Some( t ) => tile_corner_id( t, spec, source ).unwrap_or( VOID_ID ).to_owned(),
        None => VOID_ID.to_owned(),
      }
    })
  }

  /// Canonicalise three corner terrain ids: sort lexicographically so pattern
  /// matching is insensitive to triangle rotation. Returns the sorted triple
  /// and a `rotation` u8 in `0..3` capturing which original slot landed in slot
  /// 0 of the canonical form (for legacy `{rot}` sprite substitution).
  ///
  /// The `orient_to_grid` path does not use `rotation`; it derives a discrete
  /// orientation from triangle geometry (see `resolve_vertex_pass_all`).
  #[ must_use ]
  pub fn canonicalize( raw : &[ String; 3 ] ) -> ( [ String; 3 ], u8 )
  {
    // Pair each value with its original index, sort, then read out the slot.
    let mut indexed : [ ( usize, String ); 3 ] =
    [
      ( 0, raw[ 0 ].clone() ),
      ( 1, raw[ 1 ].clone() ),
      ( 2, raw[ 2 ].clone() ),
    ];
    indexed.sort_by( | a, b | a.1.cmp( &b.1 ) );

    // Fix(BUG-264): rotation must report which ORIGINAL corner's value
    // landed in canonical slot 0 (`indexed[0].0`), matching this function's
    // own doc comment ("which original slot landed in slot 0") and the
    // SPEC-level docs (`docs/format/003_anchor_placement_types.md`,
    // `docs/invariant/002_edge_and_vertex_canonical_uniqueness.md`: "the
    // permutation applied to reach"/"which cyclic permutation ... produced
    // that sorted order"). The previous formula instead searched for where
    // original corner 0 (not the value now in slot 0) ended up — the
    // *inverse* rotation, equal to the intended value only at the
    // rotation = 0 fixed point; for either non-identity cyclic rotation of
    // 3 corners it picks the wrong one of the 3 `{rot}`-substituted sprite
    // variants pre-allocated by `assets_compile`.
    // Root cause: `.position(|(orig,_)| *orig == 0)` finds corner-0's new
    // position in the sorted array, which is the inverse permutation of
    // "which corner is now first" (`indexed[0].0`).
    // Pitfall: a scalar "rotation index" can be read as a permutation or as
    // its inverse — both compile, both stay in the documented 0..3 range,
    // and no test pinned the exact value (only "some rotation matched"),
    // so only cross-checking the implementation against the doc's worked
    // wording catches the swap.
    let rotation = indexed[ 0 ].0 as u8;

    let sorted =
    [
      indexed[ 0 ].1.clone(),
      indexed[ 1 ].1.clone(),
      indexed[ 2 ].1.clone(),
    ];
    ( sorted, rotation )
  }

  /// Find the best-matching pattern from `patterns` against a canonicalised
  /// corner triple. Matches use multiset-subset matching with `"*"` wildcards
  /// absorbing any leftover canonical entries (SPEC §5.6).
  ///
  /// Resolution order (SPEC §9):
  /// 1. Specificity — fewer wildcards wins.
  /// 2. `priority` — higher wins.
  /// 3. Declaration order — earlier wins.
  ///
  /// Returns `None` if no pattern matches — the triangle emits no sprite.
  #[ must_use ]
  pub fn matching_pattern_find< 'p >
  (
    patterns : &'p [ TriBlendPattern ],
    canonical : &[ String; 3 ],
  ) -> Option< &'p TriBlendPattern >
  {
    let mut best : Option< &TriBlendPattern > = None;
    let mut best_specificity : i32 = -1;
    let mut best_priority : i32 = i32::MIN;

    for pattern in patterns
    {
      if !pattern_matches( pattern, canonical )
      {
        continue;
      }
      let specificity = 3_i32 - wildcards_in( pattern );
      let better = match best
      {
        None => true,
        Some( _ ) =>
        {
          ( specificity > best_specificity )
            || ( specificity == best_specificity && pattern.priority > best_priority )
        },
      };
      if better
      {
        best = Some( pattern );
        best_specificity = specificity;
        best_priority = pattern.priority;
      }
    }
    best
  }

  /// A pattern matches a canonical triple when every concrete entry in the
  /// pattern can be paired to a unique canonical entry of the same value;
  /// the remaining (unpaired) canonical slots are absorbed by `"*"`
  /// wildcards. Positional order is ignored — the canonical is already
  /// sorted, and wildcards can live anywhere in the pattern.
  fn pattern_matches( pattern : &TriBlendPattern, canonical : &[ String; 3 ] ) -> bool
  {
    let pat = [ &pattern.corners.0, &pattern.corners.1, &pattern.corners.2 ];
    let mut used = [ false; 3 ];

    for value in pat.iter().filter( | v | v.as_str() != "*" )
    {
      let mut matched = false;
      for ( i, c ) in canonical.iter().enumerate()
      {
        if !used[ i ] && value.as_str() == c.as_str()
        {
          used[ i ] = true;
          matched = true;
          break;
        }
      }
      if !matched
      {
        return false;
      }
    }
    // Remaining canonical slots are implicitly covered by wildcards:
    // the pattern has exactly `3 - pat_concrete_count` wildcards, and
    // that equals the number of unused canonical slots.
    true
  }

  fn wildcards_in( pattern : &TriBlendPattern ) -> i32
  {
    let c = &pattern.corners;
    [ &c.0, &c.1, &c.2 ].iter().filter( | s | s.as_str() == "*" ).count() as i32
  }
}

mod_interface::mod_interface!
{
  exposed use TriangleContext;
  exposed use triangles_enumerate;
  exposed use corners_resolve;
  exposed use canonicalize;
  exposed use matching_pattern_find;
}
