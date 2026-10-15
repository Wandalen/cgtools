# BUG-231: `Sequencer::remove()` never updates `self.state`, stranding an emptied Sequencer permanently at `Running`

- **Severity:** Medium (a caller relying on `is_completed()`/`state()` to decide when to stop
  driving a `Sequencer` never sees a "not running" signal after removing its last player -- the
  Sequencer instead silently keeps accepting `update()` calls and accumulating `time` forever;
  no crash, no data corruption, but a real desync between observable state and actual work)
- **state:** Completed
- **Affects:** Any caller of `Sequencer::remove()` that empties the player set while `state ==
  Running` -- e.g. tearing down individual animations one at a time until none remain, rather
  than calling `reset()`/dropping the `Sequencer` outright.
- **Component:** `module/helper/animation` (`src/sequencer.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** BUG-147 (`Sequencer::insert` doesn't revive a `Completed` Sequencer) --
  same file, same `state`-machine invariant, opposite direction: BUG-147 was a caller-visible
  state that failed to *advance* on insert; this bug is a caller-visible state that fails to
  *recede* on remove. This fix deliberately mirrors BUG-147's asymmetry (only auto-reached
  states are silently superseded, never a caller-requested `Paused`).

## Symptom

```rust
// pre-fix
pub fn remove( &mut self, name : &str ) -> bool
{
  self.players.remove( name ).is_some()
}
```

`remove()` never inspects or writes `self.state`. Removing the only remaining player from a
`Running` `Sequencer` leaves `state == Running` forever -- `update()`'s own completion check
(`if all_completed && !self.players.is_empty() { self.state = AnimationState::Completed; }`)
deliberately requires a non-empty player set before it will transition to `Completed`, so an
empty-but-`Running` `Sequencer` has no path back to any other state on its own.

## Impact

**Who is affected:** Any caller that removes players individually (rather than calling
`reset()` or discarding the `Sequencer`) and inspects `state()`/`is_completed()` afterward, or
keeps calling `update()` on the (now-empty) `Sequencer` and expects it to become an inert no-op
signal-wise.

**What breaks:** `is_completed()` stays `false` forever on an empty `Sequencer` (it can never
reach `Completed`, since `update()`'s guard requires non-empty players); `state()` keeps
reporting `Running` despite there being nothing left to animate; `update()` keeps accumulating
`self.time` on every call indefinitely, since the `state != Running` early-return never fires.

**Magnitude:** 1 method (`remove`), 1 missing state transition.

**Entity Scope:** None -- a code-level defect.

## How Discovered

This session's scouting pass of `animation`, reading `sequencer.rs` in full and
cross-referencing `remove()`'s handling of `self.state` against its siblings `reset()` (which
explicitly sets `Pending` on an empty player set) and `insert()` (BUG-147's already-fixed
revival guard) -- `remove()` was the one player-set-mutating method that touched `self.players`
without ever touching `self.state`.

## Minimum Reproducible Example

```rust
let mut sequencer = Sequencer::new();
sequencer.insert( "only", Tween::new( 0.0_f32, 1.0_f32, 1.0, Linear::build() ) );
assert_eq!( sequencer.state(), AnimationState::Running );

sequencer.remove( "only" );
assert_eq!( sequencer.state(), AnimationState::Pending ); // pre-fix: still Running
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/animation && cargo nextest run --all-features -E 'test(test_sequencer_remove_last_player_leaves_pending_not_stuck_running)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `remove()` never updates `self.state`, so removing the last player while `Running` leaves `state` stuck at `Running` since `update()`'s completion check requires a non-empty player set. | ✅ Root Cause | Direct read of `remove()` (pre-fix) shows it touches only `self.players`; direct read of `update()`'s completion guard confirms `!self.players.is_empty()` is required before `Completed` is ever reached; confirmed empirically via temporary-revert-and-rerun (`left: Running, right: Pending`). | E1, E2, E4 |
| H2 | This is harmless because `update()`'s early-return on `state != Running` means an empty-but-`Running` Sequencer is inert anyway, so the stuck state has no observable consequence. | ❌ Falsified | The early-return only guards the *body* of `update()` after the `Running` check passes -- since `state` never leaves `Running`, the guard never blocks anything: every `update()` call still executes the `self.time += delta_time` accumulation on an empty player set, and `is_completed()`/`state()` both keep reporting live, in-progress state to any caller checking on the Sequencer's status. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/animation/src/sequencer.rs`, `Sequencer::remove` (pre-fix, direct read) | `self.players.remove( name ).is_some()` is the entire body -- no reference to `self.state` anywhere. | H1 ✅ |
| E2 | `module/helper/animation/src/sequencer.rs`, `Sequencer::update` (direct read) | `if all_completed && !self.players.is_empty() { self.state = AnimationState::Completed; }` -- the `!self.players.is_empty()` conjunct means an empty player set can never drive this transition, by design (so a never-populated Sequencer isn't reported "completed"). Combined with E1, an emptied-via-`remove` Sequencer has no path off `Running`. | H1 ✅ |
| E3 | `module/helper/animation/src/sequencer.rs`, `Sequencer::update`'s opening guard (direct read) | `if self.state != AnimationState::Running { return; }` runs *before* the loop body -- since `state` never actually leaves `Running` in the defect scenario, this guard is a no-op here: `self.time += delta_time` still executes every call. | H2 ❌ |
| E4 | Temporary direct-source-edit revert-and-rerun (this fix) | Reverting `remove()` to the pre-fix one-liner reproduced `left: Running, right: Pending` on `test_sequencer_remove_last_player_leaves_pending_not_stuck_running`, an exact, unambiguous empirical confirmation of the stuck state. | H1 ✅ |
| E5 | `module/helper/animation/src/sequencer.rs`, `Sequencer::reset` (direct read) | `self.state = if self.players.is_empty() { AnimationState::Pending } else { AnimationState::Running };` -- establishes this file's own existing convention that an empty player set corresponds to `Pending`, which the fix mirrors. | H1 ✅ |
| E6 | `module/helper/animation/src/sequencer.rs`, `Sequencer::insert`'s `Fix(BUG-147)` comment + guard (direct read) | `insert`'s revival guard deliberately excludes `Paused` ("a caller-requested pause must stay paused across inserts; only Completed... should be silently superseded") -- the asymmetry principle this fix's own guard (`only when state == Running`) mirrors in the opposite direction. | H1 ✅ |

## Root Cause

`Sequencer::remove` only ever mutated `self.players`, with no corresponding update to
`self.state`. `Sequencer::update`'s own completion check intentionally requires a non-empty
player set before transitioning to `Completed` (so a Sequencer that was never populated, or was
emptied without ever running, isn't misreported as having "completed" work) -- but this design
choice means an empty-but-`Running` Sequencer has no mechanism to leave `Running` on its own.
Since `remove()` was the only method that could produce an empty player set while `state` was
still `Running`, and it never touched `state`, the combination left the Sequencer permanently
stuck.

## Why Not Caught

The existing `test_sequencer_remove` only ever removes one of two inserted players, so the
player set is never fully emptied and `state()` is never inspected after a removal.

## Fix Location

`module/helper/animation/src/sequencer.rs`: `Sequencer::remove` now transitions `self.state`
from `Running` to `Pending` when the removal leaves `self.players` empty, mirroring `reset()`'s
existing empty-players-means-`Pending` convention. The guard is deliberately scoped to
`state == Running` only, mirroring BUG-147's own asymmetry on `insert` -- a caller-requested
`Paused` state must survive losing its last player, since only automatically-reached states
(`Running`, reached by `insert`/`reset`, never by direct caller request) are silently
superseded.

## Prevention

`tests/sequencer_test.rs::test_sequencer_remove_last_player_leaves_pending_not_stuck_running`
pins the exact reproducer: insert one player, remove it, assert `state()` is `Pending` (not
stuck `Running`), and assert a subsequent `update()` no longer accumulates `time`.

## Pitfall

A completion/idle-state transition guarded on "the collection is non-empty" (to avoid
misreporting a never-populated or already-idle container as "completed") can leave the *reverse*
transition -- collection becomes empty while still active -- with no path back to idle, unless
every mutation that can empty the collection explicitly re-checks and updates state itself.
`update()`'s guard was correct for its own purpose; the gap was that `remove()`, the one other
method able to produce the same empty-collection condition, didn't carry the matching
responsibility.

## Generalized Version

**Broken assumption:** "the state machine's own `update`/tick step will eventually notice and
correct any stale state, so individual mutator methods don't each need to maintain the
invariant themselves."

**Confirmed general rule:** When a state transition is deliberately gated on a collection being
non-empty (to prevent a false-positive "completed"/"done" signal on an empty-by-construction
collection), every *other* method able to empty that collection while the state is still
"active" must independently re-check and transition state -- the gated transition, by design,
cannot do it for them. Audit every mutator alongside the state machine's own tick/update method,
not just the tick method in isolation.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed | Found via this session's `animation` scouting pass, reading `sequencer.rs` in full and comparing `remove()`'s handling of `self.state` against `reset()`'s and `insert()`'s (BUG-147) existing conventions. |
| 2026-08-17 | fixed | `remove()` now transitions `Running` -> `Pending` when the removal leaves the player set empty, mirroring `reset()`'s convention and BUG-147's caller-requested-`Paused`-survives asymmetry. |
| 2026-08-17 | verified | `cargo nextest run -p animation --all-features`: 41/41 passed, 0 skipped. `cargo test --doc -p animation --all-features`: 3/3 passed. `cargo clippy -p animation --all-targets --all-features -- -D warnings`: clean. Fix verified via a temporary direct-source-edit revert-and-rerun (`left: Running, right: Pending` pre-fix, passed post-fix). |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Confirming pass: deterministic MRE, no timing/randomness involved. Adversarial pass: checked whether the test's final `update()` assertion could pass for a reason unrelated to the fix (e.g. `update()` being a no-op for any empty Sequencer regardless of state) -- ruled out by E3, which shows the early-return depends on `state`, so the assertion genuinely exercises the fix. | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Correctly identified and cross-referenced BUG-147 as the same-file, same-invariant, opposite-direction sibling; fix's asymmetry (`Running` only, never `Paused`) explicitly mirrors BUG-147's own precedent rather than inventing a new rule. | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by direct reads of `remove()`, `update()`'s completion guard and opening guard, `reset()`, and `insert()`'s `Fix(BUG-147)` comment, plus empirical revert-rerun proof. | — |
| D5 | Execution Scope | — | 🟢 | Confirming pass: fix confined to `remove()`'s body plus its doc/`Fix(BUG-231)` comment. Adversarial pass: re-read `sequencer.rs` post-fix in full to confirm no other method sharing this defect shape (mutates `self.players` without touching `self.state`) remains -- `insert` (BUG-147, already fixed) and `reset` (already correct) are the only other player-set mutators, both already handle `state` correctly. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `animation::sequencer`'s `Sequencer` struct; `Sequence<T>` (the sibling homogeneous-chain type in the same file) is untouched and has no equivalent `remove` method. No downstream crate needed updating -- `remove`'s signature (`&str -> bool`) is unchanged, only its side effect is corrected. | — |

**Reproduced:** Confirmed via `cargo nextest` (fail pre-fix with `left: Running, right: Pending`,
pass post-fix) and temporary direct-source-edit revert-and-rerun. 2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/animation/src/sequencer.rs` | `Sequencer::remove` now transitions `self.state` from `Running` to `Pending` when the removal leaves `self.players` empty (full `Fix(BUG-231)` comment block). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/animation/tests/sequencer_test.rs` | Added `test_sequencer_remove_last_player_leaves_pending_not_stuck_running` (`bug_reproducer(BUG-231)`, 5-section doc comment), placed directly after the existing `test_sequencer_remove`. |

## Refs: docs/

| File | Change |
|------|--------|
| — | None -- the fix eliminates the trap rather than leaving a permanent API characteristic to document, matching this session's established convention for fixed (not by-design) defects (see BUG-230's own Refs: docs/ precedent). |
