//! Simulates a Pong-style scene entirely from Rhai (loops + branches +
//! `F32x2` vector arithmetic), then uses `animation::Tween< F32x2 >` (via
//! `scene_script::register_tween_f32x2`) to smoothly interpolate between
//! two recorded ball positions — the animation-glue layer reuses the real
//! `animation` crate, not placeholder lerp math.

use ndarray_cg::F32x2;
use scene_script::build_engine;
use std::{ cell::RefCell, rc::Rc };

#[ derive( Debug, Clone ) ]
struct Frame
{
  tick : i64,
  ball : F32x2,
  paddle_left_y : f64,
  paddle_right_y : f64,
}

fn main() -> Result< (), Box< rhai::EvalAltResult > >
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

  let frames = frames.borrow();
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
