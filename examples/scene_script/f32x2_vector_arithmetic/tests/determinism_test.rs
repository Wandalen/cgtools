//! Off-screen determinism proof for the `F32x2` vector-arithmetic script --
//! L5's "same script + same seed -> same frame sequence" contract, applied
//! to a single-value script rather than a per-tick frame sequence.
//!
//! Relocated from `src/main.rs`'s own assertion, per the all-tests-in-tests/
//! convention already applied to `pingpong_animation`.

use f32x2_vector_arithmetic::evaluate;
use ndarray_cg::F32x2;

/// Off-screen (no GPU, no browser) determinism proof for L5's contract --
/// see `docs/layer/006_l5_scene_script_and_runners.md`'s "same script +
/// same seed -> same frame sequence" Contract bullet. Runs the script twice
/// from its own fixed, hardcoded inputs and asserts both results are exactly
/// equal to each other and to the known closed-form value.
#[ test ]
fn arithmetic_is_deterministic()
{
  let run_1 = evaluate().expect( "f32x2_vector_arithmetic.rhai is bundled at compile time and must evaluate" );
  let run_2 = evaluate().expect( "f32x2_vector_arithmetic.rhai is bundled at compile time and must evaluate" );

  assert_eq!( run_1, F32x2::new( 16.0, 14.0 ), "script hardcodes a + b * 2.0 = (10,20) + (3,-3)*2.0 = (16,14)" );
  assert_eq!( run_1, run_2, "same script + same hardcoded inputs must produce the same result" );
}
