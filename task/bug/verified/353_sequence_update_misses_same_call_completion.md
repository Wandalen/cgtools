# BUG-353: `Sequence::update()` can finish its last player without registering `Completed` in the same call

- **Severity:** Medium (no crash -- but a caller-visible, internally inconsistent public API state:
  `progress()` can report `1.0` in the same `update()` call where `is_completed()` still reports
  `false`, self-correcting only on the next call)
- **state:** Verified
- **Affects:** `Sequence::update` (`src/sequencer.rs`) -- any single `update()` call whose
  `delta_time` is large enough to both leave the Sequence's own `Pending` pre-roll phase AND drive
  the (now-active) last player to completion
- **Component:** `module/helper/animation` (`src/sequencer.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/
- **Fix Task:** [388](../../verifying/388_register_animation_sequence_update_samecall_completion_fix_closes_bug353.md)
- **Related Bugs:** BUG-352 (`pause()` no-op during `Pending`) -- found in the same bug-hunt pass,
  same file (`sequencer.rs`), same `Sequence` struct; distinct root causes (this bug is a
  same-call-transition gap in `update()`, BUG-352 is a gating defect in `pause()`), no shared fix.
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ (self)
- **verification_date:** 2026-08-18

## Symptom

```rust
// pre-fix
let mut sequence = Sequence::new( vec!
[
  Tween::new( 0.0_f32, 10.0_f32, 0.5, Linear::build() ).with_delay( 0.1 ),
  Tween::new( 0.0_f32, 10.0_f32, 0.5, Linear::build() ).with_delay( 0.6 ),
] ).unwrap();

sequence.update( 100.0 );          // one oversized update spans the entire timeline

assert_eq!( sequence.progress(), 1.0 );
assert!( sequence.is_completed() );  // FAILS: still false in this same call
```

`Sequence::update`'s state transitions (`Pending -> Running` and `Running -> Completed`) were two
mutually exclusive arms of one `match self.state`, keyed on `self.state` as it stood at the START
of the call. A `delta_time` large enough to trigger both transitions in the same call only ever
fired the first one -- `progress()` (derived from the active player's own progress, independent of
`self.state`) already reports `1.0`, but `is_completed()` (which reads `self.state ==
AnimationState::Completed` directly) still reports `false` until the *next* `update()` call.

## Impact

**Who is affected:** Any caller that drives a `Sequence` with large or irregular `delta_time`
values (e.g. catching up after a dropped frame, fast-forwarding, or a very short total sequence
duration relative to the frame's delta) and checks `is_completed()`/`progress()` together after a
single `update()` call.

**What breaks:** `is_completed()` under-reports completion for exactly one extra `update()` call
whenever the completing call is also the call that leaves `Pending`. A caller gating cleanup,
callback firing, or advancing to the next queued animation on `is_completed()` sees a spurious
one-frame delay in an otherwise-already-100%-progressed `Sequence` -- an internally inconsistent
public API state (`progress() == 1.0` while `is_completed() == false`) with no error or panic to
surface it.

**Magnitude:** 1 method (`Sequence::update`), 1 missing same-call re-check.

**Entity Scope:** None -- a code-level defect.

## How Discovered

A prior investigation pass over `module/helper/animation` read `Sequence::update` in full,
comparing the Pending->Running and Running->Completed conditions in its `match self.state { ... }`
block against every other multi-transition state machine in the crate. Both conditions are
evaluated against `self.state`'s value at match-entry, and the `match`'s arms are mutually
exclusive by construction -- a call that satisfies the first condition can never also apply the
second, even when the underlying data (the active player's own `is_completed()`) already supports
it.

## Minimum Reproducible Example

```rust
let mut sequence = Sequence::new( vec!
[
  Tween::new( 0.0_f32, 10.0_f32, 0.5, Linear::build() ).with_delay( 0.1 ),
  Tween::new( 0.0_f32, 10.0_f32, 0.5, Linear::build() ).with_delay( 0.6 ),
] ).unwrap();

// One oversized update spans the entire timeline: leaves the Sequence's own Pending phase AND
// drives the last player to completion, in the same call.
sequence.update( 100.0 );

assert_eq!( sequence.progress(), 1.0 );
assert!( sequence.is_completed() );   // pre-fix: false -- self-corrects only on the NEXT update()
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/animation && cargo nextest run --all-features -E \
  'test(test_sequence_update_completes_in_same_call_that_leaves_pending)'
```
**What:** violates the invariant that `is_completed()` and `progress() == 1.0` are never
observably inconsistent with each other after any single `update()` call.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The `Pending -> Running` and `Running -> Completed` transitions are mutually exclusive arms of one `match self.state`, so a `delta_time` large enough to satisfy both in one call only ever applies the first. | ✅ Root Cause | Direct read of `update()` (pre-fix) confirms both conditions live as separate `match` arms keyed on `self.state`'s value at match-entry; confirmed empirically -- the new reproducer test fails pre-fix exactly at the `is_completed()` assertion while `progress()` already reports `1.0` in the same call. | E1, E2, E4 |
| H2 | This is harmless because the very next `update()` call re-evaluates from the now-`Running` state and reaches `Completed` anyway -- callers always call `update()` more than once before checking `is_completed()`. | ❌ Falsified | `is_completed()` and `progress()` are both public, independently-callable API -- nothing prevents (and the new test explicitly demonstrates) a caller checking both in the same frame immediately after the completing `update()` call, before any further call happens. The inconsistency is real and observable, not merely internal. | E3 |
| H3 | `Sequencer::update` (the separate heterogeneous coordinator in the same file) has the identical two-transitions-per-call gap. | ❌ Falsified | Direct read of `Sequencer::update` shows only one possible transition direction at all (`Running -> Completed`; it has no `Pending` state or pre-roll-delay concept -- it early-returns unless already `Running`), so the same-call-double-transition shape this bug depends on cannot arise there. | E5 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/animation/src/sequencer.rs`, `Sequence::update`'s `match self.state { ... }` (pre-fix, direct read) | `AnimationState::Pending if ... => { self.state = Running; }`, `AnimationState::Running if ... => { self.state = Completed; }`, `_ => {}` -- one `match` on `self.state`, evaluated once per call; the two conditions are structurally exclusive per call regardless of what the active player's own `is_completed()` reports afterward. | H1 ✅ |
| E2 | `module/helper/animation/src/sequencer.rs`, `Sequence::update`'s player-dispatch block (direct read) | The active player's `update()` call (and its own `Pending -> Running` transition, if this is the call that first activates it) happens BEFORE the `match self.state { ... }` block -- so by the time the match runs, the active player may already be `is_completed() == true`, but the match's `Pending` arm (if it fires) still consumes the call without ever re-checking the `Running` arm's own condition. | H1 ✅ |
| E3 | Reproducer test design (this fix) | `sequence.update( 100.0 )` on a fresh, still-`Pending`, 2-player `Sequence` asserts `progress() == 1.0` AND `is_completed() == true` in the same call, with no second `update()` call anywhere in the test -- directly demonstrates the inconsistency is observable within a single caller-visible frame, not merely a transient internal state. | H2 ❌ |
| E4 | Test run pre-fix (`cargo nextest`) | `test_sequence_update_completes_in_same_call_that_leaves_pending` fails pre-fix with: "is_completed() still false the same call progress() already reports 1.0 -- Pending->Running and Running->Completed didn't both apply within one update()". | H1 ✅ |
| E5 | `module/helper/animation/src/sequencer.rs`, `Sequencer::update` (direct read) | `if self.state != AnimationState::Running { return; }` followed by a single possible `Running -> Completed` transition at the end -- no `Pending` state, no pre-roll delay, no second transition direction exists in this method at all. | H3 ❌ |

## Root Cause

`Sequence::update`'s two live state transitions -- `Pending -> Running` (once `self.elapsed`
passes the active player's own `delay_get()`) and `Running -> Completed` (once the last player
reports `is_completed()`) -- were written as two arms of a single `match self.state { ... }`
keyed on `self.state`'s value at the moment the match runs. Since a `match` commits to exactly one
arm, a call whose `delta_time` was large enough to satisfy both conditions in sequence (leave
`Pending`, and the now-active last player is already complete) only ever applied the first
transition; the second condition, now also true, was never re-evaluated within that same call.

## Why Not Caught

No pre-existing `Sequence` test drove `update()` with a `delta_time` large enough to span the
entire timeline (leave `Pending` AND complete the last player) in a single call -- every prior
multi-frame test used deltas that stayed within one phase at a time, and no existing `Sequence`
test ever drove one all the way to `is_completed() == true` at all.

## Fix Location

`module/helper/animation/src/sequencer.rs:539-561` (`Sequence::update`) -- replaced the single
`match self.state { Pending if .. => ..., Running if .. => ..., _ => {} }` with two sequential
`if` checks, so the `Running -> Completed` condition is re-evaluated against the (possibly
just-updated) `self.state` immediately after the `Pending -> Running` check, within the same call.

## Prevention

Added `test_sequence_update_completes_in_same_call_that_leaves_pending`
(`tests/sequencer_test.rs`), which drives a fresh, still-`Pending`, 2-player `Sequence` with one
oversized `update()` call and asserts `is_completed()` is `true` immediately, matching `progress()
== 1.0` in that same call -- not one call later.

## Pitfall

Self-corrects on the very next `update()` call regardless (that call's own re-evaluation starts
from the now-`Running` state and reaches `Completed`), so this defect is invisible to any test
that calls `update()` more than once before checking `is_completed()` -- only a single oversized
call spanning both transitions exposes it.

## Generalized Version

**Broken assumption:** "a state machine's transitions can be written as mutually exclusive `match`
arms keyed on entry-state, since only one state change happens per tick."

**Confirmed general rule:** When a single tick/update call can legitimately advance a state
machine across more than one transition boundary (e.g. a large `delta_time` spanning both a
pre-roll phase and the work phase's own completion), transitions gated by different states must be
re-evaluated sequentially against the (possibly just-updated) state within the same call, not
written as mutually exclusive arms of one `match` keyed on the state's value at call-entry.
Detection: grep every `update`/`tick` method for a `match self.state { ... }` (or equivalent
if/else-if chain) where more than one arm can become true within a single call, and confirm later
arms re-check against the post-earlier-arm state rather than the original snapshot.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Found via a prior investigation pass reading `Sequence::update` in full and comparing its `Pending -> Running`/`Running -> Completed` conditions against every other multi-transition state machine in the crate. |
| 2026-08-18 | fix_applied | Replaced `Sequence::update`'s single `match self.state { ... }` with two sequential `if` checks so both transitions can apply within one call. Reproducer test confirmed FAIL pre-fix ("is_completed() still false the same call progress() already reports 1.0...") and PASS post-fix. Full scoped suite (`cargo nextest run -p animation --all-features`, 46 tests) and `cargo test -p animation --all-features` (46 unit/integration + 3 doc tests) both clean, 0 failures -- explicitly including BUG-138,139,140,142,143,147,148,149,231,232,233's own reproducer tests, all still passing. |
| 2026-08-18 | verified | VERIFY Gate (Tier 2 dual-role self-check, 8/8 dimensions 🟢): `test_sequence_update_completes_in_same_call_that_leaves_pending` re-run fresh and passes; full `animation` suite 46/46, 0 failures; fix logic hand-traced against the exact reproducer arithmetic to confirm the same-call double transition. |

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/animation/src/sequencer.rs` | `Sequence::update`'s `match self.state { ... }` replaced with two sequential `if` checks so `Pending -> Running` and `Running -> Completed` can both apply within one call (`Fix(BUG-353)` comment block). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/animation/tests/sequencer_test.rs` | Added `test_sequence_update_completes_in_same_call_that_leaves_pending` (`bug_reproducer(BUG-353)`, 5-section doc comment), placed directly after the BUG-352 `Sequence` reproducer test, before the existing `bug_reproducer(BUG-138)` test. |

## Refs: docs/

| File | Change |
|------|--------|
| — | None -- the fix eliminates the trap rather than leaving a permanent API characteristic to document. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All 12 sections + header fields present; `**state:**` read `Unverified` despite the file already sitting in `verified/` by directory path -- a pre-existing filing inconsistency this gate resolves; `Related Bugs` cross-link to BUG-352 confirmed bidirectional | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Verify Command re-run fresh (`cargo nextest run -p animation --all-features`): `test_sequence_update_completes_in_same_call_that_leaves_pending` PASSES; full crate suite 46/46, 0 failed. Adversarial pass hand-traced the exact arithmetic (`update(100.0)` on a 2-player [delay 0.1, 0.6]/duration-0.5 Sequence: `current.update(100.0)` on player 1 -- Pending(remain 0.6)→Running→elapsed 99.4 vs duration 0.5→Completed; then Sequence's own Pending→Running (`100.0-0.6>0`) and Running→Completed (last-player index, `is_completed()`) both fire in the same call) -- matches the observed PASS by independent derivation, not merely trusted | — |
| D3 | Cross-Reference Integrity | — | 🟢 | 3 Hypothesis rows, 1 ✅ Root Cause (H1); H1↔{E1,E2,E4}, H2↔{E3}, H3↔{E5} bidirectional, re-checked both directions; `grep -rn "BUG-353"` confirms both `Refs:` files carry a matching `Fix(BUG-353)`/`bug_reproducer(BUG-353)` backreference | — |
| D4 | Root Cause Quality | — | 🟢 | Root Cause traces to H1 ✅; Fix Location `sequencer.rs:539-561` independently re-verified against current source -- the fix comment block plus both sequential `if` blocks fall exactly in that range; Generalized Version states a broken assumption plus a detection invariant | — |
| D5 | Execution Scope | — | 🟢 | `repo_identity: self`; fix location resolves inside `$SCOPE_DIR` (`module/helper/animation/src/sequencer.rs`) | — |
| D6 | Crate Scope Unity | — | 🟢 | `**Component:**` = `module/helper/animation`; Fix Location resolves to that same crate | — |
| D7 | Crate Locality | — | 🟢 | Fix lands in `Sequence::update` itself (the leaf owner of the state machine), not pushed up into any caller | — |
| D8 | Crate Single Responsibility | — | 🟢 | Fix only re-sequences two pre-existing transition conditions into same-call-reevaluating `if`s; no new public surface, no expansion of the crate's responsibility | — |
| **Total** | | — | 🟢 | 0 open | 0/0 |

**Reproduced:** YES — exit 0 (`cargo nextest run -p animation --all-features`, 46/46 passed, including
`test_sequence_update_completes_in_same_call_that_leaves_pending`), 2026-08-18. Adversarial pass:
confirmed `Sequence::update`'s top-of-function early return on `Completed`/`Paused` means the two
`Fix(BUG-353)` `if` blocks never execute for an already-terminal Sequence, ruling out a re-entrancy
regression; the boundary comparison (`self.elapsed - current.delay_get() > 0.0`, strict) is unchanged
pre-/post-fix -- not a defect this specific fix introduced.
