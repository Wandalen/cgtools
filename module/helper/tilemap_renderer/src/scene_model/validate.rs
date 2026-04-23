//! Validation — structural checks enumerated by SPEC §16.
//!
//! This phase ships the validation **skeleton**: the trait, the error type
//! (see [`crate::scene_model::error::ValidationError`]), and entry points
//! that currently return `Ok`. Each individual rule is staked out as a
//! `// TODO SPEC §16` comment inside the relevant branch, to be filled in
//! as the implementation matures alongside the rendering phase.
//!
//! Validation collects all violations — it does not early-return on the first —
//! so a caller can present the entire failure set at once.

mod private
{
  use crate::scene_model::error::ValidationError;
  use crate::scene_model::scene::Scene;
  use crate::scene_model::spec::RenderSpec;

  /// Trait implemented by types that validate their own content against the
  /// SPEC §16 rule set.
  pub trait Validate
  {
    /// Runs all validation rules, collecting every violation found.
    ///
    /// # Errors
    ///
    /// Returns `Err` with one [`ValidationError`] per violation when any
    /// rule fails. Returns `Ok(())` when all rules pass.
    fn validate( &self ) -> Result< (), Vec< ValidationError > >;
  }

  impl Validate for RenderSpec
  {
    fn validate( &self ) -> Result< (), Vec< ValidationError > >
    {
      #[ allow( unused_mut ) ]   // populated as rules land
      let mut errors : Vec< ValidationError > = Vec::new();

      // TODO SPEC §16: every Asset.id / Tint.id / Animation.id / Effect.id / Object.id unique within its kind.
      // TODO SPEC §16: every SpriteRef( asset_id, _ ) refers to a declared asset.
      // TODO SPEC §16: every TintRef / AnimationRef / EffectRef resolves.
      // TODO SPEC §16: every NeighborBitmask.connects_with entry is a declared object id or "void".
      // TODO SPEC §16: pipeline layer ids are unique; every Object.global_layer and
      //                ObjectLayer.pipeline_layer references a declared pipeline layer.
      // TODO SPEC §16: for each object — default_animation exists in animations.
      // TODO SPEC §16: anchor ↔ sprite-source compatibility (SPEC §3, §5).
      // TODO SPEC §16: composite-in-composite illegal nesting detection.
      // TODO SPEC §16: reserved id "void" not declared as user object.
      // TODO SPEC §16: pipeline.tiling is one of the supported values
      //                (HexFlatTop / HexPointyTop in 0.2.0; Square4 / Square8 rejected).

      if errors.is_empty() { Ok( () ) } else { Err( errors ) }
    }
  }

  impl Validate for Scene
  {
    fn validate( &self ) -> Result< (), Vec< ValidationError > >
    {
      #[ allow( unused_mut ) ]
      let mut errors : Vec< ValidationError > = Vec::new();

      // TODO SPEC §16: exactly one of (tiles) vs (palette + map) is provided.
      // TODO SPEC §16: palette characters map to valid object ids in the linked render spec.
      // TODO SPEC §16: entity.owner indexes into Scene.players.
      // TODO SPEC §16: tile / edge / multihex / free / viewport instance
      //                object ids resolve in the linked render spec.
      // TODO SPEC §16: initial_global_tint (if set) resolves in the linked render spec.
      //
      // Note: cross-file checks (Scene → RenderSpec) run in a separate pass that has
      //       access to both loaded files; this method only checks Scene-internal rules.

      if errors.is_empty() { Ok( () ) } else { Err( errors ) }
    }
  }
}

mod_interface::mod_interface!
{
  own use Validate;
}
