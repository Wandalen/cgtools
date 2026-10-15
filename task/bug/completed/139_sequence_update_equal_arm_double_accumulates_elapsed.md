# BUG-139: `Sequence::update`'s `Equal` arm feeds a reconstructed absolute elapsed into the additive `update()`

- **Severity:** High (any player active for 2+ consecutive frames completes far faster than its
  declared duration — not a rare edge case, the common per-frame path)
- **state:** Completed
- **Affects:** Any caller of `Sequence<T>::update` where a player stays active across more than
  one `.update()` call — i.e. any real, multi-frame usage
- **Component:** `module/helper/animation` (`src/sequencer.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** Same function as BUG-138 (independent root cause — index selection vs.
  delta-accumulation; this bug's own regression test required BUG-138's fix already applied,
  since it relies on player 0 being correctly selected across two frames); second bug filed for
  `animation` this session

## Symptom

```rust
let players = vec!
[
  Tween::new( 0.0_f32, 10.0_f32, 2.0, Linear::build() ),                 // delay=0.0, duration=2.0
  Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ).with_delay( 2.0 ),
];
let mut sequence = Sequence::new( players ).unwrap();

sequence.update( 0.5 ); // frame 1: player 0 Pending -> Running, elapsed becomes 0.5
sequence.update( 0.5 ); // frame 2: player 0 still active -- should add only 0.5 more

// Wrong (pre-fix): sequence.current_get().unwrap().value_get() == 7.5  (elapsed reached 1.5, not 1.0)
// Correct (post-fix): sequence.current_get().unwrap().value_get() == 5.0  (elapsed == 1.0, matching wall time)
```

## Impact

**Who is affected:** Any `Sequence<T>` where a player remains the active one across 2+
consecutive `.update()` calls — true for essentially every real animation, since a player is
only ever active for a single frame in the degenerate case of a duration shorter than one frame.

**What breaks:** `AnimatablePlayer::update`'s contract is a pure incremental delta (confirmed via
`Tween::update`: `self.elapsed += remaining_time`) — callers pass only the time since the last
call, and the player accumulates it internally. The `Equal` arm (fires when the same player is
still the active one this frame) instead reconstructed an absolute "elapsed since this player
started" value from `delay_get() + progress() * duration_get()`, then called `current.update(
old_elapsed + delta_time )` — feeding the player's own already-accumulated progress back into
itself on top of the genuine new delta, every single frame after the first.

**Magnitude:** Not a crash — a silently wrong animation speed. In the concrete two-frame example
above (2 × 0.5s = 1.0s of wall time against a 2.0s duration, expected 50% progress), the player
instead reaches 75% progress after just two frames — the over-accumulation compounds every
frame, so a played-out sequence completes dramatically faster than its declared duration.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

Task #74's targeted code review of `module/helper/animation`, immediately adjacent to BUG-138 in
the same function. An `Explore` subagent dispatch flagged that the `Equal` arm's `old_elapsed +
delta_time` argument didn't match the incremental semantics `Tween::update` implements; confirmed
by direct read of `Tween::update` (`self.elapsed += remaining_time`, no absolute-time overload)
and by hand-simulating two consecutive frames on the same active player.

## Minimum Reproducible Example

```bash
cd module/helper/animation && cargo test --test sequencer_test test_sequence_update_continuing_player_receives_only_the_new_delta 2>&1 | tail -10
```

**Expected** (post-fix):
```
test tests::test_sequence_update_continuing_player_receives_only_the_new_delta ... ok
```

**Actual** (pre-fix — confirmed by temporarily reverting `current.update( delta_time )` back to
the `old_elapsed` reconstruction, then restoring the fix immediately after capturing the
failure):
```
thread 'tests::test_sequence_update_continuing_player_receives_only_the_new_delta' panicked at module/helper/animation/tests/sequencer_test.rs:344:5:
assertion `left == right` failed: continuing player's internal elapsed over-accumulated -- old (reconstructed) elapsed was added to the new delta instead of just the delta
  left: 7.5
 right: 5.0
```

**Verify Command** (≤3 lines, standalone):
```bash
cd module/helper/animation && cargo test --test sequencer_test test_sequence_update_continuing_player_receives_only_the_new_delta
# 1 passed = fixed; 1 failed (left: 7.5, right: 5.0) = bug present
```

**Known MRE limitation (check 205):** none — `Sequence<T>` is pure, synchronous,
dependency-free state; runs as an ordinary native `cargo test` against the real crate directly.
This MRE depends on BUG-138's fix already being applied (needs player 0 correctly selected on
both frames) — noted explicitly since the two bugs share a function.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The `Equal` arm passes a reconstructed absolute elapsed value, not a delta, into `update()`'s incremental-delta contract. | ✅ Root Cause | Direct read of `Sequence::update`'s `Equal` arm vs. `Tween::update`'s `self.elapsed += remaining_time` — the two are incompatible call shapes. | E1 |
| H2 | The bug is visible on a player's very first `update()` call. | ❌ Falsified | `progress()` returns `0.0` while `state == Pending` (the player's state before its first `update()`), so `old_elapsed` degenerates to `0.0` on frame 1 regardless of the bug — only a second consecutive frame on an already-`Running` player exposes it. | E2 |
| H3 | This is the same defect as BUG-138 (both in the same function). | ❌ Falsified | BUG-138 is an index-selection error (`binary_search_by`'s `Err` case); this is a delta-accumulation error in the dispatch arm that runs once the correct index is already selected — independent root causes, independently reproducible (this bug's own MRE required BUG-138 already fixed to even reach the `Equal` arm twice on the same player). | Direct trace of both fixes applied independently |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/sequencer.rs`, pre-fix `Equal` arm | `let old_elapsed = current.delay_get() + ( current.progress() * current.duration_get() ); current.update( old_elapsed + delta_time );` vs. `src/interpolation.rs`'s `Tween::update`: `self.elapsed += remaining_time` — confirms the additive/incremental contract the `Equal` arm violates. | H1 ✅ |
| E2 | MRE run, reverted logic | `left: 7.5, right: 5.0` only after TWO consecutive `update()` calls on the same player — a single-call variant of this test (matching BUG-138's own MRE) would not expose it, confirming the `Pending`-state masking on frame 1. | H1 ✅, H2 ❌ |

## Root Cause

```
Sequence::update(), Equal arm (same player still active):
  old_elapsed = current.delay_get() + current.progress() * current.duration_get()
  // old_elapsed reconstructs "how far this player already got", NOT a new increment
  current.update( old_elapsed + delta_time )
  // AnimatablePlayer::update is ADDITIVE: internally does `self.elapsed += <argument>`
  // so this adds old_elapsed *again* on top of the player's own already-stored progress
```

`AnimatablePlayer::update(delta_time)` expects only the time elapsed since the *previous* call —
the player accumulates internally. The `Equal` arm instead recomputed the player's own current
position and added it back in alongside the genuine new delta, causing double- (and,
compounding across further frames, worse-than-double-) counting.

## Why Not Caught

No existing test called `.update()` twice in a row on a `Sequence` while the same player stayed
active — the only pre-existing `Sequence` test covered solely the constructor's error path, and
this session's own BUG-138 fix (filed and fixed immediately prior) was the first to exercise
`Sequence::update` at all, with a single-call MRE that happened to land on a still-`Pending`
player, masking this second, independent defect.

## Fix Location

`module/helper/animation/src/sequencer.rs`, `Sequence::update`'s `Equal` arm:

```rust
// before
let old_elapsed = current.delay_get() + ( current.progress() * current.duration_get() );
current.update( old_elapsed + delta_time );

// after
current.update( delta_time );
```

No signature change — pure internal-logic fix; the `old_elapsed` reconstruction is removed
entirely as dead/wrong code, not merely bypassed.

## Prevention

Added `test_sequence_update_continuing_player_receives_only_the_new_delta` to
`tests/sequencer_test.rs`, driving the same player across two consecutive frames and checking its
resulting value against the wall-clock-correct expectation.

**Pitfall:** invisible on a player's very first `update()` call, since `progress()` returns `0.0`
while `Pending` — a test that only calls `.update()` once (as BUG-138's own MRE does) cannot
observe this defect; a second consecutive call on the same active player is required.

## Generalized Version

**Broken assumption:** "recomputing a value from its own current state and feeding that back
into an incremental API alongside the genuine new input is a safe no-op or a defensive
resync." False whenever the incremental API's contract is a pure delta — resyncing from derived
state and re-submitting it duplicates whatever that derived state already represents.

**Confirmed general rule:** before calling an `update(delta)`-shaped API, verify the argument is
actually a delta since the last call, not a value reconstructed from the callee's own current
state — a callee that already tracks its own accumulated state should only ever receive genuinely
new information.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered via task #74's targeted code review of `module/helper/animation`, immediately adjacent to BUG-138; confirmed by direct read of `Tween::update`'s additive contract against the `Equal` arm's absolute-reconstruction call shape. |
| 2026-08-16 | fixed | Replaced the `old_elapsed` reconstruction with a direct `current.update( delta_time )`. |
| 2026-08-16 | verified | Added `test_sequence_update_continuing_player_receives_only_the_new_delta`; confirmed it fails against the reverted pre-fix logic with the exact predicted over-accumulated value (`left: 7.5, right: 5.0`) and passes against the fix; full crate suite (34 tests incl. 6 doctests, both `cargo test`'s own doctest pass and an explicit `cargo test --doc`) + `cargo clippy --all-targets -- -D warnings` clean, jointly covering both BUG-138 and BUG-139's fixes to `sequencer.rs`. |
| 2026-08-16 | completed | Acceptance verification by a distinct session, same batch as BUG-138 (see its completed-row note for the shared 40/40 `animation` run and MAAV batch scope). Independently re-read `Sequence::update`'s `Equal` arm (confirmed `current.update( delta_time )` genuinely present, replacing the `old_elapsed` reconstruction, `Fix(BUG-139)` comment intact) and `test_sequence_update_continuing_player_receives_only_the_new_delta` (non-tautological: asserts `value_get() == 5.0` after two consecutive frames on the same active player). State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass reasoned the double-accumulation from the additive contract; adversarial pass required actually observing the FAIL against reverted code, and confirming the single-call MRE shape (matching BUG-138's) would NOT expose it — closed via revert-test-restore, captured text (`left: 7.5, right: 5.0`) matched exactly. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Shares a function with BUG-138 but an independent root cause; explicitly distinguished in H3 and cross-referenced in this report's header. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass challenged whether frame 1 alone would expose the bug (H2, falsified via the `Pending`-state masking trace) and whether this was actually the same defect as BUG-138 (H3, falsified by tracing both fixes independently). | — |
| D5 | Execution Scope | 🟢 | 🟢 | Grepped `AnimatablePlayer::update` call sites across `animation` — `Sequence::update`'s three match arms are the only production call sites; the `Less` arm's `current.update( self.elapsed )` was independently traced and confirmed correct (a fresh player's `elapsed` starts at 0, so passing the full absolute value once is equivalent to a delta-from-zero). | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `animation` `src/sequencer.rs` + `tests/sequencer_test.rs` + this bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to the `Equal` arm's body; no other arm touched. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface added — existing incremental-update contract now actually honored. | — |

**Reproduced:** YES — temporarily reverting `current.update( delta_time )` back to the
`old_elapsed` reconstruction and running
`cargo test --test sequencer_test test_sequence_update_continuing_player_receives_only_the_new_delta`
produced the exact predicted over-accumulated value (`left: 7.5, right: 5.0`); restoring the fix
returned the full suite (34 tests incl. doctests) to passing plus a clean
`cargo clippy --all-targets -- -D warnings`, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/animation/src/sequencer.rs` | `Sequence::update`'s `Equal` arm: removed the `old_elapsed` reconstruction, replaced `current.update( old_elapsed + delta_time )` with `current.update( delta_time )`. `Fix(BUG-139)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/animation/tests/sequencer_test.rs` | New test (`bug_reproducer(BUG-139)`, 5-section doc comment) — `test_sequence_update_continuing_player_receives_only_the_new_delta`. |
