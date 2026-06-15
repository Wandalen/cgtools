//! Integration tests for the retained-mode [`tilemap_scene::Scene`] API.
//!
//! Covers `spawn` / `despawn` / `move_to` / `set_state` / `set_tint` /
//! `set_visible` / `set_phase_offset` / `set_external_sprite`, plus the
//! handle-resolution helpers (`object`, `state`, `default_state`,
//! `state_name`) and the spatial indexes (`instances_at_hex`, per-anchor
//! lists).

#![ allow( clippy::min_ident_chars ) ]
#![ allow
(
  clippy::default_trait_access,
  clippy::too_many_lines,
) ]

extern crate alloc;
use alloc::sync::Arc;
use rustc_hash::FxHashMap as HashMap;

use tilemap_scene::
{
  Anchor,
  Asset,
  AssetKind,
  HexConfig,
  LayerBehaviour,
  Object,
  ObjectLayer,
  PipelineLayer,
  Placement,
  RenderPipeline,
  RenderSpec,
  Scene,
  SortMode,
  SpriteRef,
  SpriteSource,
  TilingStrategy,
};
use tilemap_renderer::types::{ MipmapMode, SamplerFilter, WrapMode };

// ────────────────────────────────────────────────────────────────────────────
// Fixture: a two-object spec — `grass` (idle/default state) and `knight`
// (with idle + walk states, default = idle). Enough variety to test
// cross-object state validation and sorted-state-name resolution.
// ────────────────────────────────────────────────────────────────────────────

fn make_layer( asset : &str, frame : &str ) -> ObjectLayer
{
  ObjectLayer
  {
    id : None,
    sprite_source : SpriteSource::Static
    (
      SpriteRef { asset : asset.into(), frame : frame.into() }
    ),
    behaviour : LayerBehaviour::default(),
    z_in_object : 0,
    pipeline_layer : None,
  }
}

fn build_spec() -> Arc< RenderSpec >
{
  let mut grass_states = HashMap::default();
  grass_states.insert( "default".into(), vec![ make_layer( "terrain", "0" ) ] );

  let mut knight_states = HashMap::default();
  knight_states.insert( "idle".into(), vec![ make_layer( "terrain", "0" ) ] );
  knight_states.insert( "walk".into(), vec![ make_layer( "terrain", "1" ) ] );

  let spec = RenderSpec
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
          columns : 2,
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
        default_state : "idle".into(),
        states : knight_states,
      },
    ],
    pipeline : RenderPipeline
    {
      hex : HexConfig { tiling : TilingStrategy::HexFlatTop, grid_stride : ( 72, 64 ) },
      layers : vec!
      [
        PipelineLayer { id : "terrain".into(), sort : SortMode::None, tint_mask : None },
      ],
      global_tint : None,
      viewport_size : None,
      clear_color : None,
    },
  };
  Arc::new( spec )
}

// ────────────────────────────────────────────────────────────────────────────
// Handle resolution
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn object_handle_resolves_for_declared_object_only()
{
  let scene = Scene::new( build_spec() );
  let grass = scene.object( "grass" ).expect( "grass declared" );
  let knight = scene.object( "knight" ).expect( "knight declared" );
  assert_ne!( grass, knight );
  assert!( scene.object( "missing" ).is_none() );
}

#[ test ]
fn state_lookup_uses_sorted_order()
{
  // knight has states { "idle", "walk" }. Sorted alphabetically, "idle"
  // comes before "walk", so default ("idle") is index 0.
  let scene = Scene::new( build_spec() );
  let knight = scene.object( "knight" ).unwrap();
  let idle = scene.state( knight, "idle" ).unwrap();
  let walk = scene.state( knight, "walk" ).unwrap();
  assert_eq!( idle.state_index, 0, "idle sorts first" );
  assert_eq!( walk.state_index, 1, "walk sorts second" );
  assert_eq!( scene.default_state( knight ), idle );
  assert_eq!( scene.state_name( idle ), Some( "idle" ) );
  assert!( scene.state( knight, "attack" ).is_none() );
}

// ────────────────────────────────────────────────────────────────────────────
// Spawn / despawn lifecycle
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn spawn_then_despawn_invalidates_handle()
{
  let mut scene = Scene::new( build_spec() );
  let grass = scene.object( "grass" ).unwrap();
  let h = scene.spawn( grass, Placement::Hex { q : 1, r : 2 } );

  // Visible immediately, length 1.
  assert_eq!( scene.len(), 1 );
  let inst = scene.instance( h ).expect( "freshly spawned must be live" );
  assert!( inst.visible );
  assert_eq!( inst.object, grass );
  assert!( matches!( inst.placement, Placement::Hex { q : 1, r : 2 } ) );
  assert_eq!( inst.state, scene.default_state( grass ) );

  scene.despawn( h );
  assert_eq!( scene.len(), 0 );
  assert!( scene.instance( h ).is_none(), "stale handle returns None" );
}

#[ test ]
fn spawn_records_clock_in_spawn_time()
{
  let mut scene = Scene::new( build_spec() );
  let grass = scene.object( "grass" ).unwrap();

  scene.tick( 2.5 );
  let h = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  assert!( ( scene.instance( h ).unwrap().spawn_time - 2.5 ).abs() < 1e-6 );
}

// ────────────────────────────────────────────────────────────────────────────
// move_to + spatial-index maintenance
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn move_to_updates_spatial_index()
{
  let mut scene = Scene::new( build_spec() );
  let grass = scene.object( "grass" ).unwrap();
  let h = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );

  // Initially indexed at (0,0).
  assert_eq!( scene.instances_at_hex( 0, 0 ).count(), 1 );
  assert_eq!( scene.instances_at_hex( 3, 1 ).count(), 0 );

  scene.move_to( h, Placement::Hex { q : 3, r : 1 } );
  // Old cell is empty, new cell has the instance.
  assert_eq!( scene.instances_at_hex( 0, 0 ).count(), 0 );
  let at_new : Vec< _ > = scene.instances_at_hex( 3, 1 ).collect();
  assert_eq!( at_new, vec![ h ] );
  assert!( matches!( scene.instance( h ).unwrap().placement, Placement::Hex { q : 3, r : 1 } ) );
}

#[ test ]
fn move_to_changes_anchor_kind()
{
  // Hex → FreePos must move the handle out of the hex spatial index and
  // out of the hex_instances list, into free_instances. (Even though the
  // owning Object declares anchor: Hex — the renderer is what dispatches
  // by Placement variant; Scene is anchor-agnostic.)
  let mut scene = Scene::new( build_spec() );
  let grass = scene.object( "grass" ).unwrap();
  let h = scene.spawn( grass, Placement::Hex { q : 2, r : 2 } );

  assert_eq!( scene.hex_instances().len(), 1 );
  assert_eq!( scene.free_instances().len(), 0 );

  scene.move_to( h, Placement::FreePos { x : 100.0, y : 50.0 } );
  assert_eq!( scene.hex_instances().len(), 0 );
  assert_eq!( scene.free_instances(), &[ h ] );
  // No longer indexed by hex.
  assert_eq!( scene.instances_at_hex( 2, 2 ).count(), 0 );
}

#[ test ]
fn instances_at_hex_returns_only_matching()
{
  // Two instances on (0,0), one on (1,1). instances_at_hex must yield
  // only the two on (0,0), in spawn order; the other cell is independent.
  let mut scene = Scene::new( build_spec() );
  let grass = scene.object( "grass" ).unwrap();
  let knight = scene.object( "knight" ).unwrap();

  let g = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  let k = scene.spawn( knight, Placement::Hex { q : 0, r : 0 } );
  let _other = scene.spawn( grass, Placement::Hex { q : 1, r : 1 } );

  let at_origin : Vec< _ > = scene.instances_at_hex( 0, 0 ).collect();
  assert_eq!( at_origin, vec![ g, k ], "spawn order preserved" );
  assert_eq!( scene.instances_at_hex( 1, 1 ).count(), 1 );
  assert_eq!( scene.instances_at_hex( 9, 9 ).count(), 0 );
}

// ────────────────────────────────────────────────────────────────────────────
// Mutation API persistence
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn set_state_persists_for_same_object()
{
  let mut scene = Scene::new( build_spec() );
  let knight = scene.object( "knight" ).unwrap();
  let h = scene.spawn( knight, Placement::Hex { q : 0, r : 0 } );
  let walk = scene.state( knight, "walk" ).unwrap();
  scene.set_state( h, walk );
  assert_eq!( scene.instance( h ).unwrap().state, walk );
}

#[ test ]
fn set_visible_toggles()
{
  let mut scene = Scene::new( build_spec() );
  let grass = scene.object( "grass" ).unwrap();
  let h = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  assert!( scene.instance( h ).unwrap().visible );
  scene.set_visible( h, false );
  assert!( !scene.instance( h ).unwrap().visible );
  // Hidden instances stay in spatial index for cheap toggle-back.
  assert_eq!( scene.instances_at_hex( 0, 0 ).count(), 1 );
}

#[ test ]
fn set_tint_and_phase_offset_persist()
{
  let mut scene = Scene::new( build_spec() );
  let grass = scene.object( "grass" ).unwrap();
  let h = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );

  assert_eq!( scene.instance( h ).unwrap().tint, None );
  scene.set_tint( h, Some( [ 1.0, 0.5, 0.25, 1.0 ] ) );
  assert_eq!( scene.instance( h ).unwrap().tint, Some( [ 1.0, 0.5, 0.25, 1.0 ] ) );

  scene.set_phase_offset( h, Some( 0.4 ) );
  assert_eq!( scene.instance( h ).unwrap().phase_offset, Some( 0.4 ) );
  scene.set_phase_offset( h, None );
  assert_eq!( scene.instance( h ).unwrap().phase_offset, None );
}

#[ test ]
fn external_sprite_round_trip()
{
  let mut scene = Scene::new( build_spec() );
  let grass = scene.object( "grass" ).unwrap();
  let h = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );

  let body = SpriteRef { asset : "terrain".into(), frame : "1".into() };
  scene.set_external_sprite( h, "body", body.clone() );

  let inst = scene.instance( h ).unwrap();
  let got = inst.external_sprites.get( "body" ).expect( "body slot populated" );
  assert_eq!( got.asset, body.asset );
  assert_eq!( got.frame, body.frame );

  // Despawn clears external sprites by virtue of dropping the Instance.
  scene.despawn( h );
  assert!( scene.instance( h ).is_none() );
}

// ────────────────────────────────────────────────────────────────────────────
// Cross-object state misuse — must NOT mutate state in debug builds we
// catch with debug_assert; in release the instance state is left alone.
// We test the "instance state is unchanged" half (works in both modes).
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
#[ cfg( not( debug_assertions ) ) ]
fn set_state_rejects_foreign_state_in_release()
{
  let mut scene = Scene::new( build_spec() );
  let grass = scene.object( "grass" ).unwrap();
  let knight = scene.object( "knight" ).unwrap();
  let h_grass = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  let knight_walk = scene.state( knight, "walk" ).unwrap();

  let before = scene.instance( h_grass ).unwrap().state;
  scene.set_state( h_grass, knight_walk );
  let after = scene.instance( h_grass ).unwrap().state;
  assert_eq!( before, after, "foreign state must be ignored, not applied" );
}

#[ test ]
#[ cfg( debug_assertions ) ]
#[ should_panic( expected = "does not belong to instance" ) ]
fn set_state_rejects_foreign_state_in_debug()
{
  let mut scene = Scene::new( build_spec() );
  let grass = scene.object( "grass" ).unwrap();
  let knight = scene.object( "knight" ).unwrap();
  let h_grass = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  let knight_walk = scene.state( knight, "walk" ).unwrap();
  scene.set_state( h_grass, knight_walk );
}

// ────────────────────────────────────────────────────────────────────────────
// Per-anchor enumeration lists used by the future Renderer.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn per_anchor_lists_track_spawn_order()
{
  let mut scene = Scene::new( build_spec() );
  let grass = scene.object( "grass" ).unwrap();

  let a = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  let b = scene.spawn( grass, Placement::FreePos { x : 1.0, y : 1.0 } );
  let c = scene.spawn( grass, Placement::Viewport );

  assert_eq!( scene.hex_instances(), &[ a ] );
  assert_eq!( scene.free_instances(), &[ b ] );
  assert_eq!( scene.viewport_instances(), &[ c ] );
  assert_eq!( scene.edge_instances().len(), 0 );
  assert_eq!( scene.multihex_instances().len(), 0 );
}

// ────────────────────────────────────────────────────────────────────────────
// Revision counter — Step 4 plumbing for per-renderer delta detection.
// Every successful mutator must bump exactly once; `tick`, query getters,
// and clock-only reads must not.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn fresh_scene_has_zero_revision()
{
  let scene = Scene::new( build_spec() );
  assert_eq!( scene.revision(), 0 );
}

#[ test ]
fn every_mutator_bumps_revision_exactly_once()
{
  let mut scene = Scene::new( build_spec() );
  let grass = scene.object( "grass" ).unwrap();
  let knight = scene.object( "knight" ).unwrap();
  let knight_walk = scene.state( knight, "walk" ).unwrap();

  let mut expected = 0u64;
  let mut check = | scene : &Scene, label : &str |
  {
    expected += 1;
    assert_eq!
    (
      scene.revision(), expected,
      "{label} must bump revision exactly once",
    );
  };

  // spawn
  let h = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  check( &scene, "spawn" );

  let h_knight = scene.spawn( knight, Placement::Hex { q : 1, r : 0 } );
  check( &scene, "spawn (2nd)" );

  // move_to
  scene.move_to( h, Placement::Hex { q : 2, r : 0 } );
  check( &scene, "move_to" );

  // set_state (valid)
  scene.set_state( h_knight, knight_walk );
  check( &scene, "set_state" );

  // set_visible
  scene.set_visible( h, false );
  check( &scene, "set_visible" );

  // set_tint
  scene.set_tint( h, Some( [ 0.5, 1.0, 1.0, 1.0 ] ) );
  check( &scene, "set_tint" );

  // set_phase_offset
  scene.set_phase_offset( h, Some( 0.25 ) );
  check( &scene, "set_phase_offset" );

  // set_external_sprite
  let body = SpriteRef { asset : "terrain".into(), frame : "1".into() };
  scene.set_external_sprite( h, "body", body );
  check( &scene, "set_external_sprite" );

  // set_global_tint
  scene.set_global_tint( Some( tilemap_scene::TintRef( "foo".into() ) ) );
  check( &scene, "set_global_tint" );

  // set_seed
  scene.set_seed( 42 );
  check( &scene, "set_seed" );

  // despawn
  scene.despawn( h_knight );
  check( &scene, "despawn" );
}

#[ test ]
fn tick_does_not_bump_revision()
{
  let mut scene = Scene::new( build_spec() );
  let grass = scene.object( "grass" ).unwrap();
  let _h = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  let r_before = scene.revision();

  let _ = scene.tick( 0.5 );
  let _ = scene.tick( 1.5 );

  assert_eq!( scene.revision(), r_before, "tick must not bump revision" );
}

#[ test ]
fn query_getters_do_not_bump_revision()
{
  let mut scene = Scene::new( build_spec() );
  let grass = scene.object( "grass" ).unwrap();
  let h = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  let r = scene.revision();

  let _ = scene.clock();
  let _ = scene.len();
  let _ = scene.is_empty();
  let _ = scene.instance( h );
  let _ = scene.hex_instances();
  let _ = scene.edge_instances();
  let _ = scene.global_tint();
  let _ = scene.seed();
  let _ = scene.spec();
  let _ : Vec< _ > = scene.instances_at_hex( 0, 0 ).collect();

  assert_eq!( scene.revision(), r, "query getters must not bump revision" );
}

#[ cfg( not( debug_assertions ) ) ]
#[ test ]
fn set_state_with_foreign_state_does_not_bump_revision()
{
  let mut scene = Scene::new( build_spec() );
  let grass = scene.object( "grass" ).unwrap();
  let knight = scene.object( "knight" ).unwrap();
  let h_grass = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  let knight_walk = scene.state( knight, "walk" ).unwrap();
  let r = scene.revision();

  scene.set_state( h_grass, knight_walk );
  assert_eq!
  (
    scene.revision(), r,
    "rejected set_state must not bump revision",
  );
}
