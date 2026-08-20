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
  use crate::layer::{ MaskTint, ObjectLayer, TintBehaviour };
  use crate::pipeline::{ RenderPipeline, TilingStrategy };
  use crate::resource::{ AnimationRef, AnimationTiming, EffectRef, TintRef };
  use crate::snapshot::SceneSnapshot;
  use crate::source::{ NeighborBitmaskSource, SpriteSource };
  use crate::spec::RenderSpec;

  /// Precomputed id sets shared by [`RenderSpec`]'s `validate()` checks —
  /// built once per call and threaded through the per-object / per-layer
  /// helper functions below instead of every helper recomputing them.
  struct SpecIds< 'a >
  {
    /// Declared `pipeline.layers[*].id` values.
    layer : HashSet< &'a str >,
    /// Declared `assets[*].id` values.
    asset : HashSet< &'a str >,
    /// Declared `objects[*].id` values.
    object : HashSet< &'a str >,
    /// Declared `tints[*].id` values.
    tint : HashSet< &'a str >,
    /// Declared `animations[*].id` values.
    anim : HashSet< &'a str >,
    /// Declared `effects[*].id` values.
    effect : HashSet< &'a str >,
  }

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
    /// - **Default state exists.** Every object's `default_state` names a
    ///   key present in its `states` map.
    /// - **Reserved ids.** The reserved id `"void"` is not declared as a
    ///   user object id.
    /// - **Id uniqueness.** No two `assets` / `tints` / `animations` /
    ///   `effects` / `objects` entries share an id within their own
    ///   collection.
    /// - **Tint / animation / effect references resolve.** Every `TintRef`
    ///   (`pipeline.global_tint`, `PipelineLayer.tint_mask`,
    ///   `LayerBehaviour.tint`'s `Flat` / `Masked.tint = Ref` forms), every
    ///   `AnimationRef` (`SpriteSource::Animation`, `VertexCorners`
    ///   pattern animations, `TintBehaviour::Masked.mask`, all reachable
    ///   recursively through `Variant` / `NeighborBitmask` /
    ///   `EdgeConnectedBitmask` / `ViewportTiled`), and every `EffectRef`
    ///   (`LayerBehaviour.effects`) resolves to a declared id.
    /// - **`connects_with` entries resolve.** Every `NeighborBitmask` /
    ///   `EdgeConnectedBitmask` `connects_with` entry, reachable anywhere
    ///   in a layer's sprite-source tree, is a declared object id or the
    ///   reserved id `"void"`.
    /// - **Composite-in-composite nesting is illegal.** A composite
    ///   source's nested-source slots (`NeighborBitmask` /
    ///   `EdgeConnectedBitmask`'s mapping/fallback, `ViewportTiled`'s
    ///   `content`) may only resolve — after unwrapping any `Variant` —
    ///   to a leaf source, never another composite.
    /// - **Tiling whitelist.** `pipeline.hex.tiling` is `HexFlatTop` or
    ///   `HexPointyTop`; `Square4` / `Square8` are rejected.
    ///
    /// **Not enforced** — see the `TODO SPEC §16` comment at the end of
    /// this impl for why anchor ↔ sprite-source compatibility is left
    /// unimplemented.
    fn validate( &self ) -> Result< (), Vec< ValidationError > >
    {
      let mut errors : Vec< ValidationError > = Vec::new();

      let ids = SpecIds
      {
        layer : self.pipeline.layers.iter().map( | l | l.id.as_str() ).collect(),
        asset : self.assets.iter().map( | a | a.id.as_str() ).collect(),
        object : self.objects.iter().map( | o | o.id.as_str() ).collect(),
        tint : self.tints.iter().map( | t | t.id.as_str() ).collect(),
        anim : self.animations.iter().map( | a | a.id.as_str() ).collect(),
        effect : self.effects.iter().map( | e | e.id.as_str() ).collect(),
      };

      for object in &self.objects
      {
        if !ids.layer.contains( object.global_layer.as_str() )
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
            layer_checks( &object.id, state_name, layer, &ids, &mut errors );
          }
        }
      }

      for anim in &self.animations
      {
        let mut visit = | asset : &str |
        {
          if !ids.asset.contains( asset )
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

      duplicate_ids_check( "asset", self.assets.iter().map( | a | a.id.as_str() ), &mut errors );
      duplicate_ids_check( "tint", self.tints.iter().map( | t | t.id.as_str() ), &mut errors );
      duplicate_ids_check( "animation", self.animations.iter().map( | a | a.id.as_str() ), &mut errors );
      duplicate_ids_check( "effect", self.effects.iter().map( | e | e.id.as_str() ), &mut errors );
      duplicate_ids_check( "object", self.objects.iter().map( | o | o.id.as_str() ), &mut errors );

      pipeline_tint_checks( &self.pipeline, &ids.tint, &mut errors );
      tiling_check( &self.pipeline, &mut errors );

      // TODO SPEC §16: anchor ↔ sprite-source compatibility (SPEC §3, §5).
      //
      // Left unimplemented — needs spec clarification, not a guess. The
      // docs disagree with each other and with the actual compile-time
      // dispatch:
      //   - `docs/format/003` says Hex accepts "all variants" and Vertex
      //     is *required* for VertexCorners sources.
      //   - `docs/format/005`'s per-variant table instead scopes each
      //     composite to exactly one anchor (NeighborBitmask /
      //     NeighborCondition → Hex, VertexCorners → Vertex,
      //     EdgeConnectedBitmask → Edge, ViewportTiled → Viewport) and
      //     marks `External` "applicable to all anchors".
      //   - `compile/frame.rs::resolve_vertex_pass_all` scans *every* object
      //     for VertexCorners layers with no anchor check at all —
      //     confirmed intentional by the explicit comment on the `blend`
      //     object in `tests/scene_model_compile_test.rs`'s
      //     `vertex_corners_three_way_blend`: "anchor type of the owning
      //     object doesn't matter for VertexCorners pass". `Placement`
      //     (`src/instance.rs`) also has no `Vertex` variant at all, so
      //     `Anchor::Vertex` can never actually be instantiated — the
      //     docs' own required anchor for VertexCorners is unreachable.
      //   - `compile/frame.rs::edge_sprite_source_resolve` has no case
      //     for `External`, so the docs' claim that External works on
      //     Edge would itself fail at compile time today.
      // Implementing this rule against the literal docs would flag a
      // deliberately-passing, intentional test as invalid. Needs the docs
      // and the compile layer reconciled first.

      if errors.is_empty() { Ok( () ) } else { Err( errors ) }
    }
  }

  impl Validate for SceneSnapshot
  {
    /// Enforces:
    ///
    /// - **Tile-source exclusivity.** `tiles` and the `(palette, map)`
    ///   shorthand are mutually exclusive. `Scene::from_snapshot` silently
    ///   prefers `tiles` and drops `map` when both are populated (see
    ///   `SceneSnapshot::palette_expand`); this rule catches that
    ///   silent-data-loss case at load time instead. A snapshot with
    ///   neither populated is not flagged — that is the ordinary shape of
    ///   `SceneSnapshot::new`.
    /// - **Entity owner bounds.** Every `entities[*].owner` is a valid
    ///   index into `players` (matches the field's own doc comment:
    ///   "index into Scene.players").
    ///
    /// **Not checked here.** Palette character coverage folds into
    /// `palette_expand`'s own `UnknownPaletteChar` error at scene-build
    /// time, not `validate()`. Three more SPEC §16 rules — palette
    /// characters resolving to a declared object, tile / edge / multihex /
    /// free / viewport instance object ids resolving, and
    /// `initial_global_tint` resolving — need `&RenderSpec` alongside the
    /// snapshot, which this trait method's `&self`-only signature can't
    /// take. They ARE enforced, just not by this method — see the
    /// `SPEC §16 note` comment near the end of this impl for where.
    fn validate( &self ) -> Result< (), Vec< ValidationError > >
    {
      let mut errors : Vec< ValidationError > = Vec::new();

      if !self.tiles.is_empty() && !self.map.is_empty()
      {
        errors.push( ValidationError::ConflictingTileSource
        {
          tiles_len : self.tiles.len(),
          map_rows : self.map.len(),
        });
      }

      for ( i, entity ) in self.entities.iter().enumerate()
      {
        if entity.owner as usize >= self.players.len()
        {
          errors.push( ValidationError::UnresolvedRef
          {
            kind : "player",
            id : entity.owner.to_string(),
            context : format!( "entities[{i}] owner" ),
          });
        }
      }

      // SPEC §16 note: three cross-file rules — palette characters
      // resolving to a declared object, tile / edge / multihex / free /
      // viewport instance object ids resolving, and `initial_global_tint`
      // resolving — all need `&RenderSpec` alongside the snapshot, which
      // this method's `&self`-only signature can't take. They are not
      // unimplemented: `crate::scene::Scene::from_snapshot` is the pass
      // that already has both documents loaded, and enforces all three —
      // unresolved instance/palette-derived object ids via
      // `SnapshotLoadError::UnknownObject` (checked at all six
      // instance-collection call sites, including palette-expanded tiles,
      // which flow through the same `tile.objects` loop as explicit
      // tiles), and an unresolved `initial_global_tint` via
      // `SnapshotLoadError::UnknownTint`. Direct test coverage in
      // `tests/scene_model_compile_test.rs`:
      // `from_snapshot_rejects_unknown_tile_object` (`UnknownObject`, tile
      // case — palette-expanded tiles share this exact code path, though
      // no test combines palette expansion with an unresolved target
      // object) and `from_snapshot_rejects_unknown_initial_global_tint` /
      // `from_snapshot_accepts_known_initial_global_tint` (`UnknownTint`).

      if errors.is_empty() { Ok( () ) } else { Err( errors ) }
    }
  }

  /// Walks a [`SpriteSource`] tree and calls `f( asset_id, where_ )` for
  /// every asset id directly named by a node. `Animation` and `External`
  /// nodes are leaves of this walk — animation bodies are validated
  /// separately, external sources are runtime-supplied.
  fn asset_refs_visit( source : &SpriteSource, f : &mut impl FnMut( &str, &str ) )
  {
    match source
    {
      SpriteSource::Static( sr ) => f( &sr.asset, "Static.sprite.asset" ),
      SpriteSource::Variant { variants, .. } =>
      {
        for v in variants { asset_refs_visit( &v.sprite, f ); }
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
            for inner in mapping.values() { asset_refs_visit( inner, f ); }
            asset_refs_visit( fallback, f );
          },
          NeighborBitmaskSource::ByAtlas { asset, .. } =>
            f( asset, "NeighborBitmaskSource::ByAtlas.asset" ),
        }
      },
      SpriteSource::ViewportTiled { content, .. } =>
        asset_refs_visit( content, f ),
      SpriteSource::Animation( _ ) | SpriteSource::External { .. } => {},
    }
  }

  /// Records a [`ValidationError::DuplicateId`] for every `ids` entry
  /// beyond its first occurrence.
  fn duplicate_ids_check< 'a >
  (
    kind : &'static str,
    ids : impl Iterator< Item = &'a str >,
    errors : &mut Vec< ValidationError >,
  )
  {
    let mut seen : HashSet< &str > = HashSet::default();
    for id in ids
    {
      if !seen.insert( id )
      {
        errors.push( ValidationError::DuplicateId { kind, id : id.to_owned() } );
      }
    }
  }

  /// Runs every per-layer SPEC §16 rule against one `ObjectLayer` —
  /// `pipeline_layer` override resolution, asset / animation / tint /
  /// effect reference resolution, `connects_with` validity, and
  /// composite-nesting legality — pushing violations into `errors`.
  fn layer_checks
  (
    object_id : &str,
    state_name : &str,
    layer : &ObjectLayer,
    ids : &SpecIds< '_ >,
    errors : &mut Vec< ValidationError >,
  )
  {
    if let Some( pl ) = layer.pipeline_layer.as_deref()
      && !ids.layer.contains( pl )
    {
      errors.push( ValidationError::UnresolvedRef
      {
        kind : "pipeline layer",
        id : pl.to_owned(),
        context : format!( "object {object_id:?} state {state_name:?} layer pipeline_layer override" ),
      });
    }

    asset_refs_visit( &layer.sprite_source, &mut | asset, where_ |
    {
      if !ids.asset.contains( asset )
      {
        errors.push( ValidationError::UnresolvedRef
        {
          kind : "asset",
          id : asset.to_owned(),
          context : format!( "object {object_id:?} state {state_name:?} sprite_source {where_}" ),
        });
      }
    });

    animation_refs_visit( &layer.sprite_source, &mut | id |
    {
      if !ids.anim.contains( id )
      {
        errors.push( ValidationError::UnresolvedRef
        {
          kind : "animation",
          id : id.to_owned(),
          context : format!( "object {object_id:?} state {state_name:?} sprite_source" ),
        });
      }
    });

    connects_with_visit( &layer.sprite_source, &mut | id |
    {
      if id != VOID_ID && !ids.object.contains( id )
      {
        errors.push( ValidationError::UnresolvedRef
        {
          kind : "object",
          id : id.to_owned(),
          context : format!( "object {object_id:?} state {state_name:?} sprite_source connects_with" ),
        });
      }
    });

    illegal_nesting_visit( &layer.sprite_source, &mut | outer, inner |
    {
      errors.push( ValidationError::IllegalSourceNesting { outer, inner } );
    });

    for EffectRef( id ) in &layer.behaviour.effects
    {
      if !ids.effect.contains( id.as_str() )
      {
        errors.push( ValidationError::UnresolvedRef
        {
          kind : "effect",
          id : id.clone(),
          context : format!( "object {object_id:?} state {state_name:?} layer behaviour.effects" ),
        });
      }
    }

    layer_tint_check( object_id, state_name, layer, ids, errors );
  }

  /// The `LayerBehaviour.tint` half of [`layer_checks`] — split out purely
  /// to keep both functions short; not an independent rule.
  fn layer_tint_check
  (
    object_id : &str,
    state_name : &str,
    layer : &ObjectLayer,
    ids : &SpecIds< '_ >,
    errors : &mut Vec< ValidationError >,
  )
  {
    match &layer.behaviour.tint
    {
      TintBehaviour::None => {},
      TintBehaviour::Flat( TintRef( id ) ) =>
      {
        if !ids.tint.contains( id.as_str() )
        {
          errors.push( ValidationError::UnresolvedRef
          {
            kind : "tint",
            id : id.clone(),
            context : format!( "object {object_id:?} state {state_name:?} layer behaviour.tint" ),
          });
        }
      },
      TintBehaviour::Masked { mask, tint } =>
      {
        if let MaskTint::Ref( TintRef( id ) ) = tint
          && !ids.tint.contains( id.as_str() )
        {
          errors.push( ValidationError::UnresolvedRef
          {
            kind : "tint",
            id : id.clone(),
            context : format!( "object {object_id:?} state {state_name:?} layer behaviour.tint (masked)" ),
          });
        }

        animation_refs_visit( mask, &mut | id |
        {
          if !ids.anim.contains( id )
          {
            errors.push( ValidationError::UnresolvedRef
            {
              kind : "animation",
              id : id.to_owned(),
              context : format!( "object {object_id:?} state {state_name:?} layer behaviour.tint mask" ),
            });
          }
        });
      },
    }
  }

  /// Checks `pipeline.global_tint` and every `PipelineLayer.tint_mask`
  /// resolve to a declared tint id.
  fn pipeline_tint_checks( pipeline : &RenderPipeline, tint_ids : &HashSet< &str >, errors : &mut Vec< ValidationError > )
  {
    if let Some( TintRef( id ) ) = &pipeline.global_tint
      && !tint_ids.contains( id.as_str() )
    {
      errors.push( ValidationError::UnresolvedRef
      {
        kind : "tint",
        id : id.clone(),
        context : "pipeline.global_tint".into(),
      });
    }
    for pl in &pipeline.layers
    {
      if let Some( TintRef( id ) ) = &pl.tint_mask
        && !tint_ids.contains( id.as_str() )
      {
        errors.push( ValidationError::UnresolvedRef
        {
          kind : "tint",
          id : id.clone(),
          context : format!( "pipeline layer {:?} tint_mask", pl.id ),
        });
      }
    }
  }

  /// Checks `pipeline.hex.tiling` is one of the supported strategies —
  /// `Square4` / `Square8` are reserved, not yet implemented.
  fn tiling_check( pipeline : &RenderPipeline, errors : &mut Vec< ValidationError > )
  {
    match pipeline.hex.tiling
    {
      TilingStrategy::HexFlatTop | TilingStrategy::HexPointyTop => {},
      TilingStrategy::Square4 | TilingStrategy::Square8 =>
      {
        errors.push( ValidationError::UnsupportedTiling( format!( "{:?}", pipeline.hex.tiling ) ) );
      },
    }
  }

  /// Walks every [`AnimationRef`] reachable within a [`SpriteSource`] tree —
  /// `Animation` leaves, `VertexCorners` pattern animations, and anything
  /// nested behind `Variant` / `NeighborBitmask` / `EdgeConnectedBitmask` /
  /// `ViewportTiled` — calling `f` with each one's id.
  fn animation_refs_visit( source : &SpriteSource, f : &mut impl FnMut( &str ) )
  {
    match source
    {
      SpriteSource::Animation( AnimationRef( id ) ) => f( id ),
      SpriteSource::Variant { variants, .. } =>
      {
        for v in variants { animation_refs_visit( &v.sprite, f ); }
      },
      SpriteSource::VertexCorners { patterns, .. } =>
      {
        for p in patterns
        {
          if let Some( AnimationRef( id ) ) = &p.animation { f( id ); }
        }
      },
      SpriteSource::NeighborBitmask { source, .. }
      | SpriteSource::EdgeConnectedBitmask { source, .. } =>
      {
        if let NeighborBitmaskSource::ByMapping { mapping, fallback } = source
        {
          for inner in mapping.values() { animation_refs_visit( inner, f ); }
          animation_refs_visit( fallback, f );
        }
      },
      SpriteSource::ViewportTiled { content, .. } => animation_refs_visit( content, f ),
      SpriteSource::Static( _ ) | SpriteSource::External { .. } | SpriteSource::NeighborCondition { .. } => {},
    }
  }

  /// Walks every `connects_with` list reachable within a [`SpriteSource`]
  /// tree (`NeighborBitmask` / `EdgeConnectedBitmask`, including instances
  /// nested behind a `Variant`), calling `f` with each entry.
  fn connects_with_visit( source : &SpriteSource, f : &mut impl FnMut( &str ) )
  {
    match source
    {
      SpriteSource::NeighborBitmask { connects_with, source }
      | SpriteSource::EdgeConnectedBitmask { connects_with, source, .. } =>
      {
        for id in connects_with { f( id ); }
        if let NeighborBitmaskSource::ByMapping { mapping, fallback } = source
        {
          for inner in mapping.values() { connects_with_visit( inner, f ); }
          connects_with_visit( fallback, f );
        }
      },
      SpriteSource::Variant { variants, .. } =>
      {
        for v in variants { connects_with_visit( &v.sprite, f ); }
      },
      SpriteSource::ViewportTiled { content, .. } => connects_with_visit( content, f ),
      SpriteSource::Static( _ ) | SpriteSource::Animation( _ ) | SpriteSource::External { .. }
      | SpriteSource::NeighborCondition { .. } | SpriteSource::VertexCorners { .. } => {},
    }
  }

  /// Returns the composite's variant name if `source` is directly a
  /// composite, or — when `source` is a `Variant` — the first composite
  /// reachable by unwrapping its alternatives (`Variant` is a leaf and
  /// composes transparently). `None` when every reachable node is a
  /// genuine leaf.
  fn composite_kind_reachable( source : &SpriteSource ) -> Option< &'static str >
  {
    match source
    {
      SpriteSource::NeighborBitmask { .. }      => Some( "NeighborBitmask" ),
      SpriteSource::NeighborCondition { .. }    => Some( "NeighborCondition" ),
      SpriteSource::VertexCorners { .. }        => Some( "VertexCorners" ),
      SpriteSource::EdgeConnectedBitmask { .. } => Some( "EdgeConnectedBitmask" ),
      SpriteSource::ViewportTiled { .. }        => Some( "ViewportTiled" ),
      SpriteSource::Variant { variants, .. } =>
        variants.iter().find_map( | v | composite_kind_reachable( &v.sprite ) ),
      SpriteSource::Static( _ ) | SpriteSource::Animation( _ ) | SpriteSource::External { .. } => None,
    }
  }

  /// Walks every composite's nested-source slot (`NeighborBitmask` /
  /// `EdgeConnectedBitmask`'s `source` mapping+fallback, `ViewportTiled`'s
  /// `content`) and calls `f( outer_kind, inner_kind )` whenever a
  /// composite is reachable there — composites may only nest leaf sources
  /// (optionally behind a `Variant`). Recurses through `Variant` at the
  /// top level too, so a composite wrapped in a `Variant` is still found
  /// and its own inner slots are checked.
  fn illegal_nesting_visit( source : &SpriteSource, f : &mut impl FnMut( &'static str, &'static str ) )
  {
    match source
    {
      SpriteSource::NeighborBitmask { source, .. } =>
        bitmask_nesting_check( "NeighborBitmask", source, f ),
      SpriteSource::EdgeConnectedBitmask { source, .. } =>
        bitmask_nesting_check( "EdgeConnectedBitmask", source, f ),
      SpriteSource::ViewportTiled { content, .. } =>
      {
        if let Some( inner ) = composite_kind_reachable( content )
        {
          f( "ViewportTiled", inner );
        }
        illegal_nesting_visit( content, f );
      },
      SpriteSource::Variant { variants, .. } =>
      {
        for v in variants { illegal_nesting_visit( &v.sprite, f ); }
      },
      SpriteSource::Static( _ ) | SpriteSource::Animation( _ ) | SpriteSource::External { .. }
      | SpriteSource::NeighborCondition { .. } | SpriteSource::VertexCorners { .. } => {},
    }
  }

  /// Shared `ByMapping` (mapping values + fallback) nesting check for
  /// [`illegal_nesting_visit`]'s `NeighborBitmask` / `EdgeConnectedBitmask`
  /// arms.
  fn bitmask_nesting_check
  (
    outer : &'static str,
    source : &NeighborBitmaskSource,
    f : &mut impl FnMut( &'static str, &'static str ),
  )
  {
    if let NeighborBitmaskSource::ByMapping { mapping, fallback } = source
    {
      for inner in mapping.values()
      {
        if let Some( inner_kind ) = composite_kind_reachable( inner ) { f( outer, inner_kind ); }
        illegal_nesting_visit( inner, f );
      }
      if let Some( inner_kind ) = composite_kind_reachable( fallback ) { f( outer, inner_kind ); }
      illegal_nesting_visit( fallback, f );
    }
  }
}

mod_interface::mod_interface!
{
  exposed use Validate;
}
