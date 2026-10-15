# BUG-140: `Tween::progress()` double-subtracts delay, undercounting progress on any delayed tween

- **Severity:** Medium (silently wrong `progress()` reading for any tween with a nonzero delay —
  not a crash, but a completed animation never reports 100%)
- **state:** Completed
- **Affects:** Any caller of `Tween::progress()` on a tween constructed with `.with_delay(d)` for
  `d > 0.0`
- **Component:** `module/helper/animation` (`src/interpolation.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — third bug filed for `animation` this session, independent of
  BUG-138/BUG-139 (different type, different file region). Two sibling `progress()`
  implementations with a visually identical `( x - delay ) / duration` formula
  (`Sequencer::progress()`, `Sequence::progress()`) were independently checked and confirmed
  correct-as-is, since both types' own `elapsed`/`time` fields are delay-*inclusive* by
  construction — unlike `Tween::elapsed`, which is delay-*exclusive*. Not filed as bugs.

## Symptom

```rust
let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ).with_delay( 0.5 );
tween.update( 0.5 ); // consumes the delay entirely
tween.update( 1.0 ); // full duration elapsed -> Completed

// Wrong (pre-fix): tween.progress() == 0.5   (a COMPLETED animation reporting 50%)
// Correct (post-fix): tween.progress() == 1.0
```

## Impact

**Who is affected:** Any caller reading `Tween::progress()` on a delayed tween (any tween built
with `.with_delay(d)`, `d > 0.0`) — e.g. a progress bar, a completion check gated on `progress()`
reaching `1.0`, or any UI/logic branching on tween progress rather than `is_completed()`.

**What breaks:** `progress()` computed `( self.elapsed - self.delay ) / self.duration`. But
`Tween::update` only ever adds to `self.elapsed` *after* the delay countdown (`self.remain`) has
been fully consumed — `elapsed` is already delay-exclusive by construction, exactly mirroring
`value_get()`'s own `self.elapsed / self.duration` (no delay subtraction there at all).
Subtracting `delay` a second time undercounts progress by `delay / duration` at every point,
worst of all at full completion: `elapsed == duration` there, so `progress()` returns `1.0 -
delay/duration` instead of `1.0` — a **fully completed** delayed tween never reports 100% via
`progress()`, even though `is_completed()` and `value_get()` both correctly reflect completion.

**Magnitude:** Not a crash — a silently wrong `f64` in `[0.0, 1.0)`. Severity is Medium rather
than High because `value_get()` (the more commonly used completion signal) is unaffected;
`progress()` specifically, on delayed tweens specifically, is the only wrong path.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

Task #74's targeted code review of `module/helper/animation`. An `Explore` subagent dispatch
flagged that `progress()`'s formula didn't match `value_get()`'s `normalized_time` computation
for the same `elapsed` field; confirmed by direct read of `Tween::update`'s delay-consumption
logic (`remain`/`Pending` handling) proving `elapsed` is delay-exclusive, and by extending the
existing `test_tween_with_delay_behavior` scenario to full completion.

## Minimum Reproducible Example

```bash
cd module/helper/animation && cargo test --test interpolation_test test_tween_progress_with_delay_reaches_full_completion 2>&1 | tail -10
```

**Expected** (post-fix):
```
test tests::test_tween_progress_with_delay_reaches_full_completion ... ok
```

**Actual** (pre-fix — confirmed by temporarily reverting `self.elapsed / self.duration` back to
`( self.elapsed - self.delay ) / self.duration`, then restoring the fix immediately after
capturing the failure):
```
thread 'tests::test_tween_progress_with_delay_reaches_full_completion' panicked at module/helper/animation/tests/interpolation_test.rs:118:5:
assertion `left == right` failed: a fully-completed delayed tween must report progress 1.0, not undercounted by a second delay subtraction
  left: 0.5
 right: 1.0
```

**Verify Command** (≤3 lines, standalone):
```bash
cd module/helper/animation && cargo test --test interpolation_test test_tween_progress_with_delay_reaches_full_completion
# 1 passed = fixed; 1 failed (left: 0.5, right: 1.0) = bug present
```

**Known MRE limitation (check 205):** none — `Tween<T>` is pure, synchronous, dependency-free
state; runs as an ordinary native `cargo test` against the real crate directly.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `progress()` subtracts `delay` from an `elapsed` that is already delay-exclusive by construction. | ✅ Root Cause | Direct read of `Tween::update`: `self.elapsed += remaining_time` only executes after `self.remain` (seeded from `delay`) reaches `0.0`; `value_get()`'s `normalized_time = self.elapsed / self.duration` performs no delay subtraction. | E1 |
| H2 | Zero-delay tweens also exhibit the bug. | ❌ Falsified | With `delay == 0.0`, the subtraction is a no-op (`elapsed - 0.0 == elapsed`) — every existing `progress()`-checking test uses a zero-delay tween, explaining why the bug went uncaught. | E2 |
| H3 | `Sequencer::progress()` and `Sequence::progress()`, which use the visually identical `( x - delay ) / duration` shape, share this same defect. | ❌ Falsified | Both types' `elapsed`/`time` fields accumulate unconditionally from `t=0` (no delay-consuming countdown, unlike `Tween`), so their own `delay` field represents an absolute offset into an already-delay-inclusive clock — the identical-looking formula is correct there. Traced both implementations directly and hand-verified with concrete elapsed/delay/duration triples. | Direct source trace, both call sites |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/interpolation.rs`, pre-fix `Tween::progress` vs. `Tween::value_get` | `progress()`: `( self.elapsed - self.delay ) / self.duration`; `value_get()`: `self.elapsed / self.duration` — same field, two different formulas for what should be the same normalized-time concept. | H1 ✅ |
| E2 | `tests/interpolation_test.rs`, pre-fix | `test_tween_initial_state`/`test_tween_progress_and_completion` check `progress()` only on zero-delay tweens; `test_tween_with_delay_behavior` uses `.with_delay(0.5)` but never calls `.progress()` at all. | H1 ✅, H2 ❌ |
| E3 | `src/sequencer.rs`, `Sequencer::progress` (inherent) and `Sequence::progress` (`AnimatablePlayer` impl) | Both use `self.time()`/`self.elapsed` fields that accumulate unconditionally from `t=0` with no delay-countdown mechanism — confirmed correct via hand-computed examples (e.g. `Sequence` with players `[delay=1,dur=1],[delay=2,dur=1]`: at `elapsed=1.5`, `(1.5-1)/2=0.25`, matching the true 25%-into-active-span answer). | H3 ❌ |

## Root Cause

```
Tween::update():
  if Pending && remain > 0.0:
    consume remain from remaining_time   // delay countdown, NOT yet touching elapsed
    if remain <= 0.0: state = Running
    else: return early                    // elapsed still untouched
  if remaining_time > 0.0 && Running:
    self.elapsed += remaining_time        // elapsed ONLY ever grows post-delay

Tween::value_get():   normalized_time = elapsed / duration                    // correct, no delay term
Tween::progress():    ( elapsed - delay ) / duration                          // BUG: delay subtracted AGAIN
```

`elapsed` already excludes the delay period by construction — `progress()`'s extra `- delay`
double-counts it, most visibly at completion (`elapsed == duration`), where the formula becomes
`1.0 - delay/duration` instead of `1.0`.

## Why Not Caught

Every existing test exercising `progress()` used a zero-delay tween (where the subtraction is a
no-op); the one test using a nonzero delay (`test_tween_with_delay_behavior`) checked `value_get()`
and `state()` only, never `progress()` itself.

## Fix Location

`module/helper/animation/src/interpolation.rs`, `Tween::progress()`:

```rust
// before
( ( self.elapsed - self.delay ) / self.duration ).clamp( 0.0, 1.0 )

// after
( self.elapsed / self.duration ).clamp( 0.0, 1.0 )
```

No signature change — pure internal-logic fix, now matching `value_get()`'s formula exactly.

## Prevention

Added `test_tween_progress_with_delay_reaches_full_completion` to `tests/interpolation_test.rs`,
driving a delayed tween all the way to `Completed` and checking `progress()` reports `1.0` there.

**Pitfall:** invisible for zero-delay tweens, and invisible if only `value_get()`/`is_completed()`
are checked on a delayed tween (both were always correct) — only reading `progress()` itself, on
a tween built with a nonzero delay, exposes the defect.

## Generalized Version

**Broken assumption:** "two methods on the same struct field, computing conceptually the same
normalized quantity, should use the same formula — if one does `x - offset`, the other probably
should too." False when the field's own accumulation semantics already bake the offset in; a
sibling method reapplying it double-counts. The tell here was that `value_get()` and `progress()`
disagreed on the very same `self.elapsed` field's relationship to `self.delay`.

**Confirmed general rule:** when two methods derive a normalized value from the same underlying
field, verify both against that field's actual accumulation contract (read the mutator, not just
the two derivations) rather than assuming formula parity between sibling methods — and when a
visually identical formula appears on a different type (as it did here, twice, on `Sequencer` and
`Sequence`), re-derive its correctness against *that* type's own field semantics rather than
assuming the same bug generalizes.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered via task #74's targeted code review of `module/helper/animation`; confirmed by direct read of `Tween::update`'s delay-consumption logic against `progress()`'s formula, and by cross-checking two sibling `progress()` implementations elsewhere in the crate that turned out NOT to share the bug. |
| 2026-08-16 | fixed | Changed `progress()` to `self.elapsed / self.duration`, matching `value_get()`. |
| 2026-08-16 | verified | Added `test_tween_progress_with_delay_reaches_full_completion`; confirmed it fails against the reverted pre-fix logic with the exact predicted undercounted value (`left: 0.5, right: 1.0`) and passes against the fix; full crate suite (35 tests incl. 6 doctests) + `cargo clippy --all-targets -- -D warnings` clean. |
| 2026-08-16 | completed | Acceptance verification by a distinct session, same batch as BUG-138 (see its completed-row note for the shared 40/40 `animation` run and MAAV batch scope). Independently re-read `Tween::progress()` (confirmed `self.elapsed / self.duration` genuinely present, `Fix(BUG-140)` comment intact) and `test_tween_progress_with_delay_reaches_full_completion` (non-tautological: asserts `progress() == 1.0` on a fully-completed delayed tween). State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass traced the delay-exclusive `elapsed` contract by reasoning; adversarial pass required actually observing the FAIL against reverted code — closed via revert-test-restore, captured text (`left: 0.5, right: 1.0`) matched exactly. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Independent of BUG-138/BUG-139 (different file region); explicitly checked and ruled out two sibling `progress()` implementations sharing the visual formula (H3, falsified) rather than assuming the fix generalizes. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass specifically hunted for the "same bug elsewhere" trap (H3) by hand-deriving `Sequencer::progress()`/`Sequence::progress()`'s own `elapsed`-accumulation semantics rather than pattern-matching on formula shape alone. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Grepped every `progress()` implementation in the crate (`Tween`, `[Tween<T>;N]`, `Sequencer`, `Sequence`) — only `Tween::progress()` shares both the formula AND the delay-exclusive-elapsed precondition that makes it wrong; the array impl (`self[0].time() - self.delay_get()`) has a separate, more complex cross-element ambiguity flagged for independent follow-up, not folded into this fix. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `animation` `src/interpolation.rs` + `tests/interpolation_test.rs` + this bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to `Tween::progress()`'s single expression; no other method touched. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface added — existing "progress 0.0 to 1.0" doc contract now actually honored for delayed tweens too. | — |

**Reproduced:** YES — temporarily reverting `self.elapsed / self.duration` back to
`( self.elapsed - self.delay ) / self.duration` and running
`cargo test --test interpolation_test test_tween_progress_with_delay_reaches_full_completion`
produced the exact predicted undercounted value (`left: 0.5, right: 1.0`); restoring the fix
returned the full suite (35 tests incl. doctests) to passing plus a clean
`cargo clippy --all-targets -- -D warnings`, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/animation/src/interpolation.rs` | `Tween::progress()`: changed `( self.elapsed - self.delay ) / self.duration` to `self.elapsed / self.duration`. `Fix(BUG-140)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/animation/tests/interpolation_test.rs` | New test (`bug_reproducer(BUG-140)`, 5-section doc comment) — `test_tween_progress_with_delay_reaches_full_completion`. |
