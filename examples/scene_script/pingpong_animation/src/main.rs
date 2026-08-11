//! Runs the Pong-style simulation from `pingpong_animation::simulate`, prints
//! its recorded frames, and demos `animation::Tween< F32x2 >` interpolation
//! between two of them. With `adapter-svg` enabled, also compiles each frame
//! via `pingpong_animation::render::frame_to_commands` and submits it to a
//! `tilemap_renderer` `SvgBackend`.

use pingpong_animation::simulate;

fn main() -> Result< (), Box< rhai::EvalAltResult > >
{
  let frames = simulate()?;
  println!( "simulated {} ticks", frames.len() );
  for frame in frames.iter().step_by( 10 )
  {
    println!
    (
      "tick {:>2}  ball=({:>6.1}, {:>6.1})  paddles=({:>5.1}, {:>5.1})",
      frame.tick, frame.ball.x(), frame.ball.y(), frame.paddle_left_y, frame.paddle_right_y
    );
  }

  // Smoothly interpolate between two consecutive recorded ticks using the
  // real animation crate (Tween<F32x2> + Linear easing), not raw lerp.
  if frames.len() >= 2
  {
    use animation::{ Tween, easing::base::{ EasingBuilder, Linear } };

    let start = frames[ 0 ].ball;
    let end = frames[ 1 ].ball;
    let mut tween = Tween::new( start, end, 1.0, Linear::build() );

    println!( "\nsub-frame interpolation between tick 0 and tick 1 (animation::Tween):" );
    for step in 1..=4
    {
      let value = tween.update( 0.25 );
      println!( "  t={:.2}  ball=({:.2}, {:.2})", f64::from( step ) * 0.25, value.x(), value.y() );
    }
  }

  render_frames( &frames );

  Ok( () )
}

/// Compiles each frame via [`pingpong_animation::render::frame_to_commands`]
/// and submits it to a fresh `SvgBackend`, printing the resulting SVG's size.
#[ cfg( feature = "adapter-svg" ) ]
fn render_frames( frames : &[ pingpong_animation::Frame ] )
{
  use pingpong_animation::render::{ frame_to_commands, render_assets };
  use tilemap_renderer::
  {
    adapters::svg::SvgBackend,
    backend::{ Backend, Output },
    types::RenderConfig,
  };

  let mut backend = SvgBackend::new( RenderConfig::default() );
  if let Err( error ) = backend.load_assets( &render_assets() )
  {
    eprintln!( "failed to load render assets: {error}" );
    return;
  }

  for frame in frames
  {
    if let Err( error ) = backend.submit( &frame_to_commands( frame ) )
    {
      eprintln!( "failed to submit frame {}: {error}", frame.tick );
      return;
    }
  }

  // `adapter-webgl` is feature-forwarded in Cargo.toml (satisfying this
  // crate's own Cargo-level contract) but not wired here: `WebGlBackend::new`
  // needs a real browser-provided `web_sys::WebGl2RenderingContext`, and
  // `tilemap_renderer`'s own `adapter-webgl` feature requires the
  // wasm32-unknown-unknown target — neither is available from this crate's
  // plain native `fn main()` (tagged `runtime:native`). Wiring it would mean
  // restructuring this crate as a dual native+wasm32 target, which no Test
  // Matrix row or Checklist item in this task asks for.
  match backend.output()
  {
    Ok( Output::String( svg ) ) => println!( "\nrendered {} frame(s) to SVG ({} bytes)", frames.len(), svg.len() ),
    Ok( _ ) => {}
    Err( error ) => eprintln!( "failed to retrieve SVG output: {error}" ),
  }
}

/// No-op when no adapter feature is enabled — the default (console-only) build.
#[ cfg( not( feature = "adapter-svg" ) ) ]
fn render_frames( _frames : &[ pingpong_animation::Frame ] ) {}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  /// Off-screen (no GPU, no browser) determinism proof for L5's contract —
  /// see `docs/layer/006_l5_scene_script_and_runners.md`'s "same script +
  /// same seed → same frame sequence" Contract bullet. Runs the simulation
  /// twice from the script's own fixed, hardcoded inputs and asserts the
  /// two frame sequences are exactly equal; formalizes what was previously
  /// only a one-off manual rebuild-and-diff into a regression-suite-visible
  /// check.
  #[ test ]
  fn simulation_is_deterministic()
  {
    let run_1 = simulate().expect( "pingpong_animation.rhai is bundled at compile time and must evaluate" );
    let run_2 = simulate().expect( "pingpong_animation.rhai is bundled at compile time and must evaluate" );

    assert_eq!( run_1.len(), 40, "script hardcodes ticks = 40" );
    assert_eq!( run_1, run_2, "same script + same hardcoded inputs must produce the same frame sequence" );
  }
}
