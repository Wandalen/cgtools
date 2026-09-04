# BUG-242: `Blender::is_completed()` never implements its own "all animations completed"
contract -- ties always report incomplete, non-ties check only one animation

- **Severity:** Medium (public API method that can never correctly answer the question its own
  doc comment claims to answer, for any Blender holding 2+ animations; no in-tree callers today,
  but a live, silently-wrong result for any external consumer of this crate's `Blender` type)
- **state:** Completed
- **Affects:** `renderer::webgl::animation::Blender::is_completed()`, any consumer with 2 or more
  weighted animations added via `Blender::add`. Single-animation `Blender`s are unaffected (that
  path was already a correct passthrough). `AnimationGraph` (this crate's own animation-graph
  type) uses plain `Sequencer`, never `Blender` -- confirmed via workspace grep -- so `Blender`
  currently has zero in-tree callers; the defect is confined to direct external users of the
  public API.
- **Component:** `module/helper/renderer` (`src/webgl/animation/blending.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

Two animations genuinely completed at the identical elapsed time -- the most common real-world
shape for blended animations meant to play in lockstep (e.g. a walk/run blend where both clips
share a duration) -- and `Blender::is_completed()` unconditionally returns `false`. Two
animations at *different* elapsed times, where only the one with the larger raw `.time()` value
happens to be completed, and it unconditionally returns `true` -- even though the other,
shorter-time animation is still running.

## Impact

**Who is affected:** Any external consumer of `renderer::webgl::animation::Blender` with two or
more weighted animations. Zero in-tree callers today (this crate's own `AnimationGraph` uses
plain `Sequencer`, not `Blender`), so no shipped rendering behavior is affected -- but this is a
public, documented API method (`pub fn is_completed`) whose doc comment makes an explicit
contract claim ("checks if all animations are completed") that the implementation never actually
honors for 2+ animations.

**What breaks:** Two independent, unrelated failure modes coexist in the same function:
- **False negative (the common case):** when every animation's `.time()` values are tied within
  `EPSILON` (0.001) -- which includes the case where all are genuinely `Completed` -- the
  function returns `false` regardless of actual completion state.
- **False positive:** when times are *not* tied, the function checks only the single animation
  with the largest raw `.time()` value and ignores every other one. Since `.time()` is raw
  elapsed wall-clock time (not normalized by each animation's own duration), a longer-duration
  animation can have a larger `.time()` while a shorter-duration animation with a smaller
  `.time()` is the one still running -- yet the function reports "completed" based solely on the
  large-time entry.

**Entity Scope:** `None` -- source-level logic defect, not entity directory instances.

## How Discovered

During this session's `renderer` crate scout (task #174), a `general-purpose` subagent fork
dispatched to review the animation subsystem (read-only, no fixes) flagged `is_completed()`'s
sort/tie-detection logic as inconsistent with its own doc comment. Independently re-derived by
directly reading `blending.rs` lines 137-167 and hand-tracing both branches against the
documented "all animations completed" contract before accepting the finding.

## Minimum Reproducible Example

No synthetic MRE needed -- `tests/blender_tests.rs` reproduces both failure modes directly against
the real `Blender` type. Two regression tests were added (see `## Fix Applied`); either
demonstrates the defect in isolation when run against the pre-fix implementation.

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p renderer --features animation --test blender_tests \
  test_is_completed_two_animations_same_time_both_genuinely_completed \
  test_is_completed_larger_time_animation_completed_smaller_time_animation_not
```
**Expected** (fixed): both pass. **Actual** (pre-fix, confirmed via temporary direct-source-edit
revert-and-rerun): both fail --
`test_is_completed_two_animations_same_time_both_genuinely_completed` panics with "two animations
completed at the identical time should report the Blender as completed" (got `false`);
`test_is_completed_larger_time_animation_completed_smaller_time_animation_not` panics with "not
all animations completed -- Blender must not report completed just because the largest-time one
is" (got `true`).

## Root Cause

`is_completed()` (pre-fix) collected every weighted animation's `.time()`, sorted descending, and
walked forward from the top counting how many entries were mutually tied within `EPSILON`:
- If **not tied at the top** (`i == 1`): returned `animations[0].is_completed()` -- the single
  animation with the largest raw elapsed time, ignoring every other animation entirely. `.time()`
  is not normalized by each animation's own duration, so "largest time" has no principled
  relationship to "closest to completion", let alone "all completed".
- If **tied at the top** (`i > 1`, including the all-tied case): returned `false`
  unconditionally, regardless of whether every tied animation -- or any animation at all -- had
  actually completed.

Neither branch ever asked the question the doc comment claims to answer: "is every animation's
own `is_completed()` true". The function conflated *timing alignment* (`.time()` proximity) with
*completion state* (`AnimationState::Completed`) -- two unrelated properties.

## Why Not Caught

`tests/blender_tests.rs` already contained 5 dedicated `is_completed`-with-multiple-animations
tests before this fix, all passing. Every one of them called `Blender::update(dt)` and checked
`is_completed()` immediately afterward. `Blender::update()` has its own, separate
auto-reset-on-completion behavior (`if animation.is_completed() { animation.reset() }`, inside its
own per-animation update loop) that resets any `Sequencer` back to `time = 0.0` /
`state = Running` the instant it completes, *within that same `update()` call* -- so by the time
`is_completed()` ran, no animation could ever still be observed in the genuine `Completed` state.
The buggy aggregation logic and a correct "all completed" check produced the *identical* `false`
answer in every existing test, for entirely unrelated reasons (the buggy code's tie-detection
branch vs. the fixed code correctly seeing every animation freshly reset to non-completed) --
masking the defect completely. One existing test's own assertion message even hints at the
confusion: `test_is_completed_multiple_animations_same_time_completed` asserts
`!blender.is_completed()` with the message "Multiple animations with same delay and duration are
completed but should be applyied reset for both" -- describing animations that ARE completed
while asserting the function reports they are not, without flagging the contradiction as a defect
in `is_completed()` itself rather than expected behavior.

Reaching the genuinely-`Completed` state observably required bypassing `Blender::update()`'s
auto-reset entirely -- driving individual `Sequencer`s to completion directly via
`animation_get_mut()`, which neither the pre-existing tests nor (per the discovering fork' notes)
any in-tree caller does.

## Fix Applied (2026-08-17)

**`src/webgl/animation/blending.rs`:** replaced the entire sort/tie-detection implementation with
a direct implementation of the documented contract:
```rust
pub fn is_completed( &self ) -> bool
{
  !self.weighted_animations.is_empty()
  && self.weighted_animations.values().all( | ( s, _ ) | s.is_completed() )
}
```
The empty-collection guard matches this codebase's established convention (see BUG-231's fix in
`animation::Sequencer`: "a genuinely-empty Sequencer is never reported as having completed
work"). The now-unused `EPSILON` constant and its only use site (the sort/tie loop) were removed
entirely -- no other code in the file referenced it.

**`tests/blender_tests.rs`:** two new regression tests, each driving its animations to genuine
completion via `animation_get_mut()` directly (bypassing `Blender::update()`'s auto-reset) --
`test_is_completed_two_animations_same_time_both_genuinely_completed` (tied-time false-negative
branch) and `test_is_completed_larger_time_animation_completed_smaller_time_animation_not`
(non-tied false-positive branch). No pre-existing test's assertion needed to change: all 20
pre-existing tests are masked identically by `Blender::update()`'s auto-reset regardless of which
`is_completed()` implementation runs underneath, confirmed by running the full pre-existing suite
unchanged against both the pre-fix and post-fix implementations.

## Verification

`longrun`-detached, from repo root:
- `cargo test -p renderer --features animation --test blender_tests` -- pre-fix (temporary
  direct-source-edit revert): 20 passed, 2 failed (the two new tests, as designed). Post-fix: 22
  passed, 0 failed.
- `verb/test_only pkg::renderer` (full scoped suite, post-fix): **136 tests run: 136 passed, 0
  skipped** (27s), including the real GPU-backed `native_render_test.rs::opaque_path_renders_lit_quad`.
- `cargo clippy -p renderer --all-features --all-targets -- -D warnings`: exit 0, clean.

## Generalized Version

**Broken assumption:** "animations reaching the same elapsed time" and "animations having
completed" are the same property, or a reliable proxy for one another. False the moment
durations differ, or the moment a caller (like `Blender::update()`'s own auto-reset) changes
`.time()` independently of completion state. When a function's doc comment states a boolean
contract over a collection ("all X" / "any X"), implement it as a direct fold over each element's
own authoritative state (`Iterator::all`/`.any()`), never via a derived proxy signal (sorted
order, timing proximity) that merely correlates with the real property in the cases first tested.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found by a scouting-fork review of `renderer`'s animation subsystem during task #174, independently re-derived from source before acceptance. Root cause: `is_completed()`'s sort/EPSILON-tie logic never implemented "all completed"; masked by `Blender::update()`'s own auto-reset-on-completion in every pre-existing test. Fixed via a direct `Iterator::all` fold; verified via 2 new regression tests (confirmed to fail pre-fix, pass post-fix, via temporary revert-and-rerun) plus the full 136/136 scoped suite and clean clippy. Closed same-session (Tier 2 Dual-Role Self-Check). |
