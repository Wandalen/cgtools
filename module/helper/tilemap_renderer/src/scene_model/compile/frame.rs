//! `compile_frame` — scene-model `Scene` → per-frame `RenderCommand` stream.
//!
//! Supported through Slice 3: hex-anchored objects with Static / Animation /
//! Variant / NeighborBitmask / NeighborCondition sources, plus per-bucket
//! dual-mesh VertexCorners triangle emission. Edge / Multihex / Viewport /
//! FreePos anchors and EdgeConnectedBitmask / ViewportTiled / External
//! sources remain rejected until later slices.

mod private
{
  use crate::commands::{ Clear, RenderCommand, Sprite };
  use crate::scene_model::anchor::Anchor;
  use crate::scene_model::compile::animation::resolve_animation_frame;
  use crate::scene_model::compile::assets::CompiledAssets;
  use crate::scene_model::compile::camera::Camera;
  use crate::scene_model::compile::conditions::evaluate_condition;
  use crate::scene_model::compile::coords::{ hex_to_world_pixel_flat, hex_to_world_pixel_pointy };
  use crate::scene_model::compile::error::CompileError;
  use crate::scene_model::compile::neighbors::
  {
    compute_neighbor_bitmask,
    dir_name,
    neighbor_offset_by_dir,
    neighbor_state_at,
    tile_lookup as build_tile_lookup,
    tile_max_priority,
  };
  use crate::scene_model::compile::vertex::
  {
    canonicalize,
    enumerate_triangles,
    find_matching_pattern,
    resolve_corners,
  };
  use crate::scene_model::hash::hash_coord;
  use crate::scene_model::layer::ObjectLayer;
  use crate::scene_model::object::Object;
  use crate::scene_model::pipeline::{ SortMode, TilingStrategy };
  use crate::scene_model::resource::SpriteRef;
  use crate::scene_model::scene::{ Scene, Tile };
  use crate::scene_model::source::{ NeighborBitmaskSource, SpriteSource, VariantSelection };
  use crate::scene_model::spec::RenderSpec;
  use crate::types::{ BlendMode, FillRef as _FillRef, Transform };
  use std::collections::HashMap;

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

    // 1. Clear the framebuffer. Slice 1 uses transparent black — callers that
    //    want a specific background issue their own `Clear` beforehand or set
    //    `RenderConfig.background` on the backend.
    commands.push( RenderCommand::Clear( Clear { color : [ 0.0, 0.0, 0.0, 0.0 ] } ) );

    // 2. Reject unsupported scene features early — clear error > confusing
    //    empty output.
    if !scene.edges.is_empty()
    {
      return Err( CompileError::UnsupportedAnchor
      {
        object : scene.edges[ 0 ].object.clone(),
        anchor : "Edge",
      });
    }
    if !scene.multihex_instances.is_empty()
    {
      return Err( CompileError::UnsupportedAnchor
      {
        object : scene.multihex_instances[ 0 ].object.clone(),
        anchor : "Multihex",
      });
    }
    if !scene.free_instances.is_empty()
    {
      return Err( CompileError::UnsupportedAnchor
      {
        object : scene.free_instances[ 0 ].object.clone(),
        anchor : "FreePos",
      });
    }
    if !scene.viewport_instances.is_empty()
    {
      return Err( CompileError::UnsupportedAnchor
      {
        object : scene.viewport_instances[ 0 ].object.clone(),
        anchor : "Viewport",
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
    let ctx = FrameContext
    {
      spec,
      compiled,
      camera,
      time_seconds,
      tile_lookup : build_tile_lookup( tiles ),
      tiling : spec.pipeline.hex.tiling,
      cell_size : spec.pipeline.hex.cell_size,
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

          // Reject anchors outside Slice 3.
          if !matches!( object.anchor, Anchor::Hex )
          {
            return Err( CompileError::UnsupportedAnchor
            {
              object : object.id.clone(),
              anchor : anchor_name( &object.anchor ),
            });
          }

          let stack = object.animations.get( &object.default_animation )
            .ok_or_else( || CompileError::MissingDefaultAnimation { object : object.id.clone() } )?;

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

      match bucket.sort
      {
        SortMode::None => {}
        SortMode::YAsc => draws.sort_by( | a, b | a.1.partial_cmp( &b.1 ).unwrap_or( core::cmp::Ordering::Equal ) ),
        SortMode::YDesc => draws.sort_by( | a, b | b.1.partial_cmp( &a.1 ).unwrap_or( core::cmp::Ordering::Equal ) ),
      }

      for ( _, _, sprite ) in draws
      {
        commands.push( RenderCommand::Sprite( sprite ) );
      }
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
    tiling : TilingStrategy,
    cell_size : ( u32, u32 ),
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
          tint : [ 1.0, 1.0, 1.0, 1.0 ],
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
      TilingStrategy::HexFlatTop   => Ok( hex_to_world_pixel_flat( q, r, ctx.cell_size ) ),
      TilingStrategy::HexPointyTop => Ok( hex_to_world_pixel_pointy( q, r, ctx.cell_size ) ),
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
    condition : &crate::scene_model::source::Condition,
    sides : &[ crate::scene_model::anchor::EdgeDirection ],
    sprite_pattern : &str,
    asset : &str,
    ctx : &FrameContext< '_ >,
  ) -> Result< Vec< ( f32, f32, Sprite ) >, CompileError >
  {
    let current_priority = tile_max_priority( tile, ctx.spec );

    let ( q, r ) = tile.pos;
    let ( wx, wy ) = hex_world_pixel( q, r, ctx, &object.id )?;
    let ( sx, sy ) = ctx.camera.project( ( wx, wy ) );
    let transform = make_transform( sx, sy, ctx.camera.zoom );

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

      out.push
      ((
        wx, wy,
        Sprite
        {
          transform,
          sprite : sprite_id,
          tint : [ 1.0, 1.0, 1.0, 1.0 ],
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
      let Some( stack ) = object.animations.get( &object.default_animation )
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
        let transform = make_transform( sx, sy, ctx.camera.zoom );

        out.push
        ((
          wx, wy,
          Sprite
          {
            transform,
            sprite : sprite_id,
            tint : [ 1.0, 1.0, 1.0, 1.0 ],
            blend : BlendMode::default(),
            clip : None,
          },
        ));
      }
    }

    Ok( out )
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

  fn anchor_name( a : &Anchor ) -> &'static str
  {
    match a
    {
      Anchor::Hex          => "Hex",
      Anchor::Edge         => "Edge",
      Anchor::Vertex       => "Vertex",
      Anchor::Multihex { .. } => "Multihex",
      Anchor::FreePos      => "FreePos",
      Anchor::Viewport     => "Viewport",
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
        let chosen = pick_variant_index( variants, *selection, pos, object )?;
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
    variants : &[ crate::scene_model::source::Variant ],
    selection : VariantSelection,
    pos : ( i32, i32 ),
    object : &Object,
  ) -> Result< usize, CompileError >
  {
    match selection
    {
      VariantSelection::HashCoord =>
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
        let hash = u64::from( hash_coord( pos.0, pos.1, 0 ) );
        let mut target = hash % total;
        for ( i, v ) in variants.iter().enumerate()
        {
          let w = u64::from( v.weight );
          if target < w { return Ok( i ); }
          target -= w;
        }
        Ok( variants.len() - 1 )    // numerical fallback — shouldn't be reached
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
        // `Random` requires a scene-level seed, which Slice 2 doesn't ship yet.
        // Treat as unsupported; callers use `HashCoord` (default) instead.
        Err( CompileError::UnsupportedSource
        {
          object : object.id.clone(),
          source : "Variant { selection: Random } (scene seed not wired yet)",
        })
      },
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
  own use compile_frame;
}
