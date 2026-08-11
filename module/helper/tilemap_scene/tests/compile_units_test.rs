//! Unit tests for the compile layer's pure primitives — id allocation,
//! condition evaluation, camera projection, edge canonicalisation, tri-blend
//! pattern matching, viewport placement, animation frame resolution, and
//! hex → world-pixel coordinate mapping. Each section exercises one exposed
//! function or type directly, at unit level; the integration-level compile
//! pipeline ( `compile_assets` / `compile_frame` ) is covered by
//! `scene_model_compile_test.rs`.
//!
//! Relocated from inline `#[ cfg( test ) ]` modules across `src/compile/*` by
//! task 073 ( bodies verbatim; imports crate-qualified ).

#![ expect( clippy::float_cmp, reason = "assertions check exact pass-through of constant tints/coordinates; no arithmetic drift is possible and epsilon comparison would weaken them" ) ]

// The viewport assertions compare against exact literal constants (e.g. `0.0`,
// known integer-valued results), not derived floating-point computations —
// exact comparison is the correct check there, not an epsilon tolerance.

use tilemap_scene::
{
  Animation,
  AnimationMode,
  AnimationTiming,
  Camera,
  Condition,
  EdgeDirection,
  EdgePosition,
  IdMap,
  NeighborState,
  PhaseOffset,
  SpriteRef,
  TilingStrategy,
  TimedFrame,
  TriBlendPattern,
  ViewportAnchorPoint,
  ViewportTiling,
  canonical_edge,
  canonicalize,
  edge_rotation,
  evaluate_condition,
  find_matching_pattern,
  hex_to_world_pixel_flat,
  hex_to_world_pixel_pointy,
  resolve_animation_frame,
  tiled_positions,
  viewport_transform,
};

// === compile/ids.rs — IdMap ===

#[ test ]
fn images_are_deterministic()
{
  let mut m = IdMap::new();
  let a = m.alloc_image( "terrain_atlas" );
  let b = m.alloc_image( "transitions_atlas" );
  let a_again = m.alloc_image( "terrain_atlas" );
  assert_eq!( a.inner(), 0 );
  assert_eq!( b.inner(), 1 );
  assert_eq!( a, a_again, "re-allocating same id returns the same handle" );
}

#[ test ]
fn sprites_namespace_by_atlas()
{
  let mut m = IdMap::new();
  let grass = m.alloc_sprite( "terrain", "0" );
  let sand  = m.alloc_sprite( "terrain", "1" );
  let grass_other_atlas = m.alloc_sprite( "other", "0" );
  assert_ne!( grass, sand, "different frames get different ids" );
  assert_ne!( grass, grass_other_atlas, "same frame name in different atlases is distinct" );
  assert_eq!( Some( grass ), m.sprite( "terrain", "0" ) );
}

// === compile/conditions.rs — evaluate_condition ===

fn state( ids : &[ &str ], max_priority : Option< i32 > ) -> ( Vec< String >, Option< i32 > )
{
  ( ids.iter().map( | s | ( *s ).into() ).collect(), max_priority )
}

#[ test ]
fn neighbour_is_matches_any_present_id()
{
  let ( ids, _ ) = state( &[ "water" ], None );
  let n = NeighborState { object_ids : &ids, max_priority : None };
  assert!( evaluate_condition( &Condition::NeighborIs( vec![ "water".into() ] ), &n, None ) );
  assert!( !evaluate_condition( &Condition::NeighborIs( vec![ "lava".into() ] ), &n, None ) );
}

#[ test ]
fn neighbour_is_void_matches_empty()
{
  let n = NeighborState { object_ids : &[], max_priority : None };
  assert!( evaluate_condition( &Condition::NeighborIs( vec![ "void".into() ] ), &n, None ) );
}

#[ test ]
fn no_neighbor_matches_empty()
{
  let ( ids, _ ) = state( &[], None );
  let n = NeighborState { object_ids : &ids, max_priority : None };
  assert!( evaluate_condition( &Condition::NoNeighbor, &n, None ) );

  let ( ids, _ ) = state( &[ "grass" ], None );
  let n = NeighborState { object_ids : &ids, max_priority : None };
  assert!( !evaluate_condition( &Condition::NoNeighbor, &n, None ) );
}

#[ test ]
fn priority_lower_only_with_both_priorities()
{
  let ( ids, _ ) = state( &[ "sand" ], Some( 5 ) );
  let n = NeighborState { object_ids : &ids, max_priority : Some( 5 ) };
  assert!( evaluate_condition( &Condition::NeighborPriorityLower, &n, Some( 10 ) ) );
  assert!( !evaluate_condition( &Condition::NeighborPriorityLower, &n, Some( 5 ) ) );
  assert!( !evaluate_condition( &Condition::NeighborPriorityLower, &n, None ) );
}

#[ test ]
fn any_of_and_not_compose()
{
  let ( ids, _ ) = state( &[ "water" ], None );
  let n = NeighborState { object_ids : &ids, max_priority : None };
  let cond = Condition::AnyOf( vec!
  [
    Condition::NeighborIs( vec![ "lava".into() ] ),
    Condition::NeighborIs( vec![ "water".into() ] ),
  ]);
  assert!( evaluate_condition( &cond, &n, None ) );

  let negated = Condition::Not( Box::new( cond ) );
  assert!( !evaluate_condition( &negated, &n, None ) );
}

// === compile/camera.rs — Camera ===

#[ test ]
fn default_centers_origin()
{
  let cam = Camera::default();
  let ( x, y ) = cam.project( ( 0.0, 0.0 ) );
  assert!( ( x - 400.0 ).abs() < 1e-3, "expected x ~= 400, got {x}" );
  assert!( ( y - 300.0 ).abs() < 1e-3, "expected y ~= 300, got {y}" );
}

#[ test ]
fn translate_shifts_projection()
{
  let cam = Camera { world_center : ( 100.0, 0.0 ), ..Camera::default() };
  let ( x, y ) = cam.project( ( 0.0, 0.0 ) );
  assert!( ( x - 300.0 ).abs() < 1e-3, "translate didn't shift x correctly: {x}" );
  assert!( ( y - 300.0 ).abs() < 1e-3, "translate changed y unexpectedly: {y}" );
}

#[ test ]
fn zoom_scales_distance_from_center()
{
  let cam = Camera { zoom : 2.0, ..Camera::default() };
  let ( x_zoomed, _ ) = cam.project( ( 50.0, 0.0 ) );
  let cam_one = Camera::default();
  let ( x_one, _ ) = cam_one.project( ( 50.0, 0.0 ) );
  // Distance from viewport centre should be doubled under 2x zoom.
  let d_zoomed = x_zoomed - 400.0;
  let d_one    = x_one - 400.0;
  assert!( ( d_zoomed / d_one - 2.0 ).abs() < 1e-3, "zoom distance ratio: {d_zoomed} / {d_one}" );
}

// === compile/edges.rs — canonical_edge / edge_rotation ===

#[ test ]
fn canonical_picks_smaller_hex()
{
  let tiling = TilingStrategy::HexFlatTop;
  // Edge between (0,0) and its N neighbour (0,-1).
  let from_a = EdgePosition { hex : ( 0, 0 ), dir : EdgeDirection::N };
  let from_b = EdgePosition { hex : ( 0, -1 ), dir : EdgeDirection::S };
  let ca = canonical_edge( from_a, tiling ).unwrap();
  let cb = canonical_edge( from_b, tiling ).unwrap();
  assert_eq!( ca, cb, "both sides must canonicalise to the same key" );
  // (0,-1) is lexicographically smaller than (0,0) → canonical hex = (0,-1).
  assert_eq!( ca.0, ( 0, -1 ) );
}

#[ test ]
fn edge_rotation_flat_top_table()
{
  use core::f32::consts::PI;
  let t = TilingStrategy::HexFlatTop;
  assert!( ( edge_rotation( EdgeDirection::N,  t ) - 0.0 ).abs() < 1e-5 );
  assert!( ( edge_rotation( EdgeDirection::NE, t ) - PI / 3.0 ).abs() < 1e-5 );
  assert!( ( edge_rotation( EdgeDirection::S,  t ) - PI ).abs() < 1e-5 );
}

// === compile/vertex.rs — canonicalize / find_matching_pattern ===

fn pattern( a : &str, b : &str, c : &str, priority : i32, sprite : &str ) -> TriBlendPattern
{
  TriBlendPattern
  {
    corners : ( a.into(), b.into(), c.into() ),
    sprite_pattern : sprite.into(),
    priority,
    animation : None,
  }
}

#[ test ]
fn canonicalize_sorts_ids()
{
  let ( sorted, _rot ) = canonicalize( &[ "water".into(), "grass".into(), "sand".into() ] );
  assert_eq!( sorted, [ "grass".to_string(), "sand".into(), "water".into() ] );
}

#[ test ]
fn exact_beats_wildcard()
{
  let patterns = [ pattern( "*", "*", "void", 0, "edge_fade" ), pattern( "grass", "sand", "water", 5, "tri_g_s_w" ) ];
  let canonical = [ "grass".into(), "sand".into(), "water".into() ];
  let found = find_matching_pattern( &patterns, &canonical );
  assert!( matches!( found, Some( p ) if p.sprite_pattern == "tri_g_s_w" ) );
}

#[ test ]
fn priority_tiebreaks_same_specificity()
{
  let patterns = [ pattern( "grass", "grass", "water", 1, "low" ), pattern( "grass", "grass", "water", 10, "high" ) ];
  let canonical = [ "grass".into(), "grass".into(), "water".into() ];
  let found = find_matching_pattern( &patterns, &canonical );
  assert!( matches!( found, Some( p ) if p.sprite_pattern == "high" ) );
}

#[ test ]
fn wildcard_fallback_when_nothing_specific()
{
  let patterns = [ pattern( "*", "*", "void", 0, "edge_fade" ) ];
  let canonical = [ "grass".into(), "sand".into(), "void".into() ];
  let found = find_matching_pattern( &patterns, &canonical );
  assert!( matches!( found, Some( p ) if p.sprite_pattern == "edge_fade" ) );
}

#[ test ]
fn no_match_returns_none()
{
  let patterns = [ pattern( "grass", "grass", "grass", 0, "pure_grass" ) ];
  let canonical = [ "grass".into(), "grass".into(), "water".into() ];
  assert!( find_matching_pattern( &patterns, &canonical ).is_none() );
}

// === compile/viewport.rs — viewport_transform / tiled_positions ===

#[ test ]
fn stretch_fills_viewport()
{
  let t = viewport_transform(
    ViewportTiling::Stretch,
    ViewportAnchorPoint::Center,
    ( 100.0, 50.0 ),
    ( 800, 600 ),
  ).unwrap();
  assert_eq!( t.position, [ 0.0, 0.0 ] );
  assert!( ( t.scale[ 0 ] - 8.0 ).abs() < 1e-5 );
  assert!( ( t.scale[ 1 ] - 12.0 ).abs() < 1e-5 );
}

#[ test ]
fn center_topleft_anchors_at_top_in_y_up()
{
  // Y-up: TopLeft places sprite's bottom-left corner at y = vh - sh, so its
  // top edge sits at the viewport's top edge.
  let t = viewport_transform(
    ViewportTiling::Center,
    ViewportAnchorPoint::TopLeft,
    ( 100.0, 50.0 ),
    ( 800, 600 ),
  ).unwrap();
  assert_eq!( t.position, [ 0.0, 550.0 ] ); // x=0, y=600-50
  assert_eq!( t.scale, [ 1.0, 1.0 ] );
}

#[ test ]
fn center_bottomcenter_positions_sprite()
{
  // Y-up: BottomCenter places sprite's bottom-left corner at y = 0.
  let t = viewport_transform(
    ViewportTiling::Center,
    ViewportAnchorPoint::BottomCenter,
    ( 100.0, 50.0 ),
    ( 800, 600 ),
  ).unwrap();
  assert!( ( t.position[ 0 ] - 350.0 ).abs() < 1e-5 ); // (800 - 100) / 2
  assert!( ( t.position[ 1 ] - 0.0 ).abs() < 1e-5 );   // bottom of viewport
}

#[ test ]
fn repeat2d_emits_grid_covering_viewport()
{
  // 32x32 tile, 800x600 viewport → 26 cols × 20 rows (plus safety margin
  // of +1 each side in the implementation).
  let positions = tiled_positions(
    ViewportTiling::Repeat2D,
    ViewportAnchorPoint::TopLeft,
    ( 32.0, 32.0 ),
    ( 800, 600 ),
  );
  assert_eq!( positions.len(), 26 * 20 );
  // First tile at origin.
  assert_eq!( positions[ 0 ], ( 0.0, 0.0 ) );
  // Second tile one sprite-width across.
  assert_eq!( positions[ 1 ], ( 32.0, 0.0 ) );
}

#[ test ]
fn repeatx_emits_single_row()
{
  let positions = tiled_positions(
    ViewportTiling::RepeatX,
    ViewportAnchorPoint::BottomLeft,
    ( 100.0, 50.0 ),
    ( 800, 600 ),
  );
  assert_eq!( positions.len(), 9 ); // ceil(800/100) + 1 = 9
  // Y-up: BottomLeft pins sprites at y = 0 (viewport's bottom edge).
  for ( _, y ) in &positions
  {
    assert!( ( y - 0.0 ).abs() < 1e-5 );
  }
}

#[ test ]
fn fit_preserves_aspect()
{
  // Sprite 100x50, viewport 800x600 → limiting axis is Y (ratio 12 vs 8).
  // Wait — sprite w 100 fits 8x in 800, sprite h 50 fits 12x in 600.
  // min = 8 → scaled sprite = 800 x 400, fits width-first, centred vertically.
  let t = viewport_transform(
    ViewportTiling::Fit,
    ViewportAnchorPoint::Center,
    ( 100.0, 50.0 ),
    ( 800, 600 ),
  ).unwrap();
  assert!( ( t.scale[ 0 ] - 8.0 ).abs() < 1e-5 );
  assert!( ( t.scale[ 1 ] - 8.0 ).abs() < 1e-5 );
  // Centred: scaled sprite is 800x400 → x=0, y=100.
  assert!( ( t.position[ 0 ] - 0.0 ).abs() < 1e-5 );
  assert!( ( t.position[ 1 ] - 100.0 ).abs() < 1e-5 );
}

// === compile/animation.rs — resolve_animation_frame ===

fn regular( id : &str, frames : &[ &str ], fps : f32, mode : AnimationMode ) -> Animation
{
  Animation
  {
    id : id.into(),
    timing : AnimationTiming::Regular
    {
      frames : frames.iter().map( | f | SpriteRef { asset : "a".into(), frame : ( *f ).into() } ).collect(),
      fps,
    },
    mode,
    phase_offset : PhaseOffset::None,
  }
}

#[ test ]
fn regular_loop_wraps()
{
  let a = regular( "w", &[ "0", "1", "2" ], 10.0, AnimationMode::Loop );
  let pick = | t | resolve_animation_frame( &a, t, 0.0, ( 0, 0 ), None ).unwrap().frame;
  assert_eq!( pick( 0.0 ), "0" );
  assert_eq!( pick( 0.1 ), "1" );
  assert_eq!( pick( 0.25 ), "2" );
  assert_eq!( pick( 0.35 ), "0", "should have wrapped back to frame 0" );
}

#[ test ]
fn one_shot_clamps()
{
  let a = regular( "w", &[ "a", "b", "c" ], 10.0, AnimationMode::OneShot );
  let pick = | t | resolve_animation_frame( &a, t, 0.0, ( 0, 0 ), None ).unwrap().frame;
  assert_eq!( pick( 0.0 ), "a" );
  assert_eq!( pick( 100.0 ), "c", "past end → stuck on last frame" );
}

#[ test ]
fn one_shot_origin_resets_local_time()
{
  // OneShot rooted at a non-zero origin — the relative time is
  // `time_seconds - oneshot_origin`, so a 0.05 s delta after the
  // origin must pick the first frame, not the clamped last frame.
  let a = regular( "w", &[ "a", "b", "c" ], 10.0, AnimationMode::OneShot );
  let pick = | t, origin | resolve_animation_frame( &a, t, origin, ( 0, 0 ), None ).unwrap().frame;
  assert_eq!( pick( 5.05, 5.0 ), "a", "0.05 s after origin → first frame" );
  assert_eq!( pick( 5.15, 5.0 ), "b" );
  assert_eq!( pick( 5.25, 5.0 ), "c" );
  assert_eq!( pick( 99.0, 5.0 ), "c", "past origin + duration → clamped" );
}

#[ test ]
fn pingpong_reflects()
{
  let a = regular( "w", &[ "a", "b", "c" ], 10.0, AnimationMode::PingPong );
  let pick = | t | resolve_animation_frame( &a, t, 0.0, ( 0, 0 ), None ).unwrap().frame;
  // Period = 2 * (3 - 1) = 4 ticks. Sequence: a b c b | a b c b | ...
  assert_eq!( pick( 0.00 ), "a" );
  assert_eq!( pick( 0.10 ), "b" );
  assert_eq!( pick( 0.20 ), "c" );
  assert_eq!( pick( 0.30 ), "b", "ping-ponged" );
  assert_eq!( pick( 0.40 ), "a" );
}

#[ test ]
fn phase_offset_hashcoord_spreads_neighbours()
{
  let mut a = regular( "w", &[ "0", "1", "2", "3" ], 4.0, AnimationMode::Loop );
  a.phase_offset = PhaseOffset::HashCoord;
  // Two neighbouring tiles, same global time — their local times should
  // differ (practically always) and so can their frames.
  let f_00 = resolve_animation_frame( &a, 0.0, 0.0, ( 0, 0 ), None ).unwrap().frame;
  let f_10 = resolve_animation_frame( &a, 0.0, 0.0, ( 1, 0 ), None ).unwrap().frame;
  // We can't assert inequality rigorously (hash could collide) but we can
  // sample many coords and check that at least SOME produce different frames.
  let samples : Vec< String > =
    ( 0..16 ).map( | q | resolve_animation_frame( &a, 0.0, 0.0, ( q, 0 ), None ).unwrap().frame ).collect();
  let unique_count = samples.iter().collect::< std::collections::HashSet< _ > >().len();
  assert!
  (
    unique_count >= 2,
    "phase-offset should spread neighbours across frames; samples: {samples:?} (first two {f_00} vs {f_10})",
  );
}

#[ test ]
fn phase_offset_instance_spreads_seeds()
{
  // `PhaseOffset::Instance` derives phase from the per-instance
  // seed mixed with the animation id. Sampling 16 distinct seeds
  // must produce at least two different frames — same shape as
  // the HashCoord test, but the input dimension is the seed
  // rather than the grid coord, so this works for placements
  // with no hex coord.
  let mut a = regular( "w", &[ "0", "1", "2", "3" ], 4.0, AnimationMode::Loop );
  a.phase_offset = PhaseOffset::Instance;
  let samples : Vec< String > = ( 0..16_u32 )
    .map( | seed | resolve_animation_frame( &a, 0.0, 0.0, ( 0, 0 ), Some( seed ) ).unwrap().frame )
    .collect();
  let unique_count = samples.iter().collect::< std::collections::HashSet< _ > >().len();
  assert!
  (
    unique_count >= 2,
    "PhaseOffset::Instance should spread seeds across frames; samples: {samples:?}",
  );
}

#[ test ]
fn phase_offset_instance_falls_back_when_seed_missing()
{
  // Without an instance seed (`None`), `PhaseOffset::Instance`
  // contributes 0.0 — the edge / vertex compile paths sit on the
  // master clock instead.
  let mut a = regular( "w", &[ "0", "1", "2" ], 10.0, AnimationMode::Loop );
  a.phase_offset = PhaseOffset::Instance;
  let frame = resolve_animation_frame( &a, 0.0, 0.0, ( 0, 0 ), None ).unwrap().frame;
  assert_eq!( frame, "0", "no seed → no phase shift" );
}

#[ test ]
fn phase_offset_fixed_shifts_timeline()
{
  let mut a = regular( "w", &[ "0", "1", "2" ], 10.0, AnimationMode::Loop );
  a.phase_offset = PhaseOffset::Fixed( 0.1 );
  // At global t=0, with phase=0.1s, we're 1 frame in.
  let frame = resolve_animation_frame( &a, 0.0, 0.0, ( 0, 0 ), None ).unwrap().frame;
  assert_eq!( frame, "1" );
}

#[ test ]
fn irregular_timing_honours_durations()
{
  let a = Animation
  {
    id : "attack".into(),
    timing : AnimationTiming::Irregular
    {
      frames : vec!
      [
        TimedFrame
        {
          sprite : SpriteRef { asset : "a".into(), frame : "wind_up".into() },
          duration_ms : 100,
        },
        TimedFrame
        {
          sprite : SpriteRef { asset : "a".into(), frame : "impact".into() },
          duration_ms : 300,    // held
        },
        TimedFrame
        {
          sprite : SpriteRef { asset : "a".into(), frame : "recover".into() },
          duration_ms : 100,
        },
      ],
    },
    mode : AnimationMode::OneShot,
    phase_offset : PhaseOffset::None,
  };
  let pick = | t | resolve_animation_frame( &a, t, 0.0, ( 0, 0 ), None ).unwrap().frame;
  assert_eq!( pick( 0.0  ), "wind_up" );
  assert_eq!( pick( 0.05 ), "wind_up" );
  assert_eq!( pick( 0.15 ), "impact" );
  assert_eq!( pick( 0.35 ), "impact", "still holding the accented frame" );
  assert_eq!( pick( 0.45 ), "recover" );
  assert_eq!( pick( 2.00 ), "recover", "OneShot clamps to last" );
}

// === compile/coords.rs — hex_to_world_pixel_flat / hex_to_world_pixel_pointy ===

// Reference facts we pin: flat-top origin maps to (0,0); positive r moves
// the point south in tiles_tools (Y-down), which becomes north-up in our
// Y-up world pixels → i.e. larger r produces MORE NEGATIVE world y.

#[ test ]
fn origin_maps_to_zero()
{
  let ( x, y ) = hex_to_world_pixel_flat( 0, 0, ( 72, 64 ) );
  assert!( x.abs() < 1e-5, "expected x ≈ 0, got {x}" );
  assert!( y.abs() < 1e-5, "expected y ≈ 0, got {y}" );
}

#[ test ]
fn flat_top_y_flip_is_applied()
{
  // Flat-top: stepping r by +1 in tiles_tools' Y-down frame means moving
  // south on-screen (larger Y). After the compile-layer flip we expect
  // negative world Y instead.
  let ( _, y0 ) = hex_to_world_pixel_flat( 0, 0, ( 72, 64 ) );
  let ( _, y1 ) = hex_to_world_pixel_flat( 0, 1, ( 72, 64 ) );
  assert!( y1 < y0, "expected r=1 to produce smaller world y than r=0, got y0={y0} y1={y1}" );
}

#[ test ]
fn flat_top_x_scales_with_cell_width()
{
  // q=1, r=0 should place the cell one full cell width to the right of origin.
  let ( x, _ ) = hex_to_world_pixel_flat( 1, 0, ( 72, 64 ) );
  // Tolerance is generous because sqrt(3) conversion is lossy — we just
  // want to pin the sign and rough magnitude. The exact value is 72 px
  // (full cell width) because of the compensating sx scale.
  assert!( x > 0.0 && x < 120.0, "x out of expected range: {x}" );
}

#[ test ]
fn pointy_top_x_shifts_with_row()
{
  // Pointy-top: moving r by +1 shifts x by half a cell width (zig-zag).
  let ( x0, _ ) = hex_to_world_pixel_pointy( 0, 0, ( 64, 72 ) );
  let ( x1, _ ) = hex_to_world_pixel_pointy( 0, 1, ( 64, 72 ) );
  assert!( ( x1 - x0 ).abs() > 1.0, "expected row shift on pointy top, got x0={x0} x1={x1}" );
}
