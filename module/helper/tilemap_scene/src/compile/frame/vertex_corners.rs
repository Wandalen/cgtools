//! The dual-grid `VertexCorners` pass of [`super`], split in two tiers: a
//! resolve that depends only on which objects stand in which hex cells —
//! cached across frames by [`VertexResolveCache`], keyed on
//! [`Scene::tiles_revision`]( crate::Scene::tiles_revision ) — and a cheap per-frame projection that folds in
//! the global tint and the `AlphaPulse` / `FadeGate` effects.

use tilemap_renderer::commands::Sprite;
use tilemap_renderer::types::{ asset, BlendMode, ResourceId };
use rustc_hash::FxHashMap as HashMap;
use alloc::sync::Arc;
use crate::compile::error::CompileError;
use crate::compile::neighbors::VOID_ID;
use crate::compile::orient::dual_orientation_index;
use crate::compile::vertex::{ canonicalize_strs, matching_pattern_find_strs, triangles_enumerate, TriangleContext };
use crate::layer::ObjectLayer;
use crate::object::Object;
use crate::resource::{ EffectKind, EffectRef };
use crate::snapshot::Tile;
use crate::source::{ SpriteSource, TriBlendPattern };
use crate::spec::RenderSpec;
use super::private::{ FrameContext, bucket_skipped, hex_world_pixel, layers_in_z_order, make_transform, pivot_apply };

/// One `VertexCorners` triangle sprite resolved in **world space**.
///
/// Everything here is a pure function of the scene's *structure* (which
/// tiles exist and who owns them) and the spec — it is **independent of
/// the camera and of the master clock**. That is the whole point: the
/// expensive part of the dual-grid vertex pass (triangle enumeration +
/// per-triangle corner resolution + pattern matching + frame-name string
/// building) only needs to re-run when the scene's hex cells change, not
/// every frame the animation clock ticks. A cache keyed on
/// [`Scene::tiles_revision`]( crate::Scene::tiles_revision ) (see [`VertexResolveCache`]) holds these; each
/// frame they are cheaply re-projected to screen `Sprite`s by
/// [`project_vertex_sprite`].
///
/// `base_tint` is the layer's OWN resolved tint (`[1;4]` when the layer has
/// no `Flat` tint); the scene-global tint is folded in at projection time so
/// a global-tint change does not invalidate the structural cache.
#[ derive( Clone ) ]
pub struct ResolvedVertexSprite
{
  /// The triangle centroid in world space — the depth-sort key.
  pub( crate ) world : ( f32, f32 ),
  /// The layer's static `offset`, added to `world` only where the sprite is
  /// drawn: like every other pass, a bucket sorts by the anchor, not by the
  /// shifted drawn position.
  pub( crate ) offset : ( f32, f32 ),
  pub( crate ) sprite : ResourceId< asset::Sprite >,
  pub( crate ) base_tint : [ f32; 4 ],
  pub( crate ) alpha : f32,
  pub( crate ) blend : BlendMode,
  pub( crate ) pivot : ( f32, f32 ),
  /// Resolved `AlphaPulse` effect `( min, max, frequency_hz, restart_on_spawn )`
  /// from the layer's `behaviour.effects`, or `None`. Static per layer, so it is
  /// cached here (does not invalidate on a clock tick); [`project_vertex_sprite`]
  /// evaluates the wave per frame off `ctx.time_seconds`, so the pulse animates
  /// without re-running the structural resolve. `min`/`max` are multipliers on
  /// the base alpha (so `min: 0, max: 1` breathes from fully transparent to the
  /// layer's own alpha and back). When `restart_on_spawn`, the wave is phased
  /// off [`Self::pulse_anchor`] instead of the global clock.
  pub( crate ) alpha_pulse : Option< ( f32, f32, f32, bool ) >,
  /// Master-clock time (seconds) captured when this sprite was resolved — i.e.
  /// at the last structural rebuild, which reruns when the hex cells change. A
  /// `restart_on_spawn` pulse uses `time_seconds - pulse_anchor` as its phase,
  /// so it restarts from `min` each time the layer's content changes.
  pub( crate ) pulse_anchor : f32,
  /// Id of the layer's resolved [`EffectKind::FadeGate`] effect, or `None`.
  /// Static per layer (which `EffectRef`s a layer names doesn't change at
  /// runtime), so it is cached here; [`project_vertex_sprite`] reads the
  /// gate's live eased value via `ctx.scene.fade_value` fresh every frame —
  /// unlike `alpha_pulse`, the gate's progress isn't a pure function of
  /// `ctx.time_seconds`, it's runtime state [`Scene::tick`]( crate::Scene::tick ) advances, so it
  /// can't be captured at resolve time without going stale.
  pub( crate ) fade_gate : Option< Arc< str > >,
}

/// Revision-keyed memo of the dual-grid vertex pass.
///
/// `buckets[i]` holds the resolved triangle sprites routing into pipeline
/// layer `i` (parallel to `spec.pipeline.layers`). Valid as long as
/// `revision` equals the scene's current [`Scene::tiles_revision`]( crate::Scene::tiles_revision ) — the
/// resolve reads nothing but which visible objects stand in which hex cells,
/// so a hex-placed spawn / despawn / move / visibility flip forces a rebuild,
/// while the clock, `FreePos` movers (boats, birds, a drag preview), state and
/// tint changes do not — an animating board with sailing boats reuses it.
pub struct VertexResolveCache
{
  pub( crate ) revision : u64,
  /// The `skip` mask the buckets were resolved with — a different mask is a
  /// miss even at the same revision (a re-enabled layer was never resolved).
  pub( crate ) skip : u64,
  pub( crate ) valid : bool,
  pub( crate ) buckets : Vec< Vec< ResolvedVertexSprite > >,
  /// Structural resolves run so far (cache misses) — see [`Self::resolves`].
  pub( crate ) resolves : u64,
}

impl VertexResolveCache
{
  /// Stores `fresh`, a structural resolve made at `revision` with the `skip` mask.
  ///
  /// A `restart_on_spawn` pulse (see `project_vertex_sprite`) anchors to when its
  /// bucket's CONTENT last changed — not to every re-resolve. The resolve reruns on
  /// any hex-cell change, including ones that don't touch this bucket at all (a
  /// capture elsewhere re-resolves every layer), which must NOT restart an unrelated
  /// layer's pulse. So each bucket's prior anchor is carried forward while its
  /// resolved tiles are unchanged; only genuinely new/changed content keeps the fresh
  /// anchor the resolve gave it.
  pub( crate ) fn store( &mut self, mut fresh : Vec< Vec< ResolvedVertexSprite > >, revision : u64, skip : u64 )
  {
    if self.valid
    {
      for ( fb, ob ) in fresh.iter_mut().zip( self.buckets.iter() )
      {
        if same_vertex_content( fb, ob )
        {
          for ( fs, os ) in fb.iter_mut().zip( ob.iter() )
          {
            fs.pulse_anchor = os.pulse_anchor;
          }
        }
      }
    }
    self.buckets = fresh;
    self.revision = revision;
    self.skip = skip;
    self.valid = true;
    self.resolves += 1;
  }

  /// A fresh, empty cache that misses on its first use.
  #[ must_use ]
  pub fn new() -> Self
  {
    Self { revision : 0, skip : 0, valid : false, buckets : Vec::new(), resolves : 0 }
  }

  /// How many times the structural resolve has run through this cache (its
  /// misses). Each one walks every dual-grid triangle for every live
  /// `VertexCorners` layer, so this is the counter to watch when profiling.
  #[ inline ]
  #[ must_use ]
  pub fn resolves( &self ) -> u64 { self.resolves }
}

impl Default for VertexResolveCache
{
  fn default() -> Self { Self::new() }
}

/// **Resolve tier** of the dual-grid vertex pass — the expensive,
/// camera/clock-INDEPENDENT half, run once across ALL pipeline buckets.
///
/// Returns one [`ResolvedVertexSprite`] list per pipeline layer (parallel to
/// `spec.pipeline.layers`), resolving every `VertexCorners` layer routed into
/// a bucket — pattern match, orientation, frame-name → sprite id — and
/// recording the result in **world space**. None of this depends on the
/// camera or the master clock, so the caller may cache the whole thing keyed
/// on [`Scene::tiles_revision`]( crate::Scene::tiles_revision ) (see [`VertexResolveCache`]) and only re-run it
/// when the hex cells change.
///
/// The board's triangles are enumerated once, and each corner channel's
/// per-cell ids are resolved once. A layer then visits only the triangles
/// touching an occupied cell of its channel — every other triangle reads
/// `(void, void, void)` — unless one of its patterns matches that triple, so
/// a small overlay costs its own size rather than the board's.
///
/// Buckets whose bit is set in `skip` (bit `i` = pipeline layer `i`) are left
/// empty: a renderer that will not draw a layer should not pay to resolve it.
/// Each layer's resolve reads only the scene tiles (its corner channel), never
/// another layer's output, so skipping one cannot change the others.
pub( crate ) fn resolve_vertex_pass_all
(
  tiles : &[ Tile ],
  ctx : &FrameContext< '_ >,
  skip : u64,
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
    for layer in layers_in_z_order( stack )
    {
      if !matches!( layer.sprite_source, SpriteSource::VertexCorners { .. } )
      {
        continue;
      }
      let effective = layer.pipeline_layer.as_deref().unwrap_or( object.global_layer.as_str() );
      if let Some( bi ) = pipeline_layers.iter().position( | b | b.id == effective )
        && !bucket_skipped( skip, bi )
      {
        bucket_layers[ bi ].push( ( object, layer ) );
      }
    }
  }

  let mut out : Vec< Vec< ResolvedVertexSprite > > =
    pipeline_layers.iter().map( | _ | Vec::new() ).collect();

  // No VertexCorners layers anywhere → skip the (allocating) enumeration.
  if bucket_layers.iter().all( Vec::is_empty )
  {
    return Ok( out );
  }

  // Objects by id, first declaration winning — the same answer as the linear
  // scan `tile_corner_id` does per lookup, in one hash probe.
  let mut object_of : HashMap< &str, &Object > = HashMap::default();
  for object in &ctx.spec.objects
  {
    object_of.entry( object.id.as_str() ).or_insert( object );
  }

  // Every distinct corner channel's per-cell id, resolved once here rather than
  // three times per triangle per layer.
  let mut channels : Vec< ( Option< &str >, ChannelCells< '_ > ) > = Vec::new();
  for ( _, layer ) in bucket_layers.iter().flatten()
  {
    let SpriteSource::VertexCorners { corner_source, .. } = &layer.sprite_source
    else { continue };
    let source = corner_source.as_deref();
    if channels.iter().all( | ( s, _ ) | *s != source )
    {
      channels.push( ( source, channel_cells( tiles, &object_of, source ) ) );
    }
  }

  let triangles = triangles_enumerate( tiles, ctx.tiling );
  let mut scratch = ResolveScratch
  {
    geometry : vec![ None; triangles.len() ],
    triangles,
    tris_at : None,
    frame_ids : HashMap::default(),
    // Any layer's object id works for the unsupported-tiling error message.
    id_for_err : bucket_layers.iter().flatten().next().map_or( "", | ( o, _ ) | o.id.as_str() ),
  };

  for ( bi, layers ) in bucket_layers.iter().enumerate()
  {
    // Layers are resolved one at a time, so their hits are re-sorted into the
    // triangle-major, layer-minor order a single board walk produces — the
    // bucket's stable depth sort keeps that order among sprites at one spot.
    let mut hits : Vec< LayerHit > = Vec::new();
    for ( li, &( object, layer ) ) in layers.iter().enumerate()
    {
      let SpriteSource::VertexCorners { corner_source, .. } = &layer.sprite_source
      else { continue };
      let source = corner_source.as_deref();
      let cells = &channels.iter().find( | ( s, _ ) | *s == source ).expect( "every channel is built above" ).1;
      layer_resolve( ctx, &mut scratch, cells, ( object, layer ), u32::try_from( li ).unwrap_or( u32::MAX ), &mut hits )?;
    }
    if layers.len() > 1
    {
      hits.sort_unstable_by_key( | h | ( h.0, h.1 ) );
    }
    out[ bi ] = hits.into_iter().map( | h | h.2 ).collect();
  }

  Ok( out )
}

/// Each occupied cell's id in one corner channel; a cell left out reads void.
type ChannelCells< 'a > = HashMap< ( i32, i32 ), &'a str >;

/// `( triangle index, layer position in its bucket, sprite )`.
type LayerHit = ( u32, u32, ResolvedVertexSprite );

/// A triangle's three corner pixel centres and its centroid.
type TriGeometry = ( [ ( f32, f32 ); 3 ], f32, f32 );

/// A layer's static draw inputs: base tint, alpha pulse, fade-gate id.
type LayerStatics = ( [ f32; 4 ], Option< ( f32, f32, f32, bool ) >, Option< Arc< str > > );

/// State shared by every layer of one [`resolve_vertex_pass_all`] call.
struct ResolveScratch< 'a >
{
  /// Every dual-mesh triangle touched by a tile.
  triangles : Vec< TriangleContext >,
  /// Triangles around each cell — built on first need, for layers that only
  /// visit the triangles touching their channel's occupied cells.
  tris_at : Option< HashMap< ( i32, i32 ), Vec< u32 > > >,
  /// Corner pixel centres + centroid per triangle, computed on first visit.
  geometry : Vec< Option< TriGeometry > >,
  /// `( pattern address, rotation )` → sprite id, so each frame-name string is
  /// built and looked up once per distinct frame, not once per triangle.
  frame_ids : HashMap< ( usize, u8 ), ResourceId< asset::Sprite > >,
  /// Object id reported by an unsupported-tiling error.
  id_for_err : &'a str,
}

/// Resolve one `VertexCorners` layer against its corner channel's `cells`,
/// appending a [`LayerHit`] (tagged with `layer_pos`) per matched triangle, in
/// ascending triangle order.
fn layer_resolve
(
  ctx : &FrameContext< '_ >,
  scratch : &mut ResolveScratch< '_ >,
  cells : &ChannelCells< '_ >,
  ( object, layer ) : ( &Object, &ObjectLayer ),
  layer_pos : u32,
  hits : &mut Vec< LayerHit >,
) -> Result< (), CompileError >
{
  let SpriteSource::VertexCorners { patterns, asset, orient_to_grid, offset, .. } = &layer.sprite_source
  else { return Ok( () ) };

  // A triangle with no occupied corner reads `(void, void, void)`. Unless a
  // pattern draws on exactly that, only triangles touching one of this
  // channel's cells can emit — usually a handful for an overlay.
  let candidates : Vec< u32 > = if matching_pattern_find_strs( patterns, &[ VOID_ID; 3 ] ).is_some()
  {
    ( 0..u32::try_from( scratch.triangles.len() ).unwrap_or( u32::MAX ) ).collect()
  }
  else
  {
    let triangles = &scratch.triangles;
    let around = scratch.tris_at.get_or_insert_with( || triangles_by_cell( triangles ) );
    let mut v : Vec< u32 > = cells.keys().filter_map( | c | around.get( c ) ).flatten().copied().collect();
    v.sort_unstable();
    v.dedup();
    v
  };

  // The object's "self" id is the value in its all-equal (X,X,X) pattern;
  // orientation counts corners matching it to tell present from void, so a
  // neighbouring object's id (e.g. an adjacent player's region) reads as
  // void instead of being mistaken for the distinguishing corner.
  let self_id = patterns.iter().find_map( TriBlendPattern::self_id );

  // Optional static world offset — shifts only the drawn sprite (a tinted,
  // nudged copy for a 2.5D wall / drop-shadow), not the corner/orientation
  // geometry resolved below.
  let offset = offset.unwrap_or( ( 0.0, 0.0 ) );

  // Tint, pulse and fade gate, resolved on the layer's first match — a layer
  // that draws nothing raises no tint error.
  let mut statics : Option< LayerStatics > = None;

  for ti in candidates
  {
    let corners = scratch.triangles[ ti as usize ].corners;
    let raw = corners.map( | c | cells.get( &c ).copied().unwrap_or( VOID_ID ) );
    let ( canonical, rotation ) = canonicalize_strs( &raw );
    let Some( pattern ) = matching_pattern_find_strs( patterns, &canonical )
    else { continue };

    let ( corner_px, wx, wy ) = triangle_geometry( ctx, scratch, ti )?;

    // Both modes substitute `{rot}`; only the index source differs.
    // Orient mode picks a pre-baked discrete orientation from triangle
    // geometry; legacy mode uses the canonical-sort rotation. Either way
    // `transform.rotation` stays 0 — no runtime sprite rotation.
    let rot_index = if *orient_to_grid
    {
      dual_orientation_index( &raw, self_id, &corner_px, wx, wy, ctx.tiling )
    }
    else
    {
      rotation
    };
    let frame_key = ( core::ptr::from_ref( pattern ) as usize, rot_index );
    let sprite_id = if let Some( &id ) = scratch.frame_ids.get( &frame_key )
    {
      id
    }
    else
    {
      let frame_name = pattern.sprite_pattern.replace( "{rot}", &rot_index.to_string() );
      let id = ctx.compiled.ids.sprite( asset, &frame_name )
        .ok_or_else( || CompileError::UnresolvedRef
        {
          kind : "sprite",
          id : format!( "{asset}:{frame_name}" ),
          context : format!( "object {:?} VertexCorners frame {frame_name}", object.id ),
        })?;
      scratch.frame_ids.insert( frame_key, id );
      id
    };

    if statics.is_none()
    {
      statics = Some( layer_statics( ctx, object, layer )? );
    }
    let ( base_tint, alpha_pulse, fade_gate ) = statics.clone().expect( "set just above" );

    hits.push( ( ti, layer_pos, ResolvedVertexSprite
    {
      world : ( wx, wy ),
      offset,
      sprite : sprite_id,
      base_tint,
      alpha : layer.behaviour.alpha,
      blend : layer.behaviour.blend,
      pivot : object.pivot,
      alpha_pulse,
      pulse_anchor : ctx.time_seconds,
      fade_gate,
    }));
  }
  Ok( () )
}

/// Corner pixel centres and centroid of triangle `ti`, computed on first use.
fn triangle_geometry( ctx : &FrameContext< '_ >, scratch : &mut ResolveScratch< '_ >, ti : u32 ) -> Result< TriGeometry, CompileError >
{
  if let Some( g ) = scratch.geometry[ ti as usize ]
  {
    return Ok( g );
  }
  let corners = scratch.triangles[ ti as usize ].corners;
  let mut corner_px = [ ( 0.0_f32, 0.0_f32 ); 3 ];
  for ( i, corner ) in corners.iter().enumerate()
  {
    corner_px[ i ] = hex_world_pixel( corner.0, corner.1, ctx, scratch.id_for_err )?;
  }
  let wx = ( corner_px[ 0 ].0 + corner_px[ 1 ].0 + corner_px[ 2 ].0 ) / 3.0;
  let wy = ( corner_px[ 0 ].1 + corner_px[ 1 ].1 + corner_px[ 2 ].1 ) / 3.0;
  let g = ( corner_px, wx, wy );
  scratch.geometry[ ti as usize ] = Some( g );
  Ok( g )
}

/// A `VertexCorners` layer's static draw inputs.
///
/// Per-object tint: a `Flat` named tint colours each VertexCorners object
/// (e.g. a per-player region) independently. Stored PRE-global; the
/// scene-global tint is folded in at projection time so it does not
/// invalidate the structural cache. `None` → identity base; `Masked` is not
/// implemented and errors.
fn layer_statics( ctx : &FrameContext< '_ >, object : &Object, layer : &ObjectLayer ) -> Result< LayerStatics, CompileError >
{
  Ok
  ((
    ctx.tints.layer_multiplier( object, &layer.behaviour )?,
    resolve_alpha_pulse( ctx.spec, &layer.behaviour.effects ),
    resolve_fade_gate( ctx.spec, &layer.behaviour.effects ).map( Arc::from ),
  ))
}

/// Each occupied cell's id in one `VertexCorners` corner channel — exactly
/// what [`tile_corner_id`] answers per cell (`source = None`: the first object
/// with a `priority`; `Some( layer )`: the first whose `global_layer` is
/// `layer`), with `object_of` resolving each object in one lookup. Cells with
/// no such object are left out (they read as [`VOID_ID`]).
fn channel_cells< 'a >
(
  tiles : &'a [ Tile ],
  object_of : &HashMap< &str, &Object >,
  source : Option< &str >,
) -> ChannelCells< 'a >
{
  let mut cells = HashMap::default();
  for tile in tiles
  {
    let id = tile.objects.iter().find( | id | object_of.get( id.as_str() ).is_some_and( | o | match source
    {
      None => o.priority.is_some(),
      Some( layer ) => o.global_layer == layer,
    }));
    if let Some( id ) = id
    {
      cells.insert( tile.pos, id.as_str() );
    }
  }
  cells
}

/// Indices of the triangles in `triangles` that have `cell` as a corner, by cell.
fn triangles_by_cell( triangles : &[ TriangleContext ] ) -> HashMap< ( i32, i32 ), Vec< u32 > >
{
  let mut by_cell : HashMap< ( i32, i32 ), Vec< u32 > > = HashMap::default();
  for ( i, tri ) in triangles.iter().enumerate()
  {
    let i = u32::try_from( i ).unwrap_or( u32::MAX );
    for corner in tri.corners
    {
      by_cell.entry( corner ).or_default().push( i );
    }
  }
  by_cell
}

/// Whether two resolved vertex buckets carry the same CONTENT — matching tile
/// count, sprite frames, and world positions. Used to decide if a bucket's
/// pulse anchor survives a re-resolve triggered by an unrelated scene mutation
/// (see the `restart_on_spawn` carry-forward in `gather_frame_emits`). Ignores
/// `pulse_anchor` (that is what's being decided) and the static per-layer tint /
/// alpha / blend (never mutated at runtime). Positions compared bit-exact —
/// both sides are recomputed identically from the same tile geometry.
pub( crate ) fn same_vertex_content( a : &[ ResolvedVertexSprite ], b : &[ ResolvedVertexSprite ] ) -> bool
{
  a.len() == b.len()
    && a.iter().zip( b.iter() ).all( | ( x, y ) |
      x.sprite == y.sprite
        && x.world.0.to_bits() == y.world.0.to_bits()
        && x.world.1.to_bits() == y.world.1.to_bits()
    )
}

/// **Project tier** of the dual-grid vertex pass — the cheap, per-frame half.
///
/// Turns one cached world-space [`ResolvedVertexSprite`] into a world-space
/// draw tuple `( sort_x, sort_y, Sprite )`. Folds the scene-global tint in
/// here (so a global-tint change does not invalidate the resolve cache) and
/// applies the anchor pivot. The camera is **not** applied here — the emitted
/// `Sprite` carries world coordinates and the backend projects it — so this is
/// now independent of pan/zoom (only the structural resolve, already cached,
/// and the tint remain).
pub( crate ) fn project_vertex_sprite
(
  rv : &ResolvedVertexSprite,
  ctx : &FrameContext< '_ >,
) -> ( f32, f32, Sprite )
{
  let ( sx, sy ) = pivot_apply( rv.world.0 + rv.offset.0, rv.world.1 + rv.offset.1, rv.pivot, rv.sprite, ctx.compiled );
  let transform = make_transform( sx, sy );
  // Base tint with the layer's static alpha.
  let mut tint = ctx.tints.layer_tint_compose( rv.base_tint, rv.alpha );

  // Per-frame alpha pulse (see `ResolvedVertexSprite::alpha_pulse`): a raised-
  // cosine wave in `[0, 1]`, at `min` when its phase `t = 0`, easing to `max`
  // at the half period (smooth, zero slope at both ends, no camera dependence).
  // `restart_on_spawn` phases off `pulse_anchor` (the clock at the last
  // structural resolve) so the wave starts at `min` each time the layer's
  // content changes — e.g. the attack overlay fading in from 0 the instant a
  // unit is selected; otherwise it free-runs off the global clock.
  //
  // The pulse fades the WHOLE tint — RGB *and* alpha by the same factor — not
  // the alpha alone: the sprite shaders sample premultiplied atlases
  // (`frag = tex * tint`), where a valid colour keeps `rgb ≤ a`. Scaling only
  // alpha would leave RGB over-bright as coverage drops, so the edge haloes
  // lighter than the fill (obvious on a neutral tint like the grey selection,
  // hidden under a saturated one like the red attack overlay). Scaling all four
  // channels is exactly a premultiplied fade toward transparent, so `min: 0,
  // max: 1` breathes cleanly from invisible to the layer's own alpha and back.
  if let Some( ( min, max, freq, restart_on_spawn ) ) = rv.alpha_pulse
  {
    let t = if restart_on_spawn { ( ctx.time_seconds - rv.pulse_anchor ).max( 0.0 ) } else { ctx.time_seconds };
    let wave = 0.5 - 0.5 * ( core::f32::consts::TAU * freq * t ).cos();
    let k = min + ( max - min ) * wave;
    tint = [ tint[ 0 ] * k, tint[ 1 ] * k, tint[ 2 ] * k, tint[ 3 ] * k ];
  }

  // Fade gate: same whole-tint-multiplier treatment as the pulse above, but
  // the factor is [`Scene::fade_value`]( crate::Scene::fade_value )'s live eased progress instead of an
  // oscillating wave — read fresh every frame since it isn't a function of
  // `ctx.time_seconds` alone (it's advanced by `Scene::tick`).
  if let Some( effect_id ) = &rv.fade_gate
  {
    let k = ctx.scene.fade_value( effect_id );
    tint = [ tint[ 0 ] * k, tint[ 1 ] * k, tint[ 2 ] * k, tint[ 3 ] * k ];
  }
  (
    rv.world.0, rv.world.1,
    Sprite
    {
      transform,
      sprite : rv.sprite,
      tint,
      blend : rv.blend,
      clip : None,
    },
  )
}

/// Resolve the first [`EffectKind::AlphaPulse`] among a layer's `effects` to
/// its `( min, max, frequency_hz )` params, or `None` if the layer references
/// no alpha-pulse effect. An `EffectRef` naming an absent / non-pulse effect is
/// simply skipped (effect resolution is best-effort — a missing effect must not
/// fail the whole frame compile). Cheap (a linear scan over the spec's few
/// effects), and called only from the revision-cached structural resolve.
fn resolve_alpha_pulse( spec : &RenderSpec, effects : &[ EffectRef ] ) -> Option< ( f32, f32, f32, bool ) >
{
  for eff_ref in effects
  {
    let Some( eff ) = spec.effects.iter().find( | e | e.id == eff_ref.0 ) else { continue };
    if let EffectKind::AlphaPulse { min, max, frequency, restart_on_spawn } = eff.kind
    {
      return Some( ( min, max, frequency, restart_on_spawn ) );
    }
  }
  None
}

/// Resolve the first [`EffectKind::FadeGate`] among a layer's `effects` to
/// its effect id (the key [`Scene::fade_value`]( crate::Scene::fade_value ) reads live state under), or
/// `None` if the layer references no fade-gate effect. Same best-effort /
/// cheap-scan contract as [`resolve_alpha_pulse`] — only the id is resolved
/// here (static, safe to cache); the gate's actual eased value is runtime
/// state read fresh per frame in [`project_vertex_sprite`].
fn resolve_fade_gate( spec : &RenderSpec, effects : &[ EffectRef ] ) -> Option< String >
{
  for eff_ref in effects
  {
    let Some( eff ) = spec.effects.iter().find( | e | e.id == eff_ref.0 ) else { continue };
    if matches!( eff.kind, EffectKind::FadeGate { .. } )
    {
      return Some( eff_ref.0.clone() );
    }
  }
  None
}
