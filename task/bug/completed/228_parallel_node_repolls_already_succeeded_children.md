# BUG-228: `ParallelNode` re-invokes already-succeeded children every tick, preventing convergence and colliding with sibling internal state

- **Severity:** High (a `ParallelNode` with children completing at different tick counts can
  fail spuriously or never converge on `Success`; reachable via entirely ordinary usage)
- **state:** Completed
- **Affects:** Every `ParallelNode` (or `parallel(...)` convenience constructor) consumer whose
  children complete at different tick counts -- not currently consumed by any other crate in
  this workspace (`grep -rn "behaviour_tree::"` outside this crate returns nothing), but a
  first-class, prominently documented public API.
- **Component:** `module/helper/behaviour_tree` (`src/lib.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** Same file/struct as BUG-145 (abandoned-`Running`-child reset on `Failure`)
  and TASK-017 (infinite-repeat livelock guard on `RepeatNode`) but an independent root cause and
  fix -- BUG-145 fixed what happens to a child abandoned mid-`Running`; this bug is about a child
  that already reached `Success` being wrongly re-invoked at all.

## Symptom

```rust
// pre-fix -- ParallelNode::execute, every tick
for i in 0 .. self.children.len()
{
  match self.children[ i ].execute( context )   // ALWAYS re-invoked, even if already Success
  {
    ...
  }
}
```

A `ParallelNode` wrapping a fast-succeeding `CooldownNode` child and a slower `WaitAction` child
returns `Failure` on the tick the slow child finishes, instead of `Success` -- even though both
children had, in truth, already succeeded.

## Impact

**Who is affected:** Any caller combining a `ParallelNode` child with a fast, immediate
completion path (a `CooldownNode`, a `SetBlackboardAction`, any instant action/condition) with a
slower child (`WaitAction`, a nested `Sequence`/`Selector` that takes multiple ticks) -- a
completely ordinary game-AI composition pattern.

**What breaks:** Two distinct failure modes from the same root cause:
1. A child that resets its own internal state on success (`WaitAction::execute` calls
   `self.reset()` right before returning `Success`, clearing `start_time`) gets silently
   restarted the next time it's polled -- `ParallelNode` can run indefinitely without ever
   landing on the exact tick every child is simultaneously in a fresh `Success` state.
2. A child whose post-success behavior differs from its pre-success behavior (`CooldownNode`
   returns `Failure`, not `Success` or `Running`, when re-polled inside its own cooldown window)
   gets misread as a fresh failure, wrongly short-circuiting the ENTIRE composite to `Failure`
   even though that child had already legitimately succeeded moments earlier.

**Magnitude:** 1 function (`ParallelNode::execute`), missing per-child terminal-status memory.

**Entity Scope:** None -- a code-level defect.

## How Discovered

This session's scouting pass of `behaviour_tree` (previously unaudited), reading `ParallelNode`
in full and comparing its every-child-every-tick re-invocation against `SequenceNode`/
`SelectorNode`'s cursor-based skip of already-resolved children in the same file.

## Minimum Reproducible Example

```rust
let mut parallel = ParallelNode::new( vec!
[
  Box::new( CooldownNode::new( Box::new( SetBlackboardAction::new( "fast_done", true ) ),
    Duration::from_secs_f32( 100.0 ) ) ),  // succeeds tick 1, fails any re-poll within 100s
  Box::new( WaitAction::new( 1.0 ) ),      // needs 1 simulated second
] );

let mut context = BehaviorContext::new();
assert_eq!( parallel.execute( &mut context ), BehaviorStatus::Running );  // tick 1

context.update( Duration::from_secs_f32( 1.1 ) );
assert_eq!( parallel.execute( &mut context ), BehaviorStatus::Success ); // tick 2 -- FAILS pre-fix
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/behaviour_tree && cargo nextest run --all-features -E 'test(test_parallel_node_does_not_repoll_already_succeeded_child)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `execute` re-invokes every child every tick with no memory of prior-tick `Success`, unlike `SequenceNode`/`SelectorNode`'s cursor-based skip. | ✅ Root Cause | Direct read of `ParallelNode::execute` (pre-fix) shows an unconditional `for i in 0..len` loop calling `.execute()` on every child every time, contrasted with `SequenceNode`/`SelectorNode`'s `while self.current_child < len` cursor that only ever touches unresolved children. | E1, E2, E3 |
| H2 | This is intentional -- a "Parallel" composite is supposed to re-evaluate every child every tick regardless of prior status, by design. | ❌ Falsified | No doc comment, test, or design note anywhere in the crate states children should be re-polled after reaching a terminal status; the struct's own doc comment ("succeeding when all succeed") implies a converging aggregate, not perpetual re-evaluation. The observed behavior (spurious `Failure`, non-convergence) is a defect, not a documented tradeoff. | E1, E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/behaviour_tree/src/lib.rs`, `ParallelNode::execute` (pre-fix, direct read) | Unconditional `self.children[ i ].execute( context )` on every child, every tick -- no per-child terminal-status tracking. | H1 ✅ |
| E2 | `module/helper/behaviour_tree/src/lib.rs`, `SequenceNode::execute`/`SelectorNode::execute` (direct read) | Both use a `current_child` cursor that only ever re-invokes the single not-yet-resolved child -- the established in-file convention `ParallelNode` diverges from. | H1 ✅ |
| E3 | `module/helper/behaviour_tree/src/lib.rs`, `WaitAction::execute`/`CooldownNode::execute` (direct read) | `WaitAction` resets `start_time` to `None` on its own `Success`; `CooldownNode` returns `Failure` (not `Success`) when re-polled inside its cooldown window -- both produce a *different* result on re-poll than on first success, confirming re-polling an already-succeeded child is observably wrong, not merely redundant. | H1 ✅ |
| E4 | `module/helper/behaviour_tree/src/lib.rs`, `ParallelNode` struct doc comment (direct read) | "Executes all children in parallel, succeeding when all succeed" -- describes a converging aggregate outcome, with no mention of continual re-evaluation past success. | H2 ❌ |

## Root Cause

`ParallelNode::execute` had no per-child memory of a prior tick's terminal `Success`, so every
tick re-invoked every child regardless of its status in any earlier tick of the same activation.
Two of this crate's own nodes (`WaitAction`, `CooldownNode`) behave differently when re-polled
after already succeeding than they did the first time, so the missing memoization wasn't just
wasted work -- it actively corrupted the aggregate result.

## Why Not Caught

`test_parallel_node` only exercises a single tick where every child completes together (two
`SetBlackboardAction`s, both instant). `test_parallel_node_resets_abandoned_running_child_on_failure`
(BUG-145's regression test) exercises the *abandon-on-failure* path, not the
*already-succeeded-child-gets-repolled* path. No existing test re-ticks a `ParallelNode` where
one child has already succeeded but another is still `Running`.

## Fix Location

`module/helper/behaviour_tree/src/lib.rs`: `ParallelNode` gained a `succeeded : Vec<bool>` field
(one entry per child, initialized `false` in both `new`/`named`). `execute` now checks
`self.succeeded[ i ]` before invoking a child -- if already `true`, the child is skipped and its
remembered success is counted directly; otherwise the child executes as before, setting
`succeeded[ i ] = true` on a fresh `Success`. `reset()` clears every `succeeded` entry back to
`false` alongside the existing `child.reset()` cascade. The pre-existing short-circuit-to-
`Failure`-within-the-same-tick behavior (BUG-145's fix) is unchanged -- a fresh `Failure` from
any not-yet-succeeded child still immediately resets and returns, exactly as before.

## Prevention

`tests/behaviour_tree_test.rs::test_parallel_node_does_not_repoll_already_succeeded_child` pins
the exact `CooldownNode`-collision scenario described in Impact: a fast-succeeding cooldown-
wrapped child paired with a slower `WaitAction`, ticked across the boundary where the slow child
finishes. Verified via a temporary direct-source-edit revert (removed the `succeeded[i]` skip
check) and rerun: the test failed with `left: Failure, right: Success`, then passed again once
the fix was restored.

## Pitfall

A composite that keeps polling every child every tick must track which children already reached
a terminal status and stop re-invoking them -- terminal, in a `Parallel` composite, has to be
sticky for the rest of the activation. Re-polling an already-succeeded child isn't merely
redundant work: if that child's own `execute` behaves differently post-success than on first
success (a reset side effect, a cooldown window, any stateful decorator), re-polling actively
corrupts the parent composite's aggregate result.

## Generalized Version

**Broken assumption:** "re-invoking a child node that already returned `Success` is harmless --
worst case it just returns `Success` again."

**Confirmed general rule:** A `BehaviorNode`'s `execute` has no idempotency contract across
repeated calls after a terminal result -- several nodes in this very crate (`WaitAction`,
`CooldownNode`) deliberately change behavior on re-poll (reset-on-success, cooldown-gating). Any
composite that re-ticks children must track terminal status itself and stop re-invoking a child
once it reaches one, exactly as `SequenceNode`/`SelectorNode` already do via their cursor.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed | Found via this session's `behaviour_tree` scouting pass, comparing `ParallelNode::execute`'s every-tick re-invocation against `SequenceNode`/`SelectorNode`'s cursor-based skip and against `WaitAction`/`CooldownNode`'s own re-poll-differs-from-first-poll behavior. |
| 2026-08-17 | fixed | Added a per-child `succeeded : Vec<bool>` flag to `ParallelNode`; `execute` skips re-invoking any child already marked succeeded, counting its remembered result instead; `reset()` clears the flags alongside existing child resets. |
| 2026-08-17 | verified | `cargo nextest run -p behaviour_tree --all-features --no-fail-fast`: 19/19 passed, 0 skipped. `cargo clippy -p behaviour_tree --all-targets --all-features -- -D warnings`: clean. Fix verified via a temporary direct-source-edit revert-and-rerun (new test failed pre-fix with `left: Failure, right: Success`, passed post-fix). |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟠 | 🟢 | Confirming pass initially considered a simple two-`WaitAction` MRE. Adversarial pass noted timing-based MREs risk sawtooth re-alignment masking the bug (both children could coincidentally re-succeed together); switched to the deterministic `CooldownNode`-collision scenario, whose pre/post-fix divergence doesn't depend on tick-delta coincidence. | Replaced timing-coincidental MRE with the deterministic cooldown-collision one. |
| D3 | Cross-Reference Integrity | — | 🟢 | Correctly distinguished from BUG-145 (abandon-on-failure reset) and TASK-017 (infinite-repeat livelock) despite sharing the same struct/file. | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by direct comparison against `SequenceNode`/`SelectorNode`'s cursor idiom and `WaitAction`/`CooldownNode`'s own re-poll-differs behavior, not assumed from the scout's summary alone. | — |
| D5 | Execution Scope | — | 🟢 | Fix confined to adding per-child success memoization; the pre-existing same-tick Failure short-circuit (BUG-145) deliberately left unchanged. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `behaviour_tree`; no downstream crate changes needed (no other crate consumes this crate yet). | — |

**Reproduced:** Confirmed via `cargo nextest` (fail pre-fix with `left: Failure, right: Success`,
pass post-fix) and temporary direct-source-edit revert-and-rerun. 2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/behaviour_tree/src/lib.rs` | `ParallelNode`: added `succeeded : Vec<bool>` field (initialized in `new`/`named`); `execute` skips already-succeeded children and counts their remembered result; `reset` clears the flags (full `Fix(BUG-228)` comment block). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/behaviour_tree/tests/behaviour_tree_test.rs` | Added `test_parallel_node_does_not_repoll_already_succeeded_child` (`bug_reproducer(BUG-228)`, 5-section doc comment). |

## Refs: docs/

| File | Change |
|------|--------|
| — | None — no pre-existing doc section described this defect. |
