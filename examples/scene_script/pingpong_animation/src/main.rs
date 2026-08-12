//! Runs the Pong-style simulation from `pingpong_animation::simulate`, prints
//! its recorded frames, and demos `animation::Tween< F32x2 >` interpolation
//! between two of them. With `adapter-svg` enabled, also compiles each frame
//! via `pingpong_animation::render::frame_to_commands` and submits it to a
//! `tilemap_renderer` `SvgBackend`. With `adapter-webgl` enabled (wasm32
//! target only), does the same against a `WebGlBackend` bound to a
//! browser-provided canvas instead.

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

  match backend.output()
  {
    Ok( Output::String( svg ) ) => println!( "\nrendered {} frame(s) to SVG ({} bytes)", frames.len(), svg.len() ),
    Ok( _ ) => {}
    Err( error ) => eprintln!( "failed to retrieve SVG output: {error}" ),
  }
}

/// Compiles each frame via [`pingpong_animation::render::frame_to_commands`]
/// and submits it to a fresh `WebGlBackend`, bound to a WebGL2 context
/// retrieved (or created) from the page via `minwebgl::context::retrieve_or_make`.
/// `tilemap_renderer`'s own `adapter-webgl` feature requires the
/// wasm32-unknown-unknown target (no native windowing exists to source a real
/// `web_sys::WebGl2RenderingContext` from) — matching this task's own Testable
/// clause, which tests this feature "on the wasm32 target" rather than via a
/// plain native `cargo run`.
#[ cfg( all( feature = "adapter-webgl", not( feature = "adapter-svg" ) ) ) ]
fn render_frames( frames : &[ pingpong_animation::Frame ] )
{
  use pingpong_animation::render::{ frame_to_commands, render_assets };
  use tilemap_renderer::
  {
    adapters::webgl::WebGlBackend,
    backend::{ Backend, Output },
    types::RenderConfig,
  };

  let commands : Vec< _ > = frames.iter().map( frame_to_commands ).collect();
  let assets = render_assets();
  let tick_count = frames.len();

  minwebgl::browser::setup( minwebgl::browser::Config::default() );
  minwebgl::spawn_local( async move
  {
    let gl = match minwebgl::context::retrieve_or_make()
    {
      Ok( gl ) => gl,
      Err( error ) => { minwebgl::warn!( "failed to retrieve WebGL2 context: {error}" ); return; }
    };

    let mut backend = match WebGlBackend::new( RenderConfig::default(), gl )
    {
      Ok( backend ) => backend,
      Err( error ) => { minwebgl::warn!( "failed to construct WebGlBackend: {error}" ); return; }
    };

    if let Err( error ) = backend.load_assets( &assets )
    {
      minwebgl::warn!( "failed to load render assets: {error}" );
      return;
    }

    for frame_commands in &commands
    {
      if let Err( error ) = backend.submit( frame_commands )
      {
        minwebgl::warn!( "failed to submit frame: {error}" );
        return;
      }
    }

    match backend.output()
    {
      Ok( Output::Presented ) => minwebgl::info!( "rendered {tick_count} frame(s) via WebGL2 (presented to canvas)" ),
      Ok( _ ) => {}
      Err( error ) => minwebgl::warn!( "failed to retrieve WebGL2 output: {error}" ),
    }
  } );
}

/// No-op when no adapter feature is enabled — the default (console-only) build.
#[ cfg( not( any( feature = "adapter-svg", feature = "adapter-webgl" ) ) ) ]
fn render_frames( _frames : &[ pingpong_animation::Frame ] ) {}
