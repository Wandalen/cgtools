//! Off-screen determinism proof for the pingpong simulation — L5's "same
//! script + same seed → same frame sequence" contract.
//!
//! Relocated from `src/main.rs`, per the all-tests-in-tests/ convention.

use pingpong_animation::simulate;

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
