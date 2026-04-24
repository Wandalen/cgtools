//! `compile_assets` — turn a scene-model `RenderSpec` into backend-ready assets.
//!
//! Walks the spec's asset and object trees to produce:
//!
//! - One [`tilemap_renderer::assets::ImageAsset`] per declared `Asset` (images only for Slice 1 —
//!   geometries, gradients, patterns, etc. are untouched).
//! - One [`tilemap_renderer::assets::SpriteAsset`] per **unique** `SpriteRef(asset_id, frame_name)`
//!   appearing anywhere in an object's layer stack.
//! - An [`crate::compile::ids::IdMap`] mapping string ids to the
//!   allocated numeric handles, so `compile_frame` can resolve draw calls.

mod private
{
  use tilemap_renderer::assets::{ Assets, ImageAsset, SpriteAsset };
  use crate::compile::error::CompileError;
  use crate::compile::ids::IdMap;
  use crate::compile::neighbors::dir_name;
  use crate::compile::resolver::AssetResolver;
  use crate::resource::{ AnimationTiming, AssetKind, SpriteRef };
  use crate::source::{ NeighborBitmaskSource, SpriteSource, Variant };
  use crate::spec::RenderSpec;
  use tilemap_renderer::types::{ asset, ResourceId };
  use rustc_hash::FxHashMap as HashMap;

  /// Output of [`compile_assets`]: backend-ready assets plus the id map.
  #[ derive( Debug ) ]
  pub struct CompiledAssets
  {
    /// Assets ready to pass to `Backend::load_assets`.
    pub assets : Assets,
    /// Allocated resource ids for use during frame compilation.
    pub ids : IdMap,
    /// Per-sprite pixel anchor point (relative to the sprite's rect
    /// top-left), populated from `FrameSpec::anchor` entries in the spec.
    /// When present, the compile-frame layer uses this point as the
    /// sprite's scene-anchor pivot instead of the owning object's
    /// normalized `pivot`.
    pub sprite_anchors : HashMap< ResourceId< asset::Sprite >, [ f32; 2 ] >,
  }

  /// Turn a `RenderSpec` into backend-ready assets.
  ///
  /// # Errors
  ///
  /// - [`CompileError::AssetResolution`] if the resolver fails for any asset.
  /// - [`CompileError::UnsupportedAssetKind`] if an asset uses `Single` /
  ///   `SpriteSheet` (Slice 1 supports `Atlas` only).
  /// - [`CompileError::UnresolvedRef`] if a sprite reference points at a
  ///   missing asset id.
  /// - [`CompileError::InvalidFrameName`] / [`CompileError::OutOfRange`] if a
  ///   sprite reference names a frame that can't be resolved to a region.
  pub fn compile_assets
  (
    spec : &RenderSpec,
    resolver : &impl AssetResolver,
  ) -> Result< CompiledAssets, CompileError >
  {
    let mut ids = IdMap::new();
    let mut images = Vec::with_capacity( spec.assets.len() );

    // Pass 1 — images.
    for asset in &spec.assets
    {
      let source = resolver.resolve( &asset.id, &asset.path, &asset.kind )?;
      let id = ids.alloc_image( &asset.id );
      images.push( ImageAsset
      {
        id,
        source,
        filter : asset.filter,
        mipmap : asset.mipmap,
        wrap : asset.wrap,
      });
    }

    // Pass 2 — sprites.
    //
    // Walk every object layer's sprite_source looking for `SpriteRef`s, AND
    // pre-expand every declared animation's frame list. Both paths converge
    // on `ensure_sprite_allocated`, which deduplicates by `(asset, frame)`.
    let mut sprites = Vec::new();
    for anim in &spec.animations
    {
      collect_animation_refs( anim, spec, &mut ids, &mut sprites )?;
    }
    for object in &spec.objects
    {
      for layers in object.states.values()
      {
        for layer in layers
        {
          collect_sprite_refs( &layer.sprite_source, spec, &mut ids, &mut sprites )?;
        }
      }
    }

    // Pass 3 — per-sprite pixel anchor points. Iterate every allocated
    // sprite, look up its asset + frame's `FrameSpec`, and record the
    // anchor (if any) in the output lookup keyed by ResourceId.
    let mut sprite_anchors : HashMap< ResourceId< asset::Sprite >, [ f32; 2 ] > = HashMap::default();
    for ( ( asset_id, frame_name ), sprite_id ) in &ids.sprites
    {
      let Some( asset ) = spec.assets.iter().find( | a | a.id == *asset_id ) else { continue };
      if let AssetKind::Atlas { frame_rects, .. } = &asset.kind
      {
        if let Some( frame ) = frame_rects.get( frame_name )
        {
          if let Some( ( ax, ay ) ) = frame.anchor
          {
            sprite_anchors.insert( *sprite_id, [ ax as f32, ay as f32 ] );
          }
        }
      }
    }

    Ok( CompiledAssets
    {
      assets : Assets
      {
        fonts : Vec::new(),
        images,
        sprites,
        geometries : Vec::new(),
        gradients : Vec::new(),
        patterns : Vec::new(),
        clip_masks : Vec::new(),
        paths : Vec::new(),
      },
      ids,
      sprite_anchors,
    })
  }

  /// Recursively collect `SpriteRef`s from any sprite source shape, allocating
  /// `SpriteAsset` entries on first encounter and deduplicating thereafter.
  fn collect_sprite_refs
  (
    source : &SpriteSource,
    spec : &RenderSpec,
    ids : &mut IdMap,
    sprites : &mut Vec< SpriteAsset >,
  ) -> Result< (), CompileError >
  {
    match source
    {
      SpriteSource::Static( sprite_ref ) =>
      {
        ensure_sprite_allocated( sprite_ref, spec, ids, sprites )
      },
      SpriteSource::Variant { variants, .. } =>
      {
        for Variant { sprite, .. } in variants
        {
          collect_sprite_refs( sprite, spec, ids, sprites )?;
        }
        Ok( () )
      },
      // Animation: the frames live on the referenced resource (`spec.animations`),
      // which we pre-expanded in pass 2 above. Nothing to do here.
      SpriteSource::Animation( _ ) => Ok( () ),
      SpriteSource::NeighborBitmask { source, .. } =>
      {
        match source
        {
          NeighborBitmaskSource::ByMapping { mapping, fallback } =>
          {
            for value in mapping.values()
            {
              collect_sprite_refs( value, spec, ids, sprites )?;
            }
            collect_sprite_refs( fallback, spec, ids, sprites )?;
          },
          NeighborBitmaskSource::ByAtlas { asset, layout : _ } =>
          {
            // Layout is `Bitmask6` in 0.2.0 — 64 entries indexed by mask.
            // Allocate a SpriteAsset for every mask value so frame-time
            // lookup is a straight `ids.sprite(asset, &mask.to_string())`.
            for mask in 0_u32..64
            {
              let name = mask.to_string();
              let sprite_ref = SpriteRef( asset.clone(), name );
              ensure_sprite_allocated( &sprite_ref, spec, ids, sprites )?;
            }
          },
        }
        Ok( () )
      },
      SpriteSource::NeighborCondition { sides, sprite_pattern, asset, .. } =>
      {
        // Substitute `{dir}` for each declared side to pre-allocate sprites.
        for dir in sides
        {
          let frame_name = sprite_pattern.replace( "{dir}", dir_name( *dir ) );
          let sprite_ref = SpriteRef( asset.clone(), frame_name );
          ensure_sprite_allocated( &sprite_ref, spec, ids, sprites )?;
        }
        Ok( () )
      },
      SpriteSource::VertexCorners { patterns, asset } =>
      {
        // Each pattern has a `{rot}` placeholder in 0..3; allocate all three
        // rotations so the frame pass has a guaranteed lookup per triangle.
        for pattern in patterns
        {
          for rot in 0_u32..3
          {
            let frame_name = pattern.sprite_pattern.replace( "{rot}", &rot.to_string() );
            let sprite_ref = SpriteRef( asset.clone(), frame_name );
            ensure_sprite_allocated( &sprite_ref, spec, ids, sprites )?;
          }
        }
        Ok( () )
      },
      SpriteSource::EdgeConnectedBitmask { source, layout : _, .. } =>
      {
        match source
        {
          NeighborBitmaskSource::ByMapping { mapping, fallback } =>
          {
            for value in mapping.values()
            {
              collect_sprite_refs( value, spec, ids, sprites )?;
            }
            collect_sprite_refs( fallback, spec, ids, sprites )?;
          },
          NeighborBitmaskSource::ByAtlas { asset, layout : _ } =>
          {
            // `EdgeConnectedLayout::EdgeHex` uses a 4-bit mask — 16 entries.
            for mask in 0_u32..16
            {
              let sprite_ref = SpriteRef( asset.clone(), mask.to_string() );
              ensure_sprite_allocated( &sprite_ref, spec, ids, sprites )?;
            }
          },
        }
        Ok( () )
      },
      SpriteSource::ViewportTiled { content, .. } =>
      {
        collect_sprite_refs( content, spec, ids, sprites )
      },
      // External — populated at runtime; nothing to pre-allocate.
      _ => Ok( () ),
    }
  }

  /// Expand every frame declared inside a top-level [`crate::resource::Animation`]
  /// into `SpriteAsset`s.
  fn collect_animation_refs
  (
    anim : &crate::resource::Animation,
    spec : &RenderSpec,
    ids : &mut IdMap,
    sprites : &mut Vec< SpriteAsset >,
  ) -> Result< (), CompileError >
  {
    match &anim.timing
    {
      AnimationTiming::Regular { frames, .. } =>
      {
        for sprite_ref in frames
        {
          ensure_sprite_allocated( sprite_ref, spec, ids, sprites )?;
        }
      },
      AnimationTiming::Irregular { frames } =>
      {
        for frame in frames
        {
          ensure_sprite_allocated( &frame.sprite, spec, ids, sprites )?;
        }
      },
      AnimationTiming::FromSheet { asset, start_frame, count, .. } =>
      {
        for i in 0..*count
        {
          let frame_name = ( *start_frame + i ).to_string();
          let sprite_ref = SpriteRef( asset.clone(), frame_name );
          ensure_sprite_allocated( &sprite_ref, spec, ids, sprites )?;
        }
      },
    }
    Ok( () )
  }

  /// Ensures a `SpriteAsset` exists for this `SpriteRef`, allocating + computing
  /// its region if not already seen.
  fn ensure_sprite_allocated
  (
    sprite_ref : &SpriteRef,
    spec : &RenderSpec,
    ids : &mut IdMap,
    sprites : &mut Vec< SpriteAsset >,
  ) -> Result< (), CompileError >
  {
    let SpriteRef( asset_id, frame_name ) = sprite_ref;

    if ids.sprite( asset_id, frame_name ).is_some()
    {
      return Ok( () );
    }

    let sheet_id = ids.image( asset_id ).ok_or_else( || CompileError::UnresolvedRef
    {
      kind : "asset",
      id : asset_id.clone(),
      context : format!( "sprite reference to frame {frame_name:?}" ),
    })?;

    let asset = spec.assets.iter().find( | a | a.id == *asset_id ).ok_or_else( ||
    {
      // Should be impossible since ids.image resolved — but guard just in case.
      CompileError::UnresolvedRef
      {
        kind : "asset",
        id : asset_id.clone(),
        context : format!( "sprite reference to frame {frame_name:?}" ),
      }
    })?;

    let region = frame_region( asset_id, frame_name, &asset.kind )?;
    let sprite_id = ids.alloc_sprite( asset_id, frame_name );
    sprites.push( SpriteAsset { id : sprite_id, sheet : sheet_id, region } );
    Ok( () )
  }

  /// Compute the pixel region `[ x, y, w, h ]` for a `( asset, frame_name )`
  /// pair under the asset's declared layout.
  ///
  /// For `AssetKind::Atlas`, frames resolve in this order:
  /// 1. Named frame in `Atlas.frames` manifest → its explicit `( col, row )`.
  /// 2. Numeric string → `( idx % columns, idx / columns )` via grid layout.
  /// 3. Otherwise → [`CompileError::InvalidFrameName`].
  ///
  /// Named and numeric frames freely coexist: atlases authored with a mix of
  /// semantic names for terrain sides / triangle blends and raw indices for
  /// autotile masks all work without separate mechanisms.
  fn frame_region
  (
    asset_id : &str,
    frame_name : &str,
    kind : &AssetKind,
  ) -> Result< [ f32; 4 ], CompileError >
  {
    match kind
    {
      AssetKind::Atlas { tile_size, columns, origin, gap, frames, frame_rects } =>
      {
        let tw = tile_size.0 as f32;
        let th = tile_size.1 as f32;
        let ox = origin.0 as f32;
        let oy = origin.1 as f32;
        let stride_x = tw + gap.0 as f32;
        let stride_y = th + gap.1 as f32;

        if let Some( frame ) = frame_rects.get( frame_name )
        {
          let [ x, y, w, h ] = frame.rect;
          return Ok( [ x as f32, y as f32, w as f32, h as f32 ] );
        }

        if let Some( &( col, row ) ) = frames.get( frame_name )
        {
          return Ok( [ ox + col as f32 * stride_x, oy + row as f32 * stride_y, tw, th ] );
        }

        if let Ok( idx ) = frame_name.parse::< u32 >()
        {
          let cols = *columns;
          if cols == 0
          {
            return Err( CompileError::OutOfRange
            {
              owner : asset_id.to_owned(),
              index : idx,
              max : 0,
            });
          }
          let col = idx % cols;
          let row = idx / cols;
          return Ok( [ ox + col as f32 * stride_x, oy + row as f32 * stride_y, tw, th ] );
        }

        Err( CompileError::InvalidFrameName
        {
          asset : asset_id.to_owned(),
          frame : frame_name.to_owned(),
        })
      },
      AssetKind::Single { size } =>
      {
        // Whole image = one frame. Frame name is informational — authors
        // pass the asset id or an empty string; any value resolves.
        let _ = frame_name;
        Ok( [ 0.0, 0.0, size.0 as f32, size.1 as f32 ] )
      },
      AssetKind::SpriteSheet { .. } =>
        Err( CompileError::UnsupportedAssetKind
        {
          asset : asset_id.to_owned(),
          kind : "SpriteSheet",
        }),
    }
  }
}

mod_interface::mod_interface!
{
  exposed use CompiledAssets;
  exposed use compile_assets;
}
