//! Runs a `.rhai` scene script that constructs and combines `F32x2` vectors
//! using ordinary `+`/`*` — demonstrating that Rhai's custom-type operator
//! overloading reuses `ndarray_cg`'s own `std::ops` implementations
//! directly, rather than reimplementing arithmetic on the Rhai side.

use ndarray_cg::F32x2;
use scene_script::engine_build;

fn main() -> Result< (), Box< rhai::EvalAltResult > >
{
  let engine = engine_build();
  let script = include_str!( "f32x2_vector_arithmetic.rhai" );

  let hub_pos : F32x2 = engine.eval::< F32x2 >( script )?;

  println!( "hub_pos = ({}, {})", hub_pos.x(), hub_pos.y() );
  assert_eq!( hub_pos, F32x2::new( 16.0, 14.0 ) );

  Ok( () )
}
