//! Runs `f32x2_vector_arithmetic::evaluate` and prints the resulting `F32x2`
//! — demonstrating that Rhai's custom-type operator overloading reuses
//! `ndarray_cg`'s own `std::ops` implementations directly, rather than
//! reimplementing arithmetic on the Rhai side.

use f32x2_vector_arithmetic::evaluate;

fn main() -> Result< (), Box< rhai::EvalAltResult > >
{
  let hub_pos = evaluate()?;
  println!( "hub_pos = ({}, {})", hub_pos.x(), hub_pos.y() );

  Ok( () )
}
