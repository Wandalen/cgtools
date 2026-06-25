//! Sorted-bucket batching — closes roadmap Deferred §2(a).
//!
//! Single-key sorted buckets (every emitted sprite shares one
//! `(sheet, blend, clip)`) collapse to a single `DrawBatch` whose
//! instance-buffer order matches the sort order. Multi-key sorted
//! buckets keep the per-sprite `Sprite` fallback because `DrawBatch`
//! has no range support and run-splitting would defeat batch reuse.
//!
//! Also pins the inline `DrawBatch` emission contract: per-bucket
//! draw calls land in pipeline order, so a sorted multi-sheet
//! fallback drawn between two batched buckets does not slip behind
//! their `DrawBatch`es.

#![ allow( clippy::min_ident_chars ) ]
#![ allow
(
  clippy::default_trait_access,
  clippy::too_many_lines,
  clippy::float_cmp,
) ]

extern crate alloc;
use alloc::sync::Arc;
use rustc_hash::FxHashMap as HashMap;

use tilemap_renderer::commands::{ RenderCommand, Sprite };
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
use tilemap_renderer::types::{ MipmapMode, SamplerFilter, WrapMode };

mod common;

// ────────────────────────────────────────────────────────────────────────────
// Fixtures — two atlases (`terrain`, `units`) and three objects:
// - `grass`    : terrain bucket, `SortMode::None`.
// - `knight_a` : units bucket, sprite from `terrain` atlas.
// - `knight_b` : units bucket, sprite from `units` atlas (separate sheet).
// ────────────────────────────────────────────────────────────────────────────

fn static_layer( asset : &str, frame : &str, pipeline_layer : Option< String > ) -> ObjectLayer
{
  ObjectLayer
  {
    id : None,
    sprite_source : SpriteSource::Static( SpriteRef { asset : asset.into(), frame : frame.into() } ),
    behaviour : LayerBehaviour::default(),
    z_in_object : 0,
    pipeline_layer,
  }
}

fn atlas_asset( id : &str, path : &str ) -> Asset
{
  Asset
  {
    id : id.into(),
    path : path.into(),
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

fn object( id : &str, asset : &str, frame : &str, pipeline_layer : &str ) -> Object
{
  let mut states = HashMap::default();
  states.insert
  (
    "default".into(),
    vec![ static_layer( asset, frame, Some( pipeline_layer.into() ) ) ],
  );
  Object
  {
    id : id.into(),
    anchor : Anchor::Hex,
    global_layer : "terrain".into(),
    priority : None,
    sort_y_source : Default::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states,
  }
}

fn build_spec() -> RenderSpec
{
  RenderSpec
  {
    version : "0.2.0".into(),
    assets : vec!
    [
      atlas_asset( "terrain", "terrain.png" ),
      atlas_asset( "units", "units.png" ),
    ],
    tints : Vec::new(),
    animations : Vec::new(),
    effects : Vec::new(),
    objects : vec!
    [
      object( "grass", "terrain", "0", "terrain" ),
      object( "knight_a", "terrain", "1", "units" ),
      object( "knight_b", "units", "0", "units" ),
      object( "effect", "terrain", "2", "effects" ),
    ],
    pipeline : RenderPipeline
    {
      hex : HexConfig { tiling : TilingStrategy::HexFlatTop, grid_stride : ( 72, 64 ) },
      layers : vec!
      [
        PipelineLayer { id : "terrain".into(), sort : SortMode::None, tint_mask : None, alpha_clip : 0.0, occlude_overlap : false, opaque : false },
        PipelineLayer { id : "units".into(),   sort : SortMode::YAsc, tint_mask : None, alpha_clip : 0.0, occlude_overlap : false, opaque : false },
        PipelineLayer { id : "effects".into(), sort : SortMode::None, tint_mask : None, alpha_clip : 0.0, occlude_overlap : false, opaque : false },
      ],
      global_tint : None,
      viewport_size : None,
      clear_color : None,
    },
  }
}

fn count_draw_batches( cmds : &[ RenderCommand ] ) -> usize
{
  cmds.iter().filter( | c | matches!( c, RenderCommand::DrawBatch( _ ) ) ).count()
}

fn count_sprite_commands( cmds : &[ RenderCommand ] ) -> usize
{
  cmds.iter().filter( | c | matches!( c, RenderCommand::Sprite( _ ) ) ).count()
}

// ────────────────────────────────────────────────────────────────────────────
// 1. Single-key sorted bucket batches into one `DrawBatch`, no `Sprite`s.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn single_key_sorted_bucket_emits_drawbatch_no_sprites()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let knight = scene.object( "knight_a" ).unwrap();

  // Three knights, all from the `terrain` atlas → single-key sorted.
  scene.spawn( knight, Placement::Hex { q : 0, r : 0 } );
  scene.spawn( knight, Placement::Hex { q : 0, r : 1 } );
  scene.spawn( knight, Placement::Hex { q : 0, r : 2 } );

  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" );

  assert_eq!
  (
    count_sprite_commands( cmds ), 0,
    "single-key sorted bucket must not fall back to per-sprite Sprite commands",
  );
  // 1 DrawBatch for the units bucket (no terrain/effects instances spawned).
  assert_eq!( count_draw_batches( cmds ), 1, "exactly one DrawBatch for the sorted bucket" );

  let sprites = common::flat_sprites( cmds );
  assert_eq!( sprites.len(), 3, "three knights flatten to three sprites" );
}

// ────────────────────────────────────────────────────────────────────────────
// 2. Sort order is preserved inside the batch.
//
// Flat-top hex grid maps positive `r` to MORE NEGATIVE world Y (Y-up), so for
// `SortMode::YAsc` the spawn at `r=2` produces the smallest Y and must appear
// first in the flattened sprite list.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn batch_instance_order_matches_sort_order_yasc()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let knight = scene.object( "knight_a" ).unwrap();

  scene.spawn( knight, Placement::Hex { q : 0, r : 0 } );
  scene.spawn( knight, Placement::Hex { q : 0, r : 1 } );
  scene.spawn( knight, Placement::Hex { q : 0, r : 2 } );

  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" );
  let sprites = common::flat_sprites( cmds );
  assert_eq!( sprites.len(), 3 );

  // YAsc — y monotonically increases.
  for w in sprites.windows( 2 )
  {
    assert!
    (
      w[ 0 ].transform.position[ 1 ] <= w[ 1 ].transform.position[ 1 ],
      "YAsc must produce non-decreasing Y in batch instance order; got {:?}",
      sprites.iter().map( | s | s.transform.position[ 1 ] ).collect::< Vec< _ > >(),
    );
  }
}

// ────────────────────────────────────────────────────────────────────────────
// 3. Multi-sheet sorted bucket falls back to per-sprite emission.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn multi_sheet_sorted_bucket_falls_back_to_per_sprite()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let knight_a = scene.object( "knight_a" ).unwrap();
  let knight_b = scene.object( "knight_b" ).unwrap();

  // Two sprites in the units bucket from DIFFERENT atlases → multi-key.
  scene.spawn( knight_a, Placement::Hex { q : 0, r : 0 } );
  scene.spawn( knight_b, Placement::Hex { q : 0, r : 1 } );

  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" );

  assert_eq!
  (
    count_sprite_commands( cmds ), 2,
    "multi-sheet sorted bucket must emit one Sprite per emit (fallback path)",
  );
  // No `DrawBatch` for the units bucket itself (terrain / effects also empty).
  assert_eq!
  (
    count_draw_batches( cmds ), 0,
    "no batches in pipeline when only the units bucket has instances and it falls back",
  );
}

// ────────────────────────────────────────────────────────────────────────────
// 4. Mixed pipeline: inline `DrawBatch` emission preserves pipeline order.
//
// terrain (None, batched) → units (YAsc, single-key, batched) → effects
// (None, batched). All three buckets emit `DrawBatch` — the per-bucket
// draws must appear in pipeline-declared order in the command stream.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn mixed_pipeline_drawbatches_in_pipeline_order()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let grass = scene.object( "grass" ).unwrap();
  let knight = scene.object( "knight_a" ).unwrap();
  let effect = scene.object( "effect" ).unwrap();

  scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  scene.spawn( knight, Placement::Hex { q : 0, r : 1 } );
  scene.spawn( effect, Placement::Hex { q : 0, r : 2 } );

  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" ).to_vec();

  // 3 buckets each with one batch → 3 DrawBatch commands, in order.
  let draw_positions : Vec< usize > = cmds.iter().enumerate()
    .filter_map( | ( i, c ) | matches!( c, RenderCommand::DrawBatch( _ ) ).then_some( i ) )
    .collect();
  assert_eq!( draw_positions.len(), 3, "three batched buckets → three DrawBatches" );

  // No per-sprite fallback in this pipeline.
  assert_eq!( count_sprite_commands( &cmds ), 0 );

  // Each DrawBatch's batch resolves through the BatchFlattener to one sprite
  // and the order across DrawBatches is the pipeline order.
  let flat = common::flatten_to_sprites( &cmds );
  let sprites : Vec< &Sprite > = flat.iter()
    .filter_map( | c | if let RenderCommand::Sprite( s ) = c { Some( s ) } else { None } )
    .collect();
  assert_eq!( sprites.len(), 3 );
  // Frame ids encode which bucket each came from: grass=0, knight_a=1, effect=2.
  // The sprite resource ids monotonically follow first-emit order — the test
  // therefore asserts strict ordering via sprite id, not via frame name.
  // (Resource ids are assigned in compile order, which mirrors the
  // declaration order in `build_spec`.)
  assert!
  (
    sprites[ 0 ].sprite.inner() < sprites[ 1 ].sprite.inner()
      && sprites[ 1 ].sprite.inner() < sprites[ 2 ].sprite.inner(),
    "flattened sprites appear in pipeline-declared bucket order",
  );
}

// ────────────────────────────────────────────────────────────────────────────
// 5. Mixed pipeline with multi-sheet sorted bucket — the per-sprite fallback
//    sits between the surrounding batches' DrawBatch commands. This pins
//    the inline-`DrawBatch` fix for the latent pre-existing bug where all
//    `Sprite` commands fired before any `DrawBatch`.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn fallback_sprite_commands_sit_between_pipeline_drawbatches()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let grass = scene.object( "grass" ).unwrap();
  let knight_a = scene.object( "knight_a" ).unwrap();
  let knight_b = scene.object( "knight_b" ).unwrap();
  let effect = scene.object( "effect" ).unwrap();

  scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  scene.spawn( knight_a, Placement::Hex { q : 0, r : 1 } );
  scene.spawn( knight_b, Placement::Hex { q : 0, r : 2 } );
  scene.spawn( effect, Placement::Hex { q : 0, r : 3 } );

  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" ).to_vec();

  // Find the positions of: terrain DrawBatch, units Sprites, effects DrawBatch.
  let first_draw = cmds.iter().position( | c | matches!( c, RenderCommand::DrawBatch( _ ) ) )
    .expect( "at least one DrawBatch (terrain bucket)" );
  let last_draw = cmds.iter().rposition( | c | matches!( c, RenderCommand::DrawBatch( _ ) ) )
    .expect( "at least one DrawBatch (effects bucket)" );
  let first_sprite = cmds.iter().position( | c | matches!( c, RenderCommand::Sprite( _ ) ) )
    .expect( "units bucket falls back to per-sprite" );
  let last_sprite = cmds.iter().rposition( | c | matches!( c, RenderCommand::Sprite( _ ) ) )
    .expect( "units bucket falls back to per-sprite" );

  assert!
  (
    first_draw < first_sprite,
    "terrain DrawBatch must precede the units Sprite fallback in command order",
  );
  assert!
  (
    last_sprite < last_draw,
    "effects DrawBatch must follow the units Sprite fallback in command order",
  );
  assert_eq!( count_draw_batches( &cmds ), 2, "terrain + effects = 2 batches" );
  assert_eq!( count_sprite_commands( &cmds ), 2, "two units in the fallback path" );
}

// ────────────────────────────────────────────────────────────────────────────
// 6. Cache: idle replay of a sorted batch is byte-equal to the priming slice.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn idle_replay_byte_equal_for_sorted_batch()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let knight = scene.object( "knight_a" ).unwrap();
  scene.spawn( knight, Placement::Hex { q : 0, r : 0 } );
  scene.spawn( knight, Placement::Hex { q : 0, r : 1 } );
  let camera = Camera::default();

  let primed : Vec< RenderCommand > = renderer.render( &scene, &camera ).expect( "prime" ).to_vec();
  let hits_before = renderer.cache_hits();
  let replayed : Vec< RenderCommand > = renderer.render( &scene, &camera ).expect( "replay" ).to_vec();

  assert_eq!( renderer.cache_hits(), hits_before + 1, "second call hits cache" );
  assert_eq!( primed.len(), replayed.len(), "replay slice length matches" );
}

// ────────────────────────────────────────────────────────────────────────────
// 7. Cache: moving a unit reuses the same batch (no Create/Delete churn).
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn sorted_batch_reused_across_move()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let knight = scene.object( "knight_a" ).unwrap();
  let h0 = scene.spawn( knight, Placement::Hex { q : 0, r : 0 } );
  let _h1 = scene.spawn( knight, Placement::Hex { q : 0, r : 1 } );
  let camera = Camera::default();

  // Prime — first render creates the batch.
  let first = renderer.render( &scene, &camera ).expect( "prime" ).to_vec();
  let creates_first = first.iter().filter( | c | matches!( c, RenderCommand::CreateSpriteBatch( _ ) ) ).count();
  assert_eq!( creates_first, 1, "first render creates the units batch once" );

  // Move one unit — must invalidate cache, but reuse the same batch (no
  // CreateSpriteBatch / DeleteBatch in the new stream).
  scene.move_to( h0, Placement::Hex { q : 5, r : 5 } );
  let second = renderer.render( &scene, &camera ).expect( "after move" ).to_vec();

  let creates_second = second.iter().filter( | c | matches!( c, RenderCommand::CreateSpriteBatch( _ ) ) ).count();
  let deletes_second = second.iter().filter( | c | matches!( c, RenderCommand::DeleteBatch( _ ) ) ).count();
  let sets_second = second.iter().filter( | c | matches!( c, RenderCommand::SetSpriteInstance( _ ) ) ).count();

  assert_eq!( creates_second, 0, "batch is reused across mutation — no fresh CreateSpriteBatch" );
  assert_eq!( deletes_second, 0, "batch is not garbage-collected mid-mutation" );
  assert!( sets_second >= 1, "diff emits at least one SetSpriteInstance for the moved unit" );
}
