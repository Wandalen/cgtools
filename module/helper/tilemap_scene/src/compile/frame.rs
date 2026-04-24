//! `compile_frame` — scene-model `Scene` → per-frame `RenderCommand` stream.
//!
//! Supported through Slice 3: hex-anchored objects with Static / Animation /
//! Variant / NeighborBitmask / NeighborCondition sources, plus per-bucket
//! dual-mesh VertexCorners triangle emission. Edge / Multihex / Viewport /
//! FreePos anchors and EdgeConnectedBitmask / ViewportTiled / External
//! sources remain rejected until later slices.

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
  use crate::layer::ObjectLayer;
  use crate::object::Object;
  use crate::pipeline::{ SortMode, TilingStrategy };
  use crate::resource::SpriteRef;
  use crate::compile::viewport::{ tiled_positions, viewport_transform };
  use crate::scene::{ EdgeInstance, Scene, Tile };
  use crate::source::{ NeighborBitmaskSource, SpriteSource, VariantSelection, ViewportTiling };
  use crate::spec::RenderSpec;
  use tilemap_renderer::types::{ BlendMode, FillRef as _FillRef, Transform };
  use rustc_hash::FxHashMap as HashMap;

  /// Compile one frame of the scene into a `RenderCommand` stream.
  ///
  /// # Errors
  ///
  /// Returns [`CompileError`] when the scene / spec uses a feature not yet
  /// supported in Slice 1 (see module-level docs) or when an id reference
  /// cannot be resolved.
  pub fn compile_frame
  (
    spec : &RenderSpec,
    scene : &Scene,
    compiled : &CompiledAssets,
    camera : &Camera,
    time_seconds : f32,
  ) -> Result< Vec< RenderCommand >, CompileError >
  {
    let mut commands : Vec< RenderCommand > = Vec::new();

    // 1. Clear the framebuffer. Colour comes from `RenderPipeline.clear_color`
    //    when set, otherwise transparent black so the backend's own background
    //    shows through.
    let clear_color = spec.pipeline.clear_color.unwrap_or( [ 0.0, 0.0, 0.0, 0.0 ] );
    commands.push( RenderCommand::Clear( Clear { color : clear_color } ) );

    // 2. Reject unsupported scene features early — clear error > confusing
    //    empty output. Slice 4 unlocks Edge / FreePos / Viewport anchors;
    //    Multihex still deferred to a later slice.
    if !scene.multihex_instances.is_empty()
    {
      return Err( CompileError::UnsupportedAnchor
      {
        object : scene.multihex_instances[ 0 ].object.clone(),
        anchor : "Multihex",
      });
    }
    if !scene.entities.is_empty()
    {
      return Err( CompileError::UnsupportedSource
      {
        object : scene.entities[ 0 ].object.clone(),
        source : "Entity (entities go through animations / runtime mutation — post-Slice-1)",
      });
    }

    // 3. Expand `(palette, map)` ASCII form into virtual `Tile`s if needed.
    //    Slice 1 maps column → q, row → r directly (no offset-coordinate
    //    correction); callers who want proper offset layouts provide
    //    `scene.tiles` explicitly.
    let tiles_owned;
    let tiles : &[ Tile ] = if scene.tiles.is_empty()
    {
      tiles_owned = expand_palette( scene );
      &tiles_owned
    }
    else
    {
      &scene.tiles
    };

    // 4. Build the per-frame context once. Everything downstream reads
    //    from it instead of taking 8+ parameters each.
    //
    // Viewport size resolution: `RenderPipeline.viewport_size` wins when set
    // (authoring-time explicit), otherwise fall back to the camera's value
    // (runtime-supplied). Used by `ViewportTiled` transform math.
    let viewport_size = spec.pipeline.viewport_size.unwrap_or( camera.viewport_size );
    let edge_lookup = build_edge_lookup( &scene.edges, spec.pipeline.hex.tiling );
    // Fold the 64-bit scene seed down to the 32-bit salt that `hash_coord`
    // expects. XOR halves keeps both halves contributing.
    let scene_seed = scene.seed.map( | s | ( s as u32 ) ^ ( ( s >> 32 ) as u32 ) ).unwrap_or( 0 );
    let global_tint = resolve_global_tint( spec )?;
    let ctx = FrameContext
    {
      spec,
      compiled,
      camera,
      time_seconds,
      tile_lookup : build_tile_lookup( tiles ),
      edge_lookup,
      tiling : spec.pipeline.hex.tiling,
      grid_stride : spec.pipeline.hex.grid_stride,
      viewport_size,
      scene_seed,
      global_tint,
    };

    // 5. Per-bucket emission.
    for bucket in &spec.pipeline.layers
    {
      let mut draws : Vec< ( f32, f32, Sprite ) > = Vec::new();

      for tile in tiles
      {
        for object_id in &tile.objects
        {
          let object = find_object( spec, object_id )?;

          // Non-Hex anchors are handled by their own passes later in this
          // bucket loop. Multihex is still rejected (unsupported).
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

          let stack = object.states.get( &object.default_state )
            .ok_or_else( || CompileError::MissingDefaultState { object : object.id.clone() } )?;

          for layer in stack
          {
            // Effective pipeline bucket: layer-level override or object-level default.
            let effective = layer.pipeline_layer.as_deref().unwrap_or( object.global_layer.as_str() );
            if effective != bucket.id
            {
              continue;
            }

            let layer_draws = compile_layer( object, layer, tile, &ctx )?;
            draws.extend( layer_draws );
          }
        }
      }

      // Second pass — vertex-anchored sprites (dual-mesh triangles).
      // `VertexCorners` lives on objects that declare `anchor: Vertex`, but
      // the format also lets an object with any anchor carry a `VertexCorners`
      // layer — the layer's `anchor` is implicit from the source kind, not
      // from the owning object. For Slice 3 we emit all vertex sprites whose
      // pattern matches, regardless of which object owns the source.
      let vertex_draws = compile_vertex_pass( bucket.id.as_str(), tiles, &ctx )?;
      draws.extend( vertex_draws );

      // Third pass — edge-anchored sprites (Slice 4).
      let edge_draws = compile_edge_pass( bucket.id.as_str(), scene, &ctx )?;
      draws.extend( edge_draws );

      // Fourth pass — FreePos-anchored sprites (Slice 4).
      let free_draws = compile_free_pass( bucket.id.as_str(), scene, &ctx )?;
      draws.extend( free_draws );

      match bucket.sort
      {
        SortMode::None => {}
        SortMode::XAsc => draws.sort_by( | a, b | a.0.partial_cmp( &b.0 ).unwrap_or( core::cmp::Ordering::Equal ) ),
        SortMode::XDesc => draws.sort_by( | a, b | b.0.partial_cmp( &a.0 ).unwrap_or( core::cmp::Ordering::Equal ) ),
        SortMode::YAsc => draws.sort_by( | a, b | a.1.partial_cmp( &b.1 ).unwrap_or( core::cmp::Ordering::Equal ) ),
        SortMode::YDesc => draws.sort_by( | a, b | b.1.partial_cmp( &a.1 ).unwrap_or( core::cmp::Ordering::Equal ) ),
        SortMode::XAscYDesc => draws.sort_by( | a, b |
          a.0.partial_cmp( &b.0 ).unwrap_or( core::cmp::Ordering::Equal )
            .then_with( || b.1.partial_cmp( &a.1 ).unwrap_or( core::cmp::Ordering::Equal ) )
        ),
        SortMode::XAscYAsc => draws.sort_by( | a, b |
          a.0.partial_cmp( &b.0 ).unwrap_or( core::cmp::Ordering::Equal )
            .then_with( || a.1.partial_cmp( &b.1 ).unwrap_or( core::cmp::Ordering::Equal ) )
        ),
        SortMode::YDescXAsc => draws.sort_by( | a, b |
          b.1.partial_cmp( &a.1 ).unwrap_or( core::cmp::Ordering::Equal )
            .then_with( || a.0.partial_cmp( &b.0 ).unwrap_or( core::cmp::Ordering::Equal ) )
        ),
        SortMode::YAscXAsc => draws.sort_by( | a, b |
          a.1.partial_cmp( &b.1 ).unwrap_or( core::cmp::Ordering::Equal )
            .then_with( || a.0.partial_cmp( &b.0 ).unwrap_or( core::cmp::Ordering::Equal ) )
        ),
      }

      for ( _, _, sprite ) in draws
      {
        // Pivot compensation for each sprite already applied at emit sites
        // (see `apply_pivot`), so `transform.position` is already the
        // backend's bottom-left anchor needed to align the object's scene
        // anchor with its configured pivot point.
        commands.push( RenderCommand::Sprite( sprite ) );
      }

      // Fifth pass — viewport-anchored sprites (Slice 4). Emitted last in
      // the bucket as `ScreenSpaceSprite` — backend skips the camera
      // transform; coordinates are already in screen pixels.
      compile_viewport_pass( bucket.id.as_str(), scene, &ctx, &mut commands )?;
    }

    Ok( commands )
  }

  /// Build a single [`Sprite`] draw call for a tile's layer. Returns a tuple
  /// `( world_x, world_y, sprite )` so the caller can apply `SortMode` without
  /// re-computing the position. The returned Sprite already has its final
  /// projected transform.
  /// Bundled per-frame context threaded into helper functions.
  ///
  /// Previously each helper took 6–10 individual parameters; consolidating
  /// them here means a single `&ctx` parameter for 95% of what the compile
  /// pipeline needs to know about. Fields are read-only — no mutation
  /// happens once the context is built in [`compile_frame`].
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

  /// Compile one layer into zero, one, or many `( world_x, world_y, Sprite )`
  /// tuples. Most sources emit exactly one sprite; `NeighborCondition` can
  /// emit up to `len(sides)` per tile (one per matching side).
  ///
  /// `VertexCorners` is intentionally handled NOT here but in the separate
  /// `compile_vertex_pass` — vertex sprites aren't anchored to a tile.
  fn compile_layer
  (
    object : &Object,
    layer : &ObjectLayer,
    tile : &Tile,
    ctx : &FrameContext< '_ >,
  ) -> Result< Vec< ( f32, f32, Sprite ) >, CompileError >
  {
    // `NeighborCondition` emits per matching side — handled separately because
    // it produces N sprites, not one.
    if let SpriteSource::NeighborCondition { condition, sides, sprite_pattern, asset } = &layer.sprite_source
    {
      return emit_neighbor_condition( object, tile, condition, sides, sprite_pattern, asset, ctx );
    }

    // `VertexCorners` is not emitted per tile.
    if matches!( &layer.sprite_source, SpriteSource::VertexCorners { .. } )
    {
      return Ok( Vec::new() );
    }

    let sprite_ref = resolve_sprite_source( &layer.sprite_source, object, tile.pos, ctx )?;

    let sprite_id = ctx.compiled.ids.sprite( &sprite_ref.0, &sprite_ref.1 )
      .ok_or_else( || CompileError::UnresolvedRef
      {
        kind : "sprite",
        id : format!( "{}:{}", sprite_ref.0, sprite_ref.1 ),
        context : format!( "object {:?} layer", object.id ),
      })?;

    let ( q, r ) = tile.pos;
    let ( wx, wy ) = hex_world_pixel( q, r, ctx, &object.id )?;
    let ( sx, sy ) = ctx.camera.project( ( wx, wy ) );
    let ( sx, sy ) = apply_pivot( sx, sy, ctx.camera.zoom, object.pivot, sprite_id, ctx.compiled );

    let transform = make_transform( sx, sy, ctx.camera.zoom );
    let _ = _FillRef::default;

    Ok( vec!
    [
      (
        wx, wy,
        Sprite
        {
          transform,
          sprite : sprite_id,
          tint : ctx.global_tint,
          blend : BlendMode::default(),
          clip : None,
        },
      ),
    ])
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

  /// Emit one sprite per matching side for a `NeighborCondition` layer.
  #[ allow( clippy::too_many_arguments ) ]   // still split across `condition` / `sides` / `sprite_pattern` / `asset` — all from the layer
  fn emit_neighbor_condition
  (
    object : &Object,
    tile : &Tile,
    condition : &crate::source::Condition,
    sides : &[ crate::anchor::EdgeDirection ],
    sprite_pattern : &str,
    asset : &str,
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
      let Some( offset ) = neighbor_offset_by_dir( ctx.tiling, side )
      else
      {
        continue;   // direction doesn't apply to this tiling — skip quietly.
      };
      let neighbour_pos = ( tile.pos.0 + offset.0, tile.pos.1 + offset.1 );
      let neighbour = neighbor_state_at( neighbour_pos, &ctx.tile_lookup, ctx.spec );
      if !evaluate_condition( condition, &neighbour, current_priority )
      {
        continue;
      }

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
          tint : ctx.global_tint,
          blend : BlendMode::default(),
          clip : None,
        },
      ));
    }
    Ok( out )
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
            tint : ctx.global_tint,
            blend : BlendMode::default(),
            clip : None,
          },
        ));
      }
    }

    Ok( out )
  }

  /// Emit sprites for every `EdgeInstance` whose owning `Object` routes
  /// into `bucket_id`. Canonicalisation dedupes duplicate declarations of
  /// the same edge from each side.
  fn compile_edge_pass
  (
    bucket_id : &str,
    scene : &Scene,
    ctx : &FrameContext< '_ >,
  ) -> Result< Vec< ( f32, f32, Sprite ) >, CompileError >
  {
    let mut out = Vec::new();
    // Deduplicate: only emit for the canonical form of each edge.
    let mut seen : rustc_hash::FxHashSet< CanonicalEdge > = rustc_hash::FxHashSet::default();

    for inst in &scene.edges
    {
      let Some( canon ) = canonical_edge( inst.at, ctx.tiling ) else { continue };
      if !seen.insert( canon ) { continue; }

      let object = find_object( ctx.spec, &inst.object )?;
      if !matches!( object.anchor, Anchor::Edge )
      {
        return Err( CompileError::UnsupportedAnchor
        {
          object : object.id.clone(),
          anchor : "Edge (object declares a different anchor)",
        });
      }

      let stack = object.states.get( &object.default_state )
        .ok_or_else( || CompileError::MissingDefaultState { object : object.id.clone() } )?;

      for layer in stack
      {
        let effective = layer.pipeline_layer.as_deref().unwrap_or( object.global_layer.as_str() );
        if effective != bucket_id { continue; }

        let sprite_ref = resolve_edge_sprite_source( &layer.sprite_source, object, canon, ctx )?;
        let sprite_id = ctx.compiled.ids.sprite( &sprite_ref.0, &sprite_ref.1 )
          .ok_or_else( || CompileError::UnresolvedRef
          {
            kind : "sprite",
            id : format!( "{}:{}", sprite_ref.0, sprite_ref.1 ),
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
            tint : ctx.global_tint,
            blend : BlendMode::default(),
            clip : None,
          },
        ));
      }
    }

    Ok( out )
  }

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
            Ok( SpriteRef( asset.clone(), mask.to_string() ) )
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
        resolve_animation_frame( anim, ctx.time_seconds, canon.0 )
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
        source : source_name( other ),
      }),
    }
  }

  /// Emit sprites for every `FreeInstance` whose owning object routes into
  /// `bucket_id`. Position comes straight from the instance; no grid math.
  fn compile_free_pass
  (
    bucket_id : &str,
    scene : &Scene,
    ctx : &FrameContext< '_ >,
  ) -> Result< Vec< ( f32, f32, Sprite ) >, CompileError >
  {
    let mut out = Vec::new();
    for inst in &scene.free_instances
    {
      let object = find_object( ctx.spec, &inst.object )?;
      if !matches!( object.anchor, Anchor::FreePos )
      {
        return Err( CompileError::UnsupportedAnchor
        {
          object : object.id.clone(),
          anchor : "FreePos (object declares a different anchor)",
        });
      }

      let stack = object.states.get( &object.default_state )
        .ok_or_else( || CompileError::MissingDefaultState { object : object.id.clone() } )?;

      for layer in stack
      {
        let effective = layer.pipeline_layer.as_deref().unwrap_or( object.global_layer.as_str() );
        if effective != bucket_id { continue; }

        // Guard: neighbour-aware sources require grid context and make no
        // sense on a free-world-pixel anchor.
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
              source : source_name( &layer.sprite_source ),
            });
          },
          _ => {},
        }

        let sprite_ref = resolve_sprite_source( &layer.sprite_source, object, ( 0, 0 ), ctx )?;
        let sprite_id = ctx.compiled.ids.sprite( &sprite_ref.0, &sprite_ref.1 )
          .ok_or_else( || CompileError::UnresolvedRef
          {
            kind : "sprite",
            id : format!( "{}:{}", sprite_ref.0, sprite_ref.1 ),
            context : format!( "object {:?} free layer", object.id ),
          })?;

        let ( wx, wy ) = inst.pos;
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
            tint : ctx.global_tint,
            blend : BlendMode::default(),
            clip : None,
          },
        ));
      }
    }
    Ok( out )
  }

  /// Emit `ScreenSpaceSprite` commands for every `ViewportInstance` whose
  /// owning object routes into `bucket_id`. Sprites bypass the camera.
  fn compile_viewport_pass
  (
    bucket_id : &str,
    scene : &Scene,
    ctx : &FrameContext< '_ >,
    commands : &mut Vec< RenderCommand >,
  ) -> Result< (), CompileError >
  {
    for inst in &scene.viewport_instances
    {
      let object = find_object( ctx.spec, &inst.object )?;
      if !matches!( object.anchor, Anchor::Viewport )
      {
        return Err( CompileError::UnsupportedAnchor
        {
          object : object.id.clone(),
          anchor : "Viewport (object declares a different anchor)",
        });
      }

      let stack = object.states.get( &object.default_state )
        .ok_or_else( || CompileError::MissingDefaultState { object : object.id.clone() } )?;

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
            source : "Viewport-anchored layer must use ViewportTiled",
          });
        };

        // Resolve the inner content to a concrete sprite_ref. Free-pos hex
        // (0,0) is a placeholder — animation phase_offset on viewport
        // objects isn't meaningful.
        let sprite_ref = resolve_sprite_source( content, object, ( 0, 0 ), ctx )?;
        let sprite_id = ctx.compiled.ids.sprite( &sprite_ref.0, &sprite_ref.1 )
          .ok_or_else( || CompileError::UnresolvedRef
          {
            kind : "sprite",
            id : format!( "{}:{}", sprite_ref.0, sprite_ref.1 ),
            context : format!( "object {:?} viewport layer", object.id ),
          })?;

        let region = ctx.compiled.assets.sprites.iter()
          .find( | s | s.id == sprite_id )
          .map( | s | ( s.region[ 2 ], s.region[ 3 ] ) )
          .unwrap_or( ( 1.0, 1.0 ) );

        // Non-repeating modes: one sprite, transform from viewport_transform.
        // Repeating modes: one sprite per tiled_positions() entry, each at
        // native pixel size.
        let is_repeat = matches!
        (
          vtiling,
          ViewportTiling::Repeat2D | ViewportTiling::RepeatX | ViewportTiling::RepeatY
        );

        if is_repeat
        {
          // Repeat tiles are rendered at **camera-zoom scale** so one texture
          // pixel matches one world-pixel-through-camera — keeps backgrounds
          // visually coherent with zoomed foreground sprites. Tile step is
          // the on-screen size (`native * zoom`), so the grid stays
          // viewport-covering regardless of zoom.
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
              tint : ctx.global_tint,
              blend : BlendMode::default(),
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
              source : "ViewportTiled (unsupported tiling)",
            });
          };
          commands.push( RenderCommand::ScreenSpaceSprite( Sprite
          {
            transform,
            sprite : sprite_id,
            tint : ctx.global_tint,
            blend : BlendMode::default(),
            clip : None,
          }));
        }
      }
    }
    Ok( () )
  }

  /// Expand an ASCII palette+map scene into concrete tiles.
  ///
  /// Simple mapping for Slice 1: `col → q`, `row → r`, whitespace ignored.
  /// No offset-coordinate correction — callers needing exact hex offset
  /// layouts provide `scene.tiles` directly.
  fn expand_palette( scene : &Scene ) -> Vec< Tile >
  {
    let mut out = Vec::new();
    for ( row_index, row ) in scene.map.iter().enumerate()
    {
      let mut col : i32 = 0;
      for ch in row.chars()
      {
        if ch.is_whitespace()
        {
          continue;
        }
        if let Some( objects ) = scene.palette.get( &ch )
        {
          out.push( Tile
          {
            pos : ( col, row_index as i32 ),
            objects : objects.clone(),
          });
        }
        col = col.saturating_add( 1 );
      }
    }
    out
  }

  fn find_object< 'spec >
  (
    spec : &'spec RenderSpec,
    object_id : &str,
  ) -> Result< &'spec Object, CompileError >
  {
    spec.objects.iter().find( | o | o.id == object_id ).ok_or_else( || CompileError::UnresolvedRef
    {
      kind : "object",
      id : object_id.to_owned(),
      context : "tile objects".into(),
    })
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
        resolve_animation_frame( anim, ctx.time_seconds, pos )
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
            Ok( SpriteRef( asset.clone(), mask.to_string() ) )
          },
        }
      },
      other => Err( CompileError::UnsupportedSource
      {
        object : object.id.clone(),
        source : source_name( other ),
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

  /// Resolve the pipeline's optional `global_tint` reference into an effective
  /// RGBA multiplier. Returns `[1,1,1,1]` when no global tint is set — every
  /// emit site can then do `sprite.tint * ctx.global_tint` unconditionally.
  ///
  /// Composition model: the tint's `strength` interpolates between identity
  /// white and the parsed colour; the result is then multiplied into each
  /// sprite. Non-`Multiply` blend modes are not yet implemented here — they
  /// fall back to multiply and log-compatible note in the error if a tint
  /// declares one (to keep callers explicit about future behaviour).
  fn resolve_global_tint( spec : &RenderSpec ) -> Result< [ f32; 4 ], CompileError >
  {
    let Some( tint_ref ) = &spec.pipeline.global_tint else { return Ok( [ 1.0, 1.0, 1.0, 1.0 ] ); };
    let id = &tint_ref.0;
    let tint = spec.tints.iter().find( | t | &t.id == id )
      .ok_or_else( || CompileError::UnresolvedRef
      {
        kind : "tint",
        id : id.clone(),
        context : "pipeline.global_tint".into(),
      })?;

    let [ r, g, b, a ] = parse_hex_rgba( &tint.color ).ok_or_else( || CompileError::UnresolvedRef
    {
      kind : "tint color",
      id : tint.color.clone(),
      context : format!( "tint {:?}", tint.id ),
    })?;

    let s = tint.strength.clamp( 0.0, 1.0 );
    // Multiply blend: lerp(white, colour, strength). Alpha follows the same
    // rule so a fully-opaque tint at strength 1.0 replaces; at strength 0.0
    // the multiplier is identity.
    Ok(
    [
      1.0 + s * ( r - 1.0 ),
      1.0 + s * ( g - 1.0 ),
      1.0 + s * ( b - 1.0 ),
      1.0 + s * ( a - 1.0 ),
    ])
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
        hex_byte( 0 )? as f32 / 255.0,
        hex_byte( 2 )? as f32 / 255.0,
        hex_byte( 4 )? as f32 / 255.0,
        1.0,
      ]),
      8 => Some(
      [
        hex_byte( 0 )? as f32 / 255.0,
        hex_byte( 2 )? as f32 / 255.0,
        hex_byte( 4 )? as f32 / 255.0,
        hex_byte( 6 )? as f32 / 255.0,
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
}

mod_interface::mod_interface!
{
  exposed use compile_frame;
}
