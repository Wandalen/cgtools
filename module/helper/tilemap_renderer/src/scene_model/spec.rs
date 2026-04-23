//! Top-level `RenderSpec` container — the root of `render_spec.ron`.
//!
//! See SPEC §10.1.

mod private
{
  use serde::{ Deserialize, Serialize };
  use crate::scene_model::object::Object;
  use crate::scene_model::pipeline::RenderPipeline;
  use crate::scene_model::resource::{ Animation, Asset, Effect, Tint };

  /// Root of a scene-model render specification. Ties together declared
  /// resources, object classes, and the pipeline.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct RenderSpec
  {
    /// Semver version of the spec file format.
    pub version : String,
    /// Asset declarations (atlases, sheets, single images).
    #[ serde( default ) ]
    pub assets : Vec< Asset >,
    /// Tint declarations (day/night, fog, highlight, etc.).
    #[ serde( default ) ]
    pub tints : Vec< Tint >,
    /// Animation declarations.
    #[ serde( default ) ]
    pub animations : Vec< Animation >,
    /// Effect declarations (procedural shaders).
    #[ serde( default ) ]
    pub effects : Vec< Effect >,
    /// Object classes declared in this spec.
    pub objects : Vec< Object >,
    /// Pipeline declaration (bucket order, tiling, global tint).
    pub pipeline : RenderPipeline,
  }
}

mod_interface::mod_interface!
{
  own use RenderSpec;
}
