//! Simulates a Pong-style scene entirely from Rhai (loops + branches +
//! `F32x2` vector arithmetic), then uses `animation::Tween< F32x2 >` (via
//! `scene_script::register_tween_f32x2`) to smoothly interpolate between
//! two recorded ball positions — the animation-glue layer reuses the real
//! `animation` crate, not placeholder lerp math.

use ndarray_cg::F32x2;
use scene_script::build_engine;
use std::{ cell::RefCell, rc::Rc };

#[ derive( Debug, Clone, PartialEq ) ]
struct Frame
{
  tick : i64,
  ball : F32x2,
  paddle_left_y : f64,
  paddle_right_y : f64,
}

/// Builds a fresh engine, evaluates the bundled script, and returns every
/// emitted frame — the entire simulation as a pure function of the script's
/// own hardcoded inputs, no external state in or out. Off-screen (no GPU, no
/// browser) and, per L5's contract, deterministic: see
/// `simulation_is_deterministic` below.
fn simulate() -> Result< Vec< Frame >, Box< rhai::EvalAltResult > >
{
  let mut engine = build_engine();

  let frames : Rc< RefCell< Vec< Frame > > > = Rc::new( RefCell::new( Vec::new() ) );
  let frames_sink = frames.clone();

  engine.register_fn
  (
    "emit_frame",
    move | tick : i64, ball : F32x2, paddle_left_y : f64, paddle_right_y : f64 |
    {
      frames_sink.borrow_mut().push( Frame { tick, ball, paddle_left_y, paddle_right_y } );
    }
  );

  let script = include_str!( "pingpong_animation.rhai" );
  let _ : rhai::Dynamic = engine.eval( script )?;

  // `engine` still holds a clone of `frames_sink` inside the registered
  // closure at this point, so `Rc::try_unwrap` would see 2 strong refs and
  // fail — clone the `Vec` out of the shared `RefCell` instead. Bound to a
  // local first: inlined into `Ok( frames.borrow().clone() )`, the `Ref`
  // guard's temporary-scope extension to the end of the block conflicts
  // with `frames` itself dropping there too (E0597).
  let recorded_frames = frames.borrow().clone();
  Ok( recorded_frames )
}

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
    let mut tween = Tween::new( start, end, 1.0, Linear::new() );

    println!( "\nsub-frame interpolation between tick 0 and tick 1 (animation::Tween):" );
    for step in 1..=4
    {
      let value = tween.update( 0.25 );
      println!( "  t={:.2}  ball=({:.2}, {:.2})", f64::from( step ) * 0.25, value.x(), value.y() );
    }
  }

  Ok( () )
}

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
