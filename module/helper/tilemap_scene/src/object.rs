//! Object — the atomic renderable unit of a scene.
//!
//! See SPEC §1.1. An object has an anchor, a pipeline bucket, and a set of
//! named visual states (each a list of layers). At runtime one state is
//! active per instance; the game calls `set_state(instance, name)` to switch
//! between them. Each state's layer stack can use any `SpriteSource` —
//! static, animated, or composite — so "state" covers both "idle vs attack"
//! character moods and simpler one-state-with-one-sprite setups.

mod private
{
  use serde::{ Deserialize, Serialize };
  use rustc_hash::FxHashMap as HashMap;
  use crate::anchor::{ Anchor, SortYSource };
  use crate::layer::ObjectLayer;

  /// Declaration of one object class. Referenced from scene data by `id`.
  ///
  /// See SPEC §1.1 and §10.2.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct Object
  {
    /// Unique id for this object within the spec.
    pub id : String,
    /// How instances are attached to the world.
    pub anchor : Anchor,
    /// Default pipeline bucket (z-layer) for this object's draw calls.
    /// Individual layers may override via [`ObjectLayer::pipeline_layer`].
    pub global_layer : String,
    /// Integer priority used by [`crate::source::Condition::NeighborPriorityLower`]
    /// and by conventions such as "lowest-priority-object-on-cell counts as
    /// terrain" when computing vertex corners.
    #[ serde( default ) ]
    pub priority : Option< i32 >,
    /// Optional Y-sort override for [`Anchor::Multihex`] objects.
    #[ serde( default ) ]
    pub sort_y_source : SortYSource,
    /// Sprite anchor point within the sprite's bounding box, in normalized
    /// `[0..1]` coords. `( 0.5, 0.5 )` = centre (default, good for tile
    /// terrain). `( 0.5, 0.0 )` = bottom-centre (feet; good for units).
    /// `( 0.0, 0.0 )` = bottom-left (raw backend anchor; rarely useful).
    /// The compile layer shifts `transform.position` by
    /// `-( pivot.0 * w * zoom, pivot.1 * h * zoom )` so the object's scene
    /// anchor (hex centre, triangle centroid, etc.) falls on this pivot
    /// point of the sprite.
    #[ serde( default = "default_pivot" ) ]
    pub pivot : ( f32, f32 ),
    /// State name active when the game hasn't issued `set_state`.
    #[ serde( default = "default_state_name" ) ]
    pub default_state : String,
    /// Map of state name → ordered list of layers to draw while that state
    /// is active. Layer order is by `z_in_object`; for equal values,
    /// declaration order is the tiebreaker. A state's layer stack may use
    /// any `SpriteSource` — a single `Static` sprite, an `Animation` ref,
    /// or a composite neighbour-aware source.
    pub states : HashMap< String, Vec< ObjectLayer > >,
  }

  #[ inline ]
  fn default_state_name() -> String { "default".into() }

  #[ inline ]
  fn default_pivot() -> ( f32, f32 ) { ( 0.5, 0.5 ) }
}

mod_interface::mod_interface!
{
  exposed use Object;
}
