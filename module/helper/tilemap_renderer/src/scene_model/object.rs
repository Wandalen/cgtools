//! Object — the atomic renderable unit of a scene.
//!
//! See SPEC §1.1. An object has an anchor, a pipeline bucket, and a set of
//! named animation stacks (each a list of layers). At runtime one named stack
//! is active per instance; the game calls `set_animation(instance, name)` to
//! switch between them.

mod private
{
  use serde::{ Deserialize, Serialize };
  use std::collections::HashMap;
  use crate::scene_model::anchor::{ Anchor, SortYSource };
  use crate::scene_model::layer::ObjectLayer;

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
    /// Integer priority used by [`crate::scene_model::source::Condition::NeighborPriorityLower`]
    /// and by conventions such as "lowest-priority-object-on-cell counts as
    /// terrain" when computing vertex corners.
    #[ serde( default ) ]
    pub priority : Option< i32 >,
    /// Optional Y-sort override for [`Anchor::Multihex`] objects.
    #[ serde( default ) ]
    pub sort_y_source : SortYSource,
    /// Animation name played when the game hasn't issued `set_animation`.
    #[ serde( default = "default_animation_name" ) ]
    pub default_animation : String,
    /// Map of animation name → ordered list of layers to draw while that
    /// animation is active. Layer order is by `z_in_object`; for equal values,
    /// declaration order is the tiebreaker.
    pub animations : HashMap< String, Vec< ObjectLayer > >,
  }

  #[ inline ]
  fn default_animation_name() -> String { "default".into() }
}

mod_interface::mod_interface!
{
  own use Object;
}
