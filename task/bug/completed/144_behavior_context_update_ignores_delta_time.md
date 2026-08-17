# BUG-144: `BehaviorContext::update()` discards `delta_time`, always resampling real wall-clock time instead

- **Severity:** High (the entire timing subsystem — every `WaitAction` and `CooldownNode` in every
  tree — is deaf to caller-controlled simulated time; only real wall-clock elapsing ever works)
- **state:** Completed
- **Affects:** Any `BehaviorContext::update(delta_time)` caller relying on `delta_time` to drive
  time-based nodes (`WaitAction`, `CooldownNode`) — i.e. any fixed-timestep simulation,
  deterministic test, fast-forward/replay system, or paused game (`delta_time = 0`) driving the
  tree
- **Component:** `module/helper/behaviour_tree` (`src/lib.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** First bug filed for `behaviour_tree` this session. Discovered via a targeted
  `Explore` review of the crate (background dispatch, task list follow-up); independent of
  BUG-145/BUG-146 (same crate, same review pass, unrelated code paths).

## Symptom

```rust
use behaviour_tree::{ BehaviorContext, WaitAction, BehaviorNode, BehaviorStatus };
use core::time::Duration;

let mut context = BehaviorContext::new();
let mut wait = WaitAction::new( 1.0 ); // needs 1 simulated second to complete

assert_eq!( wait.execute( &mut context ), BehaviorStatus::Running );

// Fast-forward the *simulation* by 5 seconds -- no real sleep anywhere:
context.update( Duration::from_secs_f32( 5.0 ) );

let status = wait.execute( &mut context );
// Wrong (pre-fix):    BehaviorStatus::Running  -- update() threw the 5s away
// Correct (post-fix): BehaviorStatus::Success  -- 5s of delta_time > 1s duration
```

## Impact

**Who is affected:** Any caller of `BehaviorContext::update(delta_time)` that expects
`delta_time` — the parameter's obvious, only stated purpose — to control how much simulated time
has passed. This includes deterministic tests, fixed-timestep game loops, fast-forward/replay
tooling, and any paused state (`delta_time = 0` every frame).

**What breaks:** `update()` unconditionally set `current_time = Instant::now()`, storing
`delta_time` into `self.delta_time` but never using it to advance `current_time`. Both time-based
nodes in the crate make their Running/Success decision purely from
`context.current_time.duration_since(...)`: `WaitAction::execute` (comparing against a captured
`start_time`) and `CooldownNode::execute` (comparing against a captured `last_execution`). Neither
can ever be advanced by anything other than real OS wall-clock elapsing — a caller supplying
synthetic `delta_time` values gets no effect at all. This directly contradicts the field's own doc
comment, `/// Current game time for time-based behaviors` — "game time" implies caller control
(pause, fast-forward, replay), not raw `Instant::now()`.

**Magnitude:** Not a crash — a silent no-op. Any caller architecture that assumes `delta_time`
drives the tree (the standard pattern for deterministic game-logic ticking) silently gets
real-time-only behavior instead, which is untestable without actual sleeping and un-pausable in a
paused game.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

A targeted `Explore` subagent review of `module/helper/behaviour_tree` (background dispatch,
covering the crate's full `src/lib.rs` and `tests/behaviour_tree_test.rs`), flagged as: the two
existing timing tests (`test_wait_action`, `test_cooldown_node`) both pair `context.update(...)`
with an immediately-preceding `std::thread::sleep(...)` of the identical duration — a strong signal
that real time, not `delta_time`, is what those tests actually depend on. Confirmed by direct read
of `update()`'s body and both call sites of `context.current_time.duration_since(...)`.

## Minimum Reproducible Example

```bash
cd module/helper/behaviour_tree && cargo test --test behaviour_tree_test test_context_update_advances_purely_from_delta_time 2>&1 | tail -10
```

**Expected** (post-fix):
```
test test_context_update_advances_purely_from_delta_time ... ok
```

**Actual** (pre-fix — confirmed by temporarily reverting `update()` back to
`self.current_time = Instant::now()`, then restoring the fix immediately after capturing the
failure):
```
thread 'test_context_update_advances_purely_from_delta_time' panicked at module/helper/behaviour_tree/tests/behaviour_tree_test.rs:201:3:
assertion `left == right` failed
  left: Running
 right: Success
```

**Verify Command** (≤3 lines, standalone):
```bash
cd module/helper/behaviour_tree && cargo test --test behaviour_tree_test test_context_update_advances_purely_from_delta_time
# 1 passed = fixed; 1 failed (Running != Success) = bug present
```

**Known MRE limitation (check 205):** none — `BehaviorContext`/`WaitAction` are pure, synchronous,
dependency-free state; the regression test runs with no real sleep and no thread spawn, as an
ordinary native `cargo test` against the real crate directly.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `update()` discards `delta_time` and resamples `Instant::now()` instead of accumulating the supplied duration onto `current_time`. | ✅ Root Cause | Direct read of `update()`'s pre-fix body: `self.current_time = Instant::now();` — `delta_time` is stored into `self.delta_time` but never referenced again in the method. | E1 |
| H2 | `WaitAction`/`CooldownNode` might read `context.delta_time` directly somewhere, making the `current_time` resampling harmless. | ❌ Falsified | Grepped both node impls: neither reads `context.delta_time` anywhere; both exclusively call `context.current_time.duration_since(...)`. | E2 |
| H3 | The existing timing tests (`test_wait_action`, `test_cooldown_node`) already exercise and would catch this — i.e. it's covered, not a real gap. | ❌ Falsified | Both tests pair `context.update(...)` with an immediately-preceding real `std::thread::sleep(...)` of the same duration, so real time elapsing satisfies the assertion regardless of whether `delta_time` does anything — confirmed both tests still pass unchanged after the fix (§ Prevention), proving they never actually pinned this behavior. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/lib.rs`, pre-fix `BehaviorContext::update` (`self.current_time = Instant::now();`) | `delta_time` assigned to `self.delta_time` then never used again in the method body. | H1 ✅ |
| E2 | `src/lib.rs`, `WaitAction::execute` (line ~862) and `CooldownNode::execute` (line ~739) | Both exclusively call `context.current_time.duration_since(...)`; neither references `context.delta_time`. | H2 ❌ |
| E3 | `tests/behaviour_tree_test.rs`, `test_wait_action`/`test_cooldown_node` (pre-existing) | Both call `std::thread::sleep(same_duration)` immediately before `context.update(same_duration)` — real elapsed time, not `delta_time`, is what makes the pre-fix assertions pass. | H3 ❌ |

## Root Cause

```
BehaviorContext::update( delta_time ):   (pre-fix)
  self.delta_time = delta_time;    // stored, but...
  self.current_time = Instant::now();  // ...never used -- resampled from the real OS clock instead

WaitAction::execute / CooldownNode::execute:
  context.current_time.duration_since( captured_instant ) >= threshold_duration
  // driven entirely by how much REAL time elapsed between the two calls, not by delta_time
```

`Instant` has no "construct at an arbitrary simulated point" API — the only sanctioned way to
derive a caller-controlled game clock from it is to accumulate `Duration`s onto an `Instant`
captured once (`Instant + Duration`, which `AddAssign` supports directly), never to re-sample
`Instant::now()` on every tick. The pre-fix code did the latter, silently decoupling `current_time`
from the very parameter meant to control it.

## Why Not Caught

The only two timing-dependent tests both paired `context.update(...)` with a real
`std::thread::sleep(...)` of the identical duration immediately before it — real time elapsing
happened to satisfy the assertions regardless of whether `delta_time` had any effect, so neither
test could distinguish "driven by `delta_time`" from "driven by real time."

## Fix Location

`module/helper/behaviour_tree/src/lib.rs`, `BehaviorContext::update`:

```rust
// before
self.delta_time = delta_time;
self.current_time = Instant::now();

// after
self.delta_time = delta_time;
self.current_time += delta_time;
```

No signature change — pure internal-logic fix. `WaitAction`/`CooldownNode`'s existing
`duration_since(...)` comparisons need no change: they already correctly compare against whatever
`current_time` holds, and now that value is caller-controlled.

## Prevention

Added `test_context_update_advances_purely_from_delta_time` to `tests/behaviour_tree_test.rs`,
advancing a `WaitAction` purely via `context.update(...)` with **no real sleep at all**, and
asserting the wait completes — only possible if `delta_time` genuinely drives `current_time`.
Confirmed the two pre-existing sleep-based timing tests (`test_wait_action`, `test_cooldown_node`)
remain passing unchanged under the fix, since the fix still advances `current_time` by at least the
swept duration (now deterministically the exact `delta_time`, rather than "real elapsed time,
usually slightly more due to OS scheduling").

**Pitfall:** invisible in any test (or gameplay loop) that happens to pair `update()` calls with
real time actually elapsing by the same or greater amount — the bug only manifests when simulated
time and real time diverge (fast-forward, replay, deterministic tests with no sleep, or a paused
game continuing to receive real-time-driven updates it shouldn't).

## Generalized Version

**Broken assumption:** "a context/state-update method's caller-supplied parameter documents intent,
even if the implementation happens to derive the same field from a different, more convenient
source (e.g. the OS clock)." False when the parameter is the field's *only* stated purpose — an
implementation that quietly substitutes a different source for the same field breaks every caller
that relies on the parameter for anything other than accidentally-correlated real-time testing.

**Confirmed general rule:** when a method takes a parameter whose name and doc directly describe
what a field should become (`delta_time` → advance `current_time`), the implementation must
actually derive the field from that parameter, not from an independent ambient source that happens
to correlate under normal real-time use. Grep every caller of the field for tests that pair the
call with an external side-channel to the "same" effect (e.g. `thread::sleep`) — that pairing is
itself a sign the field isn't actually derived from the parameter under test.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered via a targeted `Explore` review of `module/helper/behaviour_tree`; confirmed by direct read of `update()`'s body and both `duration_since(...)` call sites, and by noting both existing timing tests mask the bug with a real `thread::sleep`. |
| 2026-08-16 | fixed | Changed `update()` from `self.current_time = Instant::now()` to `self.current_time += delta_time`. |
| 2026-08-16 | verified | Added `test_context_update_advances_purely_from_delta_time` (no real sleep); confirmed it fails against the reverted pre-fix logic with the exact predicted `Running != Success` assertion panic and passes against the fix; full crate suite (16 tests incl. 1 doctest) + `cargo clippy --all-targets -- -D warnings` clean. |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-16 earlier same day, this verifier 2026-08-16). Independently re-read `BehaviorContext::update` (confirmed `self.current_time += delta_time` genuinely present, replacing the `Instant::now()` resample, `Fix(BUG-144)`/`Root cause`/`Pitfall` comment intact) and `test_context_update_advances_purely_from_delta_time` (non-tautological: advances a `WaitAction` purely via `context.update(...)` with no real sleep, asserts `Running` then `Success`). Fresh `cargo nextest run --all-features` via `longrun` (crate-wide, covering BUG-144 through BUG-146 together): 18/18 passed. `cargo clippy --all-features --all-targets -- -D warnings`: clean. MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-138 through BUG-149 together (12-bug batch spanning `animation` and `behaviour_tree`). State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass traced both `duration_since` call sites directly; adversarial pass grepped for any other `context.delta_time` read that might make the resampling harmless (H2) and checked whether the existing tests already covered this (H3) before trusting the gap was real. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | First bug filed for `behaviour_tree`; independent of BUG-145/BUG-146 from the same review pass (unrelated code paths). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass specifically checked whether the two existing timing tests would have caught this and found they structurally couldn't (both mask it with real `thread::sleep`). | — |
| D5 | Execution Scope | 🟢 | 🟢 | Grepped every `context.current_time`/`context.delta_time` read in the crate to confirm the fix's effect reaches both consumers (`WaitAction`, `CooldownNode`) with no signature change needed at either call site. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `behaviour_tree` src+test+bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to one method body. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface; existing "current game time" contract now actually driven by its own documented input. | — |

**Reproduced:** YES — temporarily reverting the fixed `update()` back to
`self.current_time = Instant::now()` and running
`cargo test --test behaviour_tree_test test_context_update_advances_purely_from_delta_time`
produced the exact predicted `Running != Success` assertion panic at
`behaviour_tree_test.rs:201:3`; restoring the fix returned the full suite (16 tests incl. doctest)
to passing plus a clean `cargo clippy --all-targets -- -D warnings`, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/behaviour_tree/src/lib.rs` | `BehaviorContext::update`: changed `self.current_time = Instant::now()` to `self.current_time += delta_time`. `Fix(BUG-144)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/behaviour_tree/tests/behaviour_tree_test.rs` | New test (`bug_reproducer(BUG-144)`, 5-section doc comment) — `test_context_update_advances_purely_from_delta_time`. |
