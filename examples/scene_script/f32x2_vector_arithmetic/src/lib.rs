//! Evaluates a `.rhai` scene script that constructs and combines `F32x2`
//! vectors using ordinary `+`/`*` -- demonstrating that Rhai's custom-type
//! operator overloading reuses `ndarray_cg`'s own `std::ops` implementations
//! directly, rather than reimplementing arithmetic on the Rhai side.

use ndarray_cg::F32x2;
use scene_script::{ engine_build, script_as_glue_load };

/// Builds a fresh engine and evaluates the bundled script, returning its
/// final `F32x2` result -- a pure function of the script's own hardcoded
/// inputs, no external state in or out. Off-screen (no GPU, no browser) and,
/// per L5's contract, deterministic: see `arithmetic_is_deterministic` in
/// `tests/determinism_test.rs`.
///
/// # Errors
///
/// Returns the Rhai evaluation error if `f32x2_vector_arithmetic.rhai` fails
/// to parse or run.
pub fn evaluate() -> Result< F32x2, Box< rhai::EvalAltResult > >
{
  let engine = engine_build();
  let script = include_str!( "f32x2_vector_arithmetic.rhai" );
  let ast = script_as_glue_load( &engine, script )
  .map_err( | err | rhai::EvalAltResult::ErrorSystem( "f32x2_vector_arithmetic.rhai".into(), Box::new( err ) ) )?;
  engine.eval_ast::< F32x2 >( &ast )
}
