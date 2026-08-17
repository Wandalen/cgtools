//! Covers `## Test Matrix` rows T01, T02, T03, T05 and anti-faking check AF2
//! from `task/completed/085_pingpong_animation_render_command_wiring.md`.
//! Whole file is `adapter-svg`-gated: `RenderCommand`/`SvgBackend` etc. only
//! exist once that feature pulls in the optional `tilemap_renderer` dependency.

#![ cfg( feature = "adapter-svg" ) ]
#![ expect( clippy::float_cmp, reason = "assertions check exact pass-through of ball coordinates into mesh transforms; no arithmetic drift is possible and epsilon comparison would weaken them" ) ]

use ndarray_cg::F32x2;
use pingpong_animation::{ render::{ frame_to_commands, render_assets, BALL_GEOMETRY }, Frame };
use tilemap_renderer::
{
  adapters::svg::SvgBackend,
  assets::Assets,
  backend::{ Backend, RenderError },
  commands::RenderCommand,
  types::RenderConfig,
};

fn sample_frame( ball_x : f32, ball_y : f32 ) -> Frame
{
  Frame
  {
    tick : 0,
    ball : F32x2::new( ball_x, ball_y ),
    paddle_left_y : 0.0,
    paddle_right_y : 0.0,
  }
}

fn empty_assets() -> Assets
{
  Assets
  {
    fonts : Vec::new(),
    images : Vec::new(),
    sprites : Vec::new(),
    geometries : Vec::new(),
    gradients : Vec::new(),
    patterns : Vec::new(),
    clip_masks : Vec::new(),
    paths : Vec::new(),
  }
}

fn ball_position( commands : &[ RenderCommand ] ) -> [ f32; 2 ]
{
  match &commands[ 0 ]
  {
    RenderCommand::Mesh( mesh ) => mesh.transform.position,
    other => panic!( "expected the first command to be the ball Mesh draw, got {other:?}" ),
  }
}

/// T01 — a representative frame compiles to exactly one ball draw and two paddle draws.
#[ test ]
fn t01_frame_to_commands_returns_ball_and_two_paddles()
{
  let frame = sample_frame( 0.0, 0.0 );
  let commands = frame_to_commands( &frame );

  assert_eq!( commands.len(), 3, "expected 1 ball + 2 paddle commands, got {commands:?}" );
  for command in &commands
  {
    assert!( matches!( command, RenderCommand::Mesh( _ ) ), "expected only Mesh draws, got {command:?}" );
  }
}

/// T02 — different `ball` positions thread through to different compiled output,
/// proving `frame_to_commands` isn't hardcoded (also backs AF1).
#[ test ]
fn t02_different_frames_produce_different_ball_position()
{
  let commands_a = frame_to_commands( &sample_frame( 0.0, 0.0 ) );
  let commands_b = frame_to_commands( &sample_frame( 5.0, 7.0 ) );

  assert_ne!( ball_position( &commands_a ), ball_position( &commands_b ) );
  assert_eq!( ball_position( &commands_a ), [ 0.0, 0.0 ] );
  assert_eq!( ball_position( &commands_b ), [ 5.0, 7.0 ] );
}

/// T03 — compiled commands submit cleanly to a fresh `SvgBackend` once its
/// real geometry assets (not empty/default) are loaded first.
#[ test ]
fn t03_compiled_commands_submit_to_svg_backend_ok()
{
  let mut backend = SvgBackend::new( RenderConfig::default() );
  backend.assets_load( &render_assets() ).expect( "render_assets() must load without error" );

  let result = backend.submit( &frame_to_commands( &sample_frame( 1.0, 2.0 ) ) );

  assert!( result.is_ok(), "submit() must succeed once real assets are loaded: {result:?}" );
}

/// T05 — under `--no-default-features --features adapter-svg` (this whole file's
/// own cfg gate) the compiler-to-backend pipeline runs end to end through
/// `output()`, proving the adapter-svg-only build is fully functional, not
/// just compiling.
#[ test ]
fn t05_adapter_svg_only_build_renders_full_pipeline()
{
  let mut backend = SvgBackend::new( RenderConfig::default() );
  backend.assets_load( &render_assets() ).expect( "render_assets() must load without error" );
  backend.submit( &frame_to_commands( &sample_frame( 3.0, 4.0 ) ) ).expect( "submit() must succeed" );

  let output = backend.output().expect( "output() must succeed after a clean submit" );
  assert!( matches!( output, tilemap_renderer::backend::Output::String( _ ) ), "SVG backend must return string output" );
}

/// AF2 — verifies real (not faked) resource threading between `frame_to_commands`'
/// geometry ids and the backend, using `SvgBackend`'s actual, current contract
/// (BUG-209, fixed 2026-08-16/17, `task/bug/completed/209_...md`): `cmd_mesh` now
/// checks the referenced geometry id against `geometries_load`'s own bookkeeping
/// *before* generating output, returning `RenderError::MissingAsset` for a `Mesh`
/// whose geometry was never loaded at all (`src/adapters/svg.rs` —
/// `cmd_mesh`/`geometry_known`), matching `svg_backend_test.rs`'s own
/// `mesh_command_missing_asset_returns_error`. This proves the same thing AF2 always
/// cared about — that `frame_to_commands`' geometry ids are real, correctly-threaded
/// references, not coincidentally-succeeding garbage — even more directly than the
/// pre-BUG-209 silent-skip contract did: an explicit `Err` naming the exact
/// unresolved id is harder to satisfy by accident than an absent `<use>` tag. Output
/// *presence* when assets are loaded is covered separately by `t03`/`t05`.
#[ test ]
fn af2_submit_without_loaded_assets_returns_missing_asset_error()
{
  let mut backend = SvgBackend::new( RenderConfig::default() );
  backend.assets_load( &empty_assets() ).expect( "loading an empty (but real) Assets value must itself succeed" );

  let result = backend.submit( &frame_to_commands( &sample_frame( 0.0, 0.0 ) ) );
  assert!(
    matches!( result, Err( RenderError::MissingAsset( id ) ) if id == BALL_GEOMETRY.inner() ),
    "ball Mesh command references a geometry id `empty_assets()` never loaded, must report it by id: {result:?}"
  );
}
