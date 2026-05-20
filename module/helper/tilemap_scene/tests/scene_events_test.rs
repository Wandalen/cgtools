//! Integration tests for [`tilemap_scene::Scene::tick`] and the
//! [`tilemap_scene::SceneEvent`] stream — Step 3 of the Path-A retained
//! migration.
//!
//! Covers `OneShot` completion crossing detection, exclusion of `Loop` /
//! `PingPong`, per-instance phase overrides, multi-layer `OneShot`s,
//! invisible-instance suppression, the `dt == 0` and "very large dt"
//! degenerates, `HashCoord` phase-driven divergence between instances at
//! different grid coordinates, and the no-re-arm-on-`set_state` rule.

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
  Animation,
  AnimationMode,
  AnimationRef,
  AnimationTiming,
  Asset,
  AssetKind,
  HexConfig,
  LayerBehaviour,
  Object,
  ObjectLayer,
  PhaseOffset,
  PipelineLayer,
  Placement,
  RenderPipeline,
  RenderSpec,
  Scene,
  SceneEvent,
  SortMode,
  SpriteRef,
  SpriteSource,
  TilingStrategy,
};
use tilemap_renderer::types::{ MipmapMode, SamplerFilter, WrapMode };

// ────────────────────────────────────────────────────────────────────────────
// Fixture helpers — small spec with one animated object whose state's
// layer stack consists entirely of `SpriteSource::Animation` layers
// referencing animations declared at the spec root.
// ────────────────────────────────────────────────────────────────────────────

fn animation_layer( anim_id : &str ) -> ObjectLayer
{
  ObjectLayer
  {
    id : None,
    sprite_source : SpriteSource::Animation( AnimationRef( anim_id.into() ) ),
    behaviour : LayerBehaviour::default(),
    z_in_object : 0,
    pipeline_layer : None,
  }
}

fn static_layer() -> ObjectLayer
{
  ObjectLayer
  {
    id : None,
    sprite_source : SpriteSource::Static
    (
      SpriteRef { asset : "atlas".into(), frame : "0".into() }
    ),
    behaviour : LayerBehaviour::default(),
    z_in_object : 0,
    pipeline_layer : None,
  }
}

fn regular_animation
(
  id : &str,
  frame_count : u32,
  fps : f32,
  mode : AnimationMode,
  phase : PhaseOffset,
) -> Animation
{
  let frames : Vec< SpriteRef > = ( 0..frame_count )
    .map( | i | SpriteRef { asset : "atlas".into(), frame : i.to_string() } )
    .collect();
  Animation
  {
    id : id.into(),
    timing : AnimationTiming::Regular { frames, fps },
    mode,
    phase_offset : phase,
  }
}

/// Build a spec with `animations` declared at the root and an object
/// `"actor"` whose default state's layer stack is `layers_for_default`.
/// A second state `"alt"` carries a single static layer (used to test
/// that switching out of the `OneShot` state still doesn't double-fire on
/// switch back).
fn build_spec( animations : Vec< Animation >, layers_for_default : Vec< ObjectLayer > ) -> Arc< RenderSpec >
{
  let mut states = HashMap::default();
  states.insert( "default".into(), layers_for_default );
  states.insert( "alt".into(), vec![ static_layer() ] );

  let spec = RenderSpec
  {
    version : "0.2.0".into(),
    assets : vec!
    [
      Asset
      {
        id : "atlas".into(),
        path : "atlas.png".into(),
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
      },
    ],
    tints : Vec::new(),
    animations,
    effects : Vec::new(),
    objects : vec!
    [
      Object
      {
        id : "actor".into(),
        anchor : Anchor::Hex,
        global_layer : "main".into(),
        priority : None,
        sort_y_source : Default::default(),
        pivot : ( 0.5, 0.5 ),
        default_state : "default".into(),
        states,
      },
    ],
    pipeline : RenderPipeline
    {
      hex : HexConfig { tiling : TilingStrategy::HexFlatTop, grid_stride : ( 72, 64 ) },
      layers : vec!
      [
        PipelineLayer { id : "main".into(), sort : SortMode::None, tint_mask : None },
      ],
      global_tint : None,
      viewport_size : None,
      clear_color : None,
    },
  };
  Arc::new( spec )
}

/// A 5-frame Regular animation at 10 fps with the given mode and the
/// default `PhaseOffset::None` — total duration is exactly 0.5 s.
fn make_simple_scene( mode : AnimationMode ) -> ( Scene, tilemap_scene::InstanceHandle )
{
  let anim = regular_animation( "spawn_fx", 5, 10.0, mode, PhaseOffset::None );
  let spec = build_spec( vec![ anim ], vec![ animation_layer( "spawn_fx" ) ] );
  let mut scene = Scene::new( spec );
  let actor = scene.object( "actor" ).expect( "actor declared" );
  let h = scene.spawn( actor, Placement::Hex { q : 0, r : 0 } );
  ( scene, h )
}

fn completed_anim_id( ev : &SceneEvent ) -> &str
{
  let SceneEvent::AnimationCompleted { animation, .. } = ev
    else { panic!( "non-AnimationCompleted event: {ev:?}" ); };
  animation.0.as_str()
}

// ────────────────────────────────────────────────────────────────────────────
// Core completion semantics
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn oneshot_emits_completion_when_duration_crossed()
{
  let ( mut scene, h ) = make_simple_scene( AnimationMode::OneShot );

  // First tick stops short of duration (0.5 s) — no event.
  let evs1 = scene.tick( 0.3 );
  assert!( evs1.is_empty(), "tick(0.3) before duration: {evs1:?}" );

  // Second tick crosses 0.5 s mark — one event.
  let evs2 = scene.tick( 0.3 );
  assert_eq!( evs2.len(), 1 );
  let SceneEvent::AnimationCompleted { instance, layer_index, animation, .. } = &evs2[ 0 ]
    else { panic!( "expected AnimationCompleted: {:?}", evs2[ 0 ] ); };
  assert_eq!( *instance, h );
  assert_eq!( *layer_index, 0 );
  assert_eq!( animation.0, "spawn_fx" );
}

#[ test ]
fn oneshot_no_event_before_completion()
{
  let ( mut scene, _h ) = make_simple_scene( AnimationMode::OneShot );
  // duration = 0.5; tick to 0.49 — no crossing.
  let evs = scene.tick( 0.49 );
  assert!( evs.is_empty(), "{evs:?}" );
}

#[ test ]
fn oneshot_no_repeat_after_completion()
{
  let ( mut scene, _h ) = make_simple_scene( AnimationMode::OneShot );
  let first = scene.tick( 0.6 );
  assert_eq!( first.len(), 1, "first tick crosses: {first:?}" );
  let second = scene.tick( 0.6 );
  assert!( second.is_empty(), "second tick must not re-fire: {second:?}" );
  let third = scene.tick( 5.0 );
  assert!( third.is_empty(), "long tick past completion still silent: {third:?}" );
}

#[ test ]
fn loop_animation_never_emits()
{
  let ( mut scene, _h ) = make_simple_scene( AnimationMode::Loop );
  // Tick across many periods.
  for _ in 0..10
  {
    let evs = scene.tick( 0.7 );
    assert!( evs.is_empty(), "Loop must never emit completion: {evs:?}" );
  }
}

#[ test ]
fn pingpong_never_emits()
{
  let ( mut scene, _h ) = make_simple_scene( AnimationMode::PingPong );
  for _ in 0..10
  {
    let evs = scene.tick( 0.7 );
    assert!( evs.is_empty(), "PingPong must never emit completion: {evs:?}" );
  }
}

// ────────────────────────────────────────────────────────────────────────────
// Per-instance phase offset
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn per_instance_phase_offset_shifts_completion()
{
  // Same animation, two instances. Default-phase one fires; the other
  // gets a negative phase offset that pushes its completion *later*.
  let anim = regular_animation( "spawn_fx", 5, 10.0, AnimationMode::OneShot, PhaseOffset::None );
  let spec = build_spec( vec![ anim ], vec![ animation_layer( "spawn_fx" ) ] );
  let mut scene = Scene::new( spec );
  let actor = scene.object( "actor" ).unwrap();

  let early = scene.spawn( actor, Placement::Hex { q : 0, r : 0 } );
  let late = scene.spawn( actor, Placement::Hex { q : 1, r : 0 } );
  // Negative offset → local time runs 0.2 s *behind* the master clock.
  scene.set_phase_offset( late, Some( -0.2 ) );

  // First tick to exactly 0.5: early instance crosses (0.0 → 0.5), late
  // is at local 0.3 (still under duration). Exactly one event, for early.
  let evs = scene.tick( 0.5 );
  assert_eq!( evs.len(), 1, "early-instance crossing only: {evs:?}" );
  let SceneEvent::AnimationCompleted { instance, .. } = &evs[ 0 ]
    else { panic!( "non-AnimationCompleted: {:?}", evs[ 0 ] ); };
  assert_eq!( *instance, early );

  // Next tick to 0.7: late local goes 0.3 → 0.5 → crossing.
  let evs2 = scene.tick( 0.2 );
  assert_eq!( evs2.len(), 1, "late-instance crossing: {evs2:?}" );
  let SceneEvent::AnimationCompleted { instance, .. } = &evs2[ 0 ]
    else { panic!( "non-AnimationCompleted: {:?}", evs2[ 0 ] ); };
  assert_eq!( *instance, late );
}

// ────────────────────────────────────────────────────────────────────────────
// Multi-layer states
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn multiple_oneshot_layers_emit_per_layer()
{
  // Two OneShot animations of different durations stacked in the same
  // state. One tick large enough to complete both → two events with
  // distinct layer_index values.
  let short = regular_animation( "short", 2, 10.0, AnimationMode::OneShot, PhaseOffset::None );
  // 2 frames @ 10fps = 0.2 s duration
  let long = regular_animation( "long", 6, 10.0, AnimationMode::OneShot, PhaseOffset::None );
  // 6 frames @ 10fps = 0.6 s duration
  let spec = build_spec
  (
    vec![ short, long ],
    vec![ animation_layer( "short" ), animation_layer( "long" ) ],
  );
  let mut scene = Scene::new( spec );
  let actor = scene.object( "actor" ).unwrap();
  let _h = scene.spawn( actor, Placement::Hex { q : 0, r : 0 } );

  let evs = scene.tick( 1.0 );
  assert_eq!( evs.len(), 2, "both OneShots fire in the same tick: {evs:?}" );
  let mut layer_indices : Vec< u16 > = evs.iter()
    .map
    (
      | ev |
      {
        let SceneEvent::AnimationCompleted { layer_index, .. } = ev
          else { panic!( "non-AnimationCompleted: {ev:?}" ); };
        *layer_index
      }
    )
    .collect();
  layer_indices.sort_unstable();
  assert_eq!( layer_indices, vec![ 0, 1 ] );
  let mut anims : Vec< &str > = evs.iter().map( completed_anim_id ).collect();
  anims.sort_unstable();
  assert_eq!( anims, vec![ "long", "short" ] );
}

// ────────────────────────────────────────────────────────────────────────────
// Visibility & degenerate ticks
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn invisible_instance_does_not_emit()
{
  let ( mut scene, h ) = make_simple_scene( AnimationMode::OneShot );
  scene.set_visible( h, false );
  let evs = scene.tick( 5.0 );
  assert!( evs.is_empty(), "hidden instance suppresses event: {evs:?}" );
}

#[ test ]
fn tick_zero_emits_nothing_and_does_not_advance_clock()
{
  let ( mut scene, _h ) = make_simple_scene( AnimationMode::OneShot );
  let before = scene.clock();
  let evs = scene.tick( 0.0 );
  assert!( evs.is_empty() );
  assert!( ( scene.clock() - before ).abs() < f32::EPSILON );
}

#[ test ]
fn tick_large_dt_emits_in_one_call()
{
  let ( mut scene, _h ) = make_simple_scene( AnimationMode::OneShot );
  // dt many times the duration: still exactly one boundary crossing.
  let evs = scene.tick( 100.0 );
  assert_eq!( evs.len(), 1, "single boundary crossing: {evs:?}" );
}

// ────────────────────────────────────────────────────────────────────────────
// HashCoord phase divergence
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn hash_coord_phase_can_separate_completions()
{
  // HashCoord spreads phase across the animation's natural period.
  // Use Linear here — easier to assert: per_q = -0.2s → instance at
  // (1, 0) has phase = -0.2s, completing 0.2s later than (0, 0).
  let anim = regular_animation
  (
    "spawn_fx",
    5,
    10.0,
    AnimationMode::OneShot,
    PhaseOffset::Linear { per_q : -0.2, per_r : 0.0 },
  );
  let spec = build_spec( vec![ anim ], vec![ animation_layer( "spawn_fx" ) ] );
  let mut scene = Scene::new( spec );
  let actor = scene.object( "actor" ).unwrap();
  let a = scene.spawn( actor, Placement::Hex { q : 0, r : 0 } );
  let b = scene.spawn( actor, Placement::Hex { q : 1, r : 0 } );

  // Tick to 0.5 — instance a (phase 0) crosses duration 0.5, b (phase
  // -0.2) is at local 0.3 — still in flight.
  let evs = scene.tick( 0.5 );
  assert_eq!( evs.len(), 1 );
  let SceneEvent::AnimationCompleted { instance, .. } = &evs[ 0 ]
    else { panic!( "non-AnimationCompleted: {:?}", evs[ 0 ] ); };
  assert_eq!( *instance, a );

  // Tick another 0.2 — b crosses.
  let evs2 = scene.tick( 0.2 );
  assert_eq!( evs2.len(), 1 );
  let SceneEvent::AnimationCompleted { instance, .. } = &evs2[ 0 ]
    else { panic!( "non-AnimationCompleted: {:?}", evs2[ 0 ] ); };
  assert_eq!( *instance, b );
}

// ────────────────────────────────────────────────────────────────────────────
// State-switch re-arm rule
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn state_switch_re_arms_oneshot()
{
  // `set_state` resets `state_entered_time` on the instance, so
  // re-entering a OneShot state restarts its animation from frame 0
  // and re-arms its `AnimationCompleted` event. This is the load-bearing
  // semantics for attack/death/pulse animations on long-lived instances.
  let ( mut scene, h ) = make_simple_scene( AnimationMode::OneShot );

  // Tick across completion — one event.
  let first = scene.tick( 0.6 );
  assert_eq!( first.len(), 1 );

  // Switch to alt (static layer) and back to default — `set_state` on
  // the second call resets the OneShot clock.
  let actor = scene.object( "actor" ).unwrap();
  let alt = scene.state( actor, "alt" ).unwrap();
  let default = scene.state( actor, "default" ).unwrap();
  scene.set_state( h, alt );
  scene.set_state( h, default );

  // Tick under duration — no crossing yet.
  let evs_mid = scene.tick( 0.3 );
  assert!( evs_mid.is_empty(), "below new-state duration: {evs_mid:?}" );

  // Cross the new state's 0.5 s duration — event re-fires.
  let evs2 = scene.tick( 0.3 );
  assert_eq!( evs2.len(), 1, "re-entering OneShot state re-arms: {evs2:?}" );
}

#[ test ]
fn oneshot_restarts_on_set_state_after_delay()
{
  // An instance lives in a non-OneShot state for much longer than
  // the OneShot duration, then switches into the OneShot state. The
  // OneShot must play from frame 0 (not the last frame) and emit
  // `AnimationCompleted` exactly once after its duration.
  let anim = regular_animation( "spawn_fx", 5, 10.0, AnimationMode::OneShot, PhaseOffset::None );
  // Default state is static; the OneShot lives on `alt`. Swap the
  // usual fixture so default-spawn doesn't pre-emit a completion.
  let spec =
  {
    let mut states = HashMap::default();
    states.insert( "default".into(), vec![ static_layer() ] );
    states.insert( "alt".into(), vec![ animation_layer( "spawn_fx" ) ] );
    let spec = RenderSpec
    {
      version : "0.2.0".into(),
      assets : vec!
      [
        Asset
        {
          id : "atlas".into(),
          path : "atlas.png".into(),
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
        },
      ],
      tints : Vec::new(),
      animations : vec![ anim ],
      effects : Vec::new(),
      objects : vec!
      [
        Object
        {
          id : "actor".into(),
          anchor : Anchor::Hex,
          global_layer : "main".into(),
          priority : None,
          sort_y_source : Default::default(),
          pivot : ( 0.5, 0.5 ),
          default_state : "default".into(),
          states,
        },
      ],
      pipeline : RenderPipeline
      {
        hex : HexConfig { tiling : TilingStrategy::HexFlatTop, grid_stride : ( 72, 64 ) },
        layers : vec!
        [
          PipelineLayer { id : "main".into(), sort : SortMode::None, tint_mask : None },
        ],
        global_tint : None,
        viewport_size : None,
        clear_color : None,
      },
    };
    Arc::new( spec )
  };
  let mut scene = Scene::new( spec );
  let actor = scene.object( "actor" ).unwrap();
  let h = scene.spawn( actor, Placement::Hex { q : 0, r : 0 } );

  // 10 s of gameplay in the static state — no events.
  let evs_idle = scene.tick( 10.0 );
  assert!( evs_idle.is_empty(), "static state must not emit: {evs_idle:?}" );

  // Trigger the OneShot.
  let alt = scene.state( actor, "alt" ).unwrap();
  scene.set_state( h, alt );

  // Half-way through the 0.5 s duration — not crossed yet.
  let evs_mid = scene.tick( 0.3 );
  assert!( evs_mid.is_empty(), "below new-state duration: {evs_mid:?}" );

  // Cross duration — exactly one event.
  let evs_done = scene.tick( 0.3 );
  assert_eq!
  (
    evs_done.len(), 1,
    "OneShot must complete `duration` seconds after set_state, not silently freeze: {evs_done:?}",
  );
}

// ────────────────────────────────────────────────────────────────────────────
// `PhaseOffset::Instance` — placement-agnostic stagger
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn phase_offset_instance_staggers_freepos_completions()
{
  // Two `Placement::FreePos` instances of the same OneShot animation.
  // `PhaseOffset::HashCoord` would degenerate (both hash `(0, 0)` →
  // same phase) but `PhaseOffset::Instance` derives phase from
  // per-instance seeds so the two instances complete in different
  // ticks. The exact ordering depends on the hash, but at least one
  // tick must distinguish them.
  let anim = regular_animation( "spawn_fx", 5, 10.0, AnimationMode::OneShot, PhaseOffset::Instance );
  let spec = build_spec( vec![ anim ], vec![ animation_layer( "spawn_fx" ) ] );
  let mut scene = Scene::new( spec );
  let actor = scene.object( "actor" ).unwrap();

  // Spawn 8 instances. Their phases come from instance_phase_seed,
  // not from grid coords, so two FreePos placements at the same
  // pixel still get different phases.
  let mut handles = Vec::new();
  for _ in 0..8
  {
    handles.push( scene.spawn( actor, Placement::FreePos { x : 0.0, y : 0.0 } ) );
  }

  // Tick enough to cross duration (0.5 s) for instances whose phase
  // puts them on the early side. Collect events from many small
  // ticks so completions fall across at least two distinct ticks.
  let mut tick_buckets : Vec< usize > = Vec::new();
  let mut total_completed = 0;
  for _ in 0..20
  {
    let evs = scene.tick( 0.1 );
    tick_buckets.push( evs.len() );
    total_completed += evs.len();
  }

  // All 8 must complete eventually.
  assert_eq!( total_completed, 8, "every instance must complete: {tick_buckets:?}" );
  // At least two ticks must have produced completions — i.e. the
  // phase actually spread the events in time. If `Instance` were
  // degenerate the way `HashCoord` is for FreePos, every completion
  // would have landed on the same tick.
  let ticks_with_events = tick_buckets.iter().filter( | &&n | n > 0 ).count();
  assert!
  (
    ticks_with_events >= 2,
    "PhaseOffset::Instance must spread per-instance phases across at least two ticks; \
     got {tick_buckets:?}",
  );
}

// ────────────────────────────────────────────────────────────────────────────
// `tick_into` — caller-owned buffer
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn tick_into_appends_to_caller_buffer()
{
  use tilemap_scene::AnimationRef;

  let ( mut scene, h ) = make_simple_scene( AnimationMode::OneShot );

  // Pre-populate the buffer with a sentinel that must survive the
  // call — `tick_into` is a *push*, not a *replace*.
  let mut events : Vec< SceneEvent > = vec!
  [
    SceneEvent::AnimationCompleted
    {
      instance : h,
      state : scene.instance( h ).unwrap().state,
      layer_index : 99,
      animation : AnimationRef( "sentinel".into() ),
    },
  ];

  // Tick under duration — buffer unchanged in length except the sentinel.
  scene.tick_into( 0.3, &mut events );
  assert_eq!( events.len(), 1, "no event yet, sentinel preserved" );

  // Cross duration — one real event pushed *after* the sentinel.
  scene.tick_into( 0.3, &mut events );
  assert_eq!( events.len(), 2 );
  let SceneEvent::AnimationCompleted { animation, .. } = &events[ 0 ]
    else { panic!() };
  assert_eq!( animation.0, "sentinel", "sentinel still at index 0" );
  let SceneEvent::AnimationCompleted { animation, .. } = &events[ 1 ]
    else { panic!() };
  assert_eq!( animation.0, "spawn_fx", "real event appended" );
}

// ────────────────────────────────────────────────────────────────────────────
// Determinism / payload shape
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn event_payload_carries_expected_state_handle()
{
  let ( mut scene, h ) = make_simple_scene( AnimationMode::OneShot );
  let evs = scene.tick( 0.6 );
  assert_eq!( evs.len(), 1 );
  let SceneEvent::AnimationCompleted { instance, state, .. } = &evs[ 0 ]
    else { panic!( "non-AnimationCompleted: {:?}", evs[ 0 ] ); };
  assert_eq!( *instance, h );
  assert_eq!( *state, scene.instance( h ).unwrap().state );
}
