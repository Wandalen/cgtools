//! `compile_assets` — turn a scene-model `RenderSpec` into backend-ready assets.
//!
//! Walks the spec's asset and object trees to produce:
//!
//! - One [`crate::assets::ImageAsset`] per declared `Asset` (images only for Slice 1 —
//!   geometries, gradients, patterns, etc. are untouched).
//! - One [`crate::assets::SpriteAsset`] per **unique** `SpriteRef(asset_id, frame_name)`
//!   appearing anywhere in an object's layer stack.
//! - An [`crate::scene_model::compile::ids::IdMap`] mapping string ids to the
//!   allocated numeric handles, so `compile_frame` can resolve draw calls.

mod private
{
  use crate::assets::{ Assets, ImageAsset, SpriteAsset };
  use crate::scene_model::compile::error::CompileError;
  use crate::scene_model::compile::ids::IdMap;
  use crate::scene_model::compile::neighbors::dir_name;
  use crate::scene_model::compile::resolver::AssetResolver;
  use crate::scene_model::resource::{ AnimationTiming, AssetKind, SpriteRef };
  use crate::scene_model::source::{ NeighborBitmaskSource, SpriteSource, Variant };
  use crate::scene_model::spec::RenderSpec;

  /// Output of [`compile_assets`]: backend-ready assets plus the id map.
  #[ derive( Debug ) ]
  pub struct CompiledAssets
  {
    /// Assets ready to pass to `Backend::load_assets`.
    pub assets : Assets,
    /// Allocated resource ids for use during frame compilation.
    pub ids : IdMap,
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
      for layers in object.animations.values()
      {
        for layer in layers
        {
          collect_sprite_refs( &layer.sprite_source, spec, &mut ids, &mut sprites )?;
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
      SpriteSource::ViewportTiled { content, .. } =>
      {
        collect_sprite_refs( content, spec, ids, sprites )
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
      // External / EdgeConnectedBitmask — not supported in this slice;
      // their sprites (if any) will be pre-allocated when they land.
      _ => Ok( () ),
    }
  }

  /// Expand every frame declared inside a top-level [`crate::scene_model::resource::Animation`]
  /// into `SpriteAsset`s.
  fn collect_animation_refs
  (
    anim : &crate::scene_model::resource::Animation,
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
      AssetKind::Atlas { tile_size, columns, frames } =>
      {
        let tw = tile_size.0 as f32;
        let th = tile_size.1 as f32;

        if let Some( &( col, row ) ) = frames.get( frame_name )
        {
          return Ok( [ col as f32 * tw, row as f32 * th, tw, th ] );
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
          return Ok( [ col as f32 * tw, row as f32 * th, tw, th ] );
        }

        Err( CompileError::InvalidFrameName
        {
          asset : asset_id.to_owned(),
          frame : frame_name.to_owned(),
        })
      },
      AssetKind::Single =>
        Err( CompileError::UnsupportedAssetKind
        {
          asset : asset_id.to_owned(),
          kind : "Single",
        }),
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
  own use CompiledAssets;
  own use compile_assets;
}
