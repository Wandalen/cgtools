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
  use crate::compile::animation::resolve_animation_frame;
  use crate::compile::assets::CompiledAssets;
  use crate::compile::camera::Camera;
  use crate::compile::conditions::evaluate_condition;
  use crate::compile::coords::{ hex_to_world_pixel_flat, hex_to_world_pixel_pointy };
  use crate::compile::edges::
  {
    CanonicalEdge,
    canonical_edge,
    compute_edge_connected_bitmask,
    edge_lookup as build_edge_lookup,
    edge_rotation,
    edge_world_pixel,
  };
  use crate::compile::error::CompileError;
  use crate::compile::neighbors::
  {
    compute_neighbor_bitmask,
    dir_name,
    neighbor_offset_by_dir,
    neighbor_state_at,
    tile_lookup as build_tile_lookup,
    tile_max_priority,
  };
  use crate::compile::vertex::
  {
    canonicalize,
    enumerate_triangles,
    find_matching_pattern,
    resolve_corners,
  };
  use crate::hash::hash_coord;
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

  /// Bundled per-frame context threaded into helper functions.
  ///
  /// Consolidates 10+ parameters into a single `&ctx`. Fields are read-only —
  /// no mutation happens once the context is built in [`render_into`].
  struct FrameContext< 'a >
  {
    spec : &'a RenderSpec,
    compiled : &'a CompiledAssets,
    camera : &'a Camera,
    time_seconds : f32,
    tile_lookup : HashMap< ( i32, i32 ), &'a Tile >,
    edge_lookup : HashMap< CanonicalEdge, &'a EdgeInstance >,
    tiling : TilingStrategy,
    grid_stride : ( u32, u32 ),
    viewport_size : ( u32, u32 ),
    /// Scene-level seed folded to `u32` for `hash_coord`. Consumed by
    /// `VariantSelection::Random`.
    scene_seed : u32,
    /// Resolved global tint multiplier — `[1,1,1,1]` when `pipeline.global_tint`
    /// is `None`. Multiplied into every emitted `Sprite.tint`.
    global_tint : [ f32; 4 ],
  }

  fn make_transform( sx : f32, sy : f32, zoom : f32 ) -> Transform
  {
    Transform
    {
      position : [ sx, sy ],
      rotation : 0.0,
      scale : [ zoom, zoom ],
      skew : [ 0.0, 0.0 ],
      depth : 0.0,
    }
  }

  /// Multiply the alpha channel of a tint by a per-layer alpha factor.
  #[ inline ]
  fn tinted( [ r, g, b, a ] : [ f32; 4 ], alpha : f32 ) -> [ f32; 4 ]
  {
    [ r, g, b, a * alpha ]
  }

  /// Shift the projected scene-anchor point so the sprite's anchor pixel
  /// lands exactly on the original scene position.
  ///
  /// Backends render sprites with a bottom-left anchor (quad extends
  /// up-right from `transform.position`). To place some arbitrary anchor
  /// point of the sprite onto the scene position, we shift
  /// `transform.position` in screen space.
  ///
  /// Priority for picking the anchor point:
  ///
  /// 1. **Per-frame pixel anchor** from `FrameSpec::anchor` — pixel offset
  ///    from the sprite's rect top-left. Used when the atlas author knows
  ///    the semantic contact point (feet, ground touch, vertex attachment).
  /// 2. **Object-level normalized `pivot`** — fraction of sprite size.
  ///    Used as a fallback when no per-frame anchor is set.
  fn apply_pivot
  (
    sx : f32,
    sy : f32,
    zoom : f32,
    pivot : ( f32, f32 ),
    sprite_id : tilemap_renderer::types::ResourceId< tilemap_renderer::types::asset::Sprite >,
    compiled : &CompiledAssets,
  ) -> ( f32, f32 )
  {
    let Some( s ) = compiled.assets.sprites.iter().find( | s | s.id == sprite_id )
    else { return ( sx, sy ); };

    let w = s.region[ 2 ];
    let h = s.region[ 3 ];

    // Pixel anchor dominates when present. `ay` is measured from the rect
    // top-left in image-y-down convention; the sprite renders with Y-up in
    // world, so the offset from sprite bottom in world is `h - ay`. That
    // flipped value is what we subtract to align the anchor pixel with
    // (sx, sy).
    if let Some( [ ax, ay ] ) = compiled.sprite_anchors.get( &sprite_id ).copied()
    {
      return ( sx - ax * zoom, sy - ( h - ay ) * zoom );
    }

    // Normalized pivot fallback.
    ( sx - pivot.0 * w * zoom, sy - pivot.1 * h * zoom )
  }

  fn hex_world_pixel
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

  /// Emit dual-mesh triangle sprites for every `VertexCorners` layer that
  /// routes into `bucket_id`. One sprite per triangle whose canonical
  /// corner tuple matches at least one pattern.
  fn compile_vertex_pass
  (
    bucket_id : &str,
    tiles : &[ Tile ],
    ctx : &FrameContext< '_ >,
  ) -> Result< Vec< ( f32, f32, Sprite ) >, CompileError >
  {
    // Gather every VertexCorners layer that belongs in this bucket.
    let mut layers : Vec< ( &Object, &ObjectLayer ) > = Vec::new();
    for object in &ctx.spec.objects
    {
      let Some( stack ) = object.states.get( &object.default_state )
      else { continue };
      for layer in stack
      {
        if !matches!( layer.sprite_source, SpriteSource::VertexCorners { .. } )
        {
          continue;
        }
        let effective = layer.pipeline_layer.as_deref().unwrap_or( object.global_layer.as_str() );
        if effective == bucket_id
        {
          layers.push( ( object, layer ) );
        }
      }
    }

    if layers.is_empty()
    {
      return Ok( Vec::new() );
    }

    let triangles = enumerate_triangles( tiles, ctx.tiling );
    let mut out : Vec< ( f32, f32, Sprite ) > = Vec::new();

    for tri in &triangles
    {
      let raw_corners = resolve_corners( tri, &ctx.tile_lookup, ctx.spec );
      let ( canonical, rotation ) = canonicalize( raw_corners );

      for ( object, layer ) in &layers
      {
        let SpriteSource::VertexCorners { patterns, asset } = &layer.sprite_source
        else { continue };

        let Some( pattern ) = find_matching_pattern( patterns, &canonical )
        else { continue };

        let frame_name = pattern.sprite_pattern.replace( "{rot}", &rotation.to_string() );
        let sprite_id = ctx.compiled.ids.sprite( asset, &frame_name )
          .ok_or_else( || CompileError::UnresolvedRef
          {
            kind : "sprite",
            id : format!( "{asset}:{frame_name}" ),
            context : format!( "object {:?} VertexCorners rotation {rotation}", object.id ),
          })?;

        // Triangle pixel centre: average the three corner hex pixel centres.
        let mut sum_x = 0.0_f32;
        let mut sum_y = 0.0_f32;
        for corner in &tri.corners
        {
          let ( cx, cy ) = hex_world_pixel( corner.0, corner.1, ctx, &object.id )?;
          sum_x += cx;
          sum_y += cy;
        }
        let wx = sum_x / 3.0;
        let wy = sum_y / 3.0;
        let ( sx, sy ) = ctx.camera.project( ( wx, wy ) );
        let ( sx, sy ) = apply_pivot( sx, sy, ctx.camera.zoom, object.pivot, sprite_id, ctx.compiled );
        let transform = make_transform( sx, sy, ctx.camera.zoom );

        out.push
        ((
          wx, wy,
          Sprite
          {
            transform,
            sprite : sprite_id,
            tint : tinted( ctx.global_tint, layer.behaviour.alpha ),
            blend : layer.behaviour.blend,
            clip : None,
          },
        ));
      }
    }

    Ok( out )
  }

  /// Emit sprites for every `EdgeInstance` whose owning `Object` routes
  /// Resolve a sprite source for an `Edge`-anchored object. Accepts the
  /// same leaf sources as the hex path (`Static` / `Animation` / `Variant`)
  /// plus the edge-specific `EdgeConnectedBitmask`.
  fn resolve_edge_sprite_source
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
        let mask = compute_edge_connected_bitmask( canon, connects_with, ctx.tiling, &ctx.edge_lookup );
        match bmsource
        {
          NeighborBitmaskSource::ByMapping { mapping, fallback } =>
          {
            match mapping.get( &mask )
            {
              Some( inner ) => resolve_edge_sprite_source( inner, object, canon, ctx ),
              None          => resolve_edge_sprite_source( fallback, object, canon, ctx ),
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
        resolve_animation_frame( anim, ctx.time_seconds, 0.0, canon.0 )
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
        let chosen = pick_variant_index( variants, *selection, canon.0, object, ctx )?;
        resolve_edge_sprite_source( &variants[ chosen ].sprite, object, canon, ctx )
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
  /// `VertexCorners` is handled by [`compile_vertex_pass`].
  fn resolve_sprite_source
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
        // per-instance variant ([`resolve_sprite_source_with_phase`])
        // threads `inst.spawn_time` through for the correct timing.
        resolve_animation_frame( anim, ctx.time_seconds, 0.0, pos )
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
        let chosen = pick_variant_index( variants, *selection, pos, object, ctx )?;
        resolve_sprite_source( &variants[ chosen ].sprite, object, pos, ctx )
      },
      SpriteSource::NeighborBitmask { connects_with, source : bmsource } =>
      {
        let mask = compute_neighbor_bitmask( pos, connects_with, ctx.tiling, &ctx.tile_lookup );
        match bmsource
        {
          NeighborBitmaskSource::ByMapping { mapping, fallback } =>
          {
            match mapping.get( &mask )
            {
              Some( inner ) => resolve_sprite_source( inner, object, pos, ctx ),
              None          => resolve_sprite_source( fallback, object, pos, ctx ),
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
  fn pick_variant_index
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
        weighted_pick( variants, object, | | u64::from( hash_coord( pos.0, pos.1, 0 ) ) )
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
        weighted_pick( variants, object, | | u64::from( hash_coord( pos.0, pos.1, ctx.scene_seed ) ) )
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

  /// Parse a `"#rrggbb"` or `"#rrggbbaa"` colour string into linear-ish
  /// `[f32; 4]`. Returns `None` on malformed input — caller decides whether
  /// to error or fall back.
  fn parse_hex_rgba( s : &str ) -> Option< [ f32; 4 ] >
  {
    let s = s.strip_prefix( '#' )?;
    let hex_byte = | i : usize | u8::from_str_radix( s.get( i..i + 2 )?, 16 ).ok();
    match s.len()
    {
      6 => Some(
      [
        f32::from( hex_byte( 0 )? ) / 255.0,
        f32::from( hex_byte( 2 )? ) / 255.0,
        f32::from( hex_byte( 4 )? ) / 255.0,
        1.0,
      ]),
      8 => Some(
      [
        f32::from( hex_byte( 0 )? ) / 255.0,
        f32::from( hex_byte( 2 )? ) / 255.0,
        f32::from( hex_byte( 4 )? ) / 255.0,
        f32::from( hex_byte( 6 )? ) / 255.0,
      ]),
      _ => None,
    }
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
  pub fn gather_frame_emits
  (
    compiled : &CompiledAssets,
    scene : &Scene,
    camera : &Camera,
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

    let synthetic_tiles = build_scene_tiles( scene );
    let synthetic_edges = build_scene_edges( scene, spec );

    let viewport_size = spec.pipeline.viewport_size.unwrap_or( camera.viewport_size );
    let edge_lookup = build_edge_lookup( &synthetic_edges, spec.pipeline.hex.tiling );
    let seed = scene.seed();
    let scene_seed = ( seed as u32 ) ^ ( ( seed >> 32 ) as u32 );
    let global_tint = resolve_scene_global_tint( spec, scene )?;
    let ctx = FrameContext
    {
      spec,
      compiled,
      camera,
      time_seconds : scene.clock(),
      tile_lookup : build_tile_lookup( &synthetic_tiles ),
      edge_lookup,
      tiling : spec.pipeline.hex.tiling,
      grid_stride : spec.pipeline.hex.grid_stride,
      viewport_size,
      scene_seed,
      global_tint,
    };

    let mut buckets = Vec::with_capacity( spec.pipeline.layers.len() );

    for bucket in &spec.pipeline.layers
    {
      let mut draws : Vec< ( f32, f32, Sprite ) > = Vec::new();

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

        let state_name = scene.state_name( inst.state ).ok_or_else( || CompileError::MissingDefaultState
        {
          object : object.id.clone(),
        })?;
        let stack = object.states.get( state_name ).ok_or_else( || CompileError::MissingDefaultState
        {
          object : object.id.clone(),
        })?;

        let Placement::Hex { q, r } = inst.placement else { continue };

        for layer in stack
        {
          let effective = layer.pipeline_layer.as_deref().unwrap_or( object.global_layer.as_str() );
          if effective != bucket.id { continue; }

          let layer_draws = compile_instance_layer( object, layer, ( q, r ), inst, &ctx )?;
          draws.extend( layer_draws );
        }
      }

      draws.extend( compile_vertex_pass( bucket.id.as_str(), &synthetic_tiles, &ctx )? );
      draws.extend( compile_edge_pass_scene( bucket.id.as_str(), scene, &ctx )? );
      draws.extend( compile_free_pass_scene( bucket.id.as_str(), scene, &ctx )? );

      apply_sort_mode( &mut draws, bucket.sort );

      let sprites : Vec< Sprite > = draws.into_iter().map( | ( _, _, s ) | s ).collect();

      // Viewport pass currently writes `ScreenSpaceSprite` commands to a
      // `Vec<RenderCommand>`; unwrap them back into raw `Sprite`s so the
      // renderer can decide whether to wrap in `ScreenSpaceSprite` or
      // (eventually) batch viewport sprites too.
      let mut tmp : Vec< RenderCommand > = Vec::new();
      compile_viewport_pass_scene( bucket.id.as_str(), scene, &ctx, &mut tmp )?;
      let screen_space : Vec< Sprite > = tmp.into_iter().filter_map( | c | match c
      {
        RenderCommand::ScreenSpaceSprite( s ) => Some( s ),
        _ => None,
      }).collect();

      buckets.push( BucketEmits { sprites, screen_space, sort : bucket.sort } );
    }

    Ok( FrameEmits { clear_color, buckets } )
  }

  /// Flatten [`FrameEmits`] into a per-sprite `RenderCommand` stream —
  /// the pre-Step-4b output shape. Kept as a thin wrapper for the
  /// renderer's fall-back path and for `flatten_to_sprites` test
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
    let emits = gather_frame_emits( compiled, scene, camera )?;
    out.push( RenderCommand::Clear( Clear { color : emits.clear_color } ) );
    for bucket in emits.buckets
    {
      for s in bucket.sprites { out.push( RenderCommand::Sprite( s ) ); }
      for s in bucket.screen_space { out.push( RenderCommand::ScreenSpaceSprite( s ) ); }
    }
    Ok( () )
  }

  /// Build a synthetic `Vec<Tile>` from the scene's hex spatial index.
  ///
  /// Only hex-placed instances contribute (edge / multihex / free / viewport
  /// stay in their own passes). Object ids are looked up via the spec; the
  /// returned tiles are owned and used as the source for `tile_lookup`
  /// in [`render_into`].
  fn build_scene_tiles( scene : &Scene ) -> Vec< Tile >
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
  fn build_scene_edges( scene : &Scene, spec : &RenderSpec ) -> Vec< EdgeInstance >
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

  /// Resolve the effective global tint, honouring `Scene`'s runtime override.
  fn resolve_scene_global_tint( spec : &RenderSpec, scene : &Scene ) -> Result< [ f32; 4 ], CompileError >
  {
    let tint_ref = scene.global_tint().cloned().or_else( || spec.pipeline.global_tint.clone() );
    let Some( tint_ref ) = tint_ref else { return Ok( [ 1.0, 1.0, 1.0, 1.0 ] ); };
    let id = &tint_ref.0;
    let tint = spec.tints.iter().find( | t | &t.id == id )
      .ok_or_else( || CompileError::UnresolvedRef
      {
        kind : "tint",
        id : id.clone(),
        context : "scene.global_tint / pipeline.global_tint".into(),
      })?;
    let [ r, g, b, a ] = parse_hex_rgba( &tint.color ).ok_or_else( || CompileError::UnresolvedRef
    {
      kind : "tint color",
      id : tint.color.clone(),
      context : format!( "tint {:?}", tint.id ),
    })?;
    let s = tint.strength.clamp( 0.0, 1.0 );
    Ok(
    [
      1.0 + s * ( r - 1.0 ),
      1.0 + s * ( g - 1.0 ),
      1.0 + s * ( b - 1.0 ),
      1.0 + s * ( a - 1.0 ),
    ])
  }

  /// Apply a bucket's sort mode to the draw list.
  fn apply_sort_mode( draws : &mut [ ( f32, f32, Sprite ) ], sort : SortMode )
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
  fn compile_instance_layer
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
      return emit_neighbor_condition_with_overrides( object, &tile, condition, sides, sprite_pattern, asset, &layer.behaviour, inst, ctx );
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
      let ( sx, sy ) = ctx.camera.project( ( wx, wy ) );
      let ( sx, sy ) = apply_pivot( sx, sy, ctx.camera.zoom, object.pivot, sprite_id, ctx.compiled );
      let transform = make_transform( sx, sy, ctx.camera.zoom );
      return Ok( vec!
      [
        (
          wx, wy,
          Sprite
          {
            transform,
            sprite : sprite_id,
            tint : final_tint( ctx.global_tint, layer.behaviour.alpha, inst.tint ),
            blend : layer.behaviour.blend,
            clip : None,
          },
        ),
      ]);
    }

    let sprite_ref = resolve_sprite_source_with_phase
    (
      &layer.sprite_source, object, pos, inst.phase_offset, inst.state_entered_time, ctx,
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
    let ( sx, sy ) = ctx.camera.project( ( wx, wy ) );
    let ( sx, sy ) = apply_pivot( sx, sy, ctx.camera.zoom, object.pivot, sprite_id, ctx.compiled );
    let transform = make_transform( sx, sy, ctx.camera.zoom );

    Ok( vec!
    [
      (
        wx, wy,
        Sprite
        {
          transform,
          sprite : sprite_id,
          tint : final_tint( ctx.global_tint, layer.behaviour.alpha, inst.tint ),
          blend : layer.behaviour.blend,
          clip : None,
        },
      ),
    ])
  }

  /// Variant of `resolve_sprite_source` that threads per-instance overrides
  /// (`phase_override`, `oneshot_origin`) into animation resolution.
  ///
  /// `oneshot_origin` is forwarded into [`resolve_animation_frame`] — it
  /// only affects `AnimationMode::OneShot` (whose local time is the elapsed
  /// since the instance entered the state), while `Loop` / `PingPong` keep
  /// riding the master clock for cross-instance harmonic phase.
  fn resolve_sprite_source_with_phase
  (
    source         : &SpriteSource,
    object         : &Object,
    pos            : ( i32, i32 ),
    phase_override : Option< f32 >,
    oneshot_origin : f32,
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
            // animation's declared `PhaseOffset` entirely.
            let mut anim_clone = anim.clone();
            anim_clone.phase_offset = crate::resource::PhaseOffset::Fixed( phase );
            resolve_animation_frame( &anim_clone, ctx.time_seconds, oneshot_origin, pos )
          },
          None => resolve_animation_frame( anim, ctx.time_seconds, oneshot_origin, pos ),
        }
      },
      SpriteSource::Variant { variants, selection } =>
      {
        if variants.is_empty()
        {
          return Err( CompileError::OutOfRange { owner : object.id.clone(), index : 0, max : 0 } );
        }
        let chosen = pick_variant_index( variants, *selection, pos, object, ctx )?;
        resolve_sprite_source_with_phase
        (
          &variants[ chosen ].sprite, object, pos, phase_override, oneshot_origin, ctx,
        )
      },
      _ => resolve_sprite_source( source, object, pos, ctx ),
    }
  }

  /// `emit_neighbor_condition` variant that composes the per-instance
  /// tint into each emitted sprite.
  #[ allow( clippy::too_many_arguments ) ]
  #[ allow( clippy::similar_names ) ]   // raw_sx / raw_sy are a coordinate pair
  fn emit_neighbor_condition_with_overrides
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
    let ( raw_sx, raw_sy ) = ctx.camera.project( ( wx, wy ) );

    let mut out = Vec::new();
    for &side in sides
    {
      let Some( offset ) = neighbor_offset_by_dir( ctx.tiling, side ) else { continue; };
      let neighbour_pos = ( tile.pos.0 + offset.0, tile.pos.1 + offset.1 );
      let neighbour = neighbor_state_at( neighbour_pos, &ctx.tile_lookup, ctx.spec );
      if !evaluate_condition( condition, &neighbour, current_priority ) { continue; }

      let frame_name = sprite_pattern.replace( "{dir}", dir_name( side ) );
      let sprite_id = ctx.compiled.ids.sprite( asset, &frame_name )
        .ok_or_else( || CompileError::UnresolvedRef
        {
          kind : "sprite",
          id : format!( "{asset}:{frame_name}" ),
          context : format!( "object {:?} NeighborCondition side {side:?}", object.id ),
        })?;

      let ( sx, sy ) = apply_pivot( raw_sx, raw_sy, ctx.camera.zoom, object.pivot, sprite_id, ctx.compiled );
      let transform = make_transform( sx, sy, ctx.camera.zoom );

      out.push
      ((
        wx, wy,
        Sprite
        {
          transform,
          sprite : sprite_id,
          tint : final_tint( ctx.global_tint, behaviour.alpha, inst.tint ),
          blend : behaviour.blend,
          clip : None,
        },
      ));
    }
    Ok( out )
  }

  /// Compose the per-sprite tint as
  /// `global * layer_alpha (alpha-channel only) * instance_tint`.
  #[ inline ]
  fn final_tint( global : [ f32; 4 ], layer_alpha : f32, inst : Option< [ f32; 4 ] > ) -> [ f32; 4 ]
  {
    let [ gr, gg, gb, ga ] = global;
    let composed = [ gr, gg, gb, ga * layer_alpha ];
    match inst
    {
      None => composed,
      Some( [ ir, ig, ib, ia ] ) =>
      [
        composed[ 0 ] * ir,
        composed[ 1 ] * ig,
        composed[ 2 ] * ib,
        composed[ 3 ] * ia,
      ],
    }
  }

  /// Emit sprites for every Scene edge handle whose owning Object routes
  /// into `bucket_id`. Mirrors `compile_edge_pass` but iterates Scene's
  /// handle list and applies per-instance overrides.
  fn compile_edge_pass_scene
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

      for layer in stack
      {
        let effective = layer.pipeline_layer.as_deref().unwrap_or( object.global_layer.as_str() );
        if effective != bucket_id { continue; }

        let sprite_ref = resolve_edge_sprite_source( &layer.sprite_source, object, canon, ctx )?;
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
        let ( sx, sy ) = ctx.camera.project( ( wx, wy ) );
        let ( sx, sy ) = apply_pivot( sx, sy, ctx.camera.zoom, object.pivot, sprite_id, ctx.compiled );

        let transform = Transform
        {
          position : [ sx, sy ],
          rotation : edge_rotation( canon.1, ctx.tiling ),
          scale : [ ctx.camera.zoom, ctx.camera.zoom ],
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
            tint : final_tint( ctx.global_tint, layer.behaviour.alpha, inst.tint ),
            blend : layer.behaviour.blend,
            clip : None,
          },
        ));
      }
    }
    Ok( out )
  }

  /// Emit sprites for every Scene free-pos handle. Mirrors `compile_free_pass`.
  fn compile_free_pass_scene
  (
    bucket_id : &str,
    scene : &Scene,
    ctx : &FrameContext< '_ >,
  ) -> Result< Vec< ( f32, f32, Sprite ) >, CompileError >
  {
    let mut out = Vec::new();
    for &handle in scene.free_instances()
    {
      let inst = scene.instance( handle ).expect( "free handle live" );
      if !inst.visible { continue; }
      let Placement::FreePos { x, y } = inst.placement else { unreachable!() };

      let object = &ctx.spec.objects[ inst.object.index() as usize ];
      if !matches!( object.anchor, Anchor::FreePos )
      {
        return Err( CompileError::UnsupportedAnchor
        {
          object : object.id.clone(),
          anchor : "FreePos (object declares a different anchor)",
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

      for layer in stack
      {
        let effective = layer.pipeline_layer.as_deref().unwrap_or( object.global_layer.as_str() );
        if effective != bucket_id { continue; }

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
          let ( sx, sy ) = ctx.camera.project( ( wx, wy ) );
          let ( sx, sy ) = apply_pivot( sx, sy, ctx.camera.zoom, object.pivot, sprite_id, ctx.compiled );
          let transform = make_transform( sx, sy, ctx.camera.zoom );
          out.push((
            wx, wy,
            Sprite
            {
              transform,
              sprite : sprite_id,
              tint : final_tint( ctx.global_tint, layer.behaviour.alpha, inst.tint ),
              blend : layer.behaviour.blend,
              clip : None,
            },
          ));
          continue;
        }

        let sprite_ref = resolve_sprite_source_with_phase
        (
          &layer.sprite_source, object, ( 0, 0 ), inst.phase_offset, inst.state_entered_time, ctx,
        )?;
        let sprite_id = ctx.compiled.ids.sprite( &sprite_ref.asset, &sprite_ref.frame )
          .ok_or_else( || CompileError::UnresolvedRef
          {
            kind : "sprite",
            id : format!( "{}:{}", sprite_ref.asset, sprite_ref.frame ),
            context : format!( "object {:?} free layer", object.id ),
          })?;

        let ( wx, wy ) = ( x, y );
        let ( sx, sy ) = ctx.camera.project( ( wx, wy ) );
        let ( sx, sy ) = apply_pivot( sx, sy, ctx.camera.zoom, object.pivot, sprite_id, ctx.compiled );
        let transform = make_transform( sx, sy, ctx.camera.zoom );

        out.push
        ((
          wx, wy,
          Sprite
          {
            transform,
            sprite : sprite_id,
            tint : final_tint( ctx.global_tint, layer.behaviour.alpha, inst.tint ),
            blend : layer.behaviour.blend,
            clip : None,
          },
        ));
      }
    }
    Ok( out )
  }

  /// Emit `ScreenSpaceSprite` commands for every Scene viewport handle.
  fn compile_viewport_pass_scene
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

      for layer in stack
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

        let sprite_ref = resolve_sprite_source( content, object, ( 0, 0 ), ctx )?;
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
              tint : final_tint( ctx.global_tint, layer.behaviour.alpha, inst.tint ),
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
            tint : final_tint( ctx.global_tint, layer.behaviour.alpha, inst.tint ),
            blend : layer.behaviour.blend,
            clip : None,
          }));
        }
      }
    }
    Ok( () )
  }
}

mod_interface::mod_interface!
{
  own use render_into;
  own use gather_frame_emits;
  own use FrameEmits;
  own use BucketEmits;
}
