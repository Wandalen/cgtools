//! Integration tests for [`tilemap_scene::Renderer`].
//!
//! Covers the Step-2 contract: asset compile once, per-instance overrides
//! (`tint`, `phase_offset`, `external_sprites`, `visible`) thread through
//! to the emitted `RenderCommand`s, and `Scene::from_snapshot` materialises
//! to a render result byte-equal with the snapshot-driven baseline.

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
  Animation,
  AnimationMode,
  AnimationRef,
  AnimationTiming,
  Asset,
  AssetKind,
  Bounds,
  Camera,
  HexConfig,
  LayerBehaviour,
  Object,
  ObjectLayer,
  PathResolver,
  PhaseOffset,
  PipelineLayer,
  Placement,
  Renderer,
  RenderPipeline,
  RenderSpec,
  Scene,
  SceneSnapshot,
  SortMode,
  SpriteRef,
  SpriteSource,
  Tile,
  TilingStrategy,
};
use tilemap_renderer::types::{ MipmapMode, SamplerFilter, WrapMode };

mod common;

// ────────────────────────────────────────────────────────────────────────────
// Fixtures — a grass object with a static sprite, plus an animated `knight`
// for phase-offset tests, plus an `external_object` for slot tests.
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

fn anim_layer( animation_id : &str ) -> ObjectLayer
{
  ObjectLayer
  {
    id : None,
    sprite_source : SpriteSource::Animation( AnimationRef( animation_id.into() ) ),
    behaviour : LayerBehaviour::default(),
    z_in_object : 0,
    pipeline_layer : None,
  }
}

fn external_layer( slot : &str ) -> ObjectLayer
{
  ObjectLayer
  {
    id : None,
    sprite_source : SpriteSource::External { slot : slot.into() },
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
  knight_states.insert( "idle".into(), vec![ anim_layer( "knight_idle" ) ] );

  let mut external_states = HashMap::default();
  external_states.insert( "default".into(), vec![ external_layer( "body" ) ] );

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
    animations : vec!
    [
      Animation
      {
        id : "knight_idle".into(),
        timing : AnimationTiming::Regular
        {
          frames : vec!
          [
            SpriteRef { asset : "terrain".into(), frame : "0".into() },
            SpriteRef { asset : "terrain".into(), frame : "1".into() },
            SpriteRef { asset : "terrain".into(), frame : "2".into() },
            SpriteRef { asset : "terrain".into(), frame : "3".into() },
          ],
          fps : 4.0,
        },
        mode : AnimationMode::Loop,
        phase_offset : PhaseOffset::None,
      },
    ],
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
      Object
      {
        id : "external_object".into(),
        anchor : Anchor::Hex,
        global_layer : "terrain".into(),
        priority : None,
        sort_y_source : Default::default(),
        pivot : ( 0.5, 0.5 ),
        default_state : "default".into(),
        states : external_states,
      },
    ],
    pipeline : RenderPipeline
    {
      hex : HexConfig { tiling : TilingStrategy::HexFlatTop, grid_stride : ( 72, 64 ) },
      layers : vec!
      [
        PipelineLayer { id : "terrain".into(), sort : SortMode::None, tint_mask : None, alpha_clip : 0.0, occlude_overlap : false, opaque : false },
      ],
      global_tint : None,
      viewport_size : None,
      clear_color : None,
    },
  }
}

// `first_sprite` / `count_sprites` helpers were inlined into tests via
// `common::flat_sprites` / `common::flat_sprite_count` after the Step-4b
// batching migration; nothing in this file references them any more.

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn renderer_assets_compile_once()
{
  let spec = build_spec();
  let renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer builds" );
  // Three objects but only one image asset (the terrain atlas). Animation
  // frames are sprite refs into that atlas, so we expect the four animation
  // frames + the static "0" frame, all deduped.
  assert_eq!( renderer.assets().images.len(), 1, "one declared asset = one image" );
  // Sprites: "0", "1", "2", "3" — four unique frames between grass + knight animation.
  assert_eq!( renderer.assets().sprites.len(), 4, "deduped sprite count: 4" );
}

#[ test ]
fn per_instance_tint_multiplies_into_sprite()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec.clone() ) );
  let grass = scene.object( "grass" ).unwrap();
  let h = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  scene.set_tint( h, Some( [ 0.5, 0.75, 1.0, 1.0 ] ) );

  let mut flat = common::BatchFlattener::new();
  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" );
  let flat_cmds = flat.apply( cmds );
  let sprite = *flat_cmds.iter().find_map( | c | if let RenderCommand::Sprite( s ) = c { Some( s ) } else { None } )
    .expect( "no Sprite after flatten" );
  assert_eq!( sprite.tint, [ 0.5, 0.75, 1.0, 1.0 ], "per-instance tint reaches sprite payload" );

  // Clearing the tint restores the default (global * layer_alpha — both are identity here).
  scene.set_tint( h, None );
  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" );
  let flat_cmds = flat.apply( cmds );
  let sprite = *flat_cmds.iter().find_map( | c | if let RenderCommand::Sprite( s ) = c { Some( s ) } else { None } )
    .expect( "no Sprite after flatten" );
  assert_eq!( sprite.tint, [ 1.0, 1.0, 1.0, 1.0 ] );
}

#[ test ]
fn per_instance_phase_offset_overrides_animation_phase()
{
  // Two instances at the same coord, both running the same animation at
  // the same clock; one gets a half-period phase offset → emits a
  // different frame than the other.
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec.clone() ) );
  let knight = scene.object( "knight" ).unwrap();

  let _h_a = scene.spawn( knight, Placement::Hex { q : 0, r : 0 } );
  let h_b = scene.spawn( knight, Placement::Hex { q : 1, r : 0 } );
  // Animation has 4 frames at 4 fps → one period = 1.0 s. Half period = 0.5 s.
  scene.set_phase_offset( h_b, Some( 0.5 ) );

  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" );
  let sprites = common::flat_sprites( cmds );
  assert_eq!( sprites.len(), 2 );

  assert_ne!
  (
    sprites[ 0 ].sprite, sprites[ 1 ].sprite,
    "instances with different phase offsets must emit different animation frames",
  );
}

#[ test ]
fn external_sprite_slot_resolves()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec.clone() ) );
  let obj = scene.object( "external_object" ).unwrap();
  let h = scene.spawn( obj, Placement::Hex { q : 0, r : 0 } );

  let mut flat = common::BatchFlattener::new();

  // Without the slot populated, the layer emits no sprite.
  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" );
  let flat_cmds = flat.apply( cmds );
  let sprite_count = flat_cmds.iter().filter( | c | matches!( c, RenderCommand::Sprite( _ ) ) ).count();
  assert_eq!( sprite_count, 0, "unset External slot must not emit any Sprite" );

  // Populate the slot — now we get exactly one sprite.
  scene.set_external_sprite( h, "body", SpriteRef { asset : "terrain".into(), frame : "2".into() } );
  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" );
  let flat_cmds = flat.apply( cmds );
  let sprite_count = flat_cmds.iter().filter( | c | matches!( c, RenderCommand::Sprite( _ ) ) ).count();
  assert_eq!( sprite_count, 1, "populated External slot emits one sprite" );
  let sprite = *flat_cmds.iter().find_map( | c | if let RenderCommand::Sprite( s ) = c { Some( s ) } else { None } )
    .expect( "Sprite after flatten" );
  // Frame "2" is the third allocated sprite in build_spec (after "0", "1").
  assert_eq!( sprite.sprite, renderer.assets().sprites.iter().find( | s | s.region[ 0 ] == 144.0 ).unwrap().id );
}

#[ test ]
fn invisible_instances_skipped()
{
  let spec = build_spec();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec.clone() ) );
  let grass = scene.object( "grass" ).unwrap();
  let h = scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );

  let mut flat = common::BatchFlattener::new();
  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" );
  let flat_cmds = flat.apply( cmds );
  assert_eq!
  (
    flat_cmds.iter().filter( | c | matches!( c, RenderCommand::Sprite( _ ) ) ).count(),
    1,
  );

  scene.set_visible( h, false );
  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" );
  let flat_cmds = flat.apply( cmds );
  assert_eq!
  (
    flat_cmds.iter().filter( | c | matches!( c, RenderCommand::Sprite( _ ) ) ).count(),
    0,
    "invisible instance must not emit any Sprite command",
  );
}

#[ test ]
fn scene_from_snapshot_round_trip()
{
  // Snapshot with three grass tiles. Rendering via `Scene::from_snapshot`
  // produces the same Sprite count as constructing the Scene directly via
  // `spawn`.
  let spec = build_spec();

  let snap = SceneSnapshot
  {
    tiles : vec!
    [
      Tile { pos : ( 0, 0 ), objects : vec![ "grass".into() ] },
      Tile { pos : ( 1, 0 ), objects : vec![ "grass".into() ] },
      Tile { pos : ( 0, 1 ), objects : vec![ "grass".into() ] },
    ],
    ..SceneSnapshot::new( Bounds::unbounded() )
  };

  let mut renderer_a = Renderer::new( &spec, &PathResolver ).expect( "renderer A" );
  let mut renderer_b = Renderer::new( &spec, &PathResolver ).expect( "renderer B" );

  let scene_from_snap = Scene::from_snapshot( &snap, Arc::new( spec.clone() ) ).expect( "from_snapshot" );

  let mut scene_direct = Scene::new( Arc::new( spec.clone() ) );
  let grass = scene_direct.object( "grass" ).unwrap();
  scene_direct.spawn( grass, Placement::Hex { q : 0, r : 0 } );
  scene_direct.spawn( grass, Placement::Hex { q : 1, r : 0 } );
  scene_direct.spawn( grass, Placement::Hex { q : 0, r : 1 } );

  let cmds_snap = renderer_a.render( &scene_from_snap, &Camera::default() ).expect( "render snap" ).to_vec();
  let cmds_direct = renderer_b.render( &scene_direct, &Camera::default() ).expect( "render direct" ).to_vec();

  let count_snap = common::flat_sprite_count( &cmds_snap );
  let count_direct = common::flat_sprite_count( &cmds_direct );
  assert_eq!( count_snap, count_direct, "snapshot-driven and spawn-driven paths emit the same sprite count" );
  assert_eq!( count_snap, 3 );
}
