# BUG-147: `Sequencer::insert` doesn't revive a `Completed` Sequencer, stranding newly-added players

- **Severity:** Medium (a `Sequencer` reused for a second batch of work after the first batch
  finished silently never runs the new batch; not a crash, not data corruption on first use)
- **state:** Completed
- **Affects:** Any `Sequencer` that reaches `AnimationState::Completed` (all contained players
  finished) and then receives a further `insert()` call without an intervening explicit
  `resume()`
- **Component:** `module/helper/animation` (`src/sequencer.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — independent of every other `animation`/`behaviour_tree` bug filed this
  session; unrelated code path (`Sequencer`'s own state-revival guard in `insert()`, not
  `Sequence<T>`'s player-selection or timing logic).

## Symptom

```rust
use animation::{ Sequencer, Tween, AnimationState, AnimatablePlayer, easing::{ base::EasingBuilder, Linear } };

let mut sequencer = Sequencer::new();
sequencer.insert( "first", Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ) );
sequencer.update( 1.0 );
assert!( sequencer.is_completed() );                          // correct: first batch finished

sequencer.insert( "second", Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ) );

assert!( !sequencer.is_completed() );
// Wrong (pre-fix):    still `true` -- state stuck at Completed despite an unstarted player
// Correct (post-fix): `false`      -- a fresh, incomplete player is now present

sequencer.update( 0.5 );
let value = sequencer.get::< Tween< f32 > >( "second" ).unwrap().value_get();
// Wrong (pre-fix):    0.0  -- update() early-returns while state == Completed; "second" never ran
// Correct (post-fix): 5.0  -- "second" advanced normally
```

## Impact

**Who is affected:** Any caller reusing a single `Sequencer` instance across more than one
independent batch of animations — e.g. a UI coordinator that inserts a fresh set of named
animations each time a new screen/state is entered, expecting the Sequencer to just pick up and
run them, the same way it did on first use.

**What breaks:** `Sequencer::update` early-returns unconditionally whenever `self.state !=
AnimationState::Running` (`sequencer.rs:103-106`). `Sequencer::insert`'s only state-revival logic
was `if self.state == AnimationState::Pending && !self.players.is_empty() { state = Running }` —
this correctly handles the very first insert (`Pending` → `Running`), but has no branch at all
for reviving from `Completed`. Once a Sequencer's last remaining player finishes and `update()`
sets `state = Completed`, every subsequent `insert()` call adds the new player to the map but
leaves `state` at `Completed` — so every subsequent `update()` call keeps early-returning before
ever reaching the loop over `self.players.values_mut()`. The newly-inserted player's `update()` is
never invoked, ever, unless the caller separately and explicitly calls `resume()`.

**Contract violation:** `is_completed()`'s own doc comment reads "Checks if the Sequencer has
completed all animations." Immediately after inserting a fresh, not-yet-run player, this is
false — a player exists that has NOT completed — yet `is_completed()` (a direct read of `state`)
continues to report `true` until an explicit `resume()` call, contradicting its documented
contract.

**Magnitude:** Silent stall, not a crash. The newly-inserted player exists in the map (`get()`,
`get_mut()`, `keys()`, `remove()` all see it normally), it just never advances no matter how many
times `update()` is called, and `is_completed()` misreports the aggregate state throughout.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

Deferred investigation task from this session's `animation` crate review (tracked as "Investigate
Sequencer::insert not reviving Completed state"). Confirmed by direct read of
`Sequencer::insert`/`Sequencer::update`'s pre-fix bodies: `insert`'s revival guard checks only
`AnimationState::Pending`, and `update`'s early-return checks `self.state != Running` — the two
conditions compose to a permanent stall for any post-`Completed` insert.

## Minimum Reproducible Example

```bash
cd module/helper/animation && cargo test --test sequencer_test test_sequencer_insert_after_completion_revives_running_state 2>&1 | tail -10
```

**Expected** (post-fix):
```
test tests::test_sequencer_insert_after_completion_revives_running_state ... ok
```

**Actual** (pre-fix — confirmed directly, before the fix was applied; no revert needed since the
defect was still live in the codebase at test-writing time):
```
thread 'tests::test_sequencer_insert_after_completion_revives_running_state' panicked at module/helper/animation/tests/sequencer_test.rs:386:5:
is_completed() still true right after inserting a fresh, not-yet-run player
```

**Verify Command** (≤3 lines, standalone):
```bash
cd module/helper/animation && cargo test --test sequencer_test test_sequencer_insert_after_completion_revives_running_state
# 1 passed = fixed; 1 failed (is_completed() stuck true) = bug present
```

**Known MRE limitation (check 205):** none — pure, synchronous, dependency-free state; the
regression test runs as an ordinary native `cargo test` against the real crate directly.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `Sequencer::insert`'s revival guard checks only `AnimationState::Pending`, so a Sequencer that already reached `Completed` never transitions back to `Running` on a further `insert()`. | ✅ Root Cause | Direct read of `insert`'s pre-fix body shows the sole guard is `if self.state == AnimationState::Pending && !self.players.is_empty()`, with no `Completed` case. | E1 |
| H2 | `Sequencer::update` would still process a newly-inserted player even while `state == Completed`, since the per-player loop doesn't itself check state. | ❌ Falsified | `update`'s very first statement is `if self.state != AnimationState::Running { return; }`, executed before the per-player loop is ever reached — the loop's own lack of a state check is irrelevant because control never gets there. | E2 |
| H3 | Widening the guard to include `Completed` would also incorrectly revive a Sequencer that was deliberately `Paused`, since `Paused` and `Completed` might share a code path. | ❌ Falsified | `Paused` and `Completed` are distinct `AnimationState` variants with no shared branch in `insert`'s guard; adding `\|\| self.state == AnimationState::Completed` to the existing `Pending` check leaves `Paused` (and any other state) completely untouched — a `Paused` Sequencer stays `Paused` across `insert()` calls exactly as before. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/sequencer.rs`, pre-fix `Sequencer::insert` | Revival guard is `state == Pending` only. | H1 ✅ |
| E2 | `src/sequencer.rs`, `Sequencer::update`'s first statement | Early-return on `state != Running` precedes the per-player loop unconditionally. | H2 ❌ |
| E3 | `src/sequencer.rs`, fixed `insert`'s guard: `state == Pending \|\| state == Completed` | `Paused` is a distinct variant, structurally excluded from the widened condition. | H3 ❌ |

## Root Cause

```
Sequencer::insert()   (pre-fix)
  self.players.insert( ... );
  if self.state == Pending && !self.players.is_empty()   // <-- only revives from Pending
  { self.state = Running; }

Sequencer::update()   (unchanged, both pre- and post-fix)
  if self.state != Running { return; }                    // <-- stuck forever once state == Completed
  ...
```

`insert()`'s revival guard was written to handle exactly one transition — the Sequencer's very
first player, `Pending` → `Running` — and never extended to cover the other state from which a
fresh player should also revive activity: `Completed`, which (unlike `Paused`) is reached
automatically rather than by explicit caller request, and therefore should not persist once new,
genuinely incomplete work exists.

## Why Not Caught

Every existing `Sequencer` test either stopped observing once `is_completed()` became `true`, or
only ever called `insert()` while the Sequencer was still `Pending`/`Running` — none inserted a
second player after a first batch had already driven the Sequencer to `Completed`.

## Fix Location

`module/helper/animation/src/sequencer.rs`, `Sequencer::insert`:

```rust
// before
pub fn insert< T >( &mut self, name : &str, player : T )
where T : AnimatablePlayer + 'static
{
  self.players.insert( name.to_string().into(), Box::new( player ) );
  if self.state == AnimationState::Pending && !self.players.is_empty()
  {
    self.state = AnimationState::Running;
  }
}

// after
pub fn insert< T >( &mut self, name : &str, player : T )
where T : AnimatablePlayer + 'static
{
  self.players.insert( name.to_string().into(), Box::new( player ) );
  if
    ( self.state == AnimationState::Pending || self.state == AnimationState::Completed )
    && !self.players.is_empty()
  {
    self.state = AnimationState::Running;
  }
}
```

Widened the revival guard to also cover `Completed`. `Paused` is deliberately left out — a
caller-requested pause must persist across inserts; only the automatically-reached `Completed`
state is silently superseded by fresh incomplete work. No signature change.

## Prevention

Added `test_sequencer_insert_after_completion_revives_running_state` to
`tests/sequencer_test.rs`: drives a `Sequencer` to `Completed` with one player, inserts a second,
and asserts both that `is_completed()` flips back to `false` immediately and that the new player
actually advances on the next `update()` call.

**Pitfall:** invisible whenever a `Sequencer` instance is used for exactly one batch of players
and discarded (or explicitly `reset()`/`resume()`d) once complete — only a `Sequencer` reused
across independent batches via `insert()` alone, with no explicit `resume()` in between, exposes
the stuck `Completed` state.

## Generalized Version

**Broken assumption:** "a state-revival guard only needs to handle the state this type of
mutation was originally designed around (adding the very first player, from `Pending`) — every
other non-`Running` state is either intentional (`Paused`, requires explicit `resume()`) or
irrelevant." False for any state reached automatically as a side effect of prior activity
(`Completed`) rather than by explicit caller request (`Paused`) — automatically-reached terminal
states must be re-examined by any mutation that could invalidate the condition that produced
them, not just the state the mutation was first written against.

**Confirmed general rule:** when a type has multiple terminal/blocking states reachable by
different means (explicit request vs. automatic derivation from contained data), a mutation that
changes the contained data and could invalidate an automatically-derived state must explicitly
revisit that state — an explicit-request state should NOT be silently overridden the same way.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Deferred investigation task from this session's `animation` crate review; confirmed by direct read of `insert`'s revival guard and `update`'s early-return composing to a permanent stall. |
| 2026-08-16 | fixed | Widened `insert`'s revival guard to also cover `AnimationState::Completed`, deliberately excluding `Paused`. |
| 2026-08-16 | verified | Added `test_sequencer_insert_after_completion_revives_running_state`; confirmed it failed pre-fix with the exact predicted `is_completed()` assertion panic and passes against the fix; full crate suite (12 tests in `sequencer_test.rs`, 39 total incl. doctests across the crate) + `cargo clippy --all-targets -- -D warnings` clean. |
| 2026-08-16 | completed | Acceptance verification by a distinct session, same batch as BUG-138 (see its completed-row note for the shared 40/40 `animation` run and MAAV batch scope). Independently re-read `Sequencer::insert` (confirmed the revival guard widened to `state == Pending || state == Completed` genuinely present, `Paused` deliberately excluded, `Fix(BUG-147)` comment intact) and `test_sequencer_insert_after_completion_revives_running_state` (non-tautological: asserts state transitions from `Completed` back to `Running` after insert, and the new player actually advances). State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass traced `insert`/`update`'s composed early-return directly from source; adversarial pass specifically checked whether the per-player loop itself might still run a new player despite the top-level early-return (H2), and whether widening the guard could wrongly revive a `Paused` Sequencer (H3), before accepting the fix as minimal and correct. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Independent of every other bug filed this session; unrelated code path (`Sequencer`'s own revival guard, not `Sequence<T>`'s player-selection/timing logic touched by BUG-138/139, and not `behaviour_tree`). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass explicitly distinguished the automatically-reached `Completed` state from the explicitly-requested `Paused` state to justify why only the former should be included in the widened guard. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Grepped every `Sequencer::insert` call site in the workspace (crate-local tests only) and confirmed the fix doesn't change behavior for the `Pending`→`Running` first-insert path or the `Paused` no-op path — both pre-existing tests (`test_sequencer_basic_flow`, `test_sequencer_pause_resume`) still pass unmodified. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `animation` src+test+bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to one method's guard condition. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface; existing "insert revives a non-Running Sequencer" contract now correctly covers both states it should. | — |

**Reproduced:** YES — the regression test was written and run against the still-unfixed codebase,
producing the exact predicted `is_completed() still true right after inserting a fresh,
not-yet-run player` assertion panic at `sequencer_test.rs:386:5`; after applying the fix, the same
test passed, and the full crate suite (12 tests in `sequencer_test.rs` incl. this one, 39 total
across the crate incl. doctests) plus `cargo clippy --all-targets -- -D warnings` came back clean,
2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/animation/src/sequencer.rs` | `Sequencer::insert`: widened the state-revival guard to also cover `AnimationState::Completed`. `Fix(BUG-147)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/animation/tests/sequencer_test.rs` | New test (`bug_reproducer(BUG-147)`, 5-section doc comment) — `test_sequencer_insert_after_completion_revives_running_state`. |
