//! `Renderer::render` on a 469-hex dual-grid board with free-moving units —
//! the frames the PR #207 performance work targets.
//!
//! - `hex_cell_change`: one hex cell flips each frame, so the `VertexCorners`
//!   resolve reruns (`Scene::tiles_revision` moved).
//! - `movers_only`: every unit moves each frame and no hex cell changes, so the
//!   scene is re-walked but the vertex resolve is reused.
//! - `clock_only`: only the clock advances — a re-walk with nothing moved.
//! - `idle_replay`: nothing changes, so the previous command buffer is replayed.
//!
//! Run with `cargo bench -p tilemap_scene --bench frame_render`. The board is
//! built the same way on every run, so successive runs are comparable;
//! criterion reports the change against the previous run.

use std::hint::black_box;
use std::sync::Arc;
use criterion::{ criterion_group, criterion_main, Criterion };
use tilemap_scene::{ Anchor, Camera, PathResolver, Placement, Renderer, Scene, SceneSnapshot, Tile };

#[ path = "../tests/common/mod.rs" ]
#[ allow( dead_code, reason = "the shared test helpers are compiled in whole; this bench uses only the fixtures" ) ]
mod common;

/// Hex-board radius: `1 + 3·R·(R+1)` cells, 469 for `R = 12`.
const RADIUS : i32 = 12;
/// Free-positioned units walking over the board.
const UNITS : usize = 64;

/// The dual-grid board (`dual_orient_spec`'s `hexagon` on every cell of a
/// radius-12 hex) plus a `unit` object anchored at a free world position.
fn board() -> ( Arc< tilemap_scene::RenderSpec >, Scene, Vec< tilemap_scene::InstanceHandle > )
{
  let mut spec = common::compile::dual_orient_spec();
  let mut unit = common::compile::grass_object();
  unit.id = "unit".into();
  unit.anchor = Anchor::FreePos;
  spec.objects.push( unit );
  let spec = Arc::new( spec );

  let mut snap = SceneSnapshot::new( tilemap_scene::Bounds { min : ( -RADIUS, -RADIUS ), max : ( RADIUS, RADIUS ) } );
  for q in -RADIUS..=RADIUS
  {
    for r in -RADIUS..=RADIUS
    {
      if ( q + r ).abs() <= RADIUS
      {
        snap.tiles.push( Tile { pos : ( q, r ), objects : vec![ "hexagon".into() ] } );
      }
    }
  }
  assert_eq!( snap.tiles.len(), 469 );

  let mut scene = Scene::from_snapshot( &snap, Arc::clone( &spec ) ).expect( "scene" );
  let unit = scene.object( "unit" ).expect( "unit object" );
  let units = ( 0..UNITS )
  .map( | i | scene.spawn( unit, unit_at( i, 0.0 ) ) )
  .collect();
  ( spec, scene, units )
}

/// Where unit `i` stands at walk step `t`: spread over the board, drifting.
#[ allow( clippy::cast_precision_loss, reason = "unit indices are tiny" ) ]
fn unit_at( i : usize, t : f32 ) -> Placement
{
  let a = i as f32 * 0.7 + t;
  Placement::FreePos { x : a.cos() * 600.0, y : a.sin() * 500.0 }
}

fn frame_render( c : &mut Criterion )
{
  let mut group = c.benchmark_group( "frame_render_469_hex" );
  let camera = Camera { viewport_size : ( 1920, 1080 ), ..Camera::default() };

  {
    let ( spec, mut scene, _ ) = board();
    let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
    let _ = renderer.render( &scene, &camera ).expect( "render" );
    let cell = scene.instances_at_hex( 0, 0 ).next().expect( "centre hex" );
    let mut visible = true;
    group.bench_function( "hex_cell_change", | b | b.iter( ||
    {
      visible = !visible;
      scene.visible_set( cell, visible );
      black_box( renderer.render( &scene, &camera ).expect( "render" ).len() )
    }));
  }

  {
    let ( spec, mut scene, units ) = board();
    let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
    let _ = renderer.render( &scene, &camera ).expect( "render" );
    let mut t = 0.0_f32;
    group.bench_function( "movers_only", | b | b.iter( ||
    {
      t += 0.01;
      for ( i, &h ) in units.iter().enumerate()
      {
        scene.placement_move( h, unit_at( i, t ) );
      }
      black_box( renderer.render( &scene, &camera ).expect( "render" ).len() )
    }));
  }

  {
    let ( spec, mut scene, _ ) = board();
    let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
    let _ = renderer.render( &scene, &camera ).expect( "render" );
    group.bench_function( "clock_only", | b | b.iter( ||
    {
      scene.tick( 1.0 / 60.0 );
      black_box( renderer.render( &scene, &camera ).expect( "render" ).len() )
    }));
  }

  {
    let ( spec, scene, _ ) = board();
    let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
    let _ = renderer.render( &scene, &camera ).expect( "render" );
    group.bench_function( "idle_replay", | b | b.iter( ||
      black_box( renderer.render( &scene, &camera ).expect( "render" ).len() )
    ));
  }

  group.finish();
}

criterion_group!( benches, frame_render );
criterion_main!( benches );
