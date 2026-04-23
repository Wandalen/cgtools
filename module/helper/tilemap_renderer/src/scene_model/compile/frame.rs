//! `compile_frame` — scene-model `Scene` → per-frame `RenderCommand` stream.
//!
//! Slice 1 behaviour: emit one [`crate::commands::Sprite`] per tile per matching
//! layer, in pipeline-bucket order. Animations, variants, masks, neighbour-
//! dependent sources, non-Hex anchors, and entities are all rejected with
//! descriptive `CompileError` variants.

mod private
{
  use crate::commands::{ Clear, RenderCommand, Sprite };
  use crate::scene_model::anchor::Anchor;
  use crate::scene_model::compile::animation::resolve_animation_frame;
  use crate::scene_model::compile::assets::CompiledAssets;
  use crate::scene_model::compile::camera::Camera;
  use crate::scene_model::compile::coords::{ hex_to_world_pixel_flat, hex_to_world_pixel_pointy };
  use crate::scene_model::compile::error::CompileError;
  use crate::scene_model::hash::hash_coord;
  use crate::scene_model::layer::ObjectLayer;
  use crate::scene_model::object::Object;
  use crate::scene_model::pipeline::{ SortMode, TilingStrategy };
  use crate::scene_model::resource::SpriteRef;
  use crate::scene_model::scene::{ Scene, Tile };
  use crate::scene_model::source::{ SpriteSource, VariantSelection };
  use crate::scene_model::spec::RenderSpec;
  use crate::types::{ BlendMode, FillRef as _FillRef, Transform };

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

    // 4. Per-bucket emission.
    let tiling = spec.pipeline.hex.tiling;
    let cell_size = spec.pipeline.hex.cell_size;
    for bucket in &spec.pipeline.layers
    {
      let mut draws : Vec< ( f32, f32, Sprite ) > = Vec::new();

      for tile in tiles
      {
        for object_id in &tile.objects
        {
          let object = find_object( spec, object_id )?;

          // Reject anchors outside Slice 1.
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

            let draw = compile_layer
            (
              object, layer, tile, compiled, camera, tiling, cell_size,
              spec, time_seconds,
            )?;
            draws.push( draw );
          }
        }
      }

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
  fn compile_layer
  (
    object : &Object,
    layer : &ObjectLayer,
    tile : &Tile,
    compiled : &CompiledAssets,
    camera : &Camera,
    tiling : TilingStrategy,
    cell_size : ( u32, u32 ),
    spec : &RenderSpec,
    time_seconds : f32,
  ) -> Result< ( f32, f32, Sprite ), CompileError >
  {
    let sprite_ref = resolve_sprite_source
    (
      &layer.sprite_source,
      object,
      spec,
      tile.pos,
      time_seconds,
    )?;

    let sprite_id = compiled.ids.sprite( &sprite_ref.0, &sprite_ref.1 )
      .ok_or_else( || CompileError::UnresolvedRef
      {
        kind : "sprite",
        id : format!( "{}:{}", sprite_ref.0, sprite_ref.1 ),
        context : format!( "object {:?} layer", object.id ),
      })?;

    let ( q, r ) = tile.pos;
    let ( wx, wy ) = match tiling
    {
      TilingStrategy::HexFlatTop    => hex_to_world_pixel_flat( q, r, cell_size ),
      TilingStrategy::HexPointyTop  => hex_to_world_pixel_pointy( q, r, cell_size ),
      TilingStrategy::Square4 | TilingStrategy::Square8 =>
        return Err( CompileError::UnsupportedAnchor
        {
          object : object.id.clone(),
          anchor : "Square (tiling strategy not implemented)",
        }),
    };
    let ( sx, sy ) = camera.project( ( wx, wy ) );

    let transform = Transform
    {
      position : [ sx, sy ],
      rotation : 0.0,
      scale : [ camera.zoom, camera.zoom ],
      skew : [ 0.0, 0.0 ],
      depth : 0.0,
    };
    let _ = _FillRef::default;

    Ok(
    (
      wx,
      wy,
      Sprite
      {
        transform,
        sprite : sprite_id,
        tint : [ 1.0, 1.0, 1.0, 1.0 ],
        blend : BlendMode::default(),
        clip : None,
      },
    ))
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
  /// Dispatches over the leaf-source shapes Slice 2 handles:
  /// `Static`, `Animation`, and `Variant`. Composite neighbour-dependent
  /// sources and `External` still return `UnsupportedSource` — those land
  /// in later slices.
  fn resolve_sprite_source
  (
    source : &SpriteSource,
    object : &Object,
    spec : &RenderSpec,
    pos : ( i32, i32 ),
    time_seconds : f32,
  ) -> Result< SpriteRef, CompileError >
  {
    match source
    {
      SpriteSource::Static( sprite_ref ) => Ok( sprite_ref.clone() ),
      SpriteSource::Animation( anim_ref ) =>
      {
        let anim = spec.animations.iter().find( | a | a.id == anim_ref.0 )
          .ok_or_else( || CompileError::UnresolvedRef
          {
            kind : "animation",
            id : anim_ref.0.clone(),
            context : format!( "object {:?} layer sprite_source", object.id ),
          })?;
        resolve_animation_frame( anim, time_seconds, pos )
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
        resolve_sprite_source( &variants[ chosen ].sprite, object, spec, pos, time_seconds )
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
