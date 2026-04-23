//! Object-level layer and its per-sprite behaviour.
//!
//! An [`ObjectLayer`] is one textured strip inside an object. An object is an
//! ordered stack of layers drawn bottom-to-top within a pipeline bucket. See
//! SPEC §1.2 and §6.

mod private
{
  use serde::{ Deserialize, Serialize };
  use crate::scene_model::resource::{ TintRef, EffectRef, BlendMode };
  use crate::scene_model::source::SpriteSource;

  /// One textured layer inside an [`crate::scene_model::object::Object`].
  ///
  /// Layers combine a [`SpriteSource`] (what to draw) with a
  /// [`LayerBehaviour`] (how to draw it). Ordering within an object is given
  /// by `z_in_object`; layers may optionally override the object's pipeline
  /// bucket via `pipeline_layer` — see SPEC §8.3.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct ObjectLayer
  {
    /// Optional human-readable id (diagnostic use; not required unique).
    #[ serde( default ) ]
    pub id : Option< String >,
    /// Rule that picks the sprite / frame for this layer each render call.
    pub sprite_source : SpriteSource,
    /// Draw-time behaviour applied to the sampled sprite.
    #[ serde( default ) ]
    pub behaviour : LayerBehaviour,
    /// Ordering within the parent object's layer stack. Higher = later (on top).
    #[ serde( default ) ]
    pub z_in_object : i32,
    /// Optional pipeline-bucket override. When `None`, the layer inherits the
    /// object's `global_layer`. When set, routes this single layer's draw calls
    /// into a different bucket — used for Wesnoth-style edge transitions and
    /// similar multi-pass idioms. See SPEC §8.3.
    #[ serde( default ) ]
    pub pipeline_layer : Option< String >,
  }

  /// Draw-time behaviour for an [`ObjectLayer`]. See SPEC §6.
  ///
  /// All fields are optional; defaults reduce to "draw the sampled sprite
  /// unmodified with normal alpha compositing".
  #[ derive( Debug, Clone, Default, Serialize, Deserialize ) ]
  pub struct LayerBehaviour
  {
    /// Colour-tinting strategy. SPEC §6.1.
    #[ serde( default ) ]
    pub tint : TintBehaviour,
    /// Compositing mode against earlier layers. SPEC §6.2.
    #[ serde( default = "default_blend_mode" ) ]
    pub blend : BlendMode,
    /// Static alpha multiplier `0.0..=1.0`. SPEC §6.5.
    #[ serde( default = "default_alpha" ) ]
    pub alpha : f32,
    /// Shader effects applied after sampling / tinting. SPEC §6.3.
    #[ serde( default ) ]
    pub effects : Vec< EffectRef >,
    /// Parallax factor — Viewport anchor only. `0.0` = pinned to screen,
    /// `1.0` = moves with world, `>1.0` = foreground parallax. SPEC §6.4.
    #[ serde( default ) ]
    pub parallax : Option< f32 >,
    /// Autonomous texture scroll in world-pixels-per-second — Viewport anchor only.
    #[ serde( default ) ]
    pub scroll_velocity : Option< ( f32, f32 ) >,
  }

  #[ inline ]
  fn default_blend_mode() -> BlendMode { BlendMode::Normal }

  #[ inline ]
  fn default_alpha() -> f32 { 1.0 }

  /// Tinting mode for a layer. SPEC §6.1.
  #[ derive( Debug, Clone, Default, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum TintBehaviour
  {
    /// No tint applied. Default.
    #[ default ]
    None,
    /// Multiply the whole sprite by a named tint.
    Flat( TintRef ),
    /// Apply a tint only where a mask's alpha is non-zero.
    ///
    /// The mask is sampled per render call using the inner [`SpriteSource`];
    /// if the mask is an [`SpriteSource::Animation`] with the same frame count
    /// as the body layer, the two stay synchronised automatically (SPEC §7.3).
    Masked
    {
      /// Sprite source sampled as the mask.
      mask : Box< SpriteSource >,
      /// Tint to apply through the mask.
      tint : MaskTint,
    },
  }

  /// Tint applied inside [`TintBehaviour::Masked`].
  ///
  /// Separate from [`TintRef`] because some tints are resolved at runtime from
  /// game state rather than declared in the spec (`TeamColor`, `FogDependent`).
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum MaskTint
  {
    /// A declared tint referenced by name.
    Ref( TintRef ),
    /// Resolved at render time from the object instance's `owner` field
    /// against `Scene.players[].color`.
    TeamColor,
    /// Resolved from fog-of-war visibility state (visible / explored / unseen).
    FogDependent,
  }
}

mod_interface::mod_interface!
{
  own use ObjectLayer;
  own use LayerBehaviour;
  own use TintBehaviour;
  own use MaskTint;
}
