//! Scene → `RenderCommand` stream — the per-frame compilation core driven
//! by [`crate::renderer::Renderer`].
//!
//! Exposes a single `pub` entry [`render_into`] (used internally by
//! `Renderer::render`) plus a collection of `pub(crate)` helpers shared
//! across passes. The module is internal-facing: consumers go through
//! [`crate::renderer::Renderer`] rather than calling helpers directly.

mod private
{
  use tilemap_renderer::commands::{ Clear, RenderCommand, Sprite };
  use crate::anchor::Anchor;
  use crate::compile::animation::animation_frame_resolve;
  use crate::compile::assets::CompiledAssets;
  use crate::compile::camera::Camera;
  use crate::compile::conditions::condition_evaluate;
  use crate::compile::coords::{ hex_to_world_pixel_flat, hex_to_world_pixel_pointy };
  use crate::compile::edges::
  {
    CanonicalEdge,
    canonical_edge,
    edge_connected_bitmask_compute,
    edge_lookup as build_edge_lookup,
    edge_rotation,
    edge_world_pixel,
  };
  use crate::compile::error::CompileError;
  use crate::compile::neighbors::
  {
    neighbor_bitmask_compute,
    dir_name,
    neighbor_offset_by_dir,
    neighbor_state_at,
    tile_lookup as build_tile_lookup,
    tile_max_priority,
  };
  use crate::compile::tint::FrameTints;
  use crate::hash::coord_hash;
  use crate::layer::{ LayerBehaviour, ObjectLayer };
  use crate::object::Object;
  use crate::pipeline::{ SortMode, TilingStrategy };
  use crate::resource::SpriteRef;
  use crate::compile::viewport::{ tiled_positions, viewport_transform };
  use crate::instance::{ Instance, Placement };
  use crate::scene::Scene;
  use crate::snapshot::{ EdgeInstance, EdgePosition, Tile };
  use crate::source::{ NeighborBitmaskSource, SpriteSource, VariantSelection, ViewportTiling };
  use crate::spec::RenderSpec;
  use tilemap_renderer::types::Transform;
  use rustc_hash::FxHashMap as HashMap;
  use super::vertex_corners::{ project_vertex_sprite, resolve_vertex_pass_all, ResolvedVertexSprite };
  pub use super::vertex_corners::VertexResolveCache;

  /// Bundled per-frame context threaded into helper functions.
  ///
  /// Consolidates 10+ parameters into a single `&ctx`. Fields are read-only —
  /// no mutation happens once the context is built in [`render_into`].
  pub( crate ) struct FrameContext< 'a >
  {
    pub( crate ) spec : &'a RenderSpec,
    pub( crate ) compiled : &'a CompiledAssets,
    pub( crate ) camera : &'a Camera,
    /// Read-only access to the live [`Scene`] — currently only for
    /// [`Scene::fade_value`] (`EffectKind::FadeGate`'s live eased progress).
    pub( crate ) scene : &'a Scene,
    pub( crate ) time_seconds : f32,
    pub( crate ) tile_lookup : HashMap< ( i32, i32 ), &'a Tile >,
    pub( crate ) edge_lookup : HashMap< CanonicalEdge, &'a EdgeInstance >,
    pub( crate ) tiling : TilingStrategy,
    pub( crate ) grid_stride : ( u32, u32 ),
    pub( crate ) viewport_size : ( u32, u32 ),
    /// Scene-level seed folded to `u32` for `coord_hash`. Consumed by
    /// `VariantSelection::Random`.
    pub( crate ) scene_seed : u32,
    /// Global tint and declared tints resolved once for this frame; the only
    /// source of an emitted `Sprite.tint` (see [`FrameTints::sprite_tint`]).
    pub( crate ) tints : FrameTints< 'a >,
  }

  /// Build a **world-space** [`Transform`] at `( wx, wy )` with unit scale and no
  /// rotation. Pan and zoom are supplied by the camera view matrix that the
  /// backend applies on the GPU (see [`crate::compile::camera::Camera::to_view_mat3`]),
  /// so a world instance always carries scale `1` and never has to be rewritten
  /// when the camera moves — that is the whole point of emitting world-space.
  pub( crate ) fn make_transform( wx : f32, wy : f32 ) -> Transform
  {
    Transform
    {
      position : [ wx, wy ],
      rotation : 0.0,
      scale : [ 1.0, 1.0 ],
      skew : [ 0.0, 0.0 ],
      depth : 0.0,
    }
  }

  /// Shift the world-space scene-anchor point so the sprite's anchor pixel
  /// lands exactly on the original scene position.
  ///
  /// Backends render sprites with a bottom-left anchor (quad extends
  /// up-right from `transform.position`). To place some arbitrary anchor
  /// point of the sprite onto the scene position, we shift
  /// `transform.position` by the anchor offset **in world pixels**. The camera
  /// view matrix (applied by the backend) then scales that offset by zoom along
  /// with the position, so the anchor stays glued at any zoom — the same result
  /// the old screen-space `offset * zoom` produced, without depending on the
  /// camera here.
  ///
  /// Priority for picking the anchor point:
  ///
  /// 1. **Per-frame pixel anchor** from `FrameSpec::anchor` — pixel offset
  ///    from the sprite's rect top-left. Used when the atlas author knows
  ///    the semantic contact point (feet, ground touch, vertex attachment).
  /// 2. **Object-level normalized `pivot`** — fraction of sprite size.
  ///    Used as a fallback when no per-frame anchor is set.
  pub( crate ) fn pivot_apply
  (
    wx : f32,
    wy : f32,
    pivot : ( f32, f32 ),
    sprite_id : tilemap_renderer::types::ResourceId< tilemap_renderer::types::asset::Sprite >,
    compiled : &CompiledAssets,
  ) -> ( f32, f32 )
  {
    let Some( s ) = compiled.assets.sprites.iter().find( | s | s.id == sprite_id )
    else { return ( wx, wy ); };

    let w = s.region[ 2 ];
    let h = s.region[ 3 ];

    // Pixel anchor dominates when present. `ay` is measured from the rect
    // top-left in image-y-down convention; the sprite renders with Y-up in
    // world, so the offset from sprite bottom in world is `h - ay`. That
    // flipped value is what we subtract to align the anchor pixel with
    // (wx, wy).
    if let Some( [ ax, ay ] ) = compiled.sprite_anchors.get( &sprite_id ).copied()
    {
      return ( wx - ax, wy - ( h - ay ) );
    }

    // Normalized pivot fallback.
    ( wx - pivot.0 * w, wy - pivot.1 * h )
  }

  pub( crate ) fn hex_world_pixel
  (
    q : i32,
    r : i32,
    ctx : &FrameContext< '_ >,
    object_id : &str,
  ) -> Result< ( f32, f32 ), CompileError >
  {
    match ctx.tiling
    {
      TilingStrategy::HexFlatTop   => Ok( hex_to_world_pixel_flat( q, r, ctx.grid_stride ) ),
      TilingStrategy::HexPointyTop => Ok( hex_to_world_pixel_pointy( q, r, ctx.grid_stride ) ),
      TilingStrategy::Square4 | TilingStrategy::Square8 =>
        Err( CompileError::UnsupportedAnchor
        {
          object : object_id.to_owned(),
          anchor : "Square (tiling strategy not implemented)",
        }),
    }
  }

  // Fix(BUG-156)
  // Root cause: `ObjectLayer::z_in_object` is documented (layer.rs, object.rs,
  // docs/format/001) as the draw order within one object's layer stack, and
  // docs/algorithm/002_scene_rendering_pass.md's own pseudocode specifies it as a
  // pre-sort applied to each object's stack before bucket-wide sort/emission -- but
  // every compile-pass call site iterated `stack` in raw declaration order, so the
  // field was read nowhere in the compile pipeline.
  // Pitfall: a documented-but-unread field compiles cleanly and produces plausible
  // output (declaration order often coincides with intended order), so this class of
  // bug has no compiler signal -- only a direct doc-vs-usage grep catches it.
  /// Returns `stack`'s layers ordered by ascending `z_in_object` (ties keep their
  /// relative declaration order, since `sort_by_key` is stable).
  pub( crate ) fn layers_in_z_order( stack : &[ ObjectLayer ] ) -> Vec< &ObjectLayer >
  {
    let mut ordered : Vec< &ObjectLayer > = stack.iter().collect();
    ordered.sort_by_key( | layer | layer.z_in_object );
    ordered
  }

  /// Emit sprites for every `EdgeInstance` whose owning `Object` routes
  /// Resolve a sprite source for an `Edge`-anchored object. Accepts the
  /// same leaf sources as the hex path (`Static` / `Animation` / `Variant`)
  /// plus the edge-specific `EdgeConnectedBitmask`.
  fn edge_sprite_source_resolve
  (
    source : &SpriteSource,
    object : &Object,
    canon : CanonicalEdge,
    ctx : &FrameContext< '_ >,
  ) -> Result< SpriteRef, CompileError >
  {
    match source
    {
      SpriteSource::EdgeConnectedBitmask { connects_with, source : bmsource, layout : _ } =>
      {
        let mask = edge_connected_bitmask_compute( canon, connects_with, ctx.tiling, &ctx.edge_lookup );
        match bmsource
        {
          NeighborBitmaskSource::ByMapping { mapping, fallback } =>
          {
            match mapping.get( &mask )
            {
              Some( inner ) => edge_sprite_source_resolve( inner, object, canon, ctx ),
              None          => edge_sprite_source_resolve( fallback, object, canon, ctx ),
            }
          },
          NeighborBitmaskSource::ByAtlas { asset, layout : _ } =>
          {
            Ok( SpriteRef { asset : asset.clone(), frame : mask.to_string() } )
          },
        }
      },
      SpriteSource::Static( sprite_ref )   => Ok( sprite_ref.clone() ),
      SpriteSource::Animation( anim_ref )  =>
      {
        let anim = ctx.spec.animations.iter().find( | a | a.id == anim_ref.0 )
          .ok_or_else( || CompileError::UnresolvedRef
          {
            kind : "animation",
            id : anim_ref.0.clone(),
            context : format!( "object {:?} edge layer sprite_source", object.id ),
          })?;
        // Edge sprites are not per-instance — pass `0.0` as the `OneShot`
        // origin (OneShot at the edge level uses absolute master time;
        // typically these are Loop animations anyway, so it doesn't matter).
        // No instance seed available here; `PhaseOffset::Instance`
        // falls back to 0.0 on edge sprites by design.
        animation_frame_resolve( anim, ctx.time_seconds, 0.0, canon.0, None )
      },
      SpriteSource::Variant { variants, selection } =>
      {
        if variants.is_empty()
        {
          return Err( CompileError::OutOfRange
          {
            owner : object.id.clone(),
            index : 0,
            max : 0,
          });
        }
        let chosen = variant_index_pick( variants, *selection, canon.0, object, ctx )?;
        edge_sprite_source_resolve( &variants[ chosen ].sprite, object, canon, ctx )
      },
      other => Err( CompileError::UnsupportedSource
      {
        object : object.id.clone(),
        source_kind : source_name( other ),
      }),
    }
  }

  /// Resolve a sprite source down to a concrete `( asset, frame )` pair.
  ///
  /// Dispatches over all non-vertex sources: `Static`, `Animation`,
  /// `Variant`, `NeighborBitmask`. `NeighborCondition` is handled by
  /// [`emit_neighbor_condition`] directly (emits multiple sprites).
  /// `VertexCorners` is handled by [`resolve_vertex_pass_all`].
  fn sprite_source_resolve
  (
    source : &SpriteSource,
    object : &Object,
    pos : ( i32, i32 ),
    ctx : &FrameContext< '_ >,
  ) -> Result< SpriteRef, CompileError >
  {
    match source
    {
      SpriteSource::Static( sprite_ref ) => Ok( sprite_ref.clone() ),
      SpriteSource::Animation( anim_ref ) =>
      {
        let anim = ctx.spec.animations.iter().find( | a | a.id == anim_ref.0 )
          .ok_or_else( || CompileError::UnresolvedRef
          {
            kind : "animation",
            id : anim_ref.0.clone(),
            context : format!( "object {:?} layer sprite_source", object.id ),
          })?;
        // Non-instance path: `OneShot` uses absolute master time. The
        // per-instance variant ([`sprite_source_with_phase_resolve`])
        // threads `inst.state_entered_time` through for the correct
        // timing. No instance seed here, so `PhaseOffset::Instance`
        // falls back to 0.0.
        animation_frame_resolve( anim, ctx.time_seconds, 0.0, pos, None )
      },
      SpriteSource::Variant { variants, selection } =>
      {
        if variants.is_empty()
        {
          return Err( CompileError::OutOfRange
          {
            owner : object.id.clone(),
            index : 0,
            max : 0,
          });
        }
        let chosen = variant_index_pick( variants, *selection, pos, object, ctx )?;
        sprite_source_resolve( &variants[ chosen ].sprite, object, pos, ctx )
      },
      SpriteSource::NeighborBitmask { connects_with, source : bmsource } =>
      {
        let mask = neighbor_bitmask_compute( pos, connects_with, ctx.tiling, &ctx.tile_lookup );
        match bmsource
        {
          NeighborBitmaskSource::ByMapping { mapping, fallback } =>
          {
            match mapping.get( &mask )
            {
              Some( inner ) => sprite_source_resolve( inner, object, pos, ctx ),
              None          => sprite_source_resolve( fallback, object, pos, ctx ),
            }
          },
          NeighborBitmaskSource::ByAtlas { asset, layout : _ } =>
          {
            Ok( SpriteRef { asset : asset.clone(), frame : mask.to_string() } )
          },
        }
      },
      other => Err( CompileError::UnsupportedSource
      {
        object : object.id.clone(),
        source_kind : source_name( other ),
      }),
    }
  }

  /// Deterministic Variant selection. See SPEC §5.2.
  fn variant_index_pick
  (
    variants : &[ crate::source::Variant ],
    selection : VariantSelection,
    pos : ( i32, i32 ),
    object : &Object,
    ctx : &FrameContext< '_ >,
  ) -> Result< usize, CompileError >
  {
    match selection
    {
      VariantSelection::HashCoord =>
      {
        weighted_pick( variants, object, | | u64::from( coord_hash( pos.0, pos.1, 0 ) ) )
      },
      VariantSelection::Fixed( idx ) =>
      {
        if idx < variants.len() { Ok( idx ) }
        else
        {
          Err( CompileError::OutOfRange
          {
            owner : object.id.clone(),
            index : idx as u32,
            max : variants.len() as u32,
          })
        }
      },
      VariantSelection::Random =>
      {
        // Deterministic pseudo-random — seeded from `Scene.seed`, salted with
        // the grid coord so different cells pick different variants. Same
        // seed + coord + variant list → same pick across frames and runs.
        weighted_pick( variants, object, | | u64::from( coord_hash( pos.0, pos.1, ctx.scene_seed ) ) )
      },
    }
  }

  /// Weighted selection shared between `HashCoord` and `Random` variants.
  fn weighted_pick< F >
  (
    variants : &[ crate::source::Variant ],
    object : &Object,
    hash_fn : F,
  ) -> Result< usize, CompileError >
  where F : FnOnce() -> u64
  {
    let total : u64 = variants.iter().map( | v | u64::from( v.weight ) ).sum();
    if total == 0
    {
      return Err( CompileError::OutOfRange
      {
        owner : object.id.clone(),
        index : 0,
        max : 0,
      });
    }
    let mut target = hash_fn() % total;
    for ( i, v ) in variants.iter().enumerate()
    {
      let w = u64::from( v.weight );
      if target < w { return Ok( i ); }
      target -= w;
    }
    Ok( variants.len() - 1 )
  }

  fn source_name( s : &SpriteSource ) -> &'static str
  {
    match s
    {
      SpriteSource::Static( _ )                   => "Static",
      SpriteSource::Variant { .. }                => "Variant",
      SpriteSource::Animation( _ )                => "Animation",
      SpriteSource::External { .. }               => "External",
      SpriteSource::NeighborBitmask { .. }        => "NeighborBitmask",
      SpriteSource::NeighborCondition { .. }      => "NeighborCondition",
      SpriteSource::VertexCorners { .. }          => "VertexCorners",
      SpriteSource::EdgeConnectedBitmask { .. }   => "EdgeConnectedBitmask",
      SpriteSource::ViewportTiled { .. }          => "ViewportTiled",
    }
  }

  // ════════════════════════════════════════════════════════════════════════
  // Scene-driven rendering — entry points called by
  // [`crate::renderer::Renderer::render`]. `gather_frame_emits` returns
  // structured per-bucket emit data the renderer turns into batched
  // commands; `render_into` is a thin wrapper that flattens emits into
  // a per-sprite command stream for tests / fall-back code paths.
  // ════════════════════════════════════════════════════════════════════════

  /// Render a `Scene` into `out` as a flat command stream.
  ///
  /// # Errors
  ///
  /// Returns [`CompileError`] when the scene uses a feature that this
  /// implementation doesn't support, or when an id reference cannot be
  /// resolved.
  ///
  /// # Panics
  ///
  /// Panics in debug builds when an instance handle stored in one of the
  /// scene's spatial indexes has no live entry — this would only happen if
  /// the indexes were corrupted by mutation outside the documented
  /// `Scene` API.
  /// Output of [`gather_frame_emits`] — per-bucket, structured emit
  /// data the renderer needs to either flatten into per-sprite
  /// `RenderCommand`s or group into batches.
  pub struct FrameEmits
  {
    /// Background clear color, sourced from `pipeline.clear_color`.
    pub clear_color : [ f32; 4 ],
    /// One entry per [`crate::pipeline::PipelineLayer`], in declaration
    /// order. Empty buckets still appear (renderer iterates them all so
    /// idle no-op buckets don't break order).
    pub buckets : Vec< BucketEmits >,
  }

  /// Per-bucket emit data. `sprites` is the sort-mode-applied world
  /// layer; `screen_space` is the viewport pass output.
  pub struct BucketEmits
  {
    /// World-space sprites in this bucket, already sorted per the
    /// bucket's `SortMode`. Order is the on-screen draw order.
    pub sprites : Vec< Sprite >,
    /// Screen-space sprites (viewport-anchored) emitted by this
    /// bucket's `Viewport` instances.
    pub screen_space : Vec< Sprite >,
    /// Bucket's sort mode — needed by the batching renderer to decide
    /// whether instance order within a batch matters.
    pub sort : SortMode,
    /// Coverage cut-off carried from the bucket's `PipelineLayer`; the
    /// renderer copies it into every `SpriteBatchParams` it emits for this
    /// bucket. `0.0` disables the discard.
    pub alpha_clip : f32,
    /// Single-coverage depth flag carried from the bucket's `PipelineLayer`
    /// (see `PipelineLayer::occlude_overlap`).
    pub occlude_overlap : bool,
    /// Opaque-pass flag carried from the bucket's `PipelineLayer` (see
    /// `PipelineLayer::opaque`). Drives the renderer's opaque/transparent
    /// split; `false` for every bucket leaves the original single-pass path.
    pub opaque : bool,
  }

  /// Every visible hex- and free-placed instance layer, routed to the
  /// non-skipped buckets it draws in; see [`instances_route`].
  struct Routed< 'a >
  {
    hex : Vec< Vec< HexWork< 'a > > >,
    free : Vec< Vec< FreeWork< 'a > > >,
    /// Whether a routed layer reads neighbouring cells (the per-cell tile table).
    reads_tiles : bool,
  }

  /// Route every visible hex- and free-placed instance's layers to the buckets
  /// they draw in, in ONE walk of the scene — per instance, its current state's
  /// stack in z order, so each bucket's list keeps the instance-major, z-minor
  /// order a per-bucket walk produced (at `buckets × instances` cost). A layer
  /// goes to every non-skipped bucket carrying its id, as that per-bucket
  /// filter did.
  fn instances_route( scene : &Scene, skip : u64 ) -> Result< Routed< '_ >, CompileError >
  {
    let spec = scene.spec();
    let bucket_count = spec.pipeline.layers.len();
    let mut bucket_of : HashMap< &str, Vec< usize > > = HashMap::default();
    for ( i, bucket ) in spec.pipeline.layers.iter().enumerate()
    {
      if !bucket_skipped( skip, i ) { bucket_of.entry( bucket.id.as_str() ).or_default().push( i ); }
    }
    let mut hex_work : Vec< Vec< HexWork< '_ > > > = ( 0..bucket_count ).map( | _ | Vec::new() ).collect();
    let mut free_work : Vec< Vec< FreeWork< '_ > > > = ( 0..bucket_count ).map( | _ | Vec::new() ).collect();
    // Whether a routed layer reads neighbouring cells (the per-cell tile table).
    let mut layers_read_tiles = false;
    if !bucket_of.is_empty()
    {
      let mut ordered : Vec< &ObjectLayer > = Vec::new();
      for &handle in scene.hex_instances()
      {
        let inst = scene.instance( handle ).expect( "hex handle live" );
        if !inst.visible { continue; }
        let object = &spec.objects[ inst.object.index() as usize ];
        match object.anchor
        {
          Anchor::Hex => {},
          Anchor::Multihex { .. } => return Err( CompileError::UnsupportedAnchor
          {
            object : object.id.clone(),
            anchor : "Multihex",
          }),
          _ => continue,
        }
        let stack = instance_stack( scene, inst, object )?;
        let Placement::Hex { q, r } = inst.placement else { continue };
        layers_z_order_into( stack, &mut ordered );
        for &layer in &ordered
        {
          // `VertexCorners` layers emit nothing per instance — the vertex pass
          // below draws them for the whole board.
          if matches!( layer.sprite_source, SpriteSource::VertexCorners { .. } ) { continue; }
          let effective = layer.pipeline_layer.as_deref().unwrap_or( object.global_layer.as_str() );
          let Some( targets ) = bucket_of.get( effective ) else { continue };
          layers_read_tiles |= source_reads_tiles( &layer.sprite_source );
          for &bi in targets
          {
            hex_work[ bi ].push( HexWork { inst, object, layer, pos : ( q, r ) } );
          }
        }
      }
      for &handle in scene.free_instances()
      {
        let inst = scene.instance( handle ).expect( "free handle live" );
        if !inst.visible { continue; }
        let Placement::FreePos { x, y } = inst.placement else { unreachable!() };
        let object = &spec.objects[ inst.object.index() as usize ];
        if !matches!( object.anchor, Anchor::FreePos )
        {
          return Err( CompileError::UnsupportedAnchor
          {
            object : object.id.clone(),
            anchor : "FreePos (object declares a different anchor)",
          });
        }
        let stack = instance_stack( scene, inst, object )?;
        layers_z_order_into( stack, &mut ordered );
        for &layer in &ordered
        {
          let effective = layer.pipeline_layer.as_deref().unwrap_or( object.global_layer.as_str() );
          let Some( targets ) = bucket_of.get( effective ) else { continue };
          for &bi in targets
          {
            free_work[ bi ].push( FreeWork { inst, object, layer, pos : ( x, y ) } );
          }
        }
      }
    }

    Ok( Routed { hex : hex_work, free : free_work, reads_tiles : layers_read_tiles } )
  }

  /// One hex-placed instance layer routed to a bucket by [`gather_frame_emits`].
  struct HexWork< 'a >
  {
    inst : &'a Instance,
    object : &'a Object,
    layer : &'a ObjectLayer,
    pos : ( i32, i32 ),
  }

  /// One free-placed instance layer routed to a bucket by [`gather_frame_emits`].
  struct FreeWork< 'a >
  {
    inst : &'a Instance,
    object : &'a Object,
    layer : &'a ObjectLayer,
    pos : ( f32, f32 ),
  }

  /// The layer stack of `inst`'s current state.
  fn instance_stack< 'a >( scene : &Scene, inst : &Instance, object : &'a Object ) -> Result< &'a [ ObjectLayer ], CompileError >
  {
    let state_name = scene.state_name( inst.state ).ok_or_else( || CompileError::MissingDefaultState
    {
      object : object.id.clone(),
    })?;
    object.states.get( state_name ).map( Vec::as_slice ).ok_or_else( || CompileError::MissingDefaultState
    {
      object : object.id.clone(),
    })
  }

  /// [`layers_in_z_order`] into a reused buffer (cleared first), so the
  /// per-instance routing walk allocates nothing once the buffer has grown.
  fn layers_z_order_into< 'a >( stack : &'a [ ObjectLayer ], out : &mut Vec< &'a ObjectLayer > )
  {
    out.clear();
    out.extend( stack.iter() );
    out.sort_by_key( | layer | layer.z_in_object );
  }

  /// Whether resolving `source` reads the cells around the instance (the
  /// per-cell tile table): neighbour sources do, directly or as a variant.
  fn source_reads_tiles( source : &SpriteSource ) -> bool
  {
    match source
    {
      SpriteSource::NeighborBitmask { .. } | SpriteSource::NeighborCondition { .. } => true,
      SpriteSource::Variant { variants, .. } => variants.iter().any( | v | source_reads_tiles( &v.sprite ) ),
      SpriteSource::Static( _ )
      | SpriteSource::Animation( _ )
      | SpriteSource::External { .. }
      | SpriteSource::VertexCorners { .. }
      | SpriteSource::EdgeConnectedBitmask { .. }
      | SpriteSource::ViewportTiled { .. } => false,
    }
  }

  /// Walk the scene and produce structured per-bucket emit data without
  /// flattening to `RenderCommand`s. This is the shared core driving
  /// both [`render_into`] (per-sprite emission, used by tests and
  /// fall-back code paths) and the batching renderer.
  ///
  /// # Errors
  ///
  /// Same error surface as the legacy `compile_frame`: unresolved
  /// sprite / animation / tint references, unsupported anchor kinds
  /// (Multihex), unsupported asset kinds, etc.
  ///
  /// # Panics
  ///
  /// Panics in debug builds if `scene` exposes an instance handle for
  /// which the underlying `Instance` is missing — only possible if the
  /// scene's spatial indexes are inconsistent with its slotmap.
  /// `vcache` is an optional revision-keyed memo for the expensive dual-grid
  /// vertex pass (see [`VertexResolveCache`]). When supplied, the structural
  /// resolve is reused across frames as long as the scene `revision` is
  /// unchanged — so an animating-but-idle board (clock ticking, nothing
  /// spawned/despawned) skips the whole triangle / pattern / string walk and
  /// only re-projects. Pass `None` for a one-shot uncached compile.
  ///
  /// `skip` masks out pipeline layers the caller will not draw (bit `i` = layer
  /// `i`, the renderer's `disabled_buckets`): their bucket comes back empty and
  /// neither the vertex resolve nor the per-bucket walk runs for them.
  pub fn gather_frame_emits
  (
    compiled : &CompiledAssets,
    scene : &Scene,
    camera : &Camera,
    vcache : Option< &mut VertexResolveCache >,
    skip : u64,
  ) -> Result< FrameEmits, CompileError >
  {
    let spec = scene.spec();
    let clear_color = spec.pipeline.clear_color.unwrap_or( [ 0.0, 0.0, 0.0, 0.0 ] );

    // Reject Multihex anchors (polish item #9 — not implemented).
    if let Some( &h ) = scene.multihex_instances().first()
    {
      let inst = scene.instance( h ).expect( "multihex handle live" );
      return Err( CompileError::UnsupportedAnchor
      {
        object : spec.objects[ inst.object.index() as usize ].id.clone(),
        anchor : "Multihex",
      });
    }

    let bucket_count = spec.pipeline.layers.len();
    let Routed { hex : hex_work, free : free_work, reads_tiles : layers_read_tiles } = instances_route( scene, skip )?;

    // The vertex resolve is reused while the scene's hex cells are unchanged.
    let revision = scene.tiles_revision();
    let vertex_miss = vcache.as_ref().is_none_or( | c | !c.valid || c.revision != revision || c.skip != skip );

    // The per-cell tile table — one `String` per hex instance — is read only by
    // the vertex resolve and by neighbour-reading sources, so build it only when
    // one of those will run this frame.
    let synthetic_tiles = if vertex_miss || layers_read_tiles { scene_tiles_build( scene ) } else { Vec::new() };
    let synthetic_edges = scene_edges_build( scene, spec );

    let viewport_size = spec.pipeline.viewport_size.unwrap_or( camera.viewport_size );
    let edge_lookup = build_edge_lookup( &synthetic_edges, spec.pipeline.hex.tiling );
    let seed = scene.seed();
    let scene_seed = ( seed as u32 ) ^ ( ( seed >> 32 ) as u32 );
    let ctx = FrameContext
    {
      spec,
      compiled,
      camera,
      scene,
      time_seconds : scene.clock(),
      tile_lookup : build_tile_lookup( &synthetic_tiles ),
      edge_lookup,
      tiling : spec.pipeline.hex.tiling,
      grid_stride : spec.pipeline.hex.grid_stride,
      viewport_size,
      scene_seed,
      tints : FrameTints::new( spec, scene )?,
    };

    // Vertex pass — resolve the camera/clock-independent structural half once,
    // reusing the cached result while the scene's hex cells are unchanged. On a
    // miss (or with no cache) it is recomputed; either way the cheap per-frame
    // projection happens below, per bucket.
    let local_resolved;
    let resolved : &[ Vec< ResolvedVertexSprite > ] = if let Some( cache ) = vcache
    {
      if vertex_miss
      {
        cache.store( resolve_vertex_pass_all( &synthetic_tiles, &ctx, skip )?, revision, skip );
      }
      &cache.buckets
    }
    else
    {
      local_resolved = resolve_vertex_pass_all( &synthetic_tiles, &ctx, skip )?;
      &local_resolved
    };
    let mut buckets = Vec::with_capacity( bucket_count );

    for ( bucket_idx, bucket ) in spec.pipeline.layers.iter().enumerate()
    {
      if bucket_skipped( skip, bucket_idx )
      {
        buckets.push( BucketEmits
        {
          sprites : Vec::new(),
          screen_space : Vec::new(),
          sort : bucket.sort,
          alpha_clip : bucket.alpha_clip,
          occlude_overlap : bucket.occlude_overlap,
          opaque : bucket.opaque,
        });
        continue;
      }
      let mut draws : Vec< ( f32, f32, Sprite ) > = Vec::new();

      for w in &hex_work[ bucket_idx ]
      {
        draws.extend( instance_layer_compile( w.object, w.layer, w.pos, w.inst, &ctx )? );
      }

      // Vertex pass: project this bucket's cached (revision-stable) resolves —
      // cheap per-frame work; the expensive resolve already ran (or was reused)
      // above. Same triangle-outer / layer-inner order as before, so sort and
      // batch grouping are unchanged.
      draws.extend( resolved[ bucket_idx ].iter().map( | rv | project_vertex_sprite( rv, &ctx ) ) );
      draws.extend( edge_pass_scene_compile( bucket.id.as_str(), scene, &ctx )? );
      free_pass_scene_compile( &free_work[ bucket_idx ], &ctx, &mut draws )?;

      sort_mode_apply( &mut draws, bucket.sort );

      let sprites : Vec< Sprite > = draws.into_iter().map( | ( _, _, s ) | s ).collect();

      // Viewport pass currently writes `ScreenSpaceSprite` commands to a
      // `Vec<RenderCommand>`; unwrap them back into raw `Sprite`s so the
      // renderer can decide whether to wrap in `ScreenSpaceSprite` or
      // (eventually) batch viewport sprites too.
      let mut tmp : Vec< RenderCommand > = Vec::new();
      viewport_pass_scene_compile( bucket.id.as_str(), scene, &ctx, &mut tmp )?;
      let screen_space : Vec< Sprite > = tmp.into_iter().filter_map( | c | match c
      {
        RenderCommand::ScreenSpaceSprite( s ) => Some( s ),
        _ => None,
      }).collect();

      buckets.push( BucketEmits
      {
        sprites,
        screen_space,
        sort : bucket.sort,
        alpha_clip : bucket.alpha_clip,
        occlude_overlap : bucket.occlude_overlap,
        opaque : bucket.opaque,
      });
    }

    Ok( FrameEmits { clear_color, buckets } )
  }

  /// Flatten [`FrameEmits`] into a per-sprite `RenderCommand` stream —
  /// the pre-Step-4b output shape. Kept as a thin wrapper for the
  /// renderer's fall-back path and for `commands_to_sprites` test
  /// helpers that compare against the historical baseline.
  ///
  /// # Errors
  ///
  /// Propagates errors from [`gather_frame_emits`].
  pub fn render_into
  (
    out : &mut Vec< RenderCommand >,
    compiled : &CompiledAssets,
    scene : &Scene,
    camera : &Camera,
  ) -> Result< (), CompileError >
  {
    let emits = gather_frame_emits( compiled, scene, camera, None, 0 )?;
    out.push( RenderCommand::Clear( Clear { color : emits.clear_color } ) );
    for bucket in emits.buckets
    {
      for s in bucket.sprites { out.push( RenderCommand::Sprite( s ) ); }
      for s in bucket.screen_space { out.push( RenderCommand::ScreenSpaceSprite( s ) ); }
    }
    Ok( () )
  }

  /// Whether pipeline layer `index` is masked out by `skip` (bit `index`; layers
  /// past bit 63 are never skipped, matching `Renderer::set_disabled_buckets`).
  pub( crate ) fn bucket_skipped( skip : u64, index : usize ) -> bool
  {
    index < 64 && skip & ( 1_u64 << index ) != 0
  }

  /// Build a synthetic `Vec<Tile>` from the scene's hex spatial index.
  ///
  /// Only hex-placed instances contribute (edge / multihex / free / viewport
  /// stay in their own passes). Object ids are looked up via the spec; the
  /// returned tiles are owned and used as the source for `tile_lookup`
  /// in [`render_into`].
  fn scene_tiles_build( scene : &Scene ) -> Vec< Tile >
  {
    let spec = scene.spec();
    let mut by_cell : HashMap< ( i32, i32 ), Vec< String > > = HashMap::default();
    for &handle in scene.hex_instances()
    {
      let inst = scene.instance( handle ).expect( "hex handle live" );
      if !inst.visible { continue; }
      let Placement::Hex { q, r } = inst.placement else { continue };
      let id = spec.objects[ inst.object.index() as usize ].id.clone();
      by_cell.entry( ( q, r ) ).or_default().push( id );
    }
    by_cell
      .into_iter()
      .map( | ( pos, objects ) | Tile { pos, objects } )
      .collect()
  }

  /// Build a synthetic `Vec<EdgeInstance>` from the scene's edge handles.
  fn scene_edges_build( scene : &Scene, spec : &RenderSpec ) -> Vec< EdgeInstance >
  {
    scene.edge_instances().iter().map( | &h |
    {
      let inst = scene.instance( h ).expect( "edge handle live" );
      let Placement::Edge { hex, dir } = inst.placement else { unreachable!() };
      EdgeInstance
      {
        at : EdgePosition { hex, dir },
        object : spec.objects[ inst.object.index() as usize ].id.clone(),
        animation : None,
      }
    }).collect()
  }

  /// Apply a bucket's sort mode to the draw list.
  fn sort_mode_apply( draws : &mut [ ( f32, f32, Sprite ) ], sort : SortMode )
  {
    use core::cmp::Ordering;
    let cmp_f = | a : f32, b : f32 | a.partial_cmp( &b ).unwrap_or( Ordering::Equal );
    match sort
    {
      SortMode::None => {}
      SortMode::XAsc      => draws.sort_by( | a, b | cmp_f( a.0, b.0 ) ),
      SortMode::XDesc     => draws.sort_by( | a, b | cmp_f( b.0, a.0 ) ),
      SortMode::YAsc      => draws.sort_by( | a, b | cmp_f( a.1, b.1 ) ),
      SortMode::YDesc     => draws.sort_by( | a, b | cmp_f( b.1, a.1 ) ),
      SortMode::XAscYDesc => draws.sort_by( | a, b | cmp_f( a.0, b.0 ).then_with( || cmp_f( b.1, a.1 ) ) ),
      SortMode::XAscYAsc  => draws.sort_by( | a, b | cmp_f( a.0, b.0 ).then_with( || cmp_f( a.1, b.1 ) ) ),
      SortMode::YDescXAsc => draws.sort_by( | a, b | cmp_f( b.1, a.1 ).then_with( || cmp_f( a.0, b.0 ) ) ),
      SortMode::YAscXAsc  => draws.sort_by( | a, b | cmp_f( a.1, b.1 ).then_with( || cmp_f( a.0, b.0 ) ) ),
    }
  }

  /// Compile one layer of a hex-anchored instance, threading the
  /// instance's per-instance overrides through the emit. Drop-in
  /// replacement for `compile_layer` where the position comes from
  /// the instance's `Placement` rather than a `Tile`.
  fn instance_layer_compile
  (
    object : &Object,
    layer : &ObjectLayer,
    pos : ( i32, i32 ),
    inst : &Instance,
    ctx : &FrameContext< '_ >,
  ) -> Result< Vec< ( f32, f32, Sprite ) >, CompileError >
  {
    // `NeighborCondition` emits multiple sprites per side.
    if let SpriteSource::NeighborCondition { condition, sides, sprite_pattern, asset } = &layer.sprite_source
    {
      // Synthesize a temporary `Tile` carrying the owning object's id so
      // the existing helper can compute current-cell priority correctly.
      let tile = Tile { pos, objects : vec![ object.id.clone() ] };
      return neighbor_condition_with_overrides_emit( object, &tile, condition, sides, sprite_pattern, asset, &layer.behaviour, inst, ctx );
    }

    // `VertexCorners` doesn't emit per-instance.
    if matches!( &layer.sprite_source, SpriteSource::VertexCorners { .. } )
    {
      return Ok( Vec::new() );
    }

    // `External` source — look up the per-instance slot map.
    if let SpriteSource::External { slot } = &layer.sprite_source
    {
      let Some( sprite_ref ) = inst.external_sprites.get( slot ) else { return Ok( Vec::new() ); };
      let sprite_id = ctx.compiled.ids.sprite( &sprite_ref.asset, &sprite_ref.frame )
        .ok_or_else( || CompileError::UnresolvedRef
        {
          kind : "sprite",
          id : format!( "{}:{}", sprite_ref.asset, sprite_ref.frame ),
          context : format!( "object {:?} external slot {slot:?}", object.id ),
        })?;
      let ( q, r ) = pos;
      let ( wx, wy ) = hex_world_pixel( q, r, ctx, &object.id )?;
      let ( sx, sy ) = pivot_apply( wx, wy, object.pivot, sprite_id, ctx.compiled );
      let transform = make_transform( sx, sy );
      return Ok( vec!
      [
        (
          wx, wy,
          Sprite
          {
            transform,
            sprite : sprite_id,
            tint : ctx.tints.sprite_tint( object, &layer.behaviour, inst.tint )?,
            blend : layer.behaviour.blend,
            clip : None,
          },
        ),
      ]);
    }

    let sprite_ref = sprite_source_with_phase_resolve
    (
      &layer.sprite_source, object, pos, inst.phase_offset, inst.state_entered_time,
      Some( inst.instance_phase_seed ), ctx,
    )?;
    let sprite_id = ctx.compiled.ids.sprite( &sprite_ref.asset, &sprite_ref.frame )
      .ok_or_else( || CompileError::UnresolvedRef
      {
        kind : "sprite",
        id : format!( "{}:{}", sprite_ref.asset, sprite_ref.frame ),
        context : format!( "object {:?} layer", object.id ),
      })?;

    let ( q, r ) = pos;
    let ( wx, wy ) = hex_world_pixel( q, r, ctx, &object.id )?;
    let ( sx, sy ) = pivot_apply( wx, wy, object.pivot, sprite_id, ctx.compiled );
    let transform = make_transform( sx, sy );

    Ok( vec!
    [
      (
        wx, wy,
        Sprite
        {
          transform,
          sprite : sprite_id,
          tint : ctx.tints.sprite_tint( object, &layer.behaviour, inst.tint )?,
          blend : layer.behaviour.blend,
          clip : None,
        },
      ),
    ])
  }

  /// Variant of `sprite_source_resolve` that threads per-instance overrides
  /// (`phase_override`, `oneshot_origin`) into animation resolution.
  ///
  /// `oneshot_origin` is forwarded into [`animation_frame_resolve`] — it
  /// only affects `AnimationMode::OneShot` (whose local time is the elapsed
  /// since the instance entered the state), while `Loop` / `PingPong` keep
  /// riding the master clock for cross-instance harmonic phase.
  fn sprite_source_with_phase_resolve
  (
    source         : &SpriteSource,
    object         : &Object,
    pos            : ( i32, i32 ),
    phase_override : Option< f32 >,
    oneshot_origin : f32,
    instance_seed  : Option< u32 >,
    ctx            : &FrameContext< '_ >,
  ) -> Result< SpriteRef, CompileError >
  {
    match source
    {
      SpriteSource::Animation( anim_ref ) =>
      {
        let anim = ctx.spec.animations.iter().find( | a | a.id == anim_ref.0 )
          .ok_or_else( || CompileError::UnresolvedRef
          {
            kind : "animation",
            id : anim_ref.0.clone(),
            context : format!( "object {:?} layer sprite_source", object.id ),
          })?;
        match phase_override
        {
          Some( phase ) =>
          {
            // Apply override by shifting the master time; bypasses the
            // animation's declared `PhaseOffset` entirely (so the
            // `instance_seed` is ignored here too, as intended for
            // explicit per-instance phase overrides).
            let mut anim_clone = anim.clone();
            anim_clone.phase_offset = crate::resource::PhaseOffset::Fixed( phase );
            animation_frame_resolve( &anim_clone, ctx.time_seconds, oneshot_origin, pos, instance_seed )
          },
          None => animation_frame_resolve( anim, ctx.time_seconds, oneshot_origin, pos, instance_seed ),
        }
      },
      SpriteSource::Variant { variants, selection } =>
      {
        if variants.is_empty()
        {
          return Err( CompileError::OutOfRange { owner : object.id.clone(), index : 0, max : 0 } );
        }
        let chosen = variant_index_pick( variants, *selection, pos, object, ctx )?;
        sprite_source_with_phase_resolve
        (
          &variants[ chosen ].sprite, object, pos, phase_override, oneshot_origin, instance_seed, ctx,
        )
      },
      _ => sprite_source_resolve( source, object, pos, ctx ),
    }
  }

  /// `emit_neighbor_condition` variant that composes the per-instance
  /// tint into each emitted sprite.
  #[ allow( clippy::too_many_arguments, reason = "each parameter is a distinct input needed to resolve and place a neighbor-conditioned sprite variant with per-instance tint; grouping into a struct would add indirection without reducing call-site complexity" ) ]
  fn neighbor_condition_with_overrides_emit
  (
    object : &Object,
    tile : &Tile,
    condition : &crate::source::Condition,
    sides : &[ crate::anchor::EdgeDirection ],
    sprite_pattern : &str,
    asset : &str,
    behaviour : &LayerBehaviour,
    inst : &Instance,
    ctx : &FrameContext< '_ >,
  ) -> Result< Vec< ( f32, f32, Sprite ) >, CompileError >
  {
    let current_priority = tile_max_priority( tile, ctx.spec );
    let ( q, r ) = tile.pos;
    let ( wx, wy ) = hex_world_pixel( q, r, ctx, &object.id )?;

    let mut out = Vec::new();
    for &side in sides
    {
      let Some( offset ) = neighbor_offset_by_dir( ctx.tiling, side ) else { continue; };
      let neighbour_pos = ( tile.pos.0 + offset.0, tile.pos.1 + offset.1 );
      let neighbour = neighbor_state_at( neighbour_pos, &ctx.tile_lookup, ctx.spec );
      if !condition_evaluate( condition, &neighbour, current_priority ) { continue; }

      let frame_name = sprite_pattern.replace( "{dir}", dir_name( side ) );
      let sprite_id = ctx.compiled.ids.sprite( asset, &frame_name )
        .ok_or_else( || CompileError::UnresolvedRef
        {
          kind : "sprite",
          id : format!( "{asset}:{frame_name}" ),
          context : format!( "object {:?} NeighborCondition side {side:?}", object.id ),
        })?;

      let ( sx, sy ) = pivot_apply( wx, wy, object.pivot, sprite_id, ctx.compiled );
      let transform = make_transform( sx, sy );

      out.push
      ((
        wx, wy,
        Sprite
        {
          transform,
          sprite : sprite_id,
          tint : ctx.tints.sprite_tint( object, behaviour, inst.tint )?,
          blend : behaviour.blend,
          clip : None,
        },
      ));
    }
    Ok( out )
  }

  /// Emit sprites for every Scene edge handle whose owning Object routes
  /// into `bucket_id`. Mirrors `compile_edge_pass` but iterates Scene's
  /// handle list and applies per-instance overrides.
  fn edge_pass_scene_compile
  (
    bucket_id : &str,
    scene : &Scene,
    ctx : &FrameContext< '_ >,
  ) -> Result< Vec< ( f32, f32, Sprite ) >, CompileError >
  {
    let mut out = Vec::new();
    let mut seen : rustc_hash::FxHashSet< CanonicalEdge > = rustc_hash::FxHashSet::default();

    for &handle in scene.edge_instances()
    {
      let inst = scene.instance( handle ).expect( "edge handle live" );
      if !inst.visible { continue; }
      let Placement::Edge { hex, dir } = inst.placement else { unreachable!() };

      let Some( canon ) = canonical_edge( EdgePosition { hex, dir }, ctx.tiling ) else { continue };
      if !seen.insert( canon ) { continue; }

      let object = &ctx.spec.objects[ inst.object.index() as usize ];
      if !matches!( object.anchor, Anchor::Edge )
      {
        return Err( CompileError::UnsupportedAnchor
        {
          object : object.id.clone(),
          anchor : "Edge (object declares a different anchor)",
        });
      }

      let state_name = scene.state_name( inst.state ).ok_or_else( || CompileError::MissingDefaultState
      {
        object : object.id.clone(),
      })?;
      let stack = object.states.get( state_name ).ok_or_else( || CompileError::MissingDefaultState
      {
        object : object.id.clone(),
      })?;

      // Fix(BUG-156): see `layers_in_z_order` doc comment for root cause.
      for layer in layers_in_z_order( stack )
      {
        let effective = layer.pipeline_layer.as_deref().unwrap_or( object.global_layer.as_str() );
        if effective != bucket_id { continue; }

        let sprite_ref = edge_sprite_source_resolve( &layer.sprite_source, object, canon, ctx )?;
        let sprite_id = ctx.compiled.ids.sprite( &sprite_ref.asset, &sprite_ref.frame )
          .ok_or_else( || CompileError::UnresolvedRef
          {
            kind : "sprite",
            id : format!( "{}:{}", sprite_ref.asset, sprite_ref.frame ),
            context : format!( "object {:?} edge layer", object.id ),
          })?;

        let Some( ( wx, wy ) ) = edge_world_pixel( canon, ctx.tiling, ctx.grid_stride )
        else
        {
          return Err( CompileError::UnsupportedAnchor
          {
            object : object.id.clone(),
            anchor : "Edge (direction not valid for tiling)",
          });
        };
        let ( sx, sy ) = pivot_apply( wx, wy, object.pivot, sprite_id, ctx.compiled );

        // World-space + unit scale; the edge's own rotation stays (the camera
        // has none, so it composes cleanly). Pan/zoom comes from the view matrix.
        let transform = Transform
        {
          position : [ sx, sy ],
          rotation : edge_rotation( canon.1, ctx.tiling ),
          scale : [ 1.0, 1.0 ],
          skew : [ 0.0, 0.0 ],
          depth : 0.0,
        };

        out.push
        ((
          wx, wy,
          Sprite
          {
            transform,
            sprite : sprite_id,
            tint : ctx.tints.sprite_tint( object, &layer.behaviour, inst.tint )?,
            blend : layer.behaviour.blend,
            clip : None,
          },
        ));
      }
    }
    Ok( out )
  }

  /// Emit sprites for every Scene free-pos handle. Mirrors `compile_free_pass`.
  fn free_pass_scene_compile
  (
    work : &[ FreeWork< '_ > ],
    ctx : &FrameContext< '_ >,
    out : &mut Vec< ( f32, f32, Sprite ) >,
  ) -> Result< (), CompileError >
  {
    for &FreeWork { inst, object, layer, pos : ( x, y ) } in work
    {
      {
        match &layer.sprite_source
        {
          SpriteSource::NeighborBitmask { .. }
          | SpriteSource::NeighborCondition { .. }
          | SpriteSource::VertexCorners { .. }
          | SpriteSource::EdgeConnectedBitmask { .. }
          | SpriteSource::ViewportTiled { .. } =>
          {
            return Err( CompileError::UnsupportedSource
            {
              object : object.id.clone(),
              source_kind : source_name( &layer.sprite_source ),
            });
          },
          _ => {}
        }
        // External slot resolution for free-pos.
        if let SpriteSource::External { slot } = &layer.sprite_source
        {
          let Some( sprite_ref ) = inst.external_sprites.get( slot ) else { continue };
          let sprite_id = ctx.compiled.ids.sprite( &sprite_ref.asset, &sprite_ref.frame )
            .ok_or_else( || CompileError::UnresolvedRef
            {
              kind : "sprite",
              id : format!( "{}:{}", sprite_ref.asset, sprite_ref.frame ),
              context : format!( "object {:?} free-pos external slot {slot:?}", object.id ),
            })?;
          let ( wx, wy ) = ( x, y );
          let ( sx, sy ) = pivot_apply( wx, wy, object.pivot, sprite_id, ctx.compiled );
          let transform = make_transform( sx, sy );
          out.push((
            wx, wy,
            Sprite
            {
              transform,
              sprite : sprite_id,
              tint : ctx.tints.sprite_tint( object, &layer.behaviour, inst.tint )?,
              blend : layer.behaviour.blend,
              clip : None,
            },
          ));
          continue;
        }

        let sprite_ref = sprite_source_with_phase_resolve
        (
          &layer.sprite_source, object, ( 0, 0 ), inst.phase_offset, inst.state_entered_time,
          Some( inst.instance_phase_seed ), ctx,
        )?;
        let sprite_id = ctx.compiled.ids.sprite( &sprite_ref.asset, &sprite_ref.frame )
          .ok_or_else( || CompileError::UnresolvedRef
          {
            kind : "sprite",
            id : format!( "{}:{}", sprite_ref.asset, sprite_ref.frame ),
            context : format!( "object {:?} free layer", object.id ),
          })?;

        let ( wx, wy ) = ( x, y );
        let ( sx, sy ) = pivot_apply( wx, wy, object.pivot, sprite_id, ctx.compiled );
        let transform = make_transform( sx, sy );

        out.push
        ((
          wx, wy,
          Sprite
          {
            transform,
            sprite : sprite_id,
            tint : ctx.tints.sprite_tint( object, &layer.behaviour, inst.tint )?,
            blend : layer.behaviour.blend,
            clip : None,
          },
        ));
      }
    }
    Ok( () )
  }

  /// Emit `ScreenSpaceSprite` commands for every Scene viewport handle.
  fn viewport_pass_scene_compile
  (
    bucket_id : &str,
    scene : &Scene,
    ctx : &FrameContext< '_ >,
    commands : &mut Vec< RenderCommand >,
  ) -> Result< (), CompileError >
  {
    for &handle in scene.viewport_instances()
    {
      let inst = scene.instance( handle ).expect( "viewport handle live" );
      if !inst.visible { continue; }

      let object = &ctx.spec.objects[ inst.object.index() as usize ];
      if !matches!( object.anchor, Anchor::Viewport )
      {
        return Err( CompileError::UnsupportedAnchor
        {
          object : object.id.clone(),
          anchor : "Viewport (object declares a different anchor)",
        });
      }

      let state_name = scene.state_name( inst.state ).ok_or_else( || CompileError::MissingDefaultState
      {
        object : object.id.clone(),
      })?;
      let stack = object.states.get( state_name ).ok_or_else( || CompileError::MissingDefaultState
      {
        object : object.id.clone(),
      })?;

      // Fix(BUG-156): see `layers_in_z_order` doc comment for root cause.
      for layer in layers_in_z_order( stack )
      {
        let effective = layer.pipeline_layer.as_deref().unwrap_or( object.global_layer.as_str() );
        if effective != bucket_id { continue; }

        let SpriteSource::ViewportTiled { content, tiling : vtiling, anchor_point } = &layer.sprite_source
        else
        {
          return Err( CompileError::UnsupportedSource
          {
            object : object.id.clone(),
            source_kind : "Viewport-anchored layer must use ViewportTiled",
          });
        };

        let sprite_ref = sprite_source_resolve( content, object, ( 0, 0 ), ctx )?;
        let sprite_id = ctx.compiled.ids.sprite( &sprite_ref.asset, &sprite_ref.frame )
          .ok_or_else( || CompileError::UnresolvedRef
          {
            kind : "sprite",
            id : format!( "{}:{}", sprite_ref.asset, sprite_ref.frame ),
            context : format!( "object {:?} viewport layer", object.id ),
          })?;

        let region = ctx.compiled.assets.sprites.iter()
          .find( | s | s.id == sprite_id )
          .map_or( ( 1.0, 1.0 ), | s | ( s.region[ 2 ], s.region[ 3 ] ) );

        let is_repeat = matches!
        (
          vtiling,
          ViewportTiling::Repeat2D | ViewportTiling::RepeatX | ViewportTiling::RepeatY
        );

        if is_repeat
        {
          let zoom = ctx.camera.zoom;
          let scaled_region = ( region.0 * zoom, region.1 * zoom );
          for ( x, y ) in tiled_positions( *vtiling, *anchor_point, scaled_region, ctx.viewport_size )
          {
            let transform = Transform
            {
              position : [ x, y ],
              rotation : 0.0,
              scale : [ zoom, zoom ],
              skew : [ 0.0, 0.0 ],
              depth : 0.0,
            };
            commands.push( RenderCommand::ScreenSpaceSprite( Sprite
            {
              transform,
              sprite : sprite_id,
              tint : ctx.tints.sprite_tint( object, &layer.behaviour, inst.tint )?,
              blend : layer.behaviour.blend,
              clip : None,
            }));
          }
        }
        else
        {
          let Some( transform ) = viewport_transform( *vtiling, *anchor_point, region, ctx.viewport_size )
          else
          {
            return Err( CompileError::UnsupportedSource
            {
              object : object.id.clone(),
              source_kind : "ViewportTiled (unsupported tiling)",
            });
          };
          commands.push( RenderCommand::ScreenSpaceSprite( Sprite
          {
            transform,
            sprite : sprite_id,
            tint : ctx.tints.sprite_tint( object, &layer.behaviour, inst.tint )?,
            blend : layer.behaviour.blend,
            clip : None,
          }));
        }
      }
    }
    Ok( () )
  }
}

mod vertex_corners;

mod_interface::mod_interface!
{
  own use render_into;
  own use gather_frame_emits;
  own use FrameEmits;
  own use BucketEmits;
  own use VertexResolveCache;
}
