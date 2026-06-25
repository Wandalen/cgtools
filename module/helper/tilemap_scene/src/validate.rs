//! Validation — structural checks enumerated by SPEC §16.
//!
//! Validation collects all violations and reports them together — it does
//! not early-return on the first — so a caller can present the entire
//! failure set at once.
//!
//! Coverage status: the partial set already enforced is listed on each
//! `impl Validate` below; the remaining rules stay as `// TODO SPEC §16`
//! markers in the same place to make the next contribution obvious.

mod private
{
  use rustc_hash::FxHashSet as HashSet;
  use crate::compile::neighbors::VOID_ID;
  use crate::error::ValidationError;
  use crate::resource::AnimationTiming;
  use crate::snapshot::SceneSnapshot;
  use crate::source::{ NeighborBitmaskSource, SpriteSource };
  use crate::spec::RenderSpec;

  /// Trait implemented by types that validate their own content against the
  /// SPEC §16 rule set.
  ///
  /// **Partial enforcement.** A subset of the SPEC §16 rules is enforced
  /// today — see each `impl Validate` for the per-type list. Rules not yet
  /// implemented are tracked as `// TODO SPEC §16` markers in the impl
  /// bodies and in `roadmap.md`. A successful `validate()` therefore proves
  /// the input is free of the implemented violations but does **not** prove
  /// full SPEC §16 conformance.
  pub trait Validate
  {
    /// Runs every implemented validation rule, collecting every violation
    /// found.
    ///
    /// # Errors
    ///
    /// Returns `Err` with one [`ValidationError`] per violation when any
    /// rule fails. Returns `Ok(())` when every implemented rule passes.
    fn validate( &self ) -> Result< (), Vec< ValidationError > >;
  }

  impl Validate for RenderSpec
  {
    /// Enforces:
    ///
    /// - **Pipeline-layer references resolve.** Every `Object.global_layer`
    ///   and every `ObjectLayer.pipeline_layer` (when set) names a
    ///   declared `pipeline.layers[*].id`.
    /// - **Asset references resolve.** Every `SpriteRef.asset` reachable
    ///   from any sprite source — including `Static`, `Variant` (recursive),
    ///   `NeighborBitmask::ByMapping` (recursive) / `ByAtlas`,
    ///   `NeighborCondition`, `VertexCorners`, `EdgeConnectedBitmask`,
    ///   `ViewportTiled` — and every asset id named by an `AnimationTiming`
    ///   variant resolves to a declared `assets[*].id`. `Animation` /
    ///   `External` sources stop the walk at their boundary; animation
    ///   bodies are validated separately when iterating `spec.animations`.
    /// - **Corner-source layers resolve.** Every `VertexCorners.corner_source`
    ///   (when set) names a layer used by at least one object's `global_layer`;
    ///   otherwise corner resolution silently falls back to `VOID_ID`.
    /// - **Default state exists.** Every object's `default_state` names a
    ///   key present in its `states` map.
    /// - **Reserved ids.** The reserved id `"void"` is not declared as a
    ///   user object id.
    fn validate( &self ) -> Result< (), Vec< ValidationError > >
    {
      let mut errors : Vec< ValidationError > = Vec::new();

      let asset_ids : HashSet< &str > =
        self.assets.iter().map( | a | a.id.as_str() ).collect();
      let layer_ids : HashSet< &str > =
        self.pipeline.layers.iter().map( | l | l.id.as_str() ).collect();
      // The set of layer names a `VertexCorners.corner_source` can resolve
      // against: `tile_corner_id` matches `corner_source` to an object's
      // `global_layer`, so a value naming no object's `global_layer` can only
      // ever fall back to `VOID_ID` for every corner (silent, geometrically
      // wrong output). Validating against this set catches such misspellings.
      let global_layer_names : HashSet< &str > =
        self.objects.iter().map( | o | o.global_layer.as_str() ).collect();

      for object in &self.objects
      {
        if !layer_ids.contains( object.global_layer.as_str() )
        {
          errors.push( ValidationError::UnresolvedRef
          {
            kind : "pipeline layer",
            id : object.global_layer.clone(),
            context : format!( "object {:?} global_layer", object.id ),
          });
        }

        for ( state_name, layers ) in &object.states
        {
          for layer in layers
          {
            if let Some( pl ) = layer.pipeline_layer.as_deref()
              && !layer_ids.contains( pl )
            {
              errors.push( ValidationError::UnresolvedRef
              {
                kind : "pipeline layer",
                id : pl.to_owned(),
                context : format!
                (
                  "object {:?} state {:?} layer pipeline_layer override",
                  object.id, state_name,
                ),
              });
            }

            if let SpriteSource::VertexCorners { corner_source : Some( cs ), .. } = &layer.sprite_source
              && !global_layer_names.contains( cs.as_str() )
            {
              errors.push( ValidationError::UnresolvedRef
              {
                kind : "corner_source layer",
                id : cs.clone(),
                context : format!
                (
                  "object {:?} state {:?} VertexCorners corner_source",
                  object.id, state_name,
                ),
              });
            }

            visit_asset_refs( &layer.sprite_source, &mut | asset, where_ |
            {
              if !asset_ids.contains( asset )
              {
                errors.push( ValidationError::UnresolvedRef
                {
                  kind : "asset",
                  id : asset.to_owned(),
                  context : format!
                  (
                    "object {:?} state {:?} sprite_source {where_}",
                    object.id, state_name,
                  ),
                });
              }
            });
          }
        }
      }

      for anim in &self.animations
      {
        let mut visit = | asset : &str |
        {
          if !asset_ids.contains( asset )
          {
            errors.push( ValidationError::UnresolvedRef
            {
              kind : "asset",
              id : asset.to_owned(),
              context : format!( "animation {:?}", anim.id ),
            });
          }
        };
        match &anim.timing
        {
          AnimationTiming::Regular { frames, .. } =>
          {
            for f in frames { visit( &f.asset ); }
          },
          AnimationTiming::FromSheet { asset, .. } => visit( asset ),
          AnimationTiming::Irregular { frames } =>
          {
            for f in frames { visit( &f.sprite.asset ); }
          },
        }
      }

      for object in &self.objects
      {
        if !object.states.contains_key( &object.default_state )
        {
          errors.push( ValidationError::MissingDefaultState
          {
            object : object.id.clone(),
            state : object.default_state.clone(),
          });
        }
        if object.id == VOID_ID
        {
          errors.push( ValidationError::ReservedId { id : object.id.clone() } );
        }
      }

      // TODO SPEC §16: every Asset.id / Tint.id / Animation.id / Effect.id / Object.id unique within its kind.
      // TODO SPEC §16: every TintRef / AnimationRef / EffectRef resolves.
      // TODO SPEC §16: every NeighborBitmask.connects_with entry is a declared object id or "void".
      // TODO SPEC §16: anchor ↔ sprite-source compatibility (SPEC §3, §5).
      // TODO SPEC §16: composite-in-composite illegal nesting detection.
      // TODO SPEC §16: pipeline.tiling is one of the supported values
      //                (HexFlatTop / HexPointyTop in 0.2.0; Square4 / Square8 rejected).

      if errors.is_empty() { Ok( () ) } else { Err( errors ) }
    }
  }

  impl Validate for SceneSnapshot
  {
    /// **Not yet enforcing.** Every Scene-internal SPEC §16 rule is still a
    /// `TODO`; the cross-file Scene → `RenderSpec` checks run in a separate
    /// pass that has access to both loaded files. Always returns `Ok(())`
    /// today.
    fn validate( &self ) -> Result< (), Vec< ValidationError > >
    {
      #[ allow( unused_mut ) ]   // mut becomes load-bearing once the TODO SceneSnapshot rules below push errors
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

  /// Walks a [`SpriteSource`] tree and calls `f( asset_id, where_ )` for
  /// every asset id directly named by a node. `Animation` and `External`
  /// nodes are leaves of this walk — animation bodies are validated
  /// separately, external sources are runtime-supplied.
  fn visit_asset_refs( source : &SpriteSource, f : &mut impl FnMut( &str, &str ) )
  {
    match source
    {
      SpriteSource::Static( sr ) => f( &sr.asset, "Static.sprite.asset" ),
      SpriteSource::Variant { variants, .. } =>
      {
        for v in variants { visit_asset_refs( &v.sprite, f ); }
      },
      SpriteSource::NeighborCondition { asset, .. } =>
        f( asset, "NeighborCondition.asset" ),
      SpriteSource::VertexCorners { asset, .. } =>
        f( asset, "VertexCorners.asset" ),
      SpriteSource::NeighborBitmask { source, .. }
      | SpriteSource::EdgeConnectedBitmask { source, .. } =>
      {
        match source
        {
          NeighborBitmaskSource::ByMapping { mapping, fallback } =>
          {
            for inner in mapping.values() { visit_asset_refs( inner, f ); }
            visit_asset_refs( fallback, f );
          },
          NeighborBitmaskSource::ByAtlas { asset, .. } =>
            f( asset, "NeighborBitmaskSource::ByAtlas.asset" ),
        }
      },
      SpriteSource::ViewportTiled { content, .. } =>
        visit_asset_refs( content, f ),
      SpriteSource::Animation( _ ) | SpriteSource::External { .. } => {},
    }
  }
}

mod_interface::mod_interface!
{
  exposed use Validate;
}
