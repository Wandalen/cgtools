# BUG-146: `RepeatNode::times(child, 0)` still executes its child once

- **Severity:** Medium (a node explicitly configured to never run its child runs it exactly once
  on first activation; not a crash, not a livelock)
- **state:** Completed
- **Affects:** Any `RepeatNode` built via `RepeatNode::times( child, 0 )` (`max_repeats ==
  Some( 0 )`)
- **Component:** `module/helper/behaviour_tree` (`src/lib.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** Third bug filed for `behaviour_tree` this session, from the same review pass
  as BUG-144/BUG-145. Independent of both — unrelated code path (decorator repeat-counting, not
  timing or composite reset semantics).

## Symptom

```rust
use behaviour_tree::*;

let mut repeat = RepeatNode::times
(
  Box::new( SetBlackboardAction::new( "ran", true ) ),
  0
);
let mut context = BehaviorContext::new();
let status = repeat.execute( &mut context );

assert_eq!( status, BehaviorStatus::Success );
assert_eq!( context.blackboard_get( "ran" ), None );
// Wrong (pre-fix):    Some(Bool(true))  -- child ran once despite a repeat count of 0
// Correct (post-fix): None              -- child never ran
```

## Impact

**Who is affected:** Any caller building a `RepeatNode::times( child, 0 )` — e.g. a
config-/data-driven tree where a repeat count is computed at runtime and can legitimately
evaluate to zero (a disabled step, an empty batch), or any test/tool that treats `times(_, 0)` as
a well-defined no-op sentinel.

**What breaks:** `RepeatNode::execute`'s completion check (`current_repeats >= max_repeats`) ran
only AFTER executing the child on each loop iteration. For any count ≥ 1 this ordering is
behavior-preserving — the check simply fires one iteration later than an equivalent check-first
loop, with the same total number of child executions either way. At exactly `max_repeats ==
Some( 0 )`, the difference becomes observable: the loop unconditionally executes the child once
on its very first iteration, before the completion check has ever had a chance to run, then
increments `current_repeats` to `1` and only then finds `1 >= 0` true and returns `Success`. A
node explicitly configured to run its child "0 times" ran it exactly once.

**Magnitude:** Silent wrong behavior, not a crash. Any side effect performed by the child
(blackboard writes, external calls performed by a custom `BehaviorNode` impl) fires once despite
the caller's explicit "never run this" configuration.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

A targeted `Explore` subagent review of `module/helper/behaviour_tree` (background dispatch,
covering the crate's full `src/lib.rs` and `tests/behaviour_tree_test.rs`), flagged
`RepeatNode::execute`'s check-after-act ordering as suspicious at the zero-count boundary.
Confirmed by direct read of `RepeatNode::execute`'s pre-fix body and by hand-tracing the loop for
`max_repeats == Some( 0 )` against `max_repeats == Some( n )` for `n >= 1`, showing the ordering
is behavior-preserving everywhere except exactly `n == 0`.

## Minimum Reproducible Example

```bash
cd module/helper/behaviour_tree && cargo test --test behaviour_tree_test test_repeat_node_zero_count_never_executes_child 2>&1 | tail -10
```

**Expected** (post-fix):
```
test test_repeat_node_zero_count_never_executes_child ... ok
```

**Actual** (pre-fix — confirmed by temporarily reverting `execute()`'s completion check back to
running after child execution, then restoring the fix immediately after capturing the failure):
```
thread 'test_repeat_node_zero_count_never_executes_child' panicked at module/helper/behaviour_tree/tests/behaviour_tree_test.rs:216:3:
assertion `left == right` failed
  left: Some(Bool(true))
 right: None
```

**Verify Command** (≤3 lines, standalone):
```bash
cd module/helper/behaviour_tree && cargo test --test behaviour_tree_test test_repeat_node_zero_count_never_executes_child
# 1 passed = fixed; 1 failed (Some(Bool(true)) != None) = bug present
```

**Known MRE limitation (check 205):** none — pure, synchronous, dependency-free state; the
regression test runs as an ordinary native `cargo test` against the real crate directly, with no
real time/sleep involved.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `RepeatNode::execute`'s completion check runs after executing the child each iteration, so a repeat count of exactly `0` cannot prevent the first child execution. | ✅ Root Cause | Direct read of `execute`'s pre-fix body shows the `if let Some( max ) = self.max_repeats && self.current_repeats >= max` check positioned inside the `Success \| Failure` match arm, after `self.current_repeats += 1`, rather than at the top of the loop. | E1 |
| H2 | The check-after-act ordering also causes an off-by-one child-execution count for `max_repeats == Some( n )`, `n >= 1` (not just `n == 0`). | ❌ Falsified | Hand-trace for `n = 3`: pre-fix executes the child on iterations where `current_repeats` is `0, 1, 2` (3 executions, check fires and stops the loop when `current_repeats` reaches `3` after the 3rd execution); post-fix executes the child on the same three iterations (check at top passes for `0, 1, 2`, fails and returns `Success` before a would-be 4th execution when `current_repeats == 3`) — identical total execution count for every `n >= 1`. | E2 |
| H3 | Moving the check to the top of the loop requires restructuring the loop's control flow beyond a simple relocation (e.g. changing the match arms). | ❌ Falsified | The fix is a pure relocation: the identical `if let Some( max ) = ... { self.reset(); return Success; }` block moved from after `self.child.reset()` inside the `Success \| Failure` arm to immediately inside the `for` loop, before the `match`. No other line changed. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/lib.rs`, pre-fix `RepeatNode::execute` | Completion check positioned after `self.current_repeats += 1; self.child.reset();`, inside the `Success \| Failure` match arm — reachable only after the child has already executed once that iteration. | H1 ✅ |
| E2 | Hand-trace of iteration counts for `max_repeats == Some(0)` vs. `Some(3)` under both check-before and check-after orderings | Check-after and check-before produce the identical execution count for every `n >= 1`; only `n == 0` diverges (1 execution vs. 0). | H2 ❌ |
| E3 | Diff between pre-fix and post-fix `execute()` bodies | Fix is a single block relocation, no other control-flow change. | H3 ❌ |

## Root Cause

```
RepeatNode::execute()   (pre-fix)
  for _ in 0 .. MAX_SYNC_ITERATIONS
  {
    match self.child.execute( context )
    {
      Running => return Running,
      Success | Failure =>
      {
        self.current_repeats += 1;
        self.child.reset();
        if let Some( max ) = self.max_repeats && self.current_repeats >= max   // <-- checked AFTER acting
        { self.reset(); return Success; }
      }
    }
  }

RepeatNode::execute()   (post-fix)
  for _ in 0 .. MAX_SYNC_ITERATIONS
  {
    if let Some( max ) = self.max_repeats && self.current_repeats >= max       // <-- checked BEFORE acting
    { self.reset(); return Success; }

    match self.child.execute( context )
    {
      Running => return Running,
      Success | Failure => { self.current_repeats += 1; self.child.reset(); }
    }
  }
```

A decorator whose completion condition can already be satisfied BEFORE any work is done (a repeat
count of exactly zero, satisfied immediately since `current_repeats` starts at `0`) must
check-then-act, not act-then-check. Checking only after acting is correct for every count ≥ 1 —
the check simply fires one iteration later, with no difference in total executions — but is
silently wrong at the `n == 0` boundary, where act-then-check guarantees at least one execution
no matter what the configured count says.

## Why Not Caught

The existing `test_repeat_node` only exercises `RepeatNode::times( child, 3 )`; nothing exercised
`RepeatNode::times( child, 0 )`, the one value at which check-before and check-after orderings
diverge.

## Fix Location

`module/helper/behaviour_tree/src/lib.rs`, `RepeatNode::execute`:

```rust
// before
for _ in 0 .. Self::MAX_SYNC_ITERATIONS
{
  match self.child.execute( context )
  {
    BehaviorStatus::Running => return BehaviorStatus::Running,
    BehaviorStatus::Success | BehaviorStatus::Failure =>
    {
      self.current_repeats += 1;
      self.child.reset();
      if let Some( max ) = self.max_repeats && self.current_repeats >= max
      { self.reset(); return BehaviorStatus::Success; }
    }
  }
}

// after
for _ in 0 .. Self::MAX_SYNC_ITERATIONS
{
  if let Some( max ) = self.max_repeats && self.current_repeats >= max
  { self.reset(); return BehaviorStatus::Success; }

  match self.child.execute( context )
  {
    BehaviorStatus::Running => return BehaviorStatus::Running,
    BehaviorStatus::Success | BehaviorStatus::Failure =>
    { self.current_repeats += 1; self.child.reset(); }
  }
}
```

Pure relocation of the existing completion-check block to the top of the loop body, before the
child executes. No signature change, no new state.

## Prevention

Added `test_repeat_node_zero_count_never_executes_child` to `tests/behaviour_tree_test.rs`:
builds `RepeatNode::times( .., 0 )` around a `SetBlackboardAction` with an observable side effect
(a blackboard write), executes once, and asserts both the returned status (`Success`) and that
the blackboard key was never set (`None`) — the latter assertion is what the pre-fix ordering
fails.

**Pitfall:** invisible whenever every exercised repeat count is `>= 1`, since check-after and
check-before are behavior-identical there — only a repeat count of exactly zero, a value easy to
omit from hand-written test cases (it looks like a "degenerate"/"nothing to test" case), exposes
the divergence.

## Generalized Version

**Broken assumption:** "checking a loop's stop condition after doing the loop body's work is
just a cosmetic reordering, since a stop condition worth having is one the loop body's own work
will eventually satisfy." False — when the stop condition can already be TRUE before any
iteration runs (a target count of zero, an already-empty collection, an already-expired
deadline), check-after guarantees at least one unwanted iteration; only check-before correctly
handles the zero/already-satisfied case.

**Confirmed general rule:** for any loop whose stop condition is a caller-supplied bound compared
against an internal counter starting at a caller-controllable value (including zero), the
condition must be checked at the TOP of the loop body, before any side-effecting work — never
after — specifically so a bound of zero (or any value the counter can already equal/exceed before
the first iteration) short-circuits with zero executions.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered via a targeted `Explore` review of `module/helper/behaviour_tree`; confirmed by direct read of `RepeatNode::execute`'s check-after-act ordering and a hand-trace showing it diverges from check-before-act only at `max_repeats == Some(0)`. |
| 2026-08-16 | fixed | Relocated the completion check to the top of the loop body, before the child executes. |
| 2026-08-16 | verified | Added `test_repeat_node_zero_count_never_executes_child`; confirmed it fails against the reverted pre-fix ordering with the exact predicted `Some(Bool(true)) != None` assertion panic and passes against the fix; full crate suite (18 tests incl. 1 doctest) + `cargo clippy --all-targets -- -D warnings` clean. |
| 2026-08-16 | completed | Acceptance verification by a distinct session, same batch as BUG-144 (see its completed-row note for the shared 18/18 `behaviour_tree` run and MAAV batch scope). Independently re-read `RepeatNode::execute` (confirmed the completion check relocated to the top of the loop body, before child execution, genuinely present, `Fix(BUG-146)` comment intact) and `test_repeat_node_zero_count_never_executes_child` (non-tautological: asserts `Success` and that the child's blackboard side effect never fired). State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass traced the check-after-act ordering directly from source; adversarial pass specifically hand-traced `n >= 1` cases (H2) to rule out a broader off-by-one before accepting the zero-boundary framing, and confirmed the fix was a pure relocation with no hidden control-flow change (H3). | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Third bug for `behaviour_tree`, from the same review pass as BUG-144/BUG-145; cross-checked for shared root cause — none, unrelated code path (decorator repeat-counting vs. timing/composite-reset). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass algebraically verified the "identical execution count for n >= 1" claim by hand-trace rather than asserting it, isolating the divergence to exactly `n == 0`. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Grepped every `RepeatNode::times`/`RepeatNode::infinite` construction site in the workspace (only this crate's own tests) and confirmed the fix does not change behavior for any existing non-zero-count usage (`test_repeat_node`, `test_repeat_node_infinite_livelock_guard` both still pass unmodified). | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `behaviour_tree` src+test+bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to one method body (single block relocation). | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface; existing "repeat count bounds total child executions" contract now holds at the zero boundary too. | — |

**Reproduced:** YES — temporarily reverting the fixed `execute()`'s completion-check position back
to after child execution (marked `// TEMPORARY BUG-146 REVERT FOR MRE VERIFICATION`) and running
`cargo test --test behaviour_tree_test test_repeat_node_zero_count_never_executes_child` produced
the exact predicted `Some(Bool(true)) != None` assertion panic at `behaviour_tree_test.rs:216:3`;
restoring the fix returned the full suite (18 tests incl. doctest) to passing plus a clean
`cargo clippy --all-targets -- -D warnings`, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/behaviour_tree/src/lib.rs` | `RepeatNode::execute`: relocated the completion check to the top of the loop body, before the child executes. `Fix(BUG-146)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/behaviour_tree/tests/behaviour_tree_test.rs` | New test (`bug_reproducer(BUG-146)`, 5-section doc comment) — `test_repeat_node_zero_count_never_executes_child`. |
