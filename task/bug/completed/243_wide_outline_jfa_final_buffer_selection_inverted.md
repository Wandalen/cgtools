# BUG-243: `WideOutlinePass::outline_pass` reads the final JFA result from the wrong ping-pong
buffer -- inverted parity, always one step stale

- **Severity:** Low (the wide-outline post-processing pass reads a one-JFA-step-stale distance
  field on every real invocation, producing a visually thinner/less-converged outline than
  requested; no panic, no crash, degraded visual quality only)
- **state:** Completed
- **Affects:** `renderer::webgl::post_processing::outline::wide_outline::WideOutlinePass::outline_pass`,
  every invocation (`num_passes` is hardcoded to `4` in `WideOutlinePass::new`, so this is not a
  configuration-dependent edge case -- it fires on 100% of real usage of this pass)
- **Component:** `module/helper/renderer` (`src/webgl/post_processing/outline/wide_outline.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`outline_pass` is the final stage of the Jump Flooding Algorithm (JFA) wide-outline pipeline: it
reads whichever of the two ping-pong framebuffers (`jfa_step_fb_color_0`/`_1`) holds the
fully-converged distance field and uses it to shade the outline. It selected the buffer by
re-deriving parity from `num_passes` directly (`num_passes % 2 == 0` selecting buffer 0) instead
of asking which buffer the *last* JFA step actually rendered into -- and got the opposite answer.

## Impact

**Who is affected:** Every consumer of `WideOutlinePass` (the `outline`/`narrow_outline` example
scenes and any application embedding this pass) -- `num_passes` is hardcoded to `4` in `new`, so
every real render hits this path, not just an edge case.

**What breaks:** `outline_pass` reads the distance field as it stood after JFA step `num_passes -
2`, one full JFA iteration behind the actual final state written by step `num_passes - 1`. JFA's
per-step search radius halves each iteration (`outline_thickness / 2^i`), so the visible effect is
a systematically thinner, less-converged outline than the configured `outline_thickness` should
produce -- not a crash, not a visibly "broken" render (a stale-but-valid distance field still
looks like *a* distance field), which is exactly why this survived undetected: nothing panics
either way.

**Entity Scope:** `None` -- source-level logic defect, not entity directory instances.

## How Discovered

During this session's `renderer` crate scout (task #174), a `general-purpose` subagent fork
dispatched to review the post-processing subsystem (read-only, no fixes) flagged a parity
mismatch between `jfa_step_pass`'s ping-pong target selection and `outline_pass`'s final-buffer
selection. Independently re-derived via a general mathematical argument before accepting the
finding (see `## Root Cause`), confirmed for `num_passes` in `{1,2,3,4}`, and confirmed the
hardcoded `num_passes = 4` in `new` means every real invocation is affected.

## Minimum Reproducible Example

No GPU-context MRE is practical or in keeping with this crate's own conventions -- see
`## Why Not Caught` for why a live-pixel test isn't used here. The defect is fully captured by a
pure parity argument, reproduced as a native unit test in `tests/webgl/jfa_buffer_selection.rs`
(no GL context required).

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p renderer --test tests jfa_buffer_selection
```
**Expected** (fixed): both tests pass --
`final_jfa_buffer_matches_last_step_actually_rendered_for_default_num_passes` asserts
`!WideOutlinePass::jfa_step_targets_fb0(3)` ( `num_passes = 4` -> last step `i = 3`, odd -> must
target `jfa_step_fb_1` ). **Actual** (pre-fix reasoning, see `## Root Cause`): the old inline
formula in `outline_pass`, `self.num_passes % 2 == 0`, evaluates to `true` for `num_passes = 4`
and selects `jfa_step_fb_color_0` -- the opposite buffer from the one this test proves the last
step actually wrote.

## Root Cause

`jfa_step_pass(i)` renders step `i` into `jfa_step_fb_0` when `i` is even (including `i == 0`),
`jfa_step_fb_1` when `i` is odd. The loop in `render()` calls `jfa_step_pass(gl, i)` for `i` in
`0..num_passes`, so the *last* step actually executed is `i = num_passes - 1`, not `num_passes`
itself.

`outline_pass` (pre-fix) computed its final-buffer selection as:
```rust
if self.num_passes % 2 == 0 { /* use jfa_step_fb_color_0 */ }
else                        { /* use jfa_step_fb_color_1 */ }
```
This checks the parity of `num_passes`, but the buffer that holds the final result is determined
by the parity of `num_passes - 1` (the last step's own index). For any `num_passes`, these two
parities are always opposite (`n` and `n - 1` differ in parity by construction) -- so the old
formula selected the buffer the last step did *not* write to, in every case, not just the
hardcoded `num_passes = 4` one. Concretely: `num_passes = 4` -> last step `i = 3` (odd) -> step 3
wrote `jfa_step_fb_1` -> `outline_pass` should read `jfa_step_fb_color_1`, but
`4 % 2 == 0` selected `jfa_step_fb_color_0` instead.

## Why Not Caught

`tests/webgl/wide_outline.rs` (BUG-179's own regression test) is this crate's only existing
`WideOutlinePass` coverage -- a `wasm_bindgen_test` that renders through a real headless WebGL2
context and asserts the call succeeds without panicking. Its own header comment already documents
that pixel-level outline correctness is "delegated to visual inspection per this crate's existing
convention for this code area," and a workspace-wide grep for `read_pixels`/`readPixels` under
`module/helper/renderer/tests/` returns zero matches -- no pixel-readback precedent exists for any
WebGL pass in this crate (unlike the native/WebGPU side, which does have real GPU pixel readback
via `native_render_test.rs`). Reading a one-step-stale-but-still-valid distance-field texture does
not panic, does not produce an obviously wrong-looking image (JFA output is inherently a somewhat
abstract intermediate texture, not the final rendered frame), and produces no type or shape
mismatch a compiler could catch -- the only way to notice is comparing the two parity derivations
against each other directly, which nothing did before this scout.

## Fix Applied (2026-08-17)

**`src/webgl/post_processing/outline/wide_outline.rs`:** extracted the ping-pong parity rule into
a single new `pub fn jfa_step_targets_fb0(i: u32) -> bool { i % 2 == 0 }` associated function on
`WideOutlinePass`, with a doc comment establishing it as the sole source of truth for "which
buffer does step `i` render into." Rewired both existing call sites to defer to it instead of each
independently re-deriving the parity:
- `jfa_step_pass`: its own render-target selection now calls `Self::jfa_step_targets_fb0(i)`; its
  read-source selection (previously duplicated inline alongside the target selection) now calls
  `Self::jfa_step_targets_fb0(i - 1)` for `i > 0` (the buffer the *previous* step wrote to), `i ==
  0` keeping its existing special case (reads the JFA-init result, not a ping-pong buffer).
- `outline_pass`: its final-buffer selection now calls
  `Self::jfa_step_targets_fb0(self.num_passes.saturating_sub(1))` -- the last step actually run --
  instead of re-deriving parity from `num_passes` itself.

This eliminates the possibility of the two call sites drifting out of sync again, since there is
now exactly one place the parity rule is spelled out.

**`tests/webgl/jfa_buffer_selection.rs`** (new file, registered in `tests/webgl/mod.rs`): two
native `#[test]` functions requiring no GL context, following the established `jfa_step_size.rs`
(BUG-180) precedent for JFA logic with no practical live-pixel test path in this crate --
`final_jfa_buffer_matches_last_step_actually_rendered_for_default_num_passes` asserts the
real-world `num_passes = 4` case directly, `jfa_step_targets_fb0_alternates_starting_true_at_zero`
asserts the general alternation pattern for `i` in `0..5`.

## Verification

`longrun`-detached, from repo root:
- `cargo test -p renderer --test tests jfa_buffer_selection` -- 2 passed, 0 failed.
- `verb/test_only pkg::renderer` (full scoped suite, post-fix): **138 tests run: 138 passed, 0
  skipped** (24s) -- up from 136 (this bug's 2 new tests), including the real GPU-backed
  `native_render_test.rs::opaque_path_renders_lit_quad` and the pre-existing wasm-gated
  `wide_outline.rs` structural test (compiles cleanly under the native target; its
  `wasm_bindgen_test` body itself only runs under a browser target, unaffected either way).
- `cargo clippy -p renderer --all-features --all-targets -- -D warnings`: exit 0, clean.

No literal revert-and-rerun against the live `outline_pass` GL call site was possible (it needs a
real WebGL2 context this crate's native test suite doesn't construct, and this crate's own
documented convention delegates pixel-level correctness for this pass to visual inspection, not
pixel-readback tests) -- verification instead rests on the direct mathematical argument in `##
Root Cause` (the old and new formulas provably select opposite buffers for every `num_passes`,
confirmed for `{1,2,3,4}`) plus the new unit test asserting the fixed function's real-world-case
answer, mirroring the BUG-180 (`jfa_step_size.rs`) precedent for this same code area.

## Generalized Version

**Broken assumption:** two independently-written expressions that are each individually plausible
will stay in agreement without a shared source. Here, "which buffer holds the final JFA result"
was computed once from "how many steps total" and once (implicitly, via the ping-pong loop logic)
from "which step ran last" -- related quantities (`num_passes` vs. `num_passes - 1`), differing by
exactly one, whose parities are *always* opposite. Whenever two call sites must agree on a
derived fact, especially one involving an index/count off-by-one relationship, extract a single
named function both defer to rather than trusting each site's own `% 2` reasoning to reach the
same conclusion independently. Structurally the same defect class as BUG-241 (two
independently-maintained things that must agree but didn't).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found by a scouting-fork review of `renderer`'s post-processing subsystem during task #174, independently re-derived via a general mathematical argument (confirmed for num_passes in {1,2,3,4}) before acceptance. Root cause: `outline_pass` selected the final JFA buffer from the parity of `num_passes` instead of `num_passes - 1` (the last step actually run) -- always the opposite buffer. Fixed by extracting a single `jfa_step_targets_fb0(i)` associated function both `jfa_step_pass` and `outline_pass` now defer to. Verified via 2 new native unit tests (no GL context needed, following the BUG-180 precedent) plus the full 138/138 scoped suite and clean clippy. Closed same-session (Tier 2 Dual-Role Self-Check). |
