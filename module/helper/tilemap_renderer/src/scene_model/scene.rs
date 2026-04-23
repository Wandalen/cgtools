//! Per-level scene data.
//!
//! A [`Scene`] is a **pure rendering description** — "what is where, to be drawn".
//! It carries no game mechanics (no HP, no AI state, no inventory). Games keep
//! their own state in their own structs and either project it into a `Scene`
//! each frame, or mutate a persistent `Scene` alongside.
//!
//! `Scene` is a plain `#[derive(Serialize, Deserialize)]` type first, a RON
//! file format second. Construct it in code directly (all fields are public),
//! deserialize it from RON / JSON / anything `serde` supports, or convert it
//! from your own format — all three are first-class. See [`Scene::new`] for
//! the common constructor.
//!
//! See SPEC §5 (file structure) and §10.3 for the companion format doc.

mod private
{
  use serde::{ Deserialize, Serialize };
  use std::collections::HashMap;
  use crate::scene_model::anchor::EdgeDirection;

  /// Top-level scene file payload.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct Scene
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
  }

  impl Scene
  {
    /// Creates an empty scene bounded by `bounds`. All collections start empty,
    /// metadata fields are `None`. The typical runtime pattern:
    ///
    /// ```ignore
    /// let mut scene = Scene::new( Bounds::unbounded() );
    /// scene.tiles.push( Tile { pos : ( 0, 0 ), objects : vec![ "grass".into() ] } );
    /// scene.entities.push( Entity { at : ( 1, 1 ), object : "knight".into(), owner : 0,
    ///                               animation : None, facing : None } );
    /// let cmds = compile_frame( &spec, &scene, &compiled, &camera, time )?;
    /// ```
    #[ inline ]
    #[ must_use ]
    pub fn new( bounds : Bounds ) -> Self
    {
      Self
      {
        meta : SceneMeta::default(),
        bounds,
        tiles : Vec::new(),
        palette : HashMap::new(),
        map : Vec::new(),
        edges : Vec::new(),
        multihex_instances : Vec::new(),
        free_instances : Vec::new(),
        viewport_instances : Vec::new(),
        entities : Vec::new(),
        players : Vec::new(),
        initial_global_tint : None,
      }
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
    /// with an explicit companion spec. Not read by [`crate::scene_model::compile::compile_frame`]
    /// — callers pass the spec separately by reference.
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

  /// Instance of an [`crate::scene_model::anchor::Anchor::Edge`] object.
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

  /// Instance of an [`crate::scene_model::anchor::Anchor::Multihex`] object.
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

  /// Instance of an [`crate::scene_model::anchor::Anchor::FreePos`] object.
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

  /// Instance of an [`crate::scene_model::anchor::Anchor::Viewport`] object.
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
  own use Scene;
  own use SceneMeta;
  own use Bounds;
  own use Tile;
  own use EdgeInstance;
  own use EdgePosition;
  own use MultihexInstance;
  own use FreeInstance;
  own use ViewportInstance;
  own use Entity;
  own use Player;
}
