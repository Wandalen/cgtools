# BUG-232: `Tween`'s finite `repeat_count` overshoots when a single large `delta_time` crosses more repeat boundaries than remain in the budget

- **Severity:** High (a frame stall, a backgrounded tab, or a deliberate fast-forward -- all
  realistic in a real-time web/game context this crate targets -- can pass a `delta_time` large
  enough to trigger this; the observable consequence is a `Tween` silently running extra,
  unrequested loops with `current_repeat()` reporting a value past `repeat_count`, not merely a
  cosmetic drift)
- **state:** Completed
- **Affects:** Any finite-`repeat_count` `Tween` (`with_repeat(n)` for `n > 0`) driven by a
  single `update(delta_time)` call whose `delta_time` spans more repeat boundaries than remain
  in the configured budget at that moment.
- **Component:** `module/helper/animation` (`src/interpolation.rs`, `Tween::repeat_handle`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** TASK-015 (`repeat_handle`'s earlier `.min(0.0)`/`.max(0.0)` overflow-elapsed
  fix, same method, different branch logic -- that fix corrected the *leftover elapsed* after a
  single crossing; this bug is about *how many crossings* a single call is allowed to consume
  against the remaining budget, an independent defect in the same function). Not related to
  BUG-231 (`Sequencer::remove`) beyond both living in `module/helper/animation`.

## Symptom

```rust
// pre-fix -- finite-repeat branch of repeat_handle
else if self.repeat_count > 0 && self.current_repeat < self.repeat_count
{
  let repeats : i32 = elapsed_repeats as i32;
  self.current_repeat += repeats;                                          // no cap vs repeat_count
  self.elapsed = ( self.elapsed - ( self.duration * elapsed_repeats ) ).max( 0.0 );
  self.state = AnimationState::Running;                                    // never re-checked against repeat_count
}
```

A `Tween::new( 0.0, 10.0, 1.0, ... ).with_repeat( 2 )` driven by one `update( 3.5 )` call
(`elapsed_repeats = 3`, but only 2 repeats remain) ends the call with `current_repeat() == 5`
(the raw crossed-boundary count added on top of two prior calls in the reproducer, or `3` in a
single fresh call) and `state() == Running` -- the Tween should instead have completed the
moment its 2-repeat budget was exhausted mid-call.

## Impact

**Who is affected:** Any caller whose `delta_time` can spike past multiple repeat boundaries in
one `update()` call -- frame stalls, backgrounded/throttled browser tabs, debugger pauses, or
deliberate fast-forward/simulation use, all ordinary occurrences in a real-time web/game
context.

**What breaks:** `current_repeat()` can report a value greater than `repeat_count()`, breaking
any caller computing progress as `current_repeat() / repeat_count()` (yields >100%) or comparing
the two directly. `state()` stays `Running` instead of `Completed` at the exact call where the
budget is exhausted, so the Tween keeps animating (with `yoyo` direction now driven by a
`current_repeat` value whose parity no longer matches any real loop count) for one or more
unrequested extra loops before a later crossing finally detects `current_repeat >=
repeat_count` and completes.

**Magnitude:** 1 branch (`repeat_handle`'s finite-repeat arm), 1 missing budget check.

**Entity Scope:** None -- a code-level defect.

## How Discovered

This session's scouting pass of `animation`, reading `Tween::repeat_handle` in full and tracing
the finite-repeat branch's arithmetic against a large-`delta_time` scenario. Initial analysis
mistakenly suspected the *ordinary* single-crossing-per-call completion timing was itself off by
one loop; re-reading the existing `test_tween_finite_repeat` test (which explicitly documents
and asserts 3 total plays for `repeat_count( 2 )`, i.e. "N repeats after the first play") showed
that behavior is correct and intentional -- the real, narrower defect only appears when a single
call's `elapsed_repeats` exceeds the *remaining* budget in one batch, which no existing test
ever drives.

## Minimum Reproducible Example

```rust
let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ).with_repeat( 2 );
tween.update( 3.5 ); // crosses 3 repeat boundaries in one call; only 2 repeats are allowed
assert!( tween.is_completed() );        // pre-fix: false (still Running)
assert_eq!( tween.current_repeat(), 2 ); // pre-fix: 3 (overshot repeat_count)
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/animation && cargo nextest run --all-features -E 'test(test_tween_finite_repeat_large_delta_completes_without_overshooting_repeat_count)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The finite-repeat branch adds this call's entire `elapsed_repeats` to `current_repeat` with no check against `repeat_count`, so a single call crossing more boundaries than remain in the budget lets `current_repeat` overshoot while `state` incorrectly stays `Running`. | ✅ Root Cause | Direct read of the pre-fix branch shows unconditional `self.current_repeat += repeats` with no comparison to `self.repeat_count`; confirmed empirically via temporary-revert-and-rerun (assertion `large delta_time should complete...` failed, `state()` still `Running`). | E1, E2, E4 |
| H2 | The *ordinary* single-crossing-per-call case is also broken -- `repeat_count( n )` should mean "n total plays," so completing only after the (n+1)-th crossing is itself a bug. | ❌ Falsified | `repeat_count`'s own doc comment reads "Number of times to *repeat*" (repeats additional to the first play), and the existing `test_tween_finite_repeat` test explicitly asserts and comments "Third loop finishes, which is the final repeat" for `repeat_count( 2 )` -- 1 initial play + 2 repeats = 3 total plays is the documented, intentional contract, not a bug. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/animation/src/interpolation.rs`, `Tween::repeat_handle`'s finite-repeat branch (pre-fix, direct read) | `self.current_repeat += repeats;` with no comparison of the result against `self.repeat_count` before unconditionally setting `state = Running`. | H1 ✅ |
| E2 | `module/helper/animation/src/interpolation.rs`, `Tween::update` (direct read) | `self.elapsed += remaining_time;` accumulates a single frame's raw `delta_time` with no internal cap -- confirms a caller-supplied large `delta_time` reaches `repeat_handle` as-is, making `elapsed_repeats > 1` a real, externally-triggerable condition, not merely theoretical. | H1 ✅ |
| E3 | `module/helper/animation/tests/interpolation_test.rs` lines 217-232, `test_tween_finite_repeat` (pre-fix, direct read) | Explicitly drives 3 sequential `update( 1.0 )` calls for `repeat_count( 2 )` and asserts `is_completed()` only after the 3rd, with the comment "Third loop finishes, which is the final repeat" -- confirms this timing is intentional, not itself a defect. | H2 ❌ |
| E4 | Temporary direct-source-edit revert-and-rerun (this fix) | Reverting the finite-repeat branch to the unconditional pre-fix form reproduced the exact failure: `test_tween_finite_repeat_large_delta_completes_without_overshooting_repeat_count` panicked on "large delta_time should complete the Tween once its repeat budget is exhausted". | H1 ✅ |

## Root Cause

`repeat_handle`'s finite-repeat branch computed `elapsed_repeats` (every repeat boundary crossed
by the current call's accumulated `delta_time`) and added the whole amount to `current_repeat`
in one step, with no check against `repeat_count`. Processing crossings one at a time would
naturally stop consuming further crossings the moment `current_repeat` reaches `repeat_count`
(the very next crossing would hit the `else` "no repeats left" branch and complete) -- but
batching them into a single arithmetic addition skipped that per-crossing check entirely,
letting a single oversized `delta_time` carry `current_repeat` arbitrarily far past
`repeat_count` while still reporting `Running`.

## Why Not Caught

Every existing repeat test drove `update()` with a `delta_time` close to exactly one `duration`
per call (one boundary crossing per call), so `elapsed_repeats` was always `1` or `0` in every
existing test -- no test ever supplied a `delta_time` large enough to cross more boundaries than
the remaining repeat budget in a single call.

## Fix Location

`module/helper/animation/src/interpolation.rs`: `Tween::repeat_handle`'s finite-repeat branch now
computes `remaining = self.repeat_count - self.current_repeat` and compares this call's `repeats`
against it. When `repeats` is within budget, behavior is unchanged (increment and stay
`Running`). When `repeats` exceeds the remaining budget, `current_repeat` is capped at exactly
`repeat_count`, `state` becomes `Completed`, and `elapsed` snaps to `duration` -- mirroring the
`else` branch's existing "no repeats left" convention and matching what processing the crossings
one at a time would have produced.

## Prevention

`tests/interpolation_test.rs::test_tween_finite_repeat_large_delta_completes_without_overshooting_repeat_count`
drives a `repeat_count( 2 )` Tween past its entire budget in one `update( 3.5 )` call (3
boundary crossings against a 2-repeat budget) and asserts completion with `current_repeat()`
capped at exactly `2` and `time()` snapped to `duration`, not left overshot.

## Pitfall

Batching multiple discrete-event crossings (repeat boundaries, here) into one arithmetic step is
only safe if every constraint that would have applied to each crossing individually (here: "stop
consuming crossings once the budget is exhausted") is re-derived for the batch as a whole. A
budget check written for the single-crossing-per-call case can look complete while silently
having no effect the moment more than one crossing arrives in the same call.

## Generalized Version

**Broken assumption:** "accumulating N discrete per-frame events into one larger step (via a
big `delta_time`) is equivalent to processing them across N separate calls, as long as the same
formula is applied to the batched total."

**Confirmed general rule:** A per-step budget or completion check (`current < limit`) evaluated
once before consuming a whole batch does not automatically bound the batch itself -- explicitly
clamp the batch's effect to the remaining budget (here: `min(repeats, remaining)`, plus
completing immediately when the batch would have exceeded it), rather than assuming the
single-step check already covers the multi-step-in-one-call case.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed | Found via this session's `animation` scouting pass, tracing `Tween::repeat_handle`'s finite-repeat branch against a large-`delta_time` scenario; an initial mis-hypothesis (ordinary single-crossing timing itself being off-by-one) was falsified by re-reading `test_tween_finite_repeat`'s own documented intent before filing. |
| 2026-08-17 | fixed | Finite-repeat branch now clamps `repeats` against the remaining budget (`repeat_count - current_repeat`); exceeding it caps `current_repeat` at `repeat_count` and completes immediately, mirroring the existing "no repeats left" convention. |
| 2026-08-17 | verified | `cargo nextest run -p animation --all-features`: 42/42 passed, 0 skipped. `cargo test --doc -p animation --all-features`: 3/3 passed. `cargo clippy -p animation --all-targets --all-features -- -D warnings`: clean. Fix verified via a temporary direct-source-edit revert-and-rerun (assertion failure pre-fix, passed post-fix). |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Confirming pass: deterministic MRE, single call, exact expected values. Adversarial pass: actively tried to falsify the bug hypothesis itself by re-reading `test_tween_finite_repeat`'s documented intent before filing -- caught and discarded an initial wrong diagnosis (see H2) rather than filing against the intended 3-total-plays semantics. | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Correctly distinguished from TASK-015 (same method, different branch-logic defect already fixed) and from BUG-231 (same crate, unrelated struct). | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by direct reads of `repeat_handle`'s finite branch and `update()`'s unbounded `elapsed +=`, plus empirical revert-rerun proof. | — |
| D5 | Execution Scope | — | 🟢 | Confirming pass: fix confined to the finite-repeat branch only (infinite-repeat branch has no budget to overshoot, correctly left untouched). Adversarial pass: re-verified the `else` "no repeats left" branch's existing `elapsed = self.duration` convention is exactly what the new completed-mid-batch path reuses, not a new invented convention. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `Tween::repeat_handle`; `Tween::with_repeat`'s public signature and `Sequence`/`Sequencer`'s consumption of `Tween` are unaffected -- confirmed by the full crate suite (42/42) passing unchanged elsewhere. | — |

**Reproduced:** Confirmed via `cargo nextest` (fail pre-fix on the large-delta assertion, pass
post-fix) and temporary direct-source-edit revert-and-rerun. 2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/animation/src/interpolation.rs` | `Tween::repeat_handle`'s finite-repeat branch now clamps `repeats` against the remaining budget (`repeat_count - current_repeat`), capping `current_repeat` and completing immediately when a batch would exceed it (full `Fix(BUG-232)` comment block). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/animation/tests/interpolation_test.rs` | Added `test_tween_finite_repeat_large_delta_completes_without_overshooting_repeat_count` (`bug_reproducer(BUG-232)`, 5-section doc comment), placed directly after `test_tween_finite_repeat_preserves_overflow_elapsed`. |

## Refs: docs/

| File | Change |
|------|--------|
| — | None -- the fix eliminates the trap rather than leaving a permanent API characteristic to document, matching this session's established convention for fixed (not by-design) defects (see BUG-230/BUG-231's own Refs: docs/ precedent). |
