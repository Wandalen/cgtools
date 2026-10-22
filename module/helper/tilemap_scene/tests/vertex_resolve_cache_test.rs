//! The revision-keyed vertex resolve cache and the per-frame vertex effects:
//! `Scene::tiles_revision`, re-resolving `VertexCorners` only when hex cells
//! change and keeping a same-cell move from reordering a cell (BUG-540),
//! re-enabling a disabled layer, `EffectKind::FadeGate` easing and
//! idle-replay invalidation, and `AlphaPulse.restart_on_spawn` restarting the
//! wave at a spawn.

#![ allow( clippy::float_cmp, reason = "the tint assertions check exact pass-through of constant tints and alphas the fade reaches exactly; epsilon comparison would weaken them" ) ]

extern crate alloc;

use alloc::sync::Arc;
use rustc_hash::FxHashMap as HashMap;

mod common;
use common::compile::{ atlas_with_frames, dual_orient_spec, minimal_scene_3x3, minimal_spec, sprite_commands };

use tilemap_renderer::commands::RenderCommand;
use tilemap_scene::
{
  Anchor,
  Asset,
  Camera,
  Effect,
  EffectKind,
  EffectRef,
  LayerBehaviour,
  MipmapMode,
  Object,
  ObjectLayer,
  PathResolver,
  PhaseOffset,
  Placement,
  Renderer,
  RenderSpec,
  SamplerFilter,
  Scene,
  SceneSnapshot,
  SortYSource,
  SpriteRef,
  SpriteSource,
  Tile,
  TriBlendPattern,
  WrapMode,
  assets_compile,
};

/// Spec + scene for a single dual-mesh vertex blend: grass at (0,0), sand at
/// (1,-1), water at (0,-1) share exactly one triangle, which a `blend`
/// `VertexCorners` object on the `terrain` layer turns into a `tri_gsw_{rot}`
/// sprite.
#[ expect( clippy::too_many_lines, reason = "linear fixture build; splitting would scatter one spec across helpers" ) ]
fn three_way_blend_fixture() -> ( RenderSpec, SceneSnapshot )
{
  let mut spec = minimal_spec();
  spec.assets.push
  (
    Asset
    {
      id : "blends".into(),
      path : "blends.png".into(),
      kind : atlas_with_frames
      (
        8,
        &[
          ( "tri_gsw_0", ( 0, 0 ) ),
          ( "tri_gsw_1", ( 1, 0 ) ),
          ( "tri_gsw_2", ( 2, 0 ) ),
        ],
      ),
      filter : SamplerFilter::default(),
      mipmap : MipmapMode::default(),
      wrap : WrapMode::default(),
      premultiplied : false,
    }
  );

  // Terrains grass/sand/water.
  for ( id, prio ) in [ ( "sand", 8 ), ( "water", 5 ) ]
  {
    spec.objects.push( Object
    {
      id : id.into(),
      anchor : Anchor::Hex,
      global_layer : "terrain".into(),
      priority : Some( prio ),
      sort_y_source : SortYSource::default(),
      pivot : ( 0.5, 0.5 ),
      default_state : "default".into(),
      states :
      {
        let mut m = HashMap::default();
        m.insert
        (
          "default".into(),
          vec!
          [
            ObjectLayer
            {
              id : None,
              sprite_source : SpriteSource::Static( SpriteRef { asset : "terrain".into(), frame : "0".into() } ),
              behaviour : LayerBehaviour::default(),
              z_in_object : 0,
              pipeline_layer : None,
            },
          ],
        );
        m
      },
    });
  }

  // VertexCorners object — its own default animation has a single layer
  // with a pattern that matches the gss/sand/water triple.
  spec.objects.push( Object
  {
    id : "blend".into(),
    anchor : Anchor::Hex,   // anchor type of the owning object doesn't matter for VertexCorners pass
    global_layer : "terrain".into(),
    priority : None,
    sort_y_source : SortYSource::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states :
    {
      let mut m = HashMap::default();
      m.insert
      (
        "default".into(),
        vec!
        [
          ObjectLayer
          {
            id : None,
            sprite_source : SpriteSource::VertexCorners
            {
              patterns : vec!
              [
                TriBlendPattern
                {
                  corners : ( "grass".into(), "sand".into(), "water".into() ),
                  sprite_pattern : "tri_gsw_{rot}".into(),
                  priority : 10,
                  animation : None,
                },
              ],
              asset : "blends".into(),
              orient_to_grid : false,
              corner_source : None,
              offset : None,
            },
            behaviour : LayerBehaviour::default(),
            z_in_object : 0,
            pipeline_layer : None,
          },
        ],
      );
      m
    },
  });
  // Instantiate the blend object once on any tile so the VertexCorners pass
  // finds it during bucket emission. (Object presence is what matters, not
  // the tile — vertex sprites are global per-bucket.)
  let scene = SceneSnapshot
  {
    tiles : vec!
    [
      Tile { pos : ( 0,  0 ), objects : vec![ "grass".into(), "blend".into() ] },
      Tile { pos : ( 1, -1 ), objects : vec![ "sand".into() ] },
      Tile { pos : ( 0, -1 ), objects : vec![ "water".into() ] },
    ],
    ..minimal_scene_3x3()
  };
  ( spec, scene )
}

/// Whether `cmds` contains any rotation of [`three_way_blend_fixture`]'s blend sprite.
fn has_three_way_blend( spec : &RenderSpec, cmds : &[ RenderCommand ] ) -> bool
{
  let compiled = assets_compile( spec, &PathResolver ).expect( "assets" );
  let sprite_ids : std::collections::HashSet< _ > = cmds.iter().filter_map( | c |
    if let RenderCommand::Sprite( s ) = c { Some( s.sprite ) } else { None }
  ).collect();
  ( 0..3 ).any( | r |
    compiled.ids.sprite( "blends", &format!( "tri_gsw_{r}" ) ).is_some_and( | id | sprite_ids.contains( &id ) )
  )
}

/// The first `fade_target_set( id, false )` snaps the gate from the implicit
/// `1.0` to `0.0` without a tick, so the clock does not move. A renderer that
/// already drew the visible frame must still re-walk the scene instead of
/// replaying it from the idle cache.
#[ test ]
fn fade_gate_first_hide_invalidates_idle_replay()
{
  let mut spec = dual_orient_spec();
  spec.effects.push( Effect
  {
    id : "dim_fade".into(),
    kind : EffectKind::FadeGate { duration_ms : 100.0 },
    phase_offset : PhaseOffset::None,
  });
  let stack = spec.objects[ 0 ].states.get_mut( "default" ).expect( "default state" );
  stack[ 0 ].behaviour.effects.push( EffectRef( "dim_fade".into() ) );

  let scene_snap = SceneSnapshot
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "hexagon".into() ] } ],
    ..minimal_scene_3x3()
  };
  let mut scene = Scene::from_snapshot( &scene_snap, Arc::new( spec.clone() ) ).expect( "scene" );
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut flattener = common::BatchFlattener::new();

  let _ = flattener.apply( renderer.render( &scene, &Camera::default() ).expect( "render" ) );
  let walks = renderer.scene_walks();

  scene.fade_target_set( "dim_fade", false );
  let hidden_cmds = flattener.apply( renderer.render( &scene, &Camera::default() ).expect( "render" ) );
  assert_eq!( renderer.scene_walks(), walks + 1, "the first hide must miss the idle-replay cache" );
  let hidden = sprite_commands( &hidden_cmds );
  assert!( !hidden.is_empty(), "lone hex must still emit corner sprites" );
  for s in &hidden
  {
    assert!( s.tint[ 3 ].abs() < 1e-5, "the re-walked frame must be fully transparent, got {:?}", s.tint );
  }
}

/// `fade_target_set` reports whether the spec declares a `FadeGate` with the
/// id: an undeclared id (a typo, or a gate the spec dropped) changes nothing
/// and returns `false`, instead of creating a gate no layer reads that would
/// ease at a made-up duration.
#[ test ]
fn fade_target_set_reports_an_undeclared_effect()
{
  let mut spec = dual_orient_spec();
  spec.effects.push( Effect
  {
    id : "dim_fade".into(),
    kind : EffectKind::FadeGate { duration_ms : 100.0 },
    phase_offset : PhaseOffset::None,
  });
  spec.effects.push( Effect
  {
    id : "pulse".into(),
    kind : EffectKind::AlphaPulse { min : 0.0, max : 1.0, frequency : 1.0, restart_on_spawn : false },
    phase_offset : PhaseOffset::None,
  });
  let mut scene = Scene::new( Arc::new( spec ) );
  let revision = scene.revision();

  assert!( !scene.fade_target_set( "dim_fdae", false ), "a typo names no effect" );
  assert!( !scene.fade_target_set( "pulse", false ), "a pulse is not a fade gate" );
  assert_eq!( scene.revision(), revision, "nothing changed" );
  assert_eq!( scene.fade_value( "dim_fdae" ), 1.0 );

  assert!( scene.fade_target_set( "dim_fade", false ), "the declared gate is driven" );
  assert_eq!( scene.fade_value( "dim_fade" ), 0.0 );
  assert_ne!( scene.revision(), revision );
}

/// `Scene::tiles_revision` moves only when the set of visible objects standing in
/// hex cells changes — `FreePos` moves, same-cell moves, states and tints leave it.
#[ test ]
fn tiles_revision_tracks_only_hex_cell_changes()
{
  let ( spec, _ ) = three_way_blend_fixture();
  let mut scene = Scene::new( Arc::new( spec ) );
  let sand = scene.object( "sand" ).expect( "sand" );

  let t = scene.tiles_revision();
  let h = scene.spawn( sand, Placement::FreePos { x : 0.0, y : 0.0 } );
  scene.placement_move( h, Placement::FreePos { x : 5.0, y : 5.0 } );
  scene.tint_set( h, Some( [ 1.0, 0.0, 0.0, 1.0 ] ) );
  assert_eq!( scene.tiles_revision(), t, "FreePos spawn / move / tint do not touch hex cells" );

  scene.placement_move( h, Placement::Hex { q : 0, r : 0 } );
  let t = scene.tiles_revision();
  assert!( t > 0, "moving onto a hex cell changes the cells" );
  scene.placement_move( h, Placement::Hex { q : 0, r : 0 } );
  scene.state_set( h, scene.default_state( sand ) );
  assert_eq!( scene.tiles_revision(), t, "a same-cell move and a state change do not" );

  scene.visible_set( h, false );
  let t = scene.tiles_revision();
  scene.visible_set( h, false );
  scene.placement_move( h, Placement::Hex { q : 1, r : 1 } );
  scene.despawn( h );
  assert_eq!( scene.tiles_revision(), t, "a hidden instance is not in any cell" );

  let h = scene.spawn( sand, Placement::Hex { q : 2, r : 2 } );
  let t = scene.tiles_revision();
  scene.despawn( h );
  assert!( scene.tiles_revision() > t, "despawning a visible hex instance changes the cells" );
}

/// `EffectKind::FadeGate`: `Scene::fade_target_set` + `Scene::tick` must ease
/// a VertexCorners layer's tint from invisible to fully visible over the
/// declared `duration_ms`, holding at the target on either side rather than
/// overshooting (task 002_smooth_dim_fade.md's board-dim fade).
#[ test ]
fn vertex_corners_fade_gate_eases_toward_target()
{
  let mut spec = dual_orient_spec();
  spec.effects.push( Effect
  {
    id : "dim_fade".into(),
    kind : EffectKind::FadeGate { duration_ms : 100.0 },
    phase_offset : PhaseOffset::None,
  });
  let stack = spec.objects[ 0 ].states.get_mut( "default" ).expect( "default state" );
  stack[ 0 ].behaviour.effects.push( EffectRef( "dim_fade".into() ) );

  let scene_snap = SceneSnapshot
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "hexagon".into() ] } ],
    ..minimal_scene_3x3()
  };
  let mut scene = Scene::from_snapshot( &scene_snap, Arc::new( spec.clone() ) ).expect( "scene" );

  // A fresh `Renderer` per call, matching `compile`/`at_time_compile` above —
  // reusing one `Renderer` across calls would exercise its idle-replay /
  // batch-diff machinery (Set/Add/Remove) instead of a full re-emit, which
  // `common::commands_to_sprites` isn't set up to reconstruct standalone.
  let render_now = | scene : &Scene |
  {
    let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
    let raw = renderer.render( scene, &Camera::default() ).expect( "render" );
    common::commands_to_sprites( raw )
  };

  // The first `fade_target_set` call for an id with no prior state snaps
  // straight to the target — no fade on the very first hide.
  scene.fade_target_set( "dim_fade", false );
  let hidden_cmds = render_now( &scene );
  let hidden = sprite_commands( &hidden_cmds );
  assert!( !hidden.is_empty(), "lone hex must still emit corner sprites — geometry, not visibility, drives presence" );
  for s in &hidden
  {
    assert!( s.tint[ 3 ].abs() < 1e-5, "target=false with no prior state must snap to fully transparent, got {:?}", s.tint );
  }

  // Flip the target on — this time a genuine ease, not a snap, since the gate
  // already has prior state.
  scene.fade_target_set( "dim_fade", true );
  let just_flipped_cmds = render_now( &scene );
  let just_flipped = sprite_commands( &just_flipped_cmds );
  for s in &just_flipped
  {
    assert!( s.tint[ 3 ].abs() < 1e-5, "immediately after flipping the target, value hasn't advanced yet: {:?}", s.tint );
  }

  // Half the fade duration (50ms of a 100ms sweep) → value ≈ 0.5.
  scene.tick( 0.050 );
  let mid_cmds = render_now( &scene );
  let mid = sprite_commands( &mid_cmds );
  for s in &mid
  {
    assert!( ( s.tint[ 3 ] - 0.5 ).abs() < 0.05, "halfway through a 100ms fade-in, alpha should be ≈0.5, got {}", s.tint[ 3 ] );
    // The shaders premultiply by the tint alpha themselves, so the fade must leave
    // the colour alone; scaling it too would darken the sprite by k² as it fades.
    assert_eq!( &s.tint[ ..3 ], &[ 1.0, 1.0, 1.0 ], "the fade scales the alpha only, got {:?}", s.tint );
  }

  // Well past the full duration → holds at fully visible (identity tint),
  // not overshooting.
  scene.tick( 1.0 );
  let done_cmds = render_now( &scene );
  let done = sprite_commands( &done_cmds );
  for s in &done
  {
    assert_eq!( s.tint, [ 1.0, 1.0, 1.0, 1.0 ], "fully faded-in must hold at identity tint, not overshoot" );
  }

  // Flip back off and step to the midpoint of the fade-out — must ease
  // symmetrically, not jump straight to 0.
  scene.fade_target_set( "dim_fade", false );
  scene.tick( 0.050 );
  let fading_out_cmds = render_now( &scene );
  let fading_out = sprite_commands( &fading_out_cmds );
  for s in &fading_out
  {
    assert!( ( s.tint[ 3 ] - 0.5 ).abs() < 0.05, "halfway through a 100ms fade-out, alpha should be ≈0.5, got {}", s.tint[ 3 ] );
  }
}

/// A disabled layer is skipped at gather time — its vertex resolve included — so
/// re-enabling it on an UNCHANGED scene (same revision) must re-resolve rather
/// than reuse the empty result cached while it was off.
#[ test ]
fn vertex_corners_reappear_when_a_disabled_layer_is_re_enabled()
{
  let ( spec, snap ) = three_way_blend_fixture();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let scene = Scene::from_snapshot( &snap, Arc::new( spec.clone() ) ).expect( "scene" );
  let terrain = spec.pipeline.layers.iter().position( | l | l.id == "terrain" ).expect( "terrain layer" );

  renderer.disabled_buckets_set( 1 << terrain );
  let off = common::commands_to_sprites( renderer.render( &scene, &Camera::default() ).expect( "render" ) );
  assert!( !has_three_way_blend( &spec, &off ), "a disabled layer emits nothing" );

  renderer.disabled_buckets_set( 0 );
  let on = common::commands_to_sprites( renderer.render( &scene, &Camera::default() ).expect( "render" ) );
  assert!( has_three_way_blend( &spec, &on ), "re-enabled layer must re-resolve its vertex sprites" );
}

/// The dual-grid resolve is cached on `tiles_revision`: mutations that leave the
/// hex cells alone reuse it (while still re-walking the scene), and a change to
/// the cells re-runs it with the new result.
#[ test ]
fn vertex_resolve_reruns_only_when_hex_cells_change()
{
  let ( spec, snap ) = three_way_blend_fixture();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::from_snapshot( &snap, Arc::new( spec.clone() ) ).expect( "scene" );
  let camera = Camera::default();
  let sand = scene.instances_at_hex( 1, -1 ).next().expect( "sand instance" );
  let mut flat = common::BatchFlattener::new();
  let mut render = | scene : &Scene, renderer : &mut Renderer |
    has_three_way_blend( &spec, &flat.apply( renderer.render( scene, &camera ).expect( "render" ) ) );

  assert!( render( &scene, &mut renderer ) );
  assert_eq!( renderer.vertex_resolves(), 1 );

  scene.tint_set( sand, Some( [ 1.0, 1.0, 1.0, 0.5 ] ) );
  assert!( render( &scene, &mut renderer ) );
  scene.tick( 0.25 );
  assert!( render( &scene, &mut renderer ) );
  assert_eq!( renderer.scene_walks(), 3, "each change still re-walks the scene" );
  assert_eq!( renderer.vertex_resolves(), 1, "but the vertex resolve is reused" );

  scene.visible_set( sand, false );
  assert!( !render( &scene, &mut renderer ), "hiding a corner tile breaks the blend" );
  assert_eq!( renderer.vertex_resolves(), 2 );

  scene.visible_set( sand, true );
  assert!( render( &scene, &mut renderer ) );
  scene.placement_move( sand, Placement::Hex { q : 2, r : 2 } );
  assert!( !render( &scene, &mut renderer ), "moving a corner tile away breaks the blend" );
  assert_eq!( renderer.vertex_resolves(), 4 );
}

/// A cell holding two corner terrains gives its corner to the first one in its
/// stack. Moving that first one onto the cell it already stands in must leave
/// the stack order alone: the move does not change `tiles_revision`, so a
/// reorder would make the cached resolve disagree with a fresh one. Here grass
/// then sand share `( 0, 0 )`; grass's corner makes the grass/sand/water blend,
/// sand's would not.
///
/// ## Root Cause
/// `Scene::placement_move` always re-indexed the instance, pushing it to the
/// end of `hex_instances`, which reordered the cell's stack. The first corner
/// terrain in the stack owns the corner, but a same-cell move does not change
/// `tiles_revision`, so the cached resolve kept the old owner.
///
/// ## Why Not Caught
/// `tiles_revision_tracks_only_hex_cell_changes` checks that a same-cell move
/// leaves the revision alone, which is right, but no test put two corner
/// terrains in one cell and compared the cached resolve with a fresh one.
///
/// ## Fix Applied
/// `placement_move` leaves the spatial indexes alone when the instance stays
/// in the hex cell it stands in.
///
/// ## Prevention
/// Compares a fresh resolve and the cached one after a same-cell move, on a
/// cell whose corner owner depends on the stack order.
///
/// ## Pitfall
/// The cache is keyed on `tiles_revision` alone, so anything that changes a
/// cell's stack order, not just its membership, must bump it.
// test_kind: bug_reproducer(BUG-540)
#[ test ]
fn vertex_resolve_cache_matches_a_fresh_resolve_after_a_same_cell_move()
{
  let ( spec, mut snap ) = three_way_blend_fixture();
  snap.tiles[ 0 ].objects = vec![ "grass".into(), "sand".into(), "blend".into() ];
  let mut scene = Scene::from_snapshot( &snap, Arc::new( spec.clone() ) ).expect( "scene" );
  let camera = Camera::default();
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut flat = common::BatchFlattener::new();
  let mut cached = | scene : &Scene |
    has_three_way_blend( &spec, &flat.apply( renderer.render( scene, &camera ).expect( "render" ) ) );
  let fresh = | scene : &Scene |
  {
    let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
    has_three_way_blend( &spec, &common::commands_to_sprites( renderer.render( scene, &camera ).expect( "render" ) ) )
  };

  assert!( cached( &scene ), "grass is first at ( 0, 0 ), so its corner makes the blend" );
  let grass = scene.instances_at_hex( 0, 0 ).next().expect( "grass instance" );
  scene.placement_move( grass, Placement::Hex { q : 0, r : 0 } );
  assert!( fresh( &scene ), "a same-cell move keeps grass first in its cell" );
  assert!( cached( &scene ), "and the cached resolve agrees" );
}

/// One renderer drawing two scenes: the fixture, then the same spec with water
/// where the fixture has sand, so no triangle makes the grass/sand/water blend.
/// Both scenes are built by the same number of spawns, so their revisions,
/// `tiles_revision` and clock are equal. The renderer must draw the second
/// scene as a fresh renderer would, whether it was cleaned up in between or
/// not: equal counters must not let it reuse the first scene's resolve or
/// replay its frame.
#[ test ]
fn a_renderer_reused_for_another_scene_does_not_reuse_its_caches()
{
  let ( spec, snap ) = three_way_blend_fixture();
  let mut swapped = snap.clone();
  swapped.tiles[ 1 ].objects = vec![ "water".into() ];
  let camera = Camera::default();

  for clean_up in [ false, true ]
  {
    let first = Scene::from_snapshot( &snap, Arc::new( spec.clone() ) ).expect( "scene" );
    let second = Scene::from_snapshot( &swapped, Arc::new( spec.clone() ) ).expect( "scene" );
    assert_eq!( ( first.revision(), first.tiles_revision() ), ( second.revision(), second.tiles_revision() ) );
    assert_ne!( first.id(), second.id() );

    let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
    let mut flat = common::BatchFlattener::new();
    assert!( has_three_way_blend( &spec, &flat.apply( renderer.render( &first, &camera ).expect( "render" ) ) ) );
    if clean_up
    {
      let _ = flat.apply( &renderer.cleanup() );
    }
    let resolves = renderer.vertex_resolves();
    let drawn = flat.apply( renderer.render( &second, &camera ).expect( "render" ) );
    assert!( !has_three_way_blend( &spec, &drawn ), "the second scene has no blend (clean_up: {clean_up})" );
    assert_eq!( renderer.vertex_resolves(), resolves + 1, "the second scene is resolved afresh (clean_up: {clean_up})" );
  }
}

/// Renders `scene` through a long-lived `renderer` (the vertex resolve cache
/// and the pulse anchor it keeps live there) and returns every sprite's alpha.
fn pulse_alphas( renderer : &mut Renderer, flattener : &mut common::BatchFlattener, scene : &Scene ) -> Vec< f32 >
{
  let cmds = flattener.apply( renderer.render( scene, &Camera::default() ).expect( "render" ) );
  sprite_commands( &cmds ).iter().map( | s | s.tint[ 3 ] ).collect()
}

/// `AlphaPulse.restart_on_spawn`: a 1 Hz pulse from 0.2 to 1.0 on a lone hex,
/// a second hex spawned at t = 0.25 s, then a look at t = 0.25 s and 0.5 s.
/// Restarting, the wave goes back to `min` at the spawn and is half-way up a
/// quarter period later. Free-running, it ignores the spawn: half-way at the
/// spawn, at `max` a quarter period later.
#[ test ]
fn alpha_pulse_restart_on_spawn_restarts_the_wave_at_a_spawn()
{
  let run = | restart_on_spawn : bool | -> ( Vec< f32 >, Vec< f32 >, Vec< f32 > )
  {
    let mut spec = dual_orient_spec();
    spec.effects.push( Effect
    {
      id : "pulse".into(),
      kind : EffectKind::AlphaPulse { min : 0.2, max : 1.0, frequency : 1.0, restart_on_spawn },
      phase_offset : PhaseOffset::None,
    });
    let stack = spec.objects[ 0 ].states.get_mut( "default" ).expect( "default state" );
    stack[ 0 ].behaviour.effects.push( EffectRef( "pulse".into() ) );

    let scene_snap = SceneSnapshot
    {
      tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "hexagon".into() ] } ],
      ..minimal_scene_3x3()
    };
    let mut scene = Scene::from_snapshot( &scene_snap, Arc::new( spec.clone() ) ).expect( "scene" );
    let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
    let mut flattener = common::BatchFlattener::new();

    let start = pulse_alphas( &mut renderer, &mut flattener, &scene );
    scene.tick( 0.25 );
    let hexagon = scene.object( "hexagon" ).expect( "hexagon object" );
    scene.spawn( hexagon, Placement::Hex { q : 1, r : 0 } );
    let at_spawn = pulse_alphas( &mut renderer, &mut flattener, &scene );
    scene.tick( 0.25 );
    let after = pulse_alphas( &mut renderer, &mut flattener, &scene );
    assert_eq!( renderer.vertex_resolves(), 2, "the spawn re-resolves once, the tick after it does not" );
    ( start, at_spawn, after )
  };
  let all_near = | alphas : &[ f32 ], want : f32, what : &str |
  {
    assert!( !alphas.is_empty(), "{what}: the hexes must emit corner sprites" );
    for &a in alphas
    {
      assert!( ( a - want ).abs() < 1e-3, "{what}: alpha {want} expected, got {a} in {alphas:?}" );
    }
  };

  let ( start, at_spawn, after ) = run( true );
  all_near( &start, 0.2, "restart: first frame" );
  all_near( &at_spawn, 0.2, "restart: the spawn sends the wave back to min" );
  all_near( &after, 0.6, "restart: a quarter period after the spawn" );

  let ( start, at_spawn, after ) = run( false );
  all_near( &start, 0.2, "free-running: first frame" );
  all_near( &at_spawn, 0.6, "free-running: the spawn does not touch the wave" );
  all_near( &after, 1.0, "free-running: half a period on the scene clock" );
}

/// `AlphaPulse.restart_on_spawn` restarts the wave only when the pulsing
/// layer's own vertex content changes. A spawn elsewhere still re-resolves the
/// vertex pass, since it changes the hex cells, but the pulsing bucket resolves
/// to the same sprites, so its wave keeps its anchor and runs on as if nothing
/// happened: half-way at the spawn, at `max` a quarter period later. A resolve
/// that restarted on any re-resolve would put it back at `min` instead.
#[ test ]
fn alpha_pulse_restart_on_spawn_carries_the_wave_across_an_unrelated_spawn()
{
  let mut spec = dual_orient_spec();
  spec.effects.push( Effect
  {
    id : "pulse".into(),
    kind : EffectKind::AlphaPulse { min : 0.2, max : 1.0, frequency : 1.0, restart_on_spawn : true },
    phase_offset : PhaseOffset::None,
  });
  let stack = spec.objects[ 0 ].states.get_mut( "default" ).expect( "default state" );
  stack[ 0 ].behaviour.effects.push( EffectRef( "pulse".into() ) );
  // A plain sprite with no corner `priority`, so it never takes part in the
  // dual-grid resolve.
  spec.objects.push( Object
  {
    id : "marker".into(),
    anchor : Anchor::Hex,
    global_layer : "terrain".into(),
    priority : None,
    sort_y_source : SortYSource::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states :
    {
      let mut m = HashMap::default();
      m.insert
      (
        "default".into(),
        vec!
        [
          ObjectLayer
          {
            id : None,
            sprite_source : SpriteSource::Static( SpriteRef { asset : "terrain".into(), frame : "0".into() } ),
            behaviour : LayerBehaviour::default(),
            z_in_object : 0,
            pipeline_layer : None,
          },
        ],
      );
      m
    },
  });

  let scene_snap = SceneSnapshot
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "hexagon".into() ] } ],
    ..minimal_scene_3x3()
  };
  let mut scene = Scene::from_snapshot( &scene_snap, Arc::new( spec.clone() ) ).expect( "scene" );
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut flattener = common::BatchFlattener::new();
  let pulse = | renderer : &mut Renderer, flattener : &mut common::BatchFlattener, scene : &Scene |
  {
    // The marker's sprite has no pulse: keep only the hexagon's corner sprites,
    // which are the ones drawn from the `dual` atlas.
    let compiled = assets_compile( &spec, &PathResolver ).expect( "assets" );
    let cmds = flattener.apply( renderer.render( scene, &Camera::default() ).expect( "render" ) );
    sprite_commands( &cmds ).iter()
      .filter( | s | compiled.ids.sprite( "terrain", "0" ) != Some( s.sprite ) )
      .map( | s | s.tint[ 3 ] )
      .collect::< Vec< f32 > >()
  };

  let start = pulse( &mut renderer, &mut flattener, &scene );
  scene.tick( 0.25 );
  let marker = scene.object( "marker" ).expect( "marker object" );
  scene.spawn( marker, Placement::Hex { q : 5, r : 5 } );
  let at_spawn = pulse( &mut renderer, &mut flattener, &scene );
  scene.tick( 0.25 );
  let after = pulse( &mut renderer, &mut flattener, &scene );
  assert_eq!( renderer.vertex_resolves(), 2, "the spawn changes the hex cells, so it re-resolves" );

  for ( alphas, want, what ) in [ ( &start, 0.2, "first frame" ), ( &at_spawn, 0.6, "the spawn leaves the wave alone" ), ( &after, 1.0, "half a period after the first frame" ) ]
  {
    assert!( !alphas.is_empty(), "{what}: the hexagon must emit corner sprites" );
    for &a in alphas
    {
      assert!( ( a - want ).abs() < 1e-3, "{what}: alpha {want} expected, got {a} in {alphas:?}" );
    }
  }
}
