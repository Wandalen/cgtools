# BUG-145: `ParallelNode::execute()` never resets children on termination, unlike every sibling composite

- **Severity:** Medium (corrupts a subsequent, independent activation of a reused `ParallelNode`;
  not a crash, not silent data corruption on the first activation)
- **state:** Completed
- **Affects:** Any `ParallelNode` reused across more than one independent activation (e.g. retried
  by an ancestor `Selector`) where at least one child was still `Running` when the node returned a
  terminal `Success`/`Failure`
- **Component:** `module/helper/behaviour_tree` (`src/lib.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** Second bug filed for `behaviour_tree` this session, from the same review pass
  as BUG-144/BUG-146. Independent of both — unrelated code path (composite reset semantics, not
  timing or decorator repeat-counting).

## Symptom

```rust
use behaviour_tree::*;
use core::time::Duration;

let mut parallel = ParallelNode::new
(
  vec!
  [
    Box::new( WaitAction::new( 10.0 ) ),                  // child 0: long-running wait
    Box::new( BlackboardCondition::new( "go", true ) ),   // child 1: fails initially
  ]
);
let mut context = BehaviorContext::new();
context.blackboard_set( "go", false );

// Tick 1: child 0 starts Running, child 1 fails -> Parallel returns Failure, abandoning
// child 0's in-flight wait.
assert_eq!( parallel.execute( &mut context ), BehaviorStatus::Failure );

context.update( Duration::from_secs_f32( 20.0 ) );
context.blackboard_set( "go", true );

// Tick 2: a fresh, independent activation.
let status = parallel.execute( &mut context );
// Wrong (pre-fix):    BehaviorStatus::Success  -- child 0's abandoned tick-1 timer looks expired
// Correct (post-fix): BehaviorStatus::Running   -- child 0 restarted fresh, needs a full 10s
```

## Impact

**Who is affected:** Any caller reusing a single `ParallelNode` across more than one independent
activation — the standard pattern for a composite living under a retrying `Selector`/`Sequence`,
or any tree ticked repeatedly across frames where a `ParallelNode` can return a terminal status,
then be revisited later.

**What breaks:** `SequenceNode::execute` (calls `self.reset()` before returning `Failure`, and again
before returning `Success`) and `SelectorNode::execute` (the mirror image) both cascade
`child.reset()` to every child on every terminal transition. `ParallelNode::execute` did neither —
it returned `Failure` immediately on the first failing child with no reset at all, and returned
`Success`/the (structurally unreachable) `Failure` fallback after the loop with no reset either. Any
child still `Running` at that moment — e.g. a `WaitAction` mid-countdown, a `CooldownNode` with
pending state, a nested `SequenceNode` partway through its `current_child` — is abandoned holding
stale internal state. The next time this same `ParallelNode` instance is independently activated,
the abandoned child resumes from its old, stale timestamp/position instead of starting fresh.

**Magnitude:** Silent wrong status, not a crash. A retried `ParallelNode` branch reports completion
prematurely (or, symmetrically, could report failure prematurely) based on leftover state from a
previous, unrelated activation.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

A targeted `Explore` subagent review of `module/helper/behaviour_tree` (background dispatch,
covering the crate's full `src/lib.rs` and `tests/behaviour_tree_test.rs`), flagged by direct
comparison against `SequenceNode`/`SelectorNode`'s own `execute` implementations in the same file,
both of which call `self.reset()` on every terminal transition. Confirmed by direct read of
`ParallelNode::execute`'s pre-fix body (no `reset()` call anywhere) and by tracing that its final
`else { Failure }` branch is structurally unreachable (since the `Failure` arm inside the loop
always returns early, by the time control reaches the post-loop aggregation no child can have
failed, so `running_count == 0` always implies `success_count == self.children.len()`).

## Minimum Reproducible Example

```bash
cd module/helper/behaviour_tree && cargo test --test behaviour_tree_test test_parallel_node_resets_abandoned_running_child_on_failure 2>&1 | tail -10
```

**Expected** (post-fix):
```
test test_parallel_node_resets_abandoned_running_child_on_failure ... ok
```

**Actual** (pre-fix — confirmed by temporarily reverting `execute()` back to the
`for child in &mut self.children` loop with no `reset()` calls, then restoring the fix immediately
after capturing the failure):
```
thread 'test_parallel_node_resets_abandoned_running_child_on_failure' panicked at module/helper/behaviour_tree/tests/behaviour_tree_test.rs:166:3:
assertion `left == right` failed
  left: Success
 right: Running
```

**Verify Command** (≤3 lines, standalone):
```bash
cd module/helper/behaviour_tree && cargo test --test behaviour_tree_test test_parallel_node_resets_abandoned_running_child_on_failure
# 1 passed = fixed; 1 failed (Success != Running) = bug present
```

**Known MRE limitation (check 205):** none — pure, synchronous, dependency-free state; the
regression test runs with no real sleep (uses `context.update(...)` directly), as an ordinary
native `cargo test` against the real crate directly.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `ParallelNode::execute` never calls `self.reset()`/`child.reset()` on any terminal path, unlike `SequenceNode`/`SelectorNode`. | ✅ Root Cause | Direct read of `execute`'s pre-fix body confirms no `reset()` call on the `Failure` early-return, the post-loop `Success` return, or the post-loop `Failure` fallback; direct read of `SequenceNode`/`SelectorNode::execute` confirms both siblings call `self.reset()` on every terminal branch. | E1 |
| H2 | The post-loop `else { Failure }` fallback branch is reachable and represents a real, intended "partial failure" state distinct from the early-return `Failure`. | ❌ Falsified | Since the early-return `Failure` triggers on the FIRST failing child, by the time control reaches post-loop aggregation, every executed child returned `Success` or `Running` — so `running_count == 0` (post-loop) algebraically forces `success_count == self.children.len()`, making the `else` branch structurally unreachable in practice. | E2 |
| H3 | `self.reset()` cannot be added inside the existing `for child in &mut self.children` loop without a borrow-checker conflict, so no minimal fix exists. | ❌ Falsified | `SequenceNode`/`SelectorNode` already solve this identical problem in this same file via index-based iteration (`self.children[ self.current_child ]`), which only re-borrows per-statement rather than holding an `IterMut` for the whole loop — the same idiom applies directly to `ParallelNode`. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/lib.rs`, pre-fix `ParallelNode::execute` vs. `SequenceNode`/`SelectorNode::execute` (both call `self.reset()` on every terminal branch) | `ParallelNode` is the only composite in the file with zero `reset()` calls in `execute`. | H1 ✅ |
| E2 | Algebraic trace of `ParallelNode::execute`'s post-loop state: `Failure` always returns early, so post-loop `running_count + success_count == self.children.len()` always holds, and `running_count == 0` implies `success_count == self.children.len()` | The final `else { Failure }` can never actually execute. | H2 ❌ |
| E3 | `SequenceNode::execute`/`SelectorNode::execute` (pre-existing), both indexing via `self.children[ self.current_child ]` rather than `for child in &mut self.children` | An established, already-compiling pattern in this exact file for calling `self.reset()` from within the same loop that executes children. | H3 ❌ |

## Root Cause

```
ParallelNode::execute()   (pre-fix)
  for child in &mut self.children     // holds an IterMut borrow of self.children for the WHOLE loop
  {
    match child.execute( context )
    {
      Failure => return Failure,      // no reset -- abandons any prior Running/Success child state
      ...
    }
  }
  ... post-loop Success/Failure ...   // no reset on either branch either

SequenceNode::execute() / SelectorNode::execute()   (pre-existing, correct)
  while self.current_child < self.children.len()
  {
    match self.children[ self.current_child ].execute( context )   // per-statement re-borrow
    {
      Failure => { self.reset(); return Failure; }   // cascades child.reset() to every child
      ...
    }
  }
```

`ParallelNode` never adopted the reset-on-terminal-transition discipline its sibling composites
both follow, leaving any child still `Running` at the moment of a terminal return holding stale
internal state (a captured `Instant`, a partial index) into its next, independent activation.

## Why Not Caught

The existing `test_parallel_node` only exercises a single activation where every child succeeds
together in one tick; nothing re-activates the same `ParallelNode` instance a second time after an
earlier activation left a child `Running` and abandoned.

## Fix Location

`module/helper/behaviour_tree/src/lib.rs`, `ParallelNode::execute`:

```rust
// before
for child in &mut self.children
{
  match child.execute( context )
  {
    BehaviorStatus::Success => success_count += 1,
    BehaviorStatus::Failure => return BehaviorStatus::Failure,
    BehaviorStatus::Running => running_count += 1,
  }
}
...
else if success_count == self.children.len() { BehaviorStatus::Success }
else { BehaviorStatus::Failure }

// after
for i in 0 .. self.children.len()
{
  match self.children[ i ].execute( context )
  {
    BehaviorStatus::Success => success_count += 1,
    BehaviorStatus::Failure => { self.reset(); return BehaviorStatus::Failure; }
    BehaviorStatus::Running => running_count += 1,
  }
}
...
else if success_count == self.children.len() { self.reset(); BehaviorStatus::Success }
else { self.reset(); BehaviorStatus::Failure }
```

Switched from `for child in &mut self.children` to index-based iteration (matching
`SequenceNode`/`SelectorNode`'s own idiom) because `self.reset()` requires `&mut self`, which
conflicts with an outstanding `IterMut` borrow held for the whole `for` loop; indexing via
`self.children[ i ]` re-borrows only per-statement. No signature change.

## Prevention

Added `test_parallel_node_resets_abandoned_running_child_on_failure` to
`tests/behaviour_tree_test.rs`: activates a `ParallelNode` where one child starts `Running` and a
sibling fails immediately (forcing `Failure` with the first child abandoned mid-wait), then
independently reactivates the same node and confirms the abandoned child restarted from scratch.

**Pitfall:** invisible whenever a `ParallelNode` is only ever activated once, or every activation
happens to have all children complete together in the same tick — only a node reused across
genuinely independent activations, with an earlier `Running` child abandoned, exposes the stale
state.

## Generalized Version

**Broken assumption:** "a composite node only needs to reset its children when the node's OWN
control-flow state needs to reset (`current_child`, an index) — a composite with no such internal
index (`ParallelNode` has none) has nothing to reset." False — `reset()`'s real job is cascading to
every CHILD's own internal state, not just the composite's own bookkeeping; a composite lacking its
own index field is not exempt from needing to cascade resets to children that may hold state of
their own.

**Confirmed general rule:** when multiple sibling implementations of the same trait establish a
convention (here: "call `self.reset()` on every terminal transition"), a new or divergent
implementation of that trait must be checked against the same convention explicitly — grep sibling
`impl`s of the trait for the pattern before treating a composite's own "no index field" as license
to skip it.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered via a targeted `Explore` review of `module/helper/behaviour_tree`; confirmed by direct comparison against `SequenceNode`/`SelectorNode`'s reset discipline and by tracing the post-loop `else` branch as structurally unreachable. |
| 2026-08-16 | fixed | Switched to index-based iteration and added `self.reset()` before every terminal `Success`/`Failure` return, matching the sibling composites. |
| 2026-08-16 | verified | Added `test_parallel_node_resets_abandoned_running_child_on_failure`; confirmed it fails against the reverted pre-fix logic with the exact predicted `Success != Running` assertion panic and passes against the fix; full crate suite (17 tests incl. 1 doctest) + `cargo clippy --all-targets -- -D warnings` clean. |
| 2026-08-16 | completed | Acceptance verification by a distinct session, same batch as BUG-144 (see its completed-row note for the shared 18/18 `behaviour_tree` run and MAAV batch scope). Independently re-read `ParallelNode::execute` (confirmed index-based iteration with `self.reset()` on every terminal `Success`/`Failure` branch genuinely present, `Fix(BUG-145)` comment intact) and `test_parallel_node_resets_abandoned_running_child_on_failure` (non-tautological: asserts `Failure` on tick 1 then `Running`, not instant `Success`, on a fresh independent reactivation). State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass traced the reset-on-terminal-transition pattern directly from `SequenceNode`/`SelectorNode`; adversarial pass specifically checked whether the post-loop `else` branch was reachable (H2) and whether a borrow-checker conflict blocked the minimal fix (H3) before trusting either. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Second bug for `behaviour_tree`, from the same review pass as BUG-144/BUG-146; cross-checked for shared root cause — none, unrelated code paths. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass algebraically proved the "unreachable else" claim rather than asserting it, and confirmed the borrow-checker constraint that necessitated the index-based rewrite. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Grepped every `ParallelNode` construction site in the workspace (none outside this crate's own tests) and confirmed the fix's iteration-style change doesn't alter observable per-tick behavior on the already-passing `test_parallel_node`. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `behaviour_tree` src+test+bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to one method body. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface; existing reset-on-terminal-transition contract now consistently enforced across all three composite node types. | — |

**Reproduced:** YES — temporarily reverting the fixed `execute()` back to the pre-fix
`for child in &mut self.children` loop with no `reset()` calls and running
`cargo test --test behaviour_tree_test test_parallel_node_resets_abandoned_running_child_on_failure`
produced the exact predicted `Success != Running` assertion panic at
`behaviour_tree_test.rs:166:3`; restoring the fix returned the full suite (17 tests incl. doctest)
to passing plus a clean `cargo clippy --all-targets -- -D warnings`, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/behaviour_tree/src/lib.rs` | `ParallelNode::execute`: switched to index-based iteration and added `self.reset()` before every terminal `Success`/`Failure` return. `Fix(BUG-145)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/behaviour_tree/tests/behaviour_tree_test.rs` | New test (`bug_reproducer(BUG-145)`, 5-section doc comment) — `test_parallel_node_resets_abandoned_running_child_on_failure`. |
