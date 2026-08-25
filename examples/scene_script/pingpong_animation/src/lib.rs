//! Simulates a Pong-style scene entirely from Rhai (loops + branches +
//! `F32x2` vector arithmetic) and exposes the recorded per-tick `Frame`
//! sequence — the animation-glue layer reuses the real `animation` crate for
//! interpolation and `tilemap_renderer` (via [`render`]) for compilation into
//! `RenderCommand`s, not placeholder math of its own.

use ndarray_cg::F32x2;
use scene_script::{ engine_build, script_as_glue_load };
use std::{ cell::RefCell, rc::Rc };

/// Compiles `Frame`s into `tilemap_renderer::commands::RenderCommand`s.
/// Backend-agnostic (no adapter-specific logic), but its types come from the
/// optional `tilemap_renderer` dependency, so it only exists once a backend
/// feature pulls that dependency in.
#[ cfg( any( feature = "adapter-svg", feature = "adapter-webgl" ) ) ]
pub mod render;

/// One simulated tick: ball position plus both paddles' vertical offsets.
#[ derive( Debug, Clone, PartialEq ) ]
pub struct Frame
{
  /// Simulation tick index, starting at 0.
  pub tick : i64,
  /// Ball position in world space.
  pub ball : F32x2,
  /// Left paddle's vertical offset.
  pub paddle_left_y : f64,
  /// Right paddle's vertical offset.
  pub paddle_right_y : f64,
}

/// Builds a fresh engine, evaluates the bundled script, and returns every
/// emitted frame — the entire simulation as a pure function of the script's
/// own hardcoded inputs, no external state in or out. Off-screen (no GPU, no
/// browser) and, per L5's contract, deterministic: see
/// `simulation_is_deterministic` in `tests/simulation_test.rs`.
///
/// # Errors
///
/// Returns the Rhai evaluation error if `pingpong_animation.rhai` fails to
/// parse or run.
pub fn simulate() -> Result< Vec< Frame >, Box< rhai::EvalAltResult > >
{
  let mut engine = engine_build();

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
  let ast = script_as_glue_load( &engine, script )
  .map_err( | err | rhai::EvalAltResult::ErrorSystem( "pingpong_animation.rhai".into(), Box::new( err ) ) )?;
  let _ : rhai::Dynamic = engine.eval_ast( &ast )?;

  // `engine` still holds a clone of `frames_sink` inside the registered
  // closure at this point, so `Rc::try_unwrap` would see 2 strong refs and
  // fail — clone the `Vec` out of the shared `RefCell` instead. Bound to a
  // local first: inlined into `Ok( frames.borrow().clone() )`, the `Ref`
  // guard's temporary-scope extension to the end of the block conflicts
  // with `frames` itself dropping there too (E0597).
  let recorded_frames = frames.borrow().clone();
  Ok( recorded_frames )
}
