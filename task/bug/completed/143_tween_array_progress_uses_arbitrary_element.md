# BUG-143: `[Tween<T>; N]::progress()` reconstructs elapsed from an arbitrary fixed element instead of the group's own pace-setter

- **Severity:** Medium (silently wrong progress fraction, including a permanently-stuck-below-1.0
  case that violates the trait's own doc contract — not a crash, not `NaN`)
- **state:** Completed
- **Affects:** Any `[Tween<T>; N]` used via its `AnimatablePlayer` impl where array elements do
  not all share the identical `delay` and identical `delay + duration` (i.e., any non-fully-
  synchronized parallel tween group)
- **Component:** `module/helper/animation` (`src/interpolation.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** Sixth bug filed for `animation` this session. Sibling to BUG-140 (`Tween::
  progress()` double-subtracts delay) — same "0.0 to 1.0" doc contract, same completion-boundary
  concern, but on the `[Tween<T>; N]` array impl rather than the scalar impl. This exact array-impl
  behavior was noticed and deliberately deferred as a possible design ambiguity during BUG-140's
  investigation (prior session); this session (task #80) confirmed it is a genuine, fixable defect
  rather than an open design question, via the sharp counter-example in E3 below.

## Symptom

```rust
use animation::{ Tween, AnimatablePlayer, easing::base::EasingBuilder, easing::Linear };

let mut tweens : [ Tween< f32 >; 2 ] =
[
  Tween::new( 0.0_f32, 1.0_f32, 2.0, Linear::build() ).with_delay( 2.0 ), // ends at t = 4.0
  Tween::new( 0.0_f32, 1.0_f32, 6.0, Linear::build() ),                   // ends at t = 6.0
];

tweens.update( 3.0 ); // global elapsed = 3.0 out of the group's 6.0 span

// Wrong (pre-fix):   tweens.progress() == 0.16666666666666666
// Correct (post-fix): tweens.progress() == 0.5
```

## Impact

**Who is affected:** Any caller using `[Tween<T>; N]`'s `AnimatablePlayer` impl (a parallel group
of independently-timed tweens) via `.progress()`, whenever array elements don't all share the
identical `delay` and identical `delay + duration` — i.e., anything other than a fully
lockstep-synchronized array.

**What breaks:** two compounding defects in one formula. (1) The pre-fix numerator,
`self[ 0 ].time() - self.delay_get()`, omitted `self[ 0 ]`'s own `delay` entirely, so the result is
wrong from the very first tick whenever element 0's delay differs from the group's true earliest
delay (`delay_get()`) — not only near completion. (2) `self[ 0 ]` is an arbitrary, possibly
non-representative index. `Tween::update`'s `Completed` branch returns early without further
advancing `elapsed`, so once `self[ 0 ]` individually completes, its own `time()` freezes
permanently at its own `duration` — even while other, longer-running array members keep animating
toward the group's real end (`duration_get()`, correctly the max `delay + duration` across every
element). A fully-completed array (`is_completed() == true` for every element) can therefore report
`progress() < 1.0` forever, directly violating `AnimatablePlayer::progress`'s own doc contract
("Gets the progress of the animated value ( 0.0 to 1.0 )") — the same contract BUG-140 restored for
the scalar `Tween` impl, left unenforced here on the array impl.

**Magnitude:** Silent wrong values, not a crash. Any caller gating logic on `progress() >= 1.0`
(state transitions, blend-weight normalization, UI progress bars) driven by an asynchronous
(non-lockstep) tween group either transitions late or never transitions at all.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

Task #80's targeted follow-up on `module/helper/animation`. The array impl's `progress()` behavior
was noticed as a possible ambiguity during BUG-140's investigation (prior session) and deliberately
deferred rather than treated as a bug at that time. This session, working through it by hand
surfaced a sharp, unambiguous counter-example (E3 below: a fully-completed array reporting
`progress() < 1.0` forever) that directly violates the trait's own stated doc contract — confirming
it as a genuine, fixable defect rather than an open design question, and following the same
investigate → root cause → fix → test → verify pattern as every other bug this session.

## Minimum Reproducible Example

```bash
cd module/helper/animation && cargo test --test interpolation_test test_tween_array_progress_uses_last_to_finish_element 2>&1 | tail -10
```

**Expected** (post-fix):
```
test tests::test_tween_array_progress_uses_last_to_finish_element ... ok
```

**Actual** (pre-fix — confirmed by temporarily reverting `progress()` back to the `self[ 0 ]`-based
formula, then restoring the fix immediately after capturing the failure):
```
thread 'tests::test_tween_array_progress_uses_last_to_finish_element' panicked at module/helper/animation/tests/interpolation_test.rs:315:5:
assertion `left == right` failed
  left: 0.16666666666666666
 right: 0.5
```

**Verify Command** (≤3 lines, standalone):
```bash
cd module/helper/animation && cargo test --test interpolation_test test_tween_array_progress_uses_last_to_finish_element
# 1 passed = fixed; 1 failed (left 0.1666... != right 0.5) = bug present
```

**Known MRE limitation (check 205):** none — `[Tween<T>; N]` is pure, synchronous, dependency-free
state; runs as an ordinary native `cargo test` against the real crate directly.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `progress()`'s numerator reconstructs elapsed from `self[ 0 ]` alone via `self[ 0 ].time() - self.delay_get()`, omitting `self[ 0 ].delay` and using a fixed, possibly non-representative index instead of the element consistent with `duration_get()`'s own max-end aggregation. | ✅ Root Cause | Direct read of the pre-fix formula compared against `duration_get()`/`delay_get()`'s own correct aggregation (max `delay+duration` / min `delay` across every element) — numerator and denominator describe two different notions of "the group." | E1 |
| H2 | The only real defect is the missing `self[ 0 ].delay` term; using `self[ 0 ]` as the reference element is otherwise fine since every element should finish "around the same time" anyway. | ❌ Falsified | The stuck-forever counter-example (E3) shows the element-selection defect is independent of, and in addition to, the delay-omission defect — even with a hypothetically-correct delay term, reading a fixed index that finishes before the group's true last element still freezes `progress()` below `1.0` after full completion. | E3 |
| H3 | The fix's element selection (`max_by` on `delay + duration`) could be wrong or non-deterministic when two elements tie for the group's latest end time. | ❌ Falsified | Algebraic check: for any actively-running element `i` (not `Pending`, not yet `Completed`), `i.delay + i.time() == i.delay + ( G - i.delay ) == G`, the true global elapsed — independent of which element is chosen, as long as it has started and hasn't finished. Tied elements therefore yield the identical numerator regardless of `max_by`'s last-element-wins tie rule (Rust's documented `max_by` semantics). | E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/interpolation.rs`, pre-fix `AnimatablePlayer for [Tween<T>; N]::progress()` (`self[ 0 ].time() - self.delay_get()`) vs. `duration_get()`/`delay_get()` (correct max-end/min-delay aggregation) | Numerator and denominator use inconsistent notions of "the group's own state." | H1 ✅ |
| E2 | `tests/interpolation_test.rs`, new regression test, plus the captured pre-fix failure text | 2-element array (delays 2.0/0.0, durations 2.0/6.0) queried at global elapsed 3.0 — neither element completed yet. Pre-fix: `0.16666...` (`self[0].time()=1.0`, `/6.0`). Post-fix: `0.5` (true `3.0/6.0`). Confirms the defect is present well before any element completes. | H1 ✅ |
| E3 | Hand-worked: same 2-element array, queried after both elements fully complete (global elapsed ≥ 6.0) | `self[ 0 ]` completes at its own `t=4.0` and freezes (`Tween::update`'s `Completed` branch stops advancing `elapsed`; `time()` stays at `self[0].duration = 2.0`). Pre-fix formula: `(2.0 - 0.0)/6.0 = 0.3333...` forever, even though `is_completed() == true` for the entire array — violates the trait's own "0.0 to 1.0" contract. Post-fix: `last` resolves to `self[1]` (the true max-end element); once completed, `last.time() == last.duration == 6.0`, giving `(0.0 + 6.0 - 0.0)/6.0 == 1.0` exactly. | H1 ✅, H2 ❌ |
| E4 | Algebraic check on `Tween`'s own `time()`/`elapsed` invariants (delay-exclusive elapsed, established for BUG-140) | For any actively-running element `i`: `i.delay + i.time() = i.delay + ( G - i.delay ) = G` (the global elapsed since the group's own zero point), independent of which running element is chosen — confirms the fix's `max_by`-selected element is not a fragile/lucky choice, and ties resolve identically regardless of `max_by`'s last-element-wins rule. | H3 ❌ |

## Root Cause

```
duration_get(): max( delay_i + duration_i ) across all i   -- correct, already fixed under TASK-015
delay_get():    min( delay_i ) across all i                -- correct

progress()  (pre-fix):
  self[ 0 ].time() - self.delay_get()   // ignores self[ 0 ].delay; ignores every i != 0 entirely
  ---------------------------------------
  self.duration_get()

Two failure modes from the same formula:
  (a) self[ 0 ].delay != delay_get()  ->  wrong numerator from tick 1 (E2)
  (b) self[ 0 ] finishes before the group's true last element  ->  frozen numerator after
      self[ 0 ]'s own completion, even while later elements (and duration_get() itself) are
      still counting toward a LATER true end (E3)

progress()  (post-fix):
  last = element maximizing ( delay_i + duration_i )   // same aggregation duration_get() uses
  ( last.delay + last.time() - self.delay_get() ) / self.duration_get()   // clamped [0.0, 1.0]
```

The numerator must be reconstructed from the SAME element that determines the group's own end
(`duration_get()`'s own `max`), including that element's own `delay` — otherwise the numerator and
denominator describe two different notions of "the group," and whichever element happens to sit at
index `0` silently becomes the (usually wrong) source of truth.

## Why Not Caught

No existing test called `.progress()` on a `[Tween<T>; N]` array at all — the only prior array-impl
test (`test_tween_array_duration_and_delay_get`) exercised `duration_get()`/`delay_get()` only.

## Fix Location

`module/helper/animation/src/interpolation.rs`, `AnimatablePlayer for [Tween<T>; N]::progress`:

```rust
// before
fn progress( &self ) -> f64
{
  if self[ 0 ].state == AnimationState::Pending
  {
    0.0
  }
  else
  {
    ( ( self[ 0 ].time() - self.delay_get() ) / self.duration_get() ).clamp( 0.0, 1.0 )
  }
}

// after
fn progress( &self ) -> f64
{
  let last = self.iter().max_by
  (
    | a, b | ( a.delay + a.duration ).partial_cmp( &( b.delay + b.duration ) ).expect( "Animation keyframes can't be NaN" )
  ).expect( "N must be greater than 0" );

  if last.state == AnimationState::Pending
  {
    0.0
  }
  else
  {
    ( ( last.delay + last.time() - self.delay_get() ) / self.duration_get() ).clamp( 0.0, 1.0 )
  }
}
```

`partial_cmp(...).expect(...)` follows the crate's only existing float-comparison precedent
(`sequencer.rs`'s `delay_get().partial_cmp(&self.elapsed).expect("Animation keyframes can't be NaN")`)
rather than introducing a new idiom. No signature change — pure internal-logic fix.

## Prevention

Added `test_tween_array_progress_uses_last_to_finish_element` to `tests/interpolation_test.rs`,
checking a 2-element array with different delays and durations mid-animation (before either
element completes) against the true global-elapsed-time answer.

**Pitfall:** invisible for any fully-synchronized array (every element sharing the same `delay` and
the same `duration`) — `self[ 0 ]` happens to coincide with the group's true pace-setter in that
special case, masking the bug until a caller actually needs staggered/asynchronous tweens in one
array, which is the array impl's entire reason to exist over a single `Tween`.

## Generalized Version

**Broken assumption:** "an aggregate accessor (`progress()`) over a collection can reuse a single
arbitrary element (`self[0]`) as its state source, as long as the collection's OTHER aggregate
accessors (`duration_get()`, `delay_get()`) are already correctly aggregated." False — every
aggregate accessor over the same collection must derive its answer from a definition of "the
group" consistent with the others; picking a fixed index for one accessor while genuinely
aggregating (`min`/`max`) for its siblings silently produces answers that agree only by coincidence
(the synchronized-array special case) and diverge as soon as that coincidence doesn't hold.

**Confirmed general rule:** when multiple accessor methods over the same collection are meant to
describe one consistent "whole," derive every one of them via the same aggregation strategy
(here: whichever element determines `duration_get()`'s own `max`), not a mix of real aggregation and
a convenience index. Grep for every accessor method on a collection type once one of them is found
to use a fixed index instead of iterating.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Deferred as a possible design ambiguity during BUG-140's investigation (prior session); confirmed this session (task #80) as a genuine, fixable defect via the stuck-forever counter-example (E3), which directly violates `AnimatablePlayer::progress`'s own doc contract. |
| 2026-08-16 | fixed | Changed `progress()` to select the element maximizing `delay + duration` (matching `duration_get()`'s own aggregation) and include that element's own `delay` in the numerator. |
| 2026-08-16 | verified | Added `test_tween_array_progress_uses_last_to_finish_element`; confirmed it fails against the reverted pre-fix logic with the exact predicted `0.16666... != 0.5` assertion panic and passes against the fix; full crate suite (44 tests incl. 3 doctests) + `cargo clippy --all-targets -- -D warnings` clean. |
| 2026-08-16 | completed | Acceptance verification by a distinct session, same batch as BUG-138 (see its completed-row note for the shared 40/40 `animation` run and MAAV batch scope). Independently re-read `[Tween<T>; N]::progress()` (confirmed the `max_by`-selected `last` element formula genuinely present, `Fix(BUG-143)` comment intact) and `test_tween_array_progress_uses_last_to_finish_element` (non-tautological: asserts `tweens.progress() == 0.5` against the true global-elapsed answer, not the pre-fix `0.16666...`). State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass derived both counter-examples (E2, E3) by hand against `Tween`'s own established `time()`/completion invariants; adversarial pass independently algebra-checked the fix's `max_by`-selection robustness under ties (H3/E4) before trusting it, then closed via revert-test-restore with the real Rust test. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Explicitly a deferred follow-up from BUG-140 (same doc contract, array-impl sibling); cross-referenced to confirm no shared root cause beyond the shared contract. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass specifically separated the two compounding defects (missing delay term vs. wrong element index) via two independent counter-examples (E2 isolates defect 1 pre-completion, E3 isolates defect 2 post-completion) rather than accepting one combined symptom as sufficient. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Grepped `progress()`/`.progress(` call sites and confirmed `duration_get()`/`delay_get()` (already correctly aggregated under TASK-015) required no further change — only the numerator's element-selection needed fixing. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `animation` src+test+bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to one method body. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface; existing "0.0 to 1.0" contract now enforced consistently with the array impl's own `duration_get()`/`delay_get()` aggregation. | — |

**Reproduced:** YES — temporarily reverting the fixed `progress()` back to the pre-fix
`self[ 0 ]`-based formula and running
`cargo test --test interpolation_test test_tween_array_progress_uses_last_to_finish_element`
produced the exact predicted `0.16666666666666666 != 0.5` assertion panic at
`interpolation_test.rs:315:5`; restoring the fix returned the full suite (44 tests incl. doctests)
to passing plus a clean `cargo clippy --all-targets -- -D warnings`, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/animation/src/interpolation.rs` | `AnimatablePlayer for [Tween<T>; N]::progress`: replaced the `self[ 0 ]`-based formula with one that selects the element maximizing `delay + duration` (matching `duration_get()`'s own aggregation) and includes that element's own `delay` in the numerator. `Fix(BUG-143)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/animation/tests/interpolation_test.rs` | New test (`bug_reproducer(BUG-143)`, 5-section doc comment) — `test_tween_array_progress_uses_last_to_finish_element`. |
