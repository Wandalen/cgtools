# BUG-352: `pause()` is a no-op while an animation is still in its `Pending` pre-roll delay phase

- **Severity:** Medium (no crash, no data corruption -- but a caller-visible behavioral defect:
  `pause()` silently fails to freeze an animation whenever it is called during a `with_delay(...)`
  pre-roll, and the animation keeps advancing exactly as if `pause()` had never been called)
- **state:** Verified
- **Affects:** `Tween::pause` (`src/interpolation.rs`) and `Sequence::pause` (`src/sequencer.rs`) --
  any call to `.pause()` while `self.state == AnimationState::Pending`, i.e. before a
  `with_delay(...)` countdown has finished
- **Component:** `module/helper/animation` (`src/interpolation.rs`, `src/sequencer.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/
- **Fix Task:** [387](../../verifying/387_register_animation_pause_during_pending_delay_phase_fix_closes_bug352.md)
- **Related Bugs:** BUG-353 (`Sequence::update` misses a same-call `Completed` transition) --
  found in the same bug-hunt pass, same file (`sequencer.rs`), same `Sequence` struct; this bug is
  a gating defect in `pause()`, BUG-353 is a same-call-transition defect in `update()` -- distinct
  root causes, no shared fix.
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ (self)
- **verification_date:** 2026-08-18

## Symptom

```rust
// pre-fix
let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ).with_delay( 5.0 );
tween.update( 1.0 );                     // still Pending -- 4.0s of delay left
assert_eq!( tween.state(), AnimationState::Pending );

tween.pause();
assert_eq!( tween.state(), AnimationState::Paused );  // FAILS: state is still Pending
```

`Tween::pause` and `Sequence::pause` both gate the `Paused` transition on
`self.state == AnimationState::Running` only. Calling `.pause()` while `state == Pending` matches
neither arm, so the call is a silent no-op -- no panic, no error, `state()` simply does not
change, and a subsequent `update()` keeps ticking the delay countdown (and, once it expires, the
animation itself) forward as if `.pause()` had never been called.

## Impact

**Who is affected:** Any caller that calls `.pause()` on a `Tween` or `Sequence` shortly after
construction/insertion, before the first `update()` call has advanced it out of `Pending` -- e.g.
pausing a whole animation timeline immediately after building it (common for "start paused, resume
on user input" setups), or pausing mid-playback of a multi-player `Sequence` whose active player
still has unconsumed delay.

**What breaks:** `state()` keeps reporting `Pending` (not `Paused`) after the call; the animation
continues consuming its delay countdown and then begins animating on subsequent `update()` calls,
with no observable signal that `.pause()` was ever requested. The caller has no loud failure to
detect this -- `pause()` returns `()` and never errors.

**Magnitude:** 2 methods (`Tween::pause`, `Sequence::pause`), same defect shape in both.

**Entity Scope:** None -- a code-level defect.

## How Discovered

A prior investigation pass over `module/helper/animation` cross-referenced every `AnimatablePlayer`
implementor's `pause()` against the `AnimationState` variants each type's own `update()` can leave
it in. `Sequencer::pause` (the separate heterogeneous coordinator in the same file) transitions to
`Paused` unconditionally, with no state guard at all; `Tween::pause` and `Sequence::pause` both
instead gated on `state == Running` specifically, which is inconsistent with `update()`'s own
`Pending` arm being a live, reachable state at the moment `.pause()` could be called.

## Minimum Reproducible Example

```rust
let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ).with_delay( 5.0 );
tween.update( 1.0 );                                   // still Pending, 4.0s of delay left
tween.pause();
assert_eq!( tween.state(), AnimationState::Paused );    // pre-fix: still Pending -- no-op

let mut sequence = Sequence::new( vec!
[
  Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ).with_delay( 5.0 ),
  Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ).with_delay( 6.0 ),
] ).unwrap();
sequence.update( 1.0 );                                 // still Pending
sequence.pause();
sequence.update( 100.0 );
assert_eq!( sequence.time(), 1.0 );                      // pre-fix: 101.0 -- elapsed kept advancing
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/animation && cargo nextest run --all-features -E \
  'test(test_tween_pause_during_pending_delay_freezes_and_resume_preserves_remaining_delay) + test(test_sequence_pause_during_pending_delay_freezes_time_and_progress)'
```
**What:** violates the invariant that `pause()` freezes an animation regardless of which live
state (`Running` or `Pending`) it was in when called.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `Tween::pause` and `Sequence::pause` gate the `Paused` transition on `state == Running` only, so calling either while `state == Pending` is a silent no-op. | ✅ Root Cause | Direct read of both `pause()` implementations (pre-fix) confirms the single-arm `if self.state == AnimationState::Running` guard, with no `Pending` arm; confirmed empirically -- both new reproducer tests fail pre-fix exactly at the `state()` assertion immediately after `.pause()`. | E1, E2, E5 |
| H2 | Widening `pause()`'s gate alone (without touching `resume()`) is sufficient -- `resume()` can stay unconditional. | ❌ Falsified | `Tween::resume` unconditionally sets `state = Running` on any `Paused` tween. Once `pause()` can freeze a `Pending` tween, `resume()` would skip the remaining delay entirely and jump straight into animating -- a fresh, reachable regression introduced by fixing `pause()` alone. `resume()` must also change, to restore `Pending` (not `Running`) whenever `self.remain > 0.0`. | E3, E4 |
| H3 | `Sequencer::pause` (the separate heterogeneous coordinator in the same file) has the identical defect. | ❌ Falsified | Direct read of `Sequencer::pause` shows it transitions to `Paused` unconditionally, with no `state ==` guard at all -- already correct, used as the fix's own reference implementation. | E6 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/animation/src/interpolation.rs`, `Tween::pause` (pre-fix, direct read) | `if self.state == AnimationState::Running { self.state = AnimationState::Paused; }` -- the entire body; no `Pending` arm. | H1 ✅ |
| E2 | `module/helper/animation/src/sequencer.rs`, `Sequence::pause` (pre-fix, direct read) | Identical single-arm shape: `if self.state == AnimationState::Running { self.state = AnimationState::Paused; }`. | H1 ✅ |
| E3 | `module/helper/animation/src/interpolation.rs`, `Tween::resume` (pre-fix, direct read) | `if self.state == AnimationState::Paused { self.state = AnimationState::Running; }` -- unconditionally jumps to `Running`, with no check of `self.remain` (the leftover delay countdown `update`'s own `Pending` arm consults). | H2 ❌ |
| E4 | `module/helper/animation/src/interpolation.rs`, `Tween::update`'s `Pending` arm (direct read) | `self.remain -= delta_time; if self.remain <= 0.0 { ... self.state = AnimationState::Running; }` -- confirms `self.remain` is the exact countdown that determines whether a `Pending` tween is still mid-delay; `resume()`'s fix reuses this same field. | H2 ❌ |
| E5 | Test run pre-fix (`cargo nextest`) | Both new reproducer tests fail exactly at the `state()`/`time()` assertion immediately after `.pause()`: `test_tween_pause_during_pending_delay_freezes_and_resume_preserves_remaining_delay` (`left: Pending, right: Paused`) and `test_sequence_pause_during_pending_delay_freezes_time_and_progress` (`left: 101.0, right: 1.0`). | H1 ✅ |
| E6 | `module/helper/animation/src/sequencer.rs`, `Sequencer::pause` (direct read) | `self.state = AnimationState::Paused;` with no preceding `if` at all -- unconditional, already correct; used as the reference implementation this fix's gate widening (rather than full unconditionality) deliberately does NOT fully copy, since `Tween`/`Sequence` must still exclude `Completed` (see Root Cause). | H3 ❌ |

## Root Cause

`Tween::pause` and `Sequence::pause` both gated the `Paused` transition on `self.state ==
AnimationState::Running` only. `Pending` (the pre-roll delay phase established by `with_delay(...)`
and consumed by each type's own `update()`) is a distinct, live `AnimationState` value that a
caller can legitimately observe and pause during -- the single-arm gate simply never considered it,
so a `.pause()` call landing on `Pending` matched no arm and silently did nothing. Fixing `pause()`
alone would introduce a fresh regression in `resume()`: `Tween::resume` and `Sequence::resume`
(the latter deliberately left unmodified -- see Fix Location) both jump to `Running`
unconditionally, which for `Tween::resume` would skip whatever delay remained (`self.remain`) when
`pause()` was called, so the fix widens `pause()`'s gate and additionally makes `Tween::resume`
delay-aware.

## Why Not Caught

The only pre-existing pause/resume test, `test_tween_pause_resume`, only ever calls `.pause()`
after the Tween has already reached `Running` (a zero-delay Tween in that test's setup). No test
for either `Tween` or `Sequence` ever called `.pause()` while `state == Pending`; `Sequence` had no
pause/resume test at all prior to this fix (the only existing `*_pause_resume` test in
`sequencer_test.rs` exercises the unrelated `Sequencer` struct).

## Fix Location

`module/helper/animation/src/interpolation.rs:359-365` (`Tween::pause`) -- widened the gate to
`matches!( self.state, AnimationState::Running | AnimationState::Pending )`.

`module/helper/animation/src/interpolation.rs:377-383` (`Tween::resume`) -- now restores `Pending`
(instead of unconditionally `Running`) whenever `self.remain > 0.0`, mirroring `update()`'s own
`Pending` arm.

`module/helper/animation/src/sequencer.rs:577-585` (`Sequence::pause`) -- widened identically to
`matches!( self.state, AnimationState::Running | AnimationState::Pending )`.

`Sequence::resume` (`sequencer.rs:587-595`) was deliberately left unmodified: `Sequence` has no
public `state()` accessor on `AnimatablePlayer`, `progress()`'s own `.clamp( 0.0, 1.0 )` absorbs
the internal `Pending`-vs-`Running` mislabeling a delay-unaware `resume()` could cause, and the
"jump past remaining delay" edge case `Tween::resume` had to guard against is not reachable the
same way here (`Sequence::update` re-derives its active player and phase from `self.elapsed` via
`binary_search_by` every call, rather than trusting a caller-set `state` to still be accurate).
Documented here as a considered decision, not an oversight.

## Prevention

Added `test_tween_pause_during_pending_delay_freezes_and_resume_preserves_remaining_delay`
(`tests/interpolation_test.rs`) and
`test_sequence_pause_during_pending_delay_freezes_time_and_progress` (`tests/sequencer_test.rs`),
both pausing mid-delay, driving a large `update()` while paused to assert nothing advances, and (for
the `Tween` case) resuming to confirm the *remaining* delay -- not the full duration -- is what is
left to consume.

## Pitfall

Invisible for any zero-delay `Tween`/`Sequence` (`Pending` is a one-tick pass-through there, see
each type's own `update()`) and for any caller that only ever pauses after the first `update()`
call already returned `Running` -- only a caller pausing during an active `.with_delay(...)`
countdown exposes it. `Completed` is deliberately still excluded from both widened gates -- pausing
an already-finished animation must not make `is_completed()` start reporting `false`.

## Generalized Version

**Broken assumption:** "an animation only needs to be pausable once it's actually
running/animating -- a pre-roll delay phase is not something a caller would pause during."

**Confirmed general rule:** Any state-machine method that freezes/suspends progress (`pause()`)
must gate on every state in which the underlying clock is still advancing, not only the state
where visible output is changing. A pre-roll/warm-up phase that consumes real time (here,
`Pending`'s delay countdown) is exactly such a state even though it produces no visible animation
output yet -- a caller expects `pause()` to freeze the clock, not merely to freeze the value.
Detection: grep every `AnimatablePlayer`/state-machine `pause()` implementation across the crate
for a single-state `==` comparison instead of a `matches!` covering every clock-advancing state,
and cross-check against that type's own `update()` to enumerate which states actually advance time.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Found via a prior investigation pass cross-referencing every `AnimatablePlayer` implementor's `pause()` gate against the live `AnimationState` values its own `update()` can leave it in. |
| 2026-08-18 | fix_applied | Widened `Tween::pause` and `Sequence::pause` to `matches!( self.state, Running \| Pending )`; made `Tween::resume` delay-aware (restores `Pending` when `self.remain > 0.0`). `Sequence::resume` deliberately left unmodified (see Fix Location). Reproducer tests confirmed FAIL pre-fix (`left: Pending, right: Paused`; `left: 101.0, right: 1.0`) and PASS post-fix. Full scoped suite (`cargo nextest run -p animation --all-features`, 46 tests) and `cargo test -p animation --all-features` (46 unit/integration + 3 doc tests) both clean, 0 failures -- explicitly including BUG-138,139,140,142,143,147,148,149,231,232,233's own reproducer tests, all still passing. |
| 2026-08-18 | verified | VERIFY Gate (Tier 2 dual-role self-check, 8/8 dimensions 🟢): both reproducer tests (`test_tween_pause_during_pending_delay_freezes_and_resume_preserves_remaining_delay`, `test_sequence_pause_during_pending_delay_freezes_time_and_progress`) re-run fresh and pass; full `animation` suite 46/46, 0 failures. |

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/animation/src/interpolation.rs` | `Tween::pause` widened to `matches!( Running \| Pending )`; `Tween::resume` made delay-aware (`Fix(BUG-352)` comment blocks). |
| `module/helper/animation/src/sequencer.rs` | `Sequence::pause` widened identically (`Fix(BUG-352)` comment block). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/animation/tests/interpolation_test.rs` | Added `test_tween_pause_during_pending_delay_freezes_and_resume_preserves_remaining_delay` (`bug_reproducer(BUG-352)`, 5-section doc comment), placed after `test_tween_pause_resume`. |
| `module/helper/animation/tests/sequencer_test.rs` | Added `test_sequence_pause_during_pending_delay_freezes_time_and_progress` (`bug_reproducer(BUG-352)`, 5-section doc comment), placed before the existing `bug_reproducer(BUG-138)` test. |

## Refs: docs/

| File | Change |
|------|--------|
| — | None -- the fix eliminates the trap rather than leaving a permanent API characteristic to document. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All 12 sections + header fields present; `**state:**` read `Unverified` despite the file already sitting in `verified/` by directory path -- a pre-existing filing inconsistency this gate resolves; `Related Bugs` cross-link to BUG-353 confirmed bidirectional | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Verify Command re-run fresh (`cargo nextest run -p animation --all-features`): both `test_tween_pause_during_pending_delay_freezes_and_resume_preserves_remaining_delay` and `test_sequence_pause_during_pending_delay_freezes_time_and_progress` PASS; full crate suite 46/46 passed, 0 failed (no regression in BUG-138/139/140/142/143/147/148/149/231/232/233's own reproducers) | — |
| D3 | Cross-Reference Integrity | — | 🟢 | 3 Hypothesis rows, 1 ✅ Root Cause (H1); H1↔{E1,E2,E5}, H2↔{E3,E4}, H3↔{E6} bidirectional, re-checked both directions; `grep -rn "BUG-352"` confirms all 5 `Refs:` files carry a matching `Fix(BUG-352)`/`bug_reproducer(BUG-352)` backreference | — |
| D4 | Root Cause Quality | — | 🟢 | Root Cause traces to H1 ✅; Fix Location line numbers independently re-verified against current source: `interpolation.rs:359-365` (`Tween::pause`) and `:377-383` (`Tween::resume`) exact; `sequencer.rs:577-585` (`Sequence::pause`) exact; Generalized Version states a broken assumption plus a detection invariant | — |
| D5 | Execution Scope | — | 🟢 | `repo_identity: self`; both fix locations resolve inside `$SCOPE_DIR` (`module/helper/animation/src/`) | — |
| D6 | Crate Scope Unity | — | 🟢 | `**Component:**` = `module/helper/animation`; both Fix Location files resolve to that same crate | — |
| D7 | Crate Locality | — | 🟢 | Fix lands in the leaf crate owning the state machine (`animation`), not pushed up into a consumer -- `scene_script`'s `tween_binding.rs` scripting bindings call `.pause()`/`.resume()` unchanged and inherit the fix for free | — |
| D8 | Crate Single Responsibility | — | 🟢 | Fix only corrects existing `pause`/`resume` state-gating; adds no new public surface, no expansion of the crate's responsibility | — |
| **Total** | | — | 🟢 | 0 open | 0/0 |

**Reproduced:** YES — exit 0 (`cargo nextest run -p animation --all-features`, 46/46 passed, including
`test_tween_pause_during_pending_delay_freezes_and_resume_preserves_remaining_delay` and
`test_sequence_pause_during_pending_delay_freezes_time_and_progress`), 2026-08-18. Adversarial pass:
confirmed no other call site in the workspace (`scene_script::tween_binding`'s 12 `.pause`/`.resume`
script bindings, `Sequencer::pause`/`resume`) depends on the old no-op-during-`Pending` behavior;
`Sequencer::pause` (`sequencer.rs:169-176`, the unconditional reference implementation cited as E6)
independently re-read and confirmed to match the report's claim exactly.
