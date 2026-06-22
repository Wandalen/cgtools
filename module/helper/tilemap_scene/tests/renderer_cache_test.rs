//! Acceptance tests for [`tilemap_scene::Renderer`]'s per-frame idle-replay
//! cache (Step 4a — feedback §9 `compile_frame` rebuilds the whole command
//! stream every frame).
//!
//! The contract under test: a `render()` call whose `(scene.revision(),
//! scene.clock(), camera signature)` matches the snapshot captured by the
//! previous call returns the previously emitted command slice verbatim
//! without re-walking the scene. Any change to revision (mutation), clock
//! (tick), or camera invalidates the cache and forces a fresh emission.

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
use tilemap_renderer::types::{ MipmapMode, SamplerFilter, WrapMode };

mod common;

// ────────────────────────────────────────────────────────────────────────────
// Fixture — a single grass tile + a single knight unit, both anchored on
// hex cells. Single bucket, no animations: keeps assertions on emitted
// commands deterministic and short.
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

fn build_spec() -> RenderSpec
{
  let mut grass_states = HashMap::default();
  grass_states.insert( "default".into(), vec![ static_layer( "terrain", "0" ) ] );

  let mut knight_states = HashMap::default();
  knight_states.insert( "default".into(), vec![ static_layer( "terrain", "1" ) ] );

  RenderSpec
  {
    version : "0.2.0".into(),
    assets : vec!
    [
      Asset
      {
        id : "terrain".into(),
        path : "terrain.png".into(),
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
      },
    ],
    tints : Vec::new(),
    animations : Vec::new(),
    effects : Vec::new(),
    objects : vec!
    [
      Object
      {
        id : "grass".into(),
        anchor : Anchor::Hex,
        global_layer : "terrain".into(),
        priority : None,
        sort_y_source : Default::default(),
        pivot : ( 0.5, 0.5 ),
        default_state : "default".into(),
        states : grass_states,
      },
      Object
      {
        id : "knight".into(),
        anchor : Anchor::Hex,
        global_layer : "terrain".into(),
        priority : None,
        sort_y_source : Default::default(),
        pivot : ( 0.5, 0.5 ),
        default_state : "default".into(),
        states : knight_states,
      },
    ],
    pipeline : RenderPipeline
    {
      hex : HexConfig { tiling : TilingStrategy::HexFlatTop, grid_stride : ( 72, 64 ) },
      layers : vec!
      [
        PipelineLayer { id : "terrain".into(), sort : SortMode::None, tint_mask : None, alpha_clip : 0.0, occlude_overlap : false },
      ],
      global_tint : None,
      viewport_size : None,
      clear_color : None,
    },
  }
}

/// Snapshot the emitted commands as an owned `Vec` so we can compare
/// across `render()` calls without juggling lifetimes.
fn snapshot( cmds : &[ RenderCommand ] ) -> Vec< RenderCommand > { cmds.to_vec() }

/// Count world-space sprites after batch-flattening — a coarse "what
/// got drawn" signal that works whether the renderer emits per-sprite
/// `Sprite` commands or batched `AddSpriteInstance` / `DrawBatch` runs.
fn sprite_count( cmds : &[ RenderCommand ] ) -> usize
{
  common::flat_sprite_count( cmds )
}

// ────────────────────────────────────────────────────────────────────────────
// 1. Idle replay — no changes between calls returns cached slice.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn idle_render_returns_cached_slice()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let grass = scene.object( "grass" ).unwrap();
  let _ = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  let camera = Camera::default();

  let first = snapshot( renderer.render( &scene, &camera ).expect( "first" ) );
  let hits_after_first = renderer.cache_hits();

  let second = snapshot( renderer.render( &scene, &camera ).expect( "second" ) );
  let hits_after_second = renderer.cache_hits();

  assert_eq!( first.len(), second.len(), "cached slice has same length" );
  assert_eq!
  (
    hits_after_second - hits_after_first, 1,
    "second call must hit cache exactly once",
  );
}

#[ test ]
fn many_idle_renders_all_hit_cache()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let grass = scene.object( "grass" ).unwrap();
  let _ = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  let camera = Camera::default();

  // First render populates cache.
  let _ = renderer.render( &scene, &camera ).expect( "prime cache" );
  assert_eq!( renderer.cache_hits(), 0, "first render is a miss, not a hit" );

  // Subsequent N calls all hit cache.
  for _ in 0..10
  {
    let _ = renderer.render( &scene, &camera ).expect( "idle" );
  }
  assert_eq!
  (
    renderer.cache_hits(), 10,
    "10 consecutive idle renders must all hit cache",
  );
}

// ────────────────────────────────────────────────────────────────────────────
// 2. First render is always a cache miss.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn first_render_is_a_miss()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let scene = Scene::new( Arc::new( spec ) );

  assert_eq!( renderer.cache_hits(), 0, "fresh renderer has zero hits" );
  let _ = renderer.render( &scene, &Camera::default() ).expect( "render" );
  assert_eq!
  (
    renderer.cache_hits(), 0,
    "first render must not be served from cache",
  );
}

// ────────────────────────────────────────────────────────────────────────────
// 3. Mutation invalidates cache.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn spawn_invalidates_cache_and_changes_output()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let grass = scene.object( "grass" ).unwrap();
  let camera = Camera::default();

  // Render with no instances.
  let before = sprite_count( &snapshot( renderer.render( &scene, &camera ).expect( "before" ) ) );

  // Spawn one — must invalidate the cache.
  let _ = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  let hits_before = renderer.cache_hits();

  let after = sprite_count( &snapshot( renderer.render( &scene, &camera ).expect( "after" ) ) );
  let hits_after = renderer.cache_hits();

  assert!( after > before, "spawned instance must appear in output" );
  assert_eq!
  (
    hits_after, hits_before,
    "spawn must invalidate cache — render after spawn cannot be a hit",
  );
}

#[ test ]
fn move_to_invalidates_cache()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let grass = scene.object( "grass" ).unwrap();
  let h = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  let camera = Camera::default();

  let _ = renderer.render( &scene, &camera ).expect( "prime" );
  scene.move_to( h, Placement::Hex { q : 5, r : 5 } );
  let hits_before = renderer.cache_hits();
  let _ = renderer.render( &scene, &camera ).expect( "after move" );
  assert_eq!
  (
    renderer.cache_hits(), hits_before,
    "move_to must invalidate cache",
  );
}

#[ test ]
fn despawn_invalidates_cache_and_drops_sprite()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let grass = scene.object( "grass" ).unwrap();
  let h = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  let camera = Camera::default();

  let before = sprite_count( &snapshot( renderer.render( &scene, &camera ).expect( "before" ) ) );
  scene.despawn( h );
  let hits_before = renderer.cache_hits();
  let after = sprite_count( &snapshot( renderer.render( &scene, &camera ).expect( "after" ) ) );

  assert!( after < before, "despawn drops a sprite" );
  assert_eq!
  (
    renderer.cache_hits(), hits_before,
    "despawn must invalidate cache",
  );
}

#[ test ]
fn set_tint_invalidates_cache()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let grass = scene.object( "grass" ).unwrap();
  let h = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  let camera = Camera::default();

  let _ = renderer.render( &scene, &camera ).expect( "prime" );
  scene.set_tint( h, Some( [ 0.5, 1.0, 1.0, 1.0 ] ) );
  let hits_before = renderer.cache_hits();
  let _ = renderer.render( &scene, &camera ).expect( "after tint" );
  assert_eq!
  (
    renderer.cache_hits(), hits_before,
    "set_tint must invalidate cache",
  );
}

// ────────────────────────────────────────────────────────────────────────────
// 4. Clock advance via tick() invalidates cache.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn tick_with_dt_invalidates_cache()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let grass = scene.object( "grass" ).unwrap();
  let _ = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  let camera = Camera::default();

  let _ = renderer.render( &scene, &camera ).expect( "prime" );
  let _ = scene.tick( 0.5 );
  let hits_before = renderer.cache_hits();
  let _ = renderer.render( &scene, &camera ).expect( "after tick" );
  assert_eq!
  (
    renderer.cache_hits(), hits_before,
    "tick with dt > 0 advances clock and must invalidate cache",
  );
}

#[ test ]
fn tick_zero_keeps_cache_warm()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let grass = scene.object( "grass" ).unwrap();
  let _ = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  let camera = Camera::default();

  let _ = renderer.render( &scene, &camera ).expect( "prime" );
  let _ = scene.tick( 0.0 );
  let hits_before = renderer.cache_hits();
  let _ = renderer.render( &scene, &camera ).expect( "after tick(0)" );
  assert_eq!
  (
    renderer.cache_hits(), hits_before + 1,
    "tick(0.0) does not advance clock — cache must still hit",
  );
}

// ────────────────────────────────────────────────────────────────────────────
// 5. Camera change invalidates cache.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn camera_pan_invalidates_cache()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let grass = scene.object( "grass" ).unwrap();
  let _ = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );

  let cam1 = Camera::default();
  let cam2 = Camera { world_center : ( 100.0, 0.0 ), ..Camera::default() };

  let _ = renderer.render( &scene, &cam1 ).expect( "prime" );
  let hits_before = renderer.cache_hits();
  let _ = renderer.render( &scene, &cam2 ).expect( "different camera" );
  assert_eq!
  (
    renderer.cache_hits(), hits_before,
    "camera pan must invalidate cache",
  );
}

#[ test ]
fn camera_zoom_invalidates_cache()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let grass = scene.object( "grass" ).unwrap();
  let _ = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );

  let cam1 = Camera::default();
  let cam2 = Camera { zoom : 2.0, ..Camera::default() };

  let _ = renderer.render( &scene, &cam1 ).expect( "prime" );
  let hits_before = renderer.cache_hits();
  let _ = renderer.render( &scene, &cam2 ).expect( "zoomed" );
  assert_eq!
  (
    renderer.cache_hits(), hits_before,
    "camera zoom must invalidate cache",
  );
}

#[ test ]
fn camera_viewport_resize_invalidates_cache()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let grass = scene.object( "grass" ).unwrap();
  let _ = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );

  let cam1 = Camera::default();
  let cam2 = Camera { viewport_size : ( 1024, 768 ), ..Camera::default() };

  let _ = renderer.render( &scene, &cam1 ).expect( "prime" );
  let hits_before = renderer.cache_hits();
  let _ = renderer.render( &scene, &cam2 ).expect( "resized" );
  assert_eq!
  (
    renderer.cache_hits(), hits_before,
    "camera viewport resize must invalidate cache",
  );
}

// ────────────────────────────────────────────────────────────────────────────
// 6. Replay correctness — cached slice is byte-equal to the slice that
//    populated it. Prevents future regressions where the replay path
//    accidentally returns stale data after the next miss.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn cached_slice_is_byte_equal_to_priming_slice()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let grass = scene.object( "grass" ).unwrap();
  let knight = scene.object( "knight" ).unwrap();
  let _ = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  let _ = scene.spawn( knight, Placement::Hex { q : 1, r : 0 } );
  let camera = Camera::default();

  let primed = snapshot( renderer.render( &scene, &camera ).expect( "prime" ) );
  let replayed = snapshot( renderer.render( &scene, &camera ).expect( "replay" ) );

  assert_eq!( primed.len(), replayed.len(), "command count stable across replay" );
  assert_eq!
  (
    sprite_count( &primed ), sprite_count( &replayed ),
    "sprite count stable across replay",
  );
  // First command of both should be Clear; nth Sprite should match by id.
  assert!( matches!( primed.first(), Some( RenderCommand::Clear( _ ) ) ) );
  assert!( matches!( replayed.first(), Some( RenderCommand::Clear( _ ) ) ) );
}

// ────────────────────────────────────────────────────────────────────────────
// 7. After invalidation, the new render correctly repopulates the cache —
//    a subsequent idle call hits again.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn miss_then_idle_hits_again()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let grass = scene.object( "grass" ).unwrap();
  let h = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  let camera = Camera::default();

  let _ = renderer.render( &scene, &camera ).expect( "prime" );
  scene.move_to( h, Placement::Hex { q : 1, r : 1 } );
  let _ = renderer.render( &scene, &camera ).expect( "miss after mutation" );
  let hits_before = renderer.cache_hits();
  let _ = renderer.render( &scene, &camera ).expect( "idle hit" );
  assert_eq!
  (
    renderer.cache_hits(), hits_before + 1,
    "cache must be re-warmed after a miss — next idle call should hit",
  );
}

// ────────────────────────────────────────────────────────────────────────────
// 8. Bit-equal Set elision — partial Deferred §1 closure.
//
// When `tick( dt )` advances the clock but no scene mutation has happened,
// `render()` takes the cache-miss path (clock changed). The renderer must
// not re-upload the GPU instance buffer for batches whose contents are
// bit-equal to last frame — `BindBatch` / `SetSpriteInstance` /
// `UnbindBatch` for that batch are skipped entirely.
// ────────────────────────────────────────────────────────────────────────────

fn count_cmd< F : Fn( &RenderCommand ) -> bool >( cmds : &[ RenderCommand ], pred : F ) -> usize
{
  cmds.iter().filter( | c | pred( c ) ).count()
}

#[ test ]
fn unchanged_batch_emits_no_set_on_cache_miss()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let grass = scene.object( "grass" ).unwrap();
  for ( q, r ) in [ ( 0, 0 ), ( 1, 0 ), ( 2, 0 ), ( 0, 1 ), ( 1, 1 ) ]
  {
    scene.spawn( grass, Placement::Hex { q, r } );
  }
  let camera = Camera::default();

  // Prime — creates the batch and emits initial Add×5.
  let _ = renderer.render( &scene, &camera ).expect( "prime" );

  // Advance the clock without mutating the scene → cache miss, but the
  // batch's instance buffer is bit-equal to last frame.
  let _ = scene.tick( 0.016 );
  let cmds = renderer.render( &scene, &camera ).expect( "after tick" ).to_vec();

  assert_eq!
  (
    count_cmd( &cmds, | c | matches!( c, RenderCommand::SetSpriteInstance( _ ) ) ),
    0,
    "no instance content changed — Set must be elided entirely",
  );
  assert_eq!
  (
    count_cmd( &cmds, | c | matches!( c, RenderCommand::BindBatch( _ ) ) ),
    0,
    "no diff to apply — BindBatch must be skipped",
  );
  assert_eq!
  (
    count_cmd( &cmds, | c | matches!( c, RenderCommand::UnbindBatch( _ ) ) ),
    0,
    "no diff to apply — UnbindBatch must be skipped",
  );
}

#[ test ]
fn single_move_emits_fewer_sets_than_full_repopulate()
{
  // Bit-equal elision provides a strict lower bound: at most one Set per
  // slot whose sprite payload actually differs between frames. Because
  // some compile passes iterate scene state in non-deterministic
  // HashMap order (e.g. `build_scene_tiles` for tile_lookup), more than
  // one slot may differ after a `move_to` — but it must never be the
  // pre-optimisation N (= full common-prefix repopulate).
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let grass = scene.object( "grass" ).unwrap();
  let mut handles = Vec::new();
  for ( q, r ) in [ ( 0, 0 ), ( 1, 0 ), ( 2, 0 ), ( 0, 1 ), ( 1, 1 ) ]
  {
    handles.push( scene.spawn( grass, Placement::Hex { q, r } ) );
  }
  let camera = Camera::default();

  let _ = renderer.render( &scene, &camera ).expect( "prime" );

  scene.move_to( handles[ 2 ], Placement::Hex { q : 5, r : 5 } );
  let cmds = renderer.render( &scene, &camera ).expect( "after move" ).to_vec();

  let sets = count_cmd( &cmds, | c | matches!( c, RenderCommand::SetSpriteInstance( _ ) ) );
  assert!
  (
    sets >= 1,
    "moved tile's slot must emit one Set (got {sets})",
  );
  assert!
  (
    sets < 5,
    "elision must skip at least one unchanged slot in a 5-tile batch (got {sets})",
  );
}

#[ test ]
fn single_tint_change_emits_exactly_one_set()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let grass = scene.object( "grass" ).unwrap();
  let mut handles = Vec::new();
  for ( q, r ) in [ ( 0, 0 ), ( 1, 0 ), ( 2, 0 ), ( 0, 1 ), ( 1, 1 ) ]
  {
    handles.push( scene.spawn( grass, Placement::Hex { q, r } ) );
  }
  let camera = Camera::default();

  let _ = renderer.render( &scene, &camera ).expect( "prime" );

  scene.set_tint( handles[ 3 ], Some( [ 0.5, 1.0, 1.0, 1.0 ] ) );
  let cmds = renderer.render( &scene, &camera ).expect( "after tint" ).to_vec();

  assert_eq!
  (
    count_cmd( &cmds, | c | matches!( c, RenderCommand::SetSpriteInstance( _ ) ) ),
    1,
    "one tile's tint changed → exactly one SetSpriteInstance",
  );
}

// ────────────────────────────────────────────────────────────────────────────
// 9. `Renderer::cleanup()` — Deferred §5.
//
// Cleanup must emit one `DeleteBatch` per live batch and reset the
// idle-replay cache so a subsequent `render()` re-allocates fresh
// batches (not replay a `cmd_buf` referencing the just-deleted ids).
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn cleanup_emits_delete_for_every_live_batch()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  let grass = scene.object( "grass" ).unwrap();
  let knight = scene.object( "knight" ).unwrap();
  scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  scene.spawn( knight, Placement::Hex { q : 1, r : 0 } );
  let camera = Camera::default();

  // Prime — produces some CreateSpriteBatch commands.
  let primed = renderer.render( &scene, &camera ).expect( "prime" ).to_vec();
  let creates = primed.iter()
    .filter( | c | matches!( c, RenderCommand::CreateSpriteBatch( _ ) ) )
    .count();
  assert!( creates >= 1, "at least one batch created during prime" );

  let drain = renderer.cleanup();
  let deletes = drain.iter()
    .filter( | c | matches!( c, RenderCommand::DeleteBatch( _ ) ) )
    .count();
  assert_eq!
  (
    deletes, creates,
    "cleanup must emit one DeleteBatch per batch created during prime",
  );

  // Subsequent render() is a guaranteed miss (cache reset) and
  // re-allocates the batches from scratch.
  let hits_before = renderer.cache_hits();
  let after = renderer.render( &scene, &camera ).expect( "after cleanup" ).to_vec();
  assert_eq!
  (
    renderer.cache_hits(), hits_before,
    "first render after cleanup must miss the cache",
  );
  let creates_after = after.iter()
    .filter( | c | matches!( c, RenderCommand::CreateSpriteBatch( _ ) ) )
    .count();
  assert_eq!
  (
    creates_after, creates,
    "post-cleanup render re-allocates the same number of batches",
  );
}
