//! Opaque/transparent two-pass split and single-coverage buckets — see
//! `docs/format/007_render_pipeline.md` (Coverage and Depth Controls).
//!
//! When any `PipelineLayer` sets `opaque: true` the renderer splits the frame:
//! opaque buckets draw first, front-to-back (topmost pipeline layer first) with
//! depth writes on; transparent buckets draw after in painter's order with depth
//! writes off — except an `occlude_overlap` bucket, which keeps writes on for its
//! own draws. Depth-write toggling rides the `SetDepthWrite` render command, and
//! each bucket's sprites are pinned to `pipeline_index / bucket_count` whenever
//! any layer is opaque or `occlude_overlap`. With no opaque layer the renderer
//! takes the single-pass path and emits no `SetDepthWrite` at all, and so does
//! a renderer told the backend has no depth buffer, whatever the flags.

#![ allow( clippy::min_ident_chars, reason = "one-letter closure bindings ( `c`, `s` ) over render commands keep the command filters on one line" ) ]
#![ allow
(
  clippy::default_trait_access,
  clippy::too_many_lines,
  clippy::float_cmp,
  reason = "spec fixtures fill unused enum fields with `Default::default()`, build one linear spec per test, and assert exact pinned depths with no arithmetic drift"
) ]

extern crate alloc;
use alloc::sync::Arc;
use rustc_hash::FxHashMap as HashMap;

use tilemap_renderer::commands::RenderCommand;
use tilemap_scene::
{
  Anchor,
  Asset,
  AssetKind,
  Camera,
  HexConfig,
  LayerBehaviour,
  Object,
  ObjectLayer,
  PathResolver,
  PipelineLayer,
  Placement,
  Renderer,
  RenderPipeline,
  RenderSpec,
  Scene,
  SortMode,
  SpriteRef,
  SpriteSource,
  TilingStrategy,
};
use tilemap_renderer::types::{ Batch, MipmapMode, ResourceId, SamplerFilter, WrapMode };

mod common;

// ────────────────────────────────────────────────────────────────────────────
// Fixtures — three objects, one per pipeline bucket (`bg`, `mid`, `top`), all
// drawn from one shared atlas. Per-layer `opaque` / `occlude_overlap` flags are
// supplied by each test so a spec can be built with exactly the pass behaviour
// under test.
// ────────────────────────────────────────────────────────────────────────────

fn static_layer( asset : &str, frame : &str ) -> ObjectLayer
{
  ObjectLayer
  {
    id : None,
    sprite_source : SpriteSource::Static( SpriteRef { asset : asset.into(), frame : frame.into() } ),
    behaviour : LayerBehaviour::default(),
    z_in_object : 0,
    pipeline_layer : None,
  }
}

fn atlas_asset( id : &str ) -> Asset
{
  Asset
  {
    id : id.into(),
    path : format!( "{id}.png" ),
    kind : AssetKind::Atlas
    {
      tile_size : ( 72, 64 ),
      columns : 4,
      origin : ( 0, 0 ),
      gap : ( 0, 0 ),
      frames : HashMap::default(),
      frame_rects : HashMap::default(),
      image_size : None,
    },
    filter : SamplerFilter::Linear,
    mipmap : MipmapMode::Off,
    wrap : WrapMode::Clamp,
    premultiplied : false,
  }
}

fn object( id : &str, frame : &str, bucket : &str ) -> Object
{
  object_on( id, "sheet", frame, bucket )
}

fn object_on( id : &str, asset : &str, frame : &str, bucket : &str ) -> Object
{
  let mut states = HashMap::default();
  states.insert( "default".into(), vec![ static_layer( asset, frame ) ] );
  Object
  {
    id : id.into(),
    anchor : Anchor::Hex,
    global_layer : bucket.into(),
    priority : None,
    sort_y_source : Default::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states,
  }
}

fn layer( id : &str, opaque : bool, occlude_overlap : bool ) -> PipelineLayer
{
  PipelineLayer
  {
    id : id.into(),
    sort : SortMode::None,
    tint_mask : None,
    alpha_clip : if opaque { 0.5 } else { 0.0 },
    occlude_overlap,
    opaque,
  }
}

/// Build a three-bucket spec. `flags[i] = (opaque, occlude_overlap)` for bucket
/// `i` in declared order: `bg` (index 0), `mid` (1), `top` (2).
fn build_spec( flags : [ ( bool, bool ); 3 ] ) -> RenderSpec
{
  RenderSpec
  {
    version : "0.2.0".into(),
    assets : vec![ atlas_asset( "sheet" ) ],
    tints : Vec::new(),
    animations : Vec::new(),
    effects : Vec::new(),
    objects : vec!
    [
      object( "bg", "0", "bg" ),
      object( "mid", "1", "mid" ),
      object( "top", "2", "top" ),
    ],
    pipeline : RenderPipeline
    {
      hex : HexConfig { tiling : TilingStrategy::HexFlatTop, grid_stride : ( 72, 64 ) },
      layers : vec!
      [
        layer( "bg", flags[ 0 ].0, flags[ 0 ].1 ),
        layer( "mid", flags[ 1 ].0, flags[ 1 ].1 ),
        layer( "top", flags[ 2 ].0, flags[ 2 ].1 ),
      ],
      global_tint : None,
      viewport_size : None,
      clear_color : None,
    },
  }
}

fn spawn_one_per_bucket( scene : &mut Scene )
{
  for ( id, r ) in [ ( "bg", 0 ), ( "mid", 1 ), ( "top", 2 ) ]
  {
    let obj = scene.object( id ).unwrap();
    scene.spawn( obj, Placement::Hex { q : 0, r } );
  }
}

/// Depth-write toggles in command order — `true`/`false` per `SetDepthWrite`.
fn depth_write_sequence( cmds : &[ RenderCommand ] ) -> Vec< bool >
{
  cmds.iter()
    .filter_map( | c | if let RenderCommand::SetDepthWrite( s ) = c { Some( s.enabled ) } else { None } )
    .collect()
}

// ────────────────────────────────────────────────────────────────────────────
// 1. With opaque layers present, the frame is bracketed by `SetDepthWrite`:
//    enable for the opaque pass, disable for the transparent pass, restore at
//    the end. Opaque buckets are pinned front-to-back (topmost layer nearest).
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn opaque_layers_bracket_passes_and_pin_depth_front_to_back()
{
  // bg + mid opaque, top transparent.
  let spec = build_spec( [ ( true, false ), ( true, false ), ( false, false ) ] );
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  spawn_one_per_bucket( &mut scene );

  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" ).to_vec();

  // Opaque pass turns writes on, transparent pass turns them off, then the
  // renderer restores them to true for the next frame's opaque pass.
  assert_eq!
  (
    depth_write_sequence( &cmds ), vec![ true, false, true ],
    "expected enable (opaque) → disable (transparent) → restore",
  );

  // Three buckets, one sprite each. Opaque pass is front-to-back: the topmost
  // opaque layer (`mid`, index 1) draws before the lower one (`bg`, index 0);
  // the transparent `top` (index 2) draws last.
  let sprites = common::flat_sprites( &cmds );
  assert_eq!( sprites.len(), 3, "one sprite per bucket" );

  let depths : Vec< f32 > = sprites.iter().map( | s | s.transform.depth ).collect();
  assert_eq!
  (
    depths, vec![ 1.0 / 3.0, 0.0, 2.0 / 3.0 ],
    "opaque pass front-to-back (mid before bg), transparent `top` last; \
     each depth is pipeline_index / bucket_count",
  );
}

// ────────────────────────────────────────────────────────────────────────────
// 2. No opaque layer → original single-pass path, no `SetDepthWrite` emitted
//    and no depth pinning (every sprite keeps the default depth 0).
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn no_opaque_layer_takes_single_pass_path()
{
  let spec = build_spec( [ ( false, false ), ( false, false ), ( false, false ) ] );
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  spawn_one_per_bucket( &mut scene );

  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" ).to_vec();

  assert!
  (
    depth_write_sequence( &cmds ).is_empty(),
    "single-pass path must not emit any SetDepthWrite",
  );

  // Painter's order, no depth pinning.
  let depths : Vec< f32 > = common::flat_sprites( &cmds ).iter().map( | s | s.transform.depth ).collect();
  assert_eq!( depths, vec![ 0.0, 0.0, 0.0 ], "no opaque pass ⇒ depths untouched" );
}

// ────────────────────────────────────────────────────────────────────────────
// 2b. A backend without a depth buffer (`Renderer::depth_buffer_set( false )`)
//     gets the single painter's pass even from a spec with opaque and
//     `occlude_overlap` layers: no `SetDepthWrite`, depth 0, declared order.
// ────────────────────────────────────────────────────────────────────────────

/// Without a depth buffer, opaque layers drawn topmost first would paint the
/// lower layers over the upper ones. So `depth_buffer_set( false )` must give
/// the frame a spec with no flags would get, and switching it on a renderer
/// that already drew must not replay the two-pass frame from the idle cache.
#[ test ]
fn without_a_depth_buffer_flagged_layers_take_the_single_pass()
{
  let flagged = build_spec( [ ( true, false ), ( true, false ), ( false, true ) ] );
  let mut renderer = Renderer::new( &flagged, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( flagged ) );
  spawn_one_per_bucket( &mut scene );
  // The second frame draws batches the first one created, so flatten both
  // through one stateful flattener.
  let mut flat = common::BatchFlattener::new();
  let two_pass = renderer.render( &scene, &Camera::default() ).expect( "render" ).to_vec();
  assert!( !depth_write_sequence( &two_pass ).is_empty(), "with a depth buffer the flags split the frame" );
  flat.apply( &two_pass );

  renderer.depth_buffer_set( false );
  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" ).to_vec();
  assert!( depth_write_sequence( &cmds ).is_empty(), "no depth buffer ⇒ no SetDepthWrite" );
  let sprites : Vec< _ > = flat.apply( &cmds ).into_iter()
    .filter_map( | c | if let RenderCommand::Sprite( s ) = c { Some( s ) } else { None } )
    .collect();

  let plain = build_spec( [ ( false, false ), ( false, false ), ( false, false ) ] );
  let mut plain_renderer = Renderer::new( &plain, &PathResolver ).expect( "renderer" );
  let mut plain_scene = Scene::new( Arc::new( plain ) );
  spawn_one_per_bucket( &mut plain_scene );
  let expected = plain_renderer.render( &plain_scene, &Camera::default() ).expect( "render" ).to_vec();
  assert_eq!
  (
    format!( "{sprites:?}" ),
    format!( "{:?}", common::flat_sprites( &expected ) ),
    "the same sprites, at depth 0, in declared order",
  );
}

// ────────────────────────────────────────────────────────────────────────────
// 3. An `occlude_overlap` transparent bucket keeps depth writes ON for its own
//    draws while the surrounding transparent pass has them off. Walk the
//    flattened stream tracking the live depth-write state at each sprite.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn occlude_overlap_keeps_depth_writes_in_transparent_pass()
{
  // bg opaque; mid plain transparent; top transparent + occlude_overlap.
  let spec = build_spec( [ ( true, false ), ( false, false ), ( false, true ) ] );
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  spawn_one_per_bucket( &mut scene );

  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" ).to_vec();

  // Walk the flattened stream, tracking the depth-write state in force at each
  // Sprite. `SetDepthWrite` passes through the flattener unchanged.
  let flat = common::commands_to_sprites( &cmds );
  let mut write = true; // backend default
  let mut by_depth : Vec< ( f32, bool ) > = Vec::new();
  for c in &flat
  {
    match c
    {
      RenderCommand::SetDepthWrite( s ) => write = s.enabled,
      RenderCommand::Sprite( s ) => by_depth.push( ( s.transform.depth, write ) ),
      _ => {},
    }
  }

  by_depth.sort_by( | a, b | a.0.partial_cmp( &b.0 ).unwrap() );
  // depth 0 = bg (opaque, writes on), 1/3 = mid (transparent, writes off),
  // 2/3 = top (occlude_overlap, writes back on).
  assert_eq!
  (
    by_depth,
    vec![ ( 0.0, true ), ( 1.0 / 3.0, false ), ( 2.0 / 3.0, true ) ],
    "opaque draws with writes on, plain transparent with writes off, \
     occlude_overlap restores writes for its own draw",
  );
}

// ────────────────────────────────────────────────────────────────────────────
// One-pixel model of the WebGL backend's depth contract, for the
// single-coverage tests below. Every sprite is taken to cover the one probed
// pixel, so the model answers "which draws reach this pixel, in which order".
//
// It mirrors the backend: the depth test is `LEQUAL`, or `LESS` while an
// `occlude_overlap` batch draws; a fragment that passes is blended and, when
// depth writes are on, stores its depth; `Clear` resets depth to the far plane
// (and the backend forces writes on there). Nearer = larger `Transform::depth`.
// There is no mid-frame depth clear: the renderer's depth layout alone must
// give single coverage and keep opaque layers hiding what lies under them.
// ────────────────────────────────────────────────────────────────────────────

/// Depth of every fragment blended into the probed pixel, in draw order.
fn painted_depths( cmds : &[ RenderCommand ] ) -> Vec< f32 >
{
  struct Pixel
  {
    stored : Option< f32 >,
    write : bool,
    painted : Vec< f32 >,
  }

  impl Pixel
  {
    fn fragment( &mut self, depth : f32, less : bool )
    {
      let pass = match self.stored
      {
        None => true,
        Some( stored ) => if less { depth > stored } else { depth >= stored },
      };
      if !pass { return; }
      self.painted.push( depth );
      if self.write { self.stored = Some( depth ); }
    }
  }

  let mut batches : HashMap< ResourceId< Batch >, ( bool, Vec< f32 > ) > = HashMap::default();
  let mut bound : Option< ResourceId< Batch > > = None;
  let mut pixel = Pixel { stored : None, write : true, painted : Vec::new() };
  for cmd in cmds
  {
    match cmd
    {
      RenderCommand::Clear( _ ) =>
      {
        pixel.stored = None;
        pixel.write = true;
      },
      RenderCommand::SetDepthWrite( s ) => pixel.write = s.enabled,
      RenderCommand::CreateSpriteBatch( c ) =>
      {
        batches.insert( c.batch, ( c.params.occlude_overlap, Vec::new() ) );
      },
      RenderCommand::BindBatch( b ) => bound = Some( b.batch ),
      RenderCommand::UnbindBatch( _ ) => bound = None,
      RenderCommand::AddSpriteInstance( a ) =>
      {
        let batch = batches.get_mut( &bound.expect( "bound batch" ) ).expect( "known batch" );
        batch.1.push( a.transform.depth );
      },
      RenderCommand::SetSpriteInstance( s ) =>
      {
        let batch = batches.get_mut( &bound.expect( "bound batch" ) ).expect( "known batch" );
        batch.1[ s.index as usize ] = s.transform.depth;
      },
      RenderCommand::RemoveInstance( r ) =>
      {
        let batch = batches.get_mut( &bound.expect( "bound batch" ) ).expect( "known batch" );
        batch.1.swap_remove( r.index as usize );
      },
      RenderCommand::DrawBatch( d ) =>
      {
        let ( occlude, depths ) = batches.get( &d.batch ).expect( "known batch" );
        for &depth in depths { pixel.fragment( depth, *occlude ); }
      },
      RenderCommand::Sprite( s ) | RenderCommand::ScreenSpaceSprite( s ) =>
      {
        pixel.fragment( s.transform.depth, false );
      },
      _ => {},
    }
  }
  pixel.painted
}

/// Number of `DrawBatch` commands emitted for `occlude_overlap` batches.
fn occlude_draw_batches( cmds : &[ RenderCommand ] ) -> usize
{
  let occluders : Vec< ResourceId< Batch > > = cmds.iter()
    .filter_map( | c | match c
    {
      RenderCommand::CreateSpriteBatch( c ) if c.params.occlude_overlap => Some( c.batch ),
      _ => None,
    })
    .collect();
  cmds.iter()
    .filter( | c | matches!( c, RenderCommand::DrawBatch( d ) if occluders.contains( &d.batch ) ) )
    .count()
}

// ────────────────────────────────────────────────────────────────────────────
// 4. An `occlude_overlap` bucket UNDER an opaque one stays hidden by it. The
//    opaque pass writes the upper layer's depth first; the occluder must not
//    wipe that depth, or it paints over the layer that should cover it.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn occlude_bucket_below_opaque_layer_stays_hidden()
{
  // bg opaque; mid = transparent occluder (e.g. a drop shadow); top opaque.
  let spec = build_spec( [ ( true, false ), ( false, true ), ( true, false ) ] );
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  spawn_one_per_bucket( &mut scene );

  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" ).to_vec();

  assert_eq!( occlude_draw_batches( &cmds ), 1, "the mid bucket draws as one occlude batch" );
  // Opaque pass front-to-back: `top` (2/3) paints, `bg` (0) is hidden by it.
  // The occluder at 1/3 sits under `top`, so it must fail the depth test too.
  assert_eq!
  (
    painted_depths( &cmds ), vec![ 2.0 / 3.0 ],
    "only the topmost opaque layer reaches the pixel; the occluder below it stays hidden",
  );
}

// ────────────────────────────────────────────────────────────────────────────
// 5. A `SortMode::None` occlude bucket whose sprites span two sheets splits
//    into two batches (one per `(sheet, blend, clip)` group). Single coverage
//    must hold across the groups, not just inside each one, and the buckets
//    above it must still draw over it.
// ────────────────────────────────────────────────────────────────────────────

fn multi_group_spec( bg_opaque : bool ) -> RenderSpec
{
  let mut spec = build_spec( [ ( bg_opaque, false ), ( false, true ), ( false, false ) ] );
  spec.assets.push( atlas_asset( "sheet2" ) );
  spec.objects.push( object_on( "mid_b", "sheet2", "1", "mid" ) );
  spec
}

fn spawn_multi_group( scene : &mut Scene )
{
  spawn_one_per_bucket( scene );
  // A second `mid` instance on the same sheet, plus one on the other sheet:
  // three overlapping occluder fragments across two batch groups.
  let mid = scene.object( "mid" ).unwrap();
  scene.spawn( mid, Placement::Hex { q : 1, r : 1 } );
  let mid_b = scene.object( "mid_b" ).unwrap();
  scene.spawn( mid_b, Placement::Hex { q : 2, r : 1 } );
}

#[ test ]
fn occlude_bucket_spanning_two_batches_paints_once_single_pass()
{
  let spec = multi_group_spec( false );
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  spawn_multi_group( &mut scene );

  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" ).to_vec();

  assert_eq!( occlude_draw_batches( &cmds ), 2, "the mid bucket splits into two occlude batches" );
  assert!
  (
    depth_write_sequence( &cmds ).is_empty(),
    "no opaque layer ⇒ single pass on the default depth-write state",
  );
  assert_eq!
  (
    painted_depths( &cmds ), vec![ 0.0, 1.0 / 3.0, 2.0 / 3.0 ],
    "bg paints, the occluder paints the pixel once across both batches, top paints over it",
  );
}

#[ test ]
fn occlude_bucket_spanning_two_batches_paints_once_two_pass()
{
  let spec = multi_group_spec( true );
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  spawn_multi_group( &mut scene );

  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" ).to_vec();

  assert_eq!( occlude_draw_batches( &cmds ), 2, "the mid bucket splits into two occlude batches" );
  assert_eq!
  (
    painted_depths( &cmds ), vec![ 0.0, 1.0 / 3.0, 2.0 / 3.0 ],
    "opaque bg, then the occluder once across both batches, then the transparent top",
  );
}

// ────────────────────────────────────────────────────────────────────────────
// 6. A per-sprite draw has no alpha clip, so in the opaque pass it would write
//    depth over its whole quad and punch holes in the layers under it. An
//    opaque bucket that has to draw per sprite (here: sorted, two sheets) is
//    moved to the transparent pass for the frame.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn opaque_bucket_drawn_per_sprite_falls_back_to_transparent_pass()
{
  let mut spec = build_spec( [ ( true, false ), ( true, false ), ( false, false ) ] );
  spec.pipeline.layers[ 1 ].sort = SortMode::YAsc;
  spec.assets.push( atlas_asset( "sheet2" ) );
  spec.objects.push( object_on( "mid_b", "sheet2", "1", "mid" ) );
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  spawn_one_per_bucket( &mut scene );
  let mid_b = scene.object( "mid_b" ).unwrap();
  scene.spawn( mid_b, Placement::Hex { q : 1, r : 1 } );

  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" ).to_vec();

  // Walk the raw stream: `mid` draws as plain `Sprite` commands; record the
  // depth-write state each one sees.
  let mut write = true;
  let mut mid_writes : Vec< bool > = Vec::new();
  for c in &cmds
  {
    match c
    {
      RenderCommand::SetDepthWrite( s ) => write = s.enabled,
      RenderCommand::Sprite( _ ) => mid_writes.push( write ),
      _ => {},
    }
  }
  assert_eq!( mid_writes, vec![ false, false ], "the per-sprite opaque bucket draws with depth writes off" );
  assert_eq!
  (
    painted_depths( &cmds ), vec![ 0.0, 1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0 ],
    "bg in the opaque pass, then mid and top in painter's order on top of it",
  );
}
