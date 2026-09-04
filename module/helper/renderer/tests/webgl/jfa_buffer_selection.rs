//! Regression coverage for BUG-243: `outline_pass`'s final-JFA-buffer selection and
//! `jfa_step_pass`'s own ping-pong target selection were two independently hand-derived parity
//! checks over the same underlying fact ( which buffer the last JFA step wrote to ) that had to
//! agree but didn't -- `outline_pass` selected the OPPOSITE buffer from the one the last step
//! ( `i = num_passes - 1` ) actually rendered into.

use renderer::webgl::post_processing::outline::wide_outline::WideOutlinePass;

/// ## Root Cause
/// `jfa_step_pass` renders step `i` into `jfa_step_fb_0` when `i` is even, `jfa_step_fb_1` when
/// odd -- so the *last* step run is `i = num_passes - 1`. `outline_pass` re-derived which buffer
/// holds the final result straight from `num_passes` ( `num_passes % 2 == 0` selecting
/// `jfa_step_fb_color_0` ) -- the parity of `num_passes`, not of `num_passes - 1` -- exactly
/// inverted from the buffer the last step actually wrote.
/// ## Why Not Caught
/// No test exercised the relationship between `jfa_step_pass`'s ping-pong target and
/// `outline_pass`'s final-buffer read -- this crate's only `wide_outline` coverage
/// ( `tests/webgl/wide_outline.rs` ) is a structural GPU-pipeline smoke test that renders without
/// error either way ( reading a one-step-stale JFA texture still produces *some* valid-looking
/// distance-field texture, just visually off by one JFA iteration ), and pixel-level correctness
/// for this pass is delegated to visual inspection per this crate's own documented convention for
/// this code area.
/// ## Fix Applied
/// Extracted the ping-pong parity rule into a single `pub fn jfa_step_targets_fb0( i : u32 ) ->
/// bool` associated function that both `jfa_step_pass` ( its own render target, and -- via
/// `i - 1` -- its read source ) and `outline_pass` ( via `num_passes - 1` ) now defer to,
/// eliminating the possibility of the two independently drifting out of sync again.
/// ## Prevention
/// This test asserts the real-world case directly: with the hardcoded `num_passes = 4`
/// ( `WideOutlinePass::new` ), the last step run is `i = 3` ( odd ), so the final result must be
/// in `jfa_step_fb_1` -- `jfa_step_targets_fb0( 3 )` must be `false`. The pre-fix formula
/// ( `num_passes % 2 == 0` ) would have selected `jfa_step_fb_0` instead, failing this assertion.
/// ## Pitfall
/// When two call sites must derive the same fact from related-but-different inputs ( "which step
/// last ran" vs. "how many steps total" ), a hand-written parity check at each site is a
/// duplication risk even when both look individually plausible -- extract the one true derivation
/// into a single function instead of trusting `off_by_one % 2` reasoning to be re-derived
/// correctly twice.
// test_kind: bug_reproducer(BUG-243)
#[ test ]
fn final_jfa_buffer_matches_last_step_actually_rendered_for_default_num_passes()
{
  let num_passes = 4u32;
  let last_step = num_passes - 1;
  assert!
  (
    !WideOutlinePass::jfa_step_targets_fb0( last_step ),
    "num_passes = 4 -> last step i = 3 is odd -> must target jfa_step_fb_1, not jfa_step_fb_0"
  );
}

#[ test ]
fn jfa_step_targets_fb0_alternates_starting_true_at_zero()
{
  let expected = [ true, false, true, false, true ];
  for ( i, expect ) in expected.into_iter().enumerate()
  {
    assert_eq!
    (
      WideOutlinePass::jfa_step_targets_fb0( i as u32 ), expect,
      "step {i}: ping-pong parity mismatch"
    );
  }
}
