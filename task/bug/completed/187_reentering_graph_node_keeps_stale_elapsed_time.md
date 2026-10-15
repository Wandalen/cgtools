# BUG-187: Re-entering a graph node keeps stale elapsed time

- **Severity:** High (playback-correctness defect -- any node revisited more than once in an
  `AnimationGraph`'s lifetime desyncs further with every re-entry, compounding without bound)
- **state:** Completed
- **Affects:** Every caller of `renderer::webgl::animation::AnimationGraph` whose graph topology
  allows a node to be entered more than once ( e.g. a state machine with a cyclic edge back to an
  earlier state, or any A -> B -> A pattern ).
- **Component:** `module/helper/renderer` (`src/webgl/animation/graph.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-16
- **Related Bugs:** None -- independent of BUG-184/BUG-185/BUG-186 (different file,
  `Scaler`/`scaling.rs`, vs. `AnimationGraph`/`graph.rs` here).

## Symptom

```rust
// before, AnimatableComposition::update's is_transited block
let time = old.borrow().in_process.as_ref().unwrap().borrow().transition_as_ref().end_ref().time();
next.borrow_mut().animation.update( time );
// -- next.animation is the target node's OWN persistent Sequencer, never reset here
```

When a transition completes, the target node's own persistent `Sequencer` is synced by calling
`.update( time )`, where `time` is the transition's own end-time reading. `Sequencer::update`
treats its argument as a delta to *add* onto `self.time`, not an absolute value to set. For a
node's first-ever entry this is harmless (`self.time` starts at 0). For a node re-entered after
having played earlier in the graph's lifetime, `self.time` already holds whatever elapsed value
that node's Sequencer was frozen at when it was previously exited (nothing resets it on exit --
only the `else` branch of the normal-playback path, which stops running for a node the instant a
transition away from it begins). The new sync value is added on top of that stale leftover
instead of starting the new activation cleanly.

## Impact

**Who is affected:** Every caller whose `AnimationGraph` topology allows revisiting a node --
e.g. an idle/walk/run state machine with edges back to `idle`, or any cycle.

**What breaks:** Each time a node is re-entered, its Sequencer's `.time()` is off by however much
stale elapsed time it was carrying from its previous activation, compounding by another such
offset on every subsequent re-entry. Since `.time()` drives which segment of a multi-segment
Sequence is currently sampled (and its progress within that segment), re-entry playback runs
increasingly out of sync with what it should show, worsening without bound the more a graph
cycles through its states.

**Magnitude:** Every re-entry adds error; a graph that never revisits a node is unaffected, but
any state-machine-shaped graph ( the whole point of `AnimationGraph` existing as a graph rather
than a flat list ) revisits nodes routinely.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Continuing the backlog item filed as task #139 (`re-entering a graph node keeps stale elapsed
time`). Reading `AnimatableComposition::update`'s `is_transited` block in full, then cross
-checking `Sequencer::update`'s exact semantics (`self.time += delta_time`, a pure incremental
add, confirmed by reading `sequencer.rs` directly) against what the block passes it (an absolute
transition-end-time reading, not a delta) surfaced the missing reset. The codebase's own
established "reset-before-use" idiom -- already present in this same file's normal-playback
branch (`if current.animation.is_completed() { current.animation.reset(); }`) and in
`Transition::update` (`if self.start.is_completed() { self.start.reset(); }` /
same for `end`) -- confirmed a reset was the intended pattern here too, just missing.

## Minimum Reproducible Example

```rust
// "a" accumulates 3.0s of elapsed time, transitions to "b", "b" plays a while, transitions
// back to "a" -- re-entering "a" while its own Sequencer still holds that stale 3.0s.
graph.update( 1.0 ); graph.update( 1.0 ); graph.update( 1.0 ); // a.animation.time() == 3.0
graph.edge_add( "a", "b", "ab", instant_tween.clone(), true_condition );
graph.update( 0.5 ); graph.update( 0.5 );                      // now current == "b"
graph.update( 1.0 ); graph.update( 1.0 );                      // b.animation.time() == 2.5
graph.edge_add( "b", "a", "ba", instant_tween, true_condition );
graph.update( 0.5 ); graph.update( 0.5 );                      // now current == "a" again
// pre-fix: graph.node_get( "a" ).unwrap().time() == 6.5 ( 3.5 added onto the stale 3.0 )
// post-fix: graph.node_get( "a" ).unwrap().time() == 3.5 ( clean reset, then synced )
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo nextest run --all-features --test animation_graph_tests animation_graph_reentry_resets_stale_elapsed_time_test
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `next.animation.update( time )` adds `time` onto whatever stale elapsed value `next.animation` already holds from a previous activation, since nothing resets it on exit or before this sync. | ✅ Root Cause | Confirmed by reading `Sequencer::update`'s exact `self.time += delta_time` semantics and tracing that `is_transited` never touches `next.animation` before this one call. | E1 |
| H2 | The addition is intentional -- `time` is meant to represent a small correction/offset on top of existing elapsed time, not an absolute resync. | ❌ Falsified | `time` is read as `transition.end_ref().time()` -- the transition's OWN independently-tracked end-clone's elapsed value, entirely unrelated in magnitude to whatever `next.animation`'s own pre-existing elapsed happens to be; treating it as an additive correction has no coherent interpretation for a first-time entry ( where it works only because the base is exactly 0 ). | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/renderer/src/webgl/animation/graph.rs`, `AnimatableComposition::update` (pre-fix) | `is_transited` block calls `next.borrow_mut().animation.update( time )` with no preceding reset; `next.animation` is the SAME persistent Sequencer stored in `animation_nodes`, not a fresh clone. | H1 ✅ |
| E2 | `module/helper/animation/src/sequencer.rs`, `Sequencer::update` | `self.time += delta_time;` -- confirmed a pure incremental add, unconditional on any notion of "small correction." | H2 ❌ |
| E3 | `module/helper/renderer/src/webgl/animation/graph.rs`, normal-playback branch (line 332-335) and `module/helper/renderer/src/webgl/animation/transition.rs`, `Transition::update` | Both already reset a Sequencer before reusing it once completed -- an established idiom in this exact codebase, absent only at this one call site. | H1 ✅ |

## Root Cause

`AnimatableComposition::update`'s transition-completion handler synced the re-entered node's own
persistent Sequencer via an additive `.update( time )` call without ever resetting it first. A
node's own Sequencer is never touched while a transition away from it is in progress (the normal
-playback `else` branch that would run it stops being reached the instant `in_process` becomes
`Some`), so it simply sits frozen at its exit-time value until the next re-entry adds on top of
it.

## Why Not Caught

No pre-existing test in `animation_graph_tests.rs` drove the graph through a re-entry ( A -> B ->
A ) scenario, nor asserted on any node's own Sequencer elapsed time after a transition -- all 8
prior tests only check `current_name_get()` or node/edge existence via `node_get`/`edge_get`.

## Fix Location

`module/helper/renderer/src/webgl/animation/graph.rs`, `AnimatableComposition::update`'s
`is_transited` block: added `next.borrow_mut().animation.reset();` immediately before the
existing `next.borrow_mut().animation.update( time );` call.

## Prevention

New test `animation_graph_reentry_resets_stale_elapsed_time_test` drives a 2-node graph through
an A -> B -> A cycle, letting "a" accumulate 3.0s of elapsed time before its first exit, and
asserts its Sequencer reads exactly 3.5 ( the transition's own end time, from a clean reset )
rather than 6.5 ( that same value added onto the stale 3.0s ) after being re-entered. Verified
empirically, not just by construction: the `reset()` call was temporarily removed via a direct
source edit (no `git stash` -- outside the git whitelist), the test was re-run and failed with
`got 6.5` -- an exact match to the hand-derived pre-fix value -- then the fix was restored and
the test re-confirmed passing.

## Pitfall

A per-node persistent Sequencer that free-runs while its node is not `current` is easy to assume
"hasn't moved" since nothing is actively animating it -- but nothing suspends or resets it
either; it simply sits at whatever value it was left at. Any later code that reads it back and
treats that value as a delta rather than a snapshot silently corrupts state. `Sequencer::update`'s
own contract (`self.time += delta_time`, a pure delta) makes this sharp: passing anything other
than a genuine per-frame delta -- an absolute reading from elsewhere -- to `update()` requires a
reset immediately before, or the "elsewhere" value nonsensically compounds onto whatever was
already there.

## Generalized Version

**Broken assumption:** "A struct field that isn't the ordinary per-frame update path for it must
be inert while that path is skipped, so reading or writing it later relative to its own prior
value is still low-risk."

**Confirmed general rule:** When a value is normally advanced by *deltas* every frame but a
special-case path instead needs to set it to something derived from *elsewhere* (a sibling
object's reading, a cross-reference), that path must explicitly reset the value first -- an
additive `update()`-shaped call is only safe against a known-zero baseline.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Continuing backlog task #139; confirmed via reading `update()`'s `is_transited` block and `Sequencer::update`'s exact delta semantics. |
| 2026-08-16 | fixed | Added `next.borrow_mut().animation.reset();` immediately before the existing sync call. |
| 2026-08-16 | verified | Empirically confirmed via temporary fix removal ( direct source edit, not git ): new test failed pre-fix (`got 6.5`, exact match to hand-derivation), passed post-fix. `cargo nextest run -p renderer --test animation_graph_tests --all-features`: 9/9 passed. `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: clean. Full workspace: `cargo nextest run --workspace --all-features --exclude object_picking`: 1912/1912 passed, doctests all `ok`, `cargo clippy --workspace --all-targets --all-features --exclude object_picking -- -D warnings`: clean (0 warnings). |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming: new test passes against fixed code, 9/9 in-crate. Adversarial: attempted to show the test might pass for reasons unrelated to the fix (e.g. floating-point coincidence, or `current_id`/segment-selection side effects) -- ruled out by using only exact multiples of 0.5 ( no floating-point drift possible ) and by temporarily removing the `reset()` call, confirming the test fails with `got 6.5`, an EXACT match to the hand-derived pre-fix value, then restoring and reconfirming PASS. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Checked against BUG-184/BUG-185/BUG-186 -- confirmed independent (different file, different mechanism). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct reading of `Sequencer::update`'s exact accumulation formula and the `is_transited` block's exact call sequence, not assumed. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is the one-line `reset()` insertion only; no unrelated refactor. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Entirely within `renderer`'s own `graph.rs`. | — |
| D7 | Crate Locality | 🟢 | 🟢 | The `is_transited` sync has exactly one call site, fixed there. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix applies the codebase's own existing reset-before-use idiom to the one place it was missing, without adding unrelated scope. | — |

**Reproduced:** YES -- new test fails pre-fix (`got 6.5`, an exact match to the hand-derived
stale-accumulation value) and passes post-fix (`got 3.5`), confirmed via a temporary
direct-source-edit removal-and-rerun. Scoped suite (9/9), full workspace (1912/1912), doctests,
and clippy all clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/animation/graph.rs` | `AnimatableComposition::update`'s `is_transited` block: added `next.borrow_mut().animation.reset();` before the existing sync call, with a `Fix(BUG-187)` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/animation_graph_tests.rs` | Added `long_animation_create()` helper and `animation_graph_reentry_resets_stale_elapsed_time_test`. |
