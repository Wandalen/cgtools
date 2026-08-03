//! Pure-data scene snapshot — the RON-deserialisable representation.
//!
//! A [`SceneSnapshot`] is a **point-in-time data description** of what is
//! drawn where: tiles with object stacks, edge / multihex / free / viewport
//! instances, players, seed. It carries no runtime state — no instance
//! handles, no clock, no spatial indexes — and is therefore cheap to
//! serialize, diff, or stamp into version control.
//!
//! The retained-mode [`crate::scene::Scene`] is the runtime counterpart:
//! mutable, handle-indexed, owned by the consumer. Game pipelines that load
//! their world from disk deserialize into `SceneSnapshot` and then build a
//! `Scene` from it; pipelines that construct the world programmatically skip
//! the snapshot entirely.
//!
//! Companion types ([`Tile`], [`EdgeInstance`], …) are the on-disk
//! shape of placements; [`crate::scene::Scene::from_snapshot`] consumes
//! a `SceneSnapshot` and spawns the equivalent retained-mode instances.
//! Consumers that build their world programmatically can ignore this
//! module entirely.

mod private
{
  use serde::{ Deserialize, Serialize };
  use rustc_hash::FxHashMap as HashMap;
  use crate::anchor::EdgeDirection;

  /// Top-level scene file payload.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct SceneSnapshot
  {
    /// Scene metadata (name, render spec path, etc.).
    pub meta : SceneMeta,
    /// Axial-coordinate bounds for the grid.
    pub bounds : Bounds,

    /// Explicit list of tiles. Mutually exclusive with `palette` + `map`.
    #[ serde( default ) ]
    pub tiles : Vec< Tile >,

    /// Palette mapping ASCII characters to object id stacks. Used with `map`.
    #[ serde( default ) ]
    pub palette : HashMap< char, Vec< String > >,
    /// ASCII grid (one row per entry). Requires `palette` to be set.
    #[ serde( default ) ]
    pub map : Vec< String >,

    /// Edge-anchored object instances.
    #[ serde( default ) ]
    pub edges : Vec< EdgeInstance >,
    /// Multihex-anchored object instances.
    #[ serde( default ) ]
    pub multihex_instances : Vec< MultihexInstance >,
    /// Free-world-space instances (projectiles, floating numbers).
    #[ serde( default ) ]
    pub free_instances : Vec< FreeInstance >,
    /// Viewport-anchored instances (skyboxes, weather overlays).
    #[ serde( default ) ]
    pub viewport_instances : Vec< ViewportInstance >,

    /// "The player's pieces" — hex-anchored instances that move at runtime.
    /// Stored separately because runtime systems track them differently from
    /// static tile objects.
    #[ serde( default ) ]
    pub entities : Vec< Entity >,

    /// Player / faction declarations. `Entity.owner` indexes into this list.
    #[ serde( default ) ]
    pub players : Vec< Player >,

    /// Initial global tint (references a tint id in the render spec).
    /// Game code may change this at runtime.
    #[ serde( default ) ]
    pub initial_global_tint : Option< String >,
    /// Per-scene seed for deterministic pseudo-randomness.
    ///
    /// Consumed by `Variant::Random` (SPEC §5.2) — seeds the hash that
    /// picks a variant per instance. `None` / `0` is a valid, fully
    /// deterministic seed. Stays stable across frames so the chosen
    /// variant for each `(q, r)` doesn't flicker.
    #[ serde( default ) ]
    pub seed : Option< u64 >,
  }

  impl SceneSnapshot
  {
    /// Creates an empty snapshot bounded by `bounds`. All collections start
    /// empty, metadata fields are `None`.
    #[ inline ]
    #[ must_use ]
    pub fn new( bounds : Bounds ) -> Self
    {
      Self
      {
        meta : SceneMeta::default(),
        bounds,
        tiles : Vec::new(),
        palette : HashMap::default(),
        map : Vec::new(),
        edges : Vec::new(),
        multihex_instances : Vec::new(),
        free_instances : Vec::new(),
        viewport_instances : Vec::new(),
        entities : Vec::new(),
        players : Vec::new(),
        initial_global_tint : None,
        seed : None,
      }
    }

    /// Materialise the effective tile list — either `self.tiles` directly,
    /// or the expanded form of `palette + map` when explicit tiles are
    /// absent.
    ///
    /// The ASCII expansion maps `col → q`, `row → r` without offset-
    /// coordinate correction; callers that need exact hex offset layouts
    /// populate `self.tiles` explicitly. Whitespace in `map` rows is
    /// ignored.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::SnapshotLoadError::UnknownPaletteChar`]
    /// when an ASCII `map` row uses a character missing from `palette`.
    pub fn expand_palette( &self ) -> Result< Vec< Tile >, crate::error::SnapshotLoadError >
    {
      use crate::error::SnapshotLoadError;
      let mut out = Vec::new();
      for ( row_index, row ) in self.map.iter().enumerate()
      {
        let mut col : i32 = 0;
        for ch in row.chars()
        {
          if ch.is_whitespace() { continue; }
          if let Some( objects ) = self.palette.get( &ch )
          {
            out.push( Tile { pos : ( col, row_index as i32 ), objects : objects.clone() } );
          }
          else
          {
            return Err( SnapshotLoadError::UnknownPaletteChar
            {
              ch,
              pos : ( col, row_index as i32 ),
            });
          }
          col = col.saturating_add( 1 );
        }
      }
      Ok( out )
    }
  }

  /// Scene metadata — **optional, file-centric**.
  ///
  /// Both fields are plumbing for loading scenes from disk. For
  /// runtime-constructed scenes, leave them as [`None`] (see
  /// [`SceneMeta::default`]). The compile layer never reads them.
  #[ derive( Debug, Clone, Default, Serialize, Deserialize ) ]
  pub struct SceneMeta
  {
    /// Human-readable scene name. Informational only — used by editors and
    /// logs; the compile / render pipeline ignores it.
    #[ serde( default ) ]
    pub name : Option< String >,
    /// Path to the render spec this scene was authored against, relative to
    /// the scene file. Only meaningful when the scene was loaded from disk
    /// with an explicit companion spec.
    #[ serde( default ) ]
    pub render_spec : Option< String >,
  }

  /// Inclusive axial-coordinate bounds of the grid.
  ///
  /// Used for culling optimisations. Use [`Bounds::unbounded`] when you don't
  /// want culling (e.g. when building a scene incrementally and the extent
  /// is unknown).
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize ) ]
  pub struct Bounds
  {
    /// Lower bound `( q_min, r_min )`.
    pub min : ( i32, i32 ),
    /// Upper bound `( q_max, r_max )`.
    pub max : ( i32, i32 ),
  }

  impl Bounds
  {
    /// A bounds rectangle covering the entire `i32` axial coordinate space —
    /// effectively "no culling".
    #[ inline ]
    #[ must_use ]
    pub fn unbounded() -> Self
    {
      Self { min : ( i32::MIN, i32::MIN ), max : ( i32::MAX, i32::MAX ) }
    }
  }

  /// One hex cell with its object stack. Rendered bottom-to-top.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct Tile
  {
    /// Cell position in axial `( q, r )` coordinates.
    pub pos : ( i32, i32 ),
    /// Objects present on this cell, by id (terrain first by convention, then overlays).
    pub objects : Vec< String >,
  }

  /// Instance of an [`crate::anchor::Anchor::Edge`] object.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct EdgeInstance
  {
    /// The edge location.
    pub at : EdgePosition,
    /// Object id referenced.
    pub object : String,
    /// Optional non-default animation name.
    #[ serde( default ) ]
    pub animation : Option< String >,
  }

  /// An edge identified by `( hex, direction )` — the canonical form is picked
  /// at validation time, but scene files may use either side of the edge.
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize ) ]
  pub struct EdgePosition
  {
    /// Owning hex in axial coordinates.
    pub hex : ( i32, i32 ),
    /// Direction to the neighbour across the edge.
    pub dir : EdgeDirection,
  }

  /// Instance of an [`crate::anchor::Anchor::Multihex`] object.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct MultihexInstance
  {
    /// Anchor cell the shape is placed relative to.
    pub anchor : ( i32, i32 ),
    /// Object id referenced.
    pub object : String,
    /// Optional non-default animation name.
    #[ serde( default ) ]
    pub animation : Option< String >,
  }

  /// Instance of an [`crate::anchor::Anchor::FreePos`] object.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct FreeInstance
  {
    /// World-space pixel position `( x, y )`.
    pub pos : ( f32, f32 ),
    /// Object id referenced.
    pub object : String,
    /// Optional non-default animation name.
    #[ serde( default ) ]
    pub animation : Option< String >,
  }

  /// Instance of an [`crate::anchor::Anchor::Viewport`] object.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct ViewportInstance
  {
    /// Object id referenced.
    pub object : String,
    /// Optional non-default animation name.
    #[ serde( default ) ]
    pub animation : Option< String >,
  }

  /// A game-piece placed on a hex cell (typically a unit).
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct Entity
  {
    /// Cell position in axial `( q, r )`.
    pub at : ( i32, i32 ),
    /// Object id referenced (declared in render spec).
    pub object : String,
    /// Owning player (index into `Scene.players`).
    pub owner : u32,
    /// Optional animation override.
    #[ serde( default ) ]
    pub animation : Option< String >,
    /// Facing direction, for sprite mirroring (game-logic concern).
    #[ serde( default ) ]
    pub facing : Option< EdgeDirection >,
  }

  /// A player / faction declaration. Referenced by `Entity.owner`.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct Player
  {
    /// Player id (usually matches array index).
    pub id : u32,
    /// Team colour (`"#rrggbb"` / `"#rrggbbaa"`). Feeds `MaskTint::TeamColor`.
    pub color : String,
    /// Human-readable name.
    pub name : String,
  }
}

mod_interface::mod_interface!
{
  exposed use SceneSnapshot;
  exposed use SceneMeta;
  exposed use Bounds;
  exposed use Tile;
  exposed use EdgeInstance;
  exposed use EdgePosition;
  exposed use MultihexInstance;
  exposed use FreeInstance;
  exposed use ViewportInstance;
  exposed use Entity;
  exposed use Player;
}
