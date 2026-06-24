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
  use crate::layer::{ LayerBehaviour, ObjectLayer, TintBehaviour };
  use crate::object::Object;
  use crate::pipeline::{ SortMode, TilingStrategy };
  use crate::resource::{ SpriteRef, TintRef };
  use crate::compile::viewport::{ tiled_positions, viewport_transform };
  use crate::instance::{ Instance, Placement };
  use crate::scene::Scene;
  use crate::snapshot::{ EdgeInstance, EdgePosition, Tile };
  use crate::source::{ NeighborBitmaskSource, SpriteSource, VariantSelection, ViewportTiling };
  use crate::spec::RenderSpec;
  use tilemap_renderer::types::{ asset, BlendMode, ResourceId, Transform };
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

  /// Discrete dual-grid orientation index for a triangle, in `orient_to_grid`
  /// mode. The regular hex grid's dual triangles occur in six 60°-orientations,
  /// each pre-baked as its own frame; this picks which one to draw.
  ///
  /// We align the *distinguishing* corner to its baked reference axis, then round
  /// the residual to the nearest 60° step. The baker lays the sorted corner slots
  /// at 60°/180°/300° (slot k at 60°+120°·k) and bakes orientation `o` by rotating
  /// the shape; crucially the export's PNG save flips vertically, so the baker's
  /// CCW `u_rot` reads as CLOCKWISE in world. A frame's reference corner therefore
  /// points at `base − 60°·o` in world space, index `round((base − bearing)/60°)`:
  ///   • corner tile (1 present)          → align the lone PRESENT corner,  base 60°,  6 frames
  ///   • edge tile   (2 present, 1 absent) → align the single ABSENT corner, base 300°, 6 frames
  ///   • full tile   (3 present)           → base 60°, 2 frames (▲/▽ parity only)
  ///
  /// "Present" means *this object's own id* (`self_id`, taken from its `(X,X,X)`
  /// full pattern), NOT a lexicographic property of the canonical triple. That
  /// distinction matters once a triangle holds two DIFFERENT non-void ids — e.g.
  /// two adjacent players' regions: for `region_1`'s edge tile the corners are
  /// `(region_1, region_1, region_0)`, and `"region_0" < "region_1"` sorts the
  /// foreign id FIRST, so the old canonical-order test misread the edge as a
  /// corner and pointed the petals at the neighbour's centre. Counting matches of
  /// `self_id` instead is exactly what the matched pattern meant by self vs.
  /// wildcard, so terrain (`self_id = "hexagon"`, absent = `"void"`) is unchanged
  /// while cross-region boundaries orient correctly. When `self_id` is `None`
  /// (object has no `(X,X,X)` pattern) we fall back to the canonical-order rule.
  ///
  /// NOTE: still assumes at most two distinct ids per triangle drive one object's
  /// shape (present vs. not-present). A genuine three-id chiral junction's ▲/▽
  /// mirror pair is out of scope (would need a parity-keyed reflected frame).
  fn dual_orientation_index
  (
    raw : &[ String; 3 ],
    canonical : &[ String; 3 ],
    self_id : Option< &str >,
    corner_px : &[ ( f32, f32 ); 3 ],
    wx : f32,
    wy : f32,
  ) -> u8
  {
    use core::f32::consts::FRAC_PI_3;
    let ( base, period, dist_idx ) = if let Some( sid ) = self_id
    {
      // Classify by how many corners are THIS object's own id ("present").
      let present = [ raw[ 0 ] == sid, raw[ 1 ] == sid, raw[ 2 ] == sid ];
      match present.iter().filter( | p | **p ).count()
      {
        // edge: the lone NOT-present corner is the distinguishing (void) one.
        2 =>
        {
          let idx = present.iter().position( | p | !*p ).unwrap_or( 0 );
          ( FRAC_PI_3 * 5.0, 6_i32, idx )
        }
        // corner: the lone PRESENT corner is the distinguishing one.
        1 =>
        {
          let idx = present.iter().position( | p | *p ).unwrap_or( 0 );
          ( FRAC_PI_3, 6, idx )
        }
        // full (3) — or the degenerate 0 — are 3-fold symmetric: parity only.
        _ => ( FRAC_PI_3, 2, 0 ),
      }
    }
    else
    {
      // Legacy fallback: derive the distinguishing corner from canonical order
      // (valid when the absent id sorts after the present id, e.g. literal void).
      let ( unique, base, period ) =
        if canonical[ 0 ] == canonical[ 2 ]      { ( None,                  FRAC_PI_3,       2_i32 ) }
        else if canonical[ 0 ] == canonical[ 1 ] { ( Some( &canonical[ 2 ] ), FRAC_PI_3 * 5.0, 6 ) }
        else                                     { ( Some( &canonical[ 0 ] ), FRAC_PI_3,       6 ) };
      let dist_idx = unique
        .and_then( | v | raw.iter().position( | c | c == v ) )
        .unwrap_or( 0 );
      ( base, period, dist_idx )
    };
    let ( cx, cy ) = corner_px[ dist_idx ];
    let bearing = ( cy - wy ).atan2( cx - wx );
    // `base − bearing` (not `bearing − base`): the baked frames advance
    // clockwise in world because the atlas export flips the PNG vertically.
    let steps = ( ( base - bearing ) / FRAC_PI_3 ).round() as i32;
    steps.rem_euclid( period ) as u8
  }

  /// **Resolve tier** of the dual-grid vertex pass — the expensive,
  /// camera/clock-INDEPENDENT half, run once across ALL pipeline buckets.
  ///
  /// Returns one [`ResolvedVertexSprite`] list per pipeline layer (parallel to
  /// `spec.pipeline.layers`). For each triangle it enumerates the board ONCE,
  /// computes the corner pixel centres ONCE, and then resolves every
  /// `VertexCorners` layer routed into any bucket — pattern match, orientation,
  /// frame-name → sprite id — recording the result in **world space**. None of
  /// this depends on the camera or the master clock, so the caller may cache
  /// the whole thing keyed on the scene `revision` (see [`VertexResolveCache`])
  /// and only re-run it when the board structurally changes.
  ///
  /// This is the consolidation of the former per-bucket `compile_vertex_pass`:
  /// triangle enumeration and corner-pixel projection happened once per bucket
  /// before (terrain + every player region + selection + attack overlays);
  /// now they happen once total.
  fn resolve_vertex_pass_all
  (
    tiles : &[ Tile ],
    ctx : &FrameContext< '_ >,
  ) -> Result< Vec< Vec< ResolvedVertexSprite > >, CompileError >
  {
    let pipeline_layers = &ctx.spec.pipeline.layers;

    // Per-bucket VertexCorners (object, layer) lists, parallel to the pipeline.
    // Built once; `bucket_layers[i]` feeds output bucket `i`.
    let mut bucket_layers : Vec< Vec< ( &Object, &ObjectLayer ) > > =
      pipeline_layers.iter().map( | _ | Vec::new() ).collect();
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
        if let Some( bi ) = pipeline_layers.iter().position( | b | b.id == effective )
        {
          bucket_layers[ bi ].push( ( object, layer ) );
        }
      }
    }

    let mut out : Vec< Vec< ResolvedVertexSprite > > =
      pipeline_layers.iter().map( | _ | Vec::new() ).collect();

    // No VertexCorners layers anywhere → skip the (allocating) enumeration.
    if bucket_layers.iter().all( | v | v.is_empty() )
    {
      return Ok( out );
    }

    // Any layer's object id works for the unsupported-tiling error message.
    let id_for_err : &str = bucket_layers.iter().flatten().next()
      .map_or( "", | ( o, _ ) | o.id.as_str() );

    let triangles = enumerate_triangles( tiles, ctx.tiling );

    for tri in &triangles
    {
      // Corner hex pixel centres — once per triangle, shared across every
      // bucket and layer (the cross-bucket hoist).
      let mut corner_px = [ ( 0.0_f32, 0.0_f32 ); 3 ];
      for ( i, corner ) in tri.corners.iter().enumerate()
      {
        corner_px[ i ] = hex_world_pixel( corner.0, corner.1, ctx, id_for_err )?;
      }
      let wx = ( corner_px[ 0 ].0 + corner_px[ 1 ].0 + corner_px[ 2 ].0 ) / 3.0;
      let wy = ( corner_px[ 0 ].1 + corner_px[ 1 ].1 + corner_px[ 2 ].1 ) / 3.0;

      for ( bi, layers ) in bucket_layers.iter().enumerate()
      {
        for ( object, layer ) in layers
        {
          let SpriteSource::VertexCorners { patterns, asset, orient_to_grid, corner_source, offset } = &layer.sprite_source
          else { continue };

          // Resolve corners from THIS layer's channel (terrain id by default, or
          // the named draw layer). Per-layer so independent dual grids — e.g.
          // base terrain and per-player regions — coexist on the same cells.
          let raw_corners = resolve_corners( tri, &ctx.tile_lookup, ctx.spec, corner_source.as_deref() );
          let ( canonical, rotation ) = canonicalize( raw_corners.clone() );

          let Some( pattern ) = find_matching_pattern( patterns, &canonical )
          else { continue };

          // Both modes substitute `{rot}`; only the index source differs.
          // Orient mode picks a pre-baked discrete orientation from triangle
          // geometry; legacy mode uses the canonical-sort rotation. Either way
          // `transform.rotation` stays 0 — no runtime sprite rotation.
          let rot_index = if *orient_to_grid
          {
            // The object's "self" id is the value in its all-equal (X,X,X) pattern;
            // orientation counts corners matching it to tell present from void, so a
            // neighbouring object's id (e.g. an adjacent player's region) reads as
            // void instead of being mistaken for the distinguishing corner.
            let self_id = patterns.iter().find_map( | p |
              ( p.corners.0 == p.corners.1 && p.corners.1 == p.corners.2 && p.corners.0 != "*" )
                .then_some( p.corners.0.as_str() ) );
            dual_orientation_index( &raw_corners, &canonical, self_id, &corner_px, wx, wy )
          }
          else
          {
            rotation
          };
          let frame_name = pattern.sprite_pattern.replace( "{rot}", &rot_index.to_string() );

          let sprite_id = ctx.compiled.ids.sprite( asset, &frame_name )
            .ok_or_else( || CompileError::UnresolvedRef
            {
              kind : "sprite",
              id : format!( "{asset}:{frame_name}" ),
              context : format!( "object {:?} VertexCorners frame {frame_name}", object.id ),
            })?;

          // Optional static world offset — shifts only the drawn sprite (a tinted,
          // nudged copy for a 2.5D wall / drop-shadow), not the corner/orientation
          // geometry resolved above.
          let ( ox, oy ) = offset.unwrap_or( ( 0.0, 0.0 ) );

          // Per-object tint: a `Flat` named tint colours each VertexCorners
          // object (e.g. a per-player region) independently. Stored PRE-global;
          // the scene-global tint is folded in at projection time so it does not
          // invalidate this structural cache. `None`/`Masked` → identity base.
          let base_tint = match &layer.behaviour.tint
          {
            TintBehaviour::Flat( tref ) => resolve_tint_ref( ctx.spec, tref )?,
            _ => [ 1.0, 1.0, 1.0, 1.0 ],
          };

          out[ bi ].push( ResolvedVertexSprite
          {
            world : ( wx + ox, wy + oy ),
            sprite : sprite_id,
            base_tint,
            alpha : layer.behaviour.alpha,
            blend : layer.behaviour.blend,
            pivot : object.pivot,
          });
        }
      }
    }

    Ok( out )
  }

  /// **Project tier** of the dual-grid vertex pass — the cheap, per-frame half.
  ///
  /// Turns one cached world-space [`ResolvedVertexSprite`] into a screen-space
  /// draw tuple `( sort_x, sort_y, Sprite )`. Folds the scene-global tint in
  /// here (so a global-tint change does not invalidate the resolve cache) and
  /// applies the camera (`project` + pivot + zoom). Bit-identical to the tail
  /// of the former `compile_vertex_pass`.
  fn project_vertex_sprite
  (
    rv : &ResolvedVertexSprite,
    ctx : &FrameContext< '_ >,
  ) -> ( f32, f32, Sprite )
  {
    let ( sx, sy ) = ctx.camera.project( rv.world );
    let ( sx, sy ) = apply_pivot( sx, sy, ctx.camera.zoom, rv.pivot, rv.sprite, ctx.compiled );
    let transform = make_transform( sx, sy, ctx.camera.zoom );
    let layer_tint =
    [
      ctx.global_tint[ 0 ] * rv.base_tint[ 0 ],
      ctx.global_tint[ 1 ] * rv.base_tint[ 1 ],
      ctx.global_tint[ 2 ] * rv.base_tint[ 2 ],
      ctx.global_tint[ 3 ] * rv.base_tint[ 3 ],
    ];
    (
      rv.world.0, rv.world.1,
      Sprite
      {
        transform,
        sprite : rv.sprite,
        tint : tinted( layer_tint, rv.alpha ),
        blend : rv.blend,
        clip : None,
      },
    )
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
        // No instance seed available here; `PhaseOffset::Instance`
        // falls back to 0.0 on edge sprites by design.
        resolve_animation_frame( anim, ctx.time_seconds, 0.0, canon.0, None )
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
        // threads `inst.state_entered_time` through for the correct
        // timing. No instance seed here, so `PhaseOffset::Instance`
        // falls back to 0.0.
        resolve_animation_frame( anim, ctx.time_seconds, 0.0, pos, None )
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
    /// Coverage cut-off carried from the bucket's `PipelineLayer`; the
    /// renderer copies it into every `SpriteBatchParams` it emits for this
    /// bucket. `0.0` disables the discard.
    pub alpha_clip : f32,
    /// Single-coverage depth flag carried from the bucket's `PipelineLayer`
    /// (see `PipelineLayer::occlude_overlap`).
    pub occlude_overlap : bool,
  }

  /// One `VertexCorners` triangle sprite resolved in **world space**.
  ///
  /// Everything here is a pure function of the scene's *structure* (which
  /// tiles exist and who owns them) and the spec — it is **independent of
  /// the camera and of the master clock**. That is the whole point: the
  /// expensive part of the dual-grid vertex pass (triangle enumeration +
  /// per-triangle corner resolution + pattern matching + frame-name string
  /// building) only needs to re-run when the scene's `revision` changes, not
  /// every frame the animation clock ticks. A cache keyed on `revision`
  /// (see [`VertexResolveCache`]) holds these; each frame they are cheaply
  /// re-projected to screen `Sprite`s by [`project_vertex_sprite`].
  ///
  /// `base_tint` is the layer's OWN resolved tint (`[1;4]` when the layer has
  /// no `Flat` tint); the scene-global tint is folded in at projection time so
  /// a global-tint change does not invalidate the structural cache.
  #[ derive( Clone ) ]
  pub struct ResolvedVertexSprite
  {
    world : ( f32, f32 ),
    sprite : ResourceId< asset::Sprite >,
    base_tint : [ f32; 4 ],
    alpha : f32,
    blend : BlendMode,
    pivot : ( f32, f32 ),
  }

  /// Revision-keyed memo of the dual-grid vertex pass.
  ///
  /// `buckets[i]` holds the resolved triangle sprites routing into pipeline
  /// layer `i` (parallel to `spec.pipeline.layers`). Valid as long as
  /// `revision` equals the scene's current `revision()`; any structural
  /// mutation (spawn / despawn / move / owner change) bumps the scene
  /// revision and forces a rebuild, while a pure clock tick (animation
  /// advance) does not — so an idle, animating board reuses this every frame.
  pub struct VertexResolveCache
  {
    revision : u64,
    valid : bool,
    buckets : Vec< Vec< ResolvedVertexSprite > >,
  }

  impl VertexResolveCache
  {
    /// A fresh, empty cache that misses on its first use.
    #[ must_use ]
    pub fn new() -> Self
    {
      Self { revision : 0, valid : false, buckets : Vec::new() }
    }
  }

  impl Default for VertexResolveCache
  {
    fn default() -> Self { Self::new() }
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
  pub fn gather_frame_emits
  (
    compiled : &CompiledAssets,
    scene : &Scene,
    camera : &Camera,
    vcache : Option< &mut VertexResolveCache >,
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

    // Vertex pass — resolve the camera/clock-independent structural half once,
    // reusing the cached result while the scene `revision` is unchanged. On a
    // miss (or with no cache) it is recomputed; either way the cheap per-frame
    // projection happens below, per bucket.
    let revision = scene.revision();
    let resolved : &[ Vec< ResolvedVertexSprite > ];
    let local_resolved;
    match vcache
    {
      Some( cache ) =>
      {
        if !cache.valid || cache.revision != revision
        {
          cache.buckets = resolve_vertex_pass_all( &synthetic_tiles, &ctx )?;
          cache.revision = revision;
          cache.valid = true;
        }
        resolved = &cache.buckets;
      }
      None =>
      {
        local_resolved = resolve_vertex_pass_all( &synthetic_tiles, &ctx )?;
        resolved = &local_resolved;
      }
    }

    let mut buckets = Vec::with_capacity( spec.pipeline.layers.len() );

    for ( bucket_idx, bucket ) in spec.pipeline.layers.iter().enumerate()
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

      // Vertex pass: project this bucket's cached (revision-stable) resolves —
      // cheap per-frame work; the expensive resolve already ran (or was reused)
      // above. Same triangle-outer / layer-inner order as before, so sort and
      // batch grouping are unchanged.
      draws.extend( resolved[ bucket_idx ].iter().map( | rv | project_vertex_sprite( rv, &ctx ) ) );
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

      buckets.push( BucketEmits
      {
        sprites,
        screen_space,
        sort : bucket.sort,
        alpha_clip : bucket.alpha_clip,
        occlude_overlap : bucket.occlude_overlap,
      });
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
    let emits = gather_frame_emits( compiled, scene, camera, None )?;
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

  /// Resolve a named [`TintRef`] to a strength-blended multiplier `[r,g,b,a]`.
  ///
  /// `strength` interpolates the parsed colour towards identity `[1,1,1,1]`, so
  /// the result is ready to multiply straight into a `Sprite.tint`.
  fn resolve_tint_ref( spec : &RenderSpec, tint_ref : &TintRef ) -> Result< [ f32; 4 ], CompileError >
  {
    let id = &tint_ref.0;
    let tint = spec.tints.iter().find( | t | &t.id == id )
      .ok_or_else( || CompileError::UnresolvedRef
      {
        kind : "tint",
        id : id.clone(),
        context : "tint reference".into(),
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

  /// Resolve the effective global tint, honouring `Scene`'s runtime override.
  fn resolve_scene_global_tint( spec : &RenderSpec, scene : &Scene ) -> Result< [ f32; 4 ], CompileError >
  {
    let tint_ref = scene.global_tint().cloned().or_else( || spec.pipeline.global_tint.clone() );
    let Some( tint_ref ) = tint_ref else { return Ok( [ 1.0, 1.0, 1.0, 1.0 ] ); };
    resolve_tint_ref( spec, &tint_ref )
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
            resolve_animation_frame( &anim_clone, ctx.time_seconds, oneshot_origin, pos, instance_seed )
          },
          None => resolve_animation_frame( anim, ctx.time_seconds, oneshot_origin, pos, instance_seed ),
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
          &variants[ chosen ].sprite, object, pos, phase_override, oneshot_origin, instance_seed, ctx,
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
  own use VertexResolveCache;
}
