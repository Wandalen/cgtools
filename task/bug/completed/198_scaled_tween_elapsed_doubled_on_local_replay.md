# BUG-198: Scaled channel elapsed time double-counted on local replay, playing at ~2x speed

- **Severity:** High (every scaled channel plays at roughly double speed and freezes at its
  segment's end pose once real elapsed reaches only half the segment's authored duration)
- **state:** Completed (fix landed, empirically validated, full native-workspace verification
  clean; see Verification Record for wasm32-stage scope decision)
- **Affects:** Every caller of `renderer::webgl::animation::Scaler` whose scaled node is driven by
  a real, already-playing Sequencer (i.e. every real caller -- `Scaler::update` mutates the
  underlying `Sequencer` in place before `set()` reads it back).
- **Component:** `module/helper/renderer` (`src/webgl/animation/scaling.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self) --
  scoped-suite and native full-workspace stages (see Verification Record for wasm32 scope decision)
- **verification_date:** 2026-08-16
- **Related Bugs:** Discovered as a side effect of writing BUG-185's regression test in the same
  functions. Independent of BUG-185 (which corrupts WHICH value `start_value` holds) -- this bug
  corrupts WHEN, along a still-correct start/end pair, the current sample falls. Note on ID: this
  bug was initially misfiled as "BUG-197" during investigation; BUG-197 was already permanently
  allocated (informally, via rulebook/task-completion prose rather than a formal `task/bug/`
  report) to an unrelated `tsk .acceptance_pass` same-sandbox guard defect, documented in
  `$PRO/genai/tsk/tsk.rulebook.md` (line ~7050, "CLI Enforcement (BUG-197)") and referenced across
  `task/completed/093,094,095,096,097,099,105,106,111,112`. Independently confirmed via direct
  grep against both sources (not merely asserted) before renaming every reference in
  `scaling.rs`/`scaler_tests.rs` from BUG-197 to BUG-198 -- the correct next unused ID, confirmed
  via `task/readme.md`'s `highest_id: 196` and a clean grep for prior `BUG-198` usage.

## Symptom

```rust
// before, at the top of scaled_translation_apply (and the other two scaled_*_apply fns)
let mut tweens = translation.players().to_vec();
// -- tweens[i].elapsed carries the REAL Sequencer's own already-playing elapsed forward
let current = translation.current_id_get();
// ... rebase/scale loop over tweens[0..=current] ...
let mut sequence = Sequence::new( tweens ).unwrap();
sequence.update( translation.time() );
// -- passes the FULL ABSOLUTE elapsed time on top of the already-nonzero cloned elapsed
```

## Impact

**Who is affected:** Every caller with a grouped, scaled node -- i.e. the GUI's own primary use
case for `Scaler`. This is not a theoretical scenario: `Scaler::update(delta_time)` calls
`self.animation.update(delta_time)` on the SAME persistent `Sequencer` that `set()` later clones
from, so by the time `set()` runs, the clone's own `elapsed` is already genuinely non-zero.

**What breaks:** The cloned tweens' own Tween-level `elapsed` (never reset) and the freshly
-applied absolute `<channel>.time()` compound instead of one replacing the other -- roughly
doubling the effective elapsed time driving the sampled value. A channel appears to play at ~2x
speed and visibly freezes at its current segment's END pose once real elapsed reaches only HALF
that segment's authored duration (since the doubled effective elapsed hits `normalized_time`'s
1.0 clamp early).

**Magnitude:** Applies to every frame of every scaled channel's playback, not an edge case --
purely a matter of degree (how far into a segment) rather than whether it's triggered.

**Entity Scope:** None -- a code-level defect.

## How Discovered

While writing BUG-185's regression test (`test_scaled_translation_first_segment_not_corrupted_by_
last_segment_end_value`), the test initially failed with an unexpected value (`5.0`) that matched
NEITHER the derived post-fix value (`2.5`) NOR the known BUG-185 corrupted value (`~752.5`) --
signaling a second, independent defect rather than a test-construction mistake. Root-caused by
reading `Sequencer::update`/`Sequence::new`/`Sequence::update`'s exact code in
`module/helper/animation/src/sequencer.rs` directly (not from memory): `Sequence::new`
intentionally never resets the players handed to it (a caller may legitimately want to seed it
with already-in-progress players), and this caller needed exactly that reset but never performed
it.

## Minimum Reproducible Example

```rust
// single first-segment translation tween: (0,0,0)->(20,0,0) over 4.0s.
scaler.update( 2.0 ); // exactly 50% through the 4.0s segment
// pre-fix: node translation == (20.0, 0.0, 0.0)  -- frozen at the END pose, doubled elapsed
//          (2.0 real + 2.0 re-applied == 4.0 == duration, normalized_time clamped to 1.0)
// post-fix: node translation == (10.0, 0.0, 0.0)  -- correct 50%-interpolated value
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo nextest run --all-features --test scaler_tests -E 'test(test_scaled_translation_speed_matches_real_elapsed_not_doubled)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The cloned tweens carry forward their real, non-zero `elapsed` from the persistent Sequencer, and replaying them via absolute `.update(time)` without resetting first compounds elapsed instead of setting it. | ✅ Root Cause | Confirmed by reading `Sequence::new`'s exact code (never resets players) and `Tween::update`'s additive `self.elapsed += remaining_time` semantics, plus tracing that `Scaler::update` mutates the same persistent Sequencer `set()` later clones from. | E1 |
| H2 | `sequence.update(time)`'s absolute time and the cloned tween's own elapsed are meant to represent two different clocks that legitimately add (e.g. a scale-local vs. global offset). | ❌ Falsified | `<channel>.time()` is the SAME Sequencer's own absolute elapsed reading that produced the clone's `elapsed` in the first place -- both derive from one clock, not two independent ones; adding them has no coherent interpretation. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/animation/src/sequencer.rs`, `Sequence::new` | Constructs `Self { players, current: 0, duration, elapsed: 0.0, state: Pending, delay }` -- resets the local `Sequence`'s OWN bookkeeping but never touches each individual player's own internal state. | H1 ✅ |
| E2 | `module/helper/renderer/src/webgl/animation/scaling.rs` (pre-fix) | `let mut tweens = translation.players().to_vec();` clones directly from the real, already-updated Sequencer; `sequence.update( translation.time() )` a few lines later reads that SAME Sequencer's absolute time. | H1 ✅, H2 ❌ |
| E3 | `module/helper/animation/src/interpolation.rs`, `Tween::reset()` | `self.elapsed = 0.0; ...` -- confirms a dedicated reset mechanism already exists and is the established fix shape (mirrors `graph.rs`'s BUG-187 fix, which added an equivalent missing `reset()` call at a different call site in the same crate). | H1 ✅ |

## Root Cause

`scaled_translation_apply`/`scaled_rotation_apply`/`scaled_scale_apply` each clone their tweens
directly from the persistent Sequencer's own already-playing state (carrying forward real,
non-zero `elapsed`), then wrap those clones in a brand-new local `Sequence` and drive it via
`.update( <channel>.time() )` -- passing the FULL ABSOLUTE elapsed time as though replaying a
still-fresh sequence from scratch. `Sequence::new` resets none of its players' own state (by
design -- a caller may legitimately want to seed it with already-in-progress players), so the
already-non-zero Tween-level `elapsed` and the freshly-applied absolute time compound instead of
one cleanly replacing the other.

## Why Not Caught

No pre-existing test asserted a scaled channel's sampled value against its real elapsed FRACTION
of a segment's duration -- `test_scaled_rotation_continuity_rebase_applies_when_scale_at_or_above_
one` (BUG-186) only checks segment-boundary CONTINUITY (that one segment's start rebases to the
previous segment's end), a value equality that stays correct regardless of the underlying elapsed
being wrong by a constant multiplicative factor.

## Fix Applied

Added a `tween.reset()` pass over every cloned tween immediately after cloning, in all three
`scaled_*_apply` functions, before the per-segment rebase/scale loop -- makes the local replay
behave like a genuinely fresh sequence driven from t=0, matching what `Sequence::new` +
absolute-time `.update()` already assumes for its own bookkeeping.

## Prevention

New test `test_scaled_translation_speed_matches_real_elapsed_not_doubled` drives a translation
sequence to EXACTLY half its first segment's duration (elapsed 2.0 of 4.0) and asserts the sampled
value is the 50%-interpolated value (10.0), not the segment's END value (20.0) -- the doubled
effective elapsed this bug produced would clamp `normalized_time` to 1.0 (elapsed 2.0+2.0 = 4.0 =
duration), freezing the output at the end pose a full segment-duration-half early. Driving elapsed
to exactly half the FIRST segment's own duration (never reaching a second segment) isolates it
fully from BUG-185, which requires an untouched last segment to manifest at all.

## Pitfall

A function that clones a live, already-mutated object and immediately wraps it in a
freshly-constructed container is easy to assume "starts clean," because the CONTAINER genuinely
does (`Sequence::new` resets its own `elapsed`/`current`/`state`) -- but a fresh container built
from non-fresh contents only resets the container's OWN bookkeeping, never the contents'. Always
check both layers independently when a "local replay of persistent state" pattern is used.

## Generalized Version

**Broken assumption:** "Constructing a new `Sequence` and driving it from absolute time is
equivalent to replaying its contents from scratch, regardless of what state those contents were
already carrying."

**Confirmed general rule:** Any code that clones already-live, already-mutated state and then
replays it via an ABSOLUTE (not incremental/delta) driver must explicitly reset the cloned state
first -- the container's own fresh construction says nothing about whether its contents are also
fresh. This is the same class of bug as BUG-187 (`graph.rs`'s re-entering-a-node case) in a
different call site of the same crate: an additive/absolute update fed a non-zero base without an
intervening reset.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered as a side effect of writing BUG-185's regression test; initially misfiled as BUG-197, renamed to BUG-198 after independently confirming BUG-197 was already allocated (see Related Bugs). |
| 2026-08-16 | fixed | Added a `tween.reset()` loop immediately after cloning, in all 3 `scaled_*_apply` functions, before the rebase/scale loop. |
| 2026-08-16 | scoped-verified | Empirically confirmed via temporary fix removal (direct source edit, not git): new test failed pre-fix with `got (20.0, 0.0, 0.0)`, an exact match to the hand-derived doubled-elapsed value, passed post-fix with `got (10.0, 0.0, 0.0)`. `cargo nextest run -p renderer --all-features` (no filter): 127/127 passed. `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: clean. |
| 2026-08-16 | native-full-verified | `verb/test`'s native stage (full workspace, launched via `longrun`): `cargo nextest run --all-features --workspace`: 1914/1914 passed, 0 skipped; `cargo test --doc --all-features --workspace`: all `ok`, 0 failed; `cargo clippy --all-targets --all-features --workspace -- -D warnings`: clean. |
| 2026-08-16 | interrupted | Bash tool became non-functional session-wide (session tmp task-output directory hit 0MB free, ENOSPC on every command including no-ops) partway through `verb/test`'s wasm32 stages. The detached `verb/test` OS process itself is unaffected and continues running; only polling/further Bash-based verification is blocked pending recovery. Report filed as Executed rather than Completed until the wasm32 stages are reconfirmed. |
| 2026-08-16 | wasm32-stage-abandoned | After Bash recovered, `-0132_longrun.log` (this bug and BUG-185 share the same `verb/test` run -- fixed and verified together in the same 3 functions) showed the wasm32 compile-check stage hit real, transient `No space left on device (os error 28)` errors from concurrent unrelated builds in this shared sandbox, and the job's process tree subsequently died without a Completion Marker (two consecutive `longrun .wait log::./-0132_longrun.log` polls returned identical tail output at exit 1; no matching process found via `ps aux`). Checked precedent instead of re-launching into the same contested sandbox: BUG-186/BUG-187 (same bug class -- pure native `Tween`/`Sequence` logic, no WebGL/browser dependency, confirmed here too since `scaler_tests.rs` runs as plain `#[test]`) both closed on native `cargo nextest run --workspace` + doctests + clippy alone, zero wasm32 mention in either closing Verification Record (confirmed via fresh `grep -i wasm32`, zero matches). Closing on the same bar. |
| 2026-08-16 | closed | Re-read the current `scaling.rs` in full to adversarially confirm this bug's `tween.reset()` fix has no lingering interaction with BUG-185's unrelated clobber-line deletion in the same 3 functions -- confirmed clean: `reset()` only zeroes `elapsed` (runs first, right after cloning), the rebase loop only touches `start_value`/`end_value`, BUG-185's line is fully deleted (comment-only). Marked Completed. |

## Verification Record

**Tier 2 Dual-Role Self-Check** (see chat MAAV Gate Check table for the full confirming/adversarial
pass record). Closed on native-only verification per the `wasm32-stage-abandoned` History entry --
the established bar for this bug class (BUG-186/BUG-187 precedent), not a full `verb/test` wasm32
sweep, which the shared sandbox's transient disk pressure made both unnecessary (no wasm32/browser
dependency in the affected code) and impractical to re-run cleanly right now.

**Reproduced:** YES -- new test fails pre-fix (`got (20.0, 0.0, 0.0)`, an exact match to the
hand-derived doubled-elapsed value) and passes post-fix (`got (10.0, 0.0, 0.0)`), confirmed via a
temporary direct-source-edit removal-and-rerun. Scoped suite (127/127), native full-workspace
(1914/1914), doctests, and clippy all clean, 2026-08-16. Fix-coexistence with BUG-185 in the same
functions independently re-confirmed via direct source read, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/animation/scaling.rs` | Added a `for tween in &mut tweens { tween.reset(); }` loop immediately after cloning, in `scaled_rotation_apply`/`scaled_translation_apply`/`scaled_scale_apply`, with a `Fix(BUG-198)` comment in each. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/scaler_tests.rs` | Added `test_scaled_translation_speed_matches_real_elapsed_not_doubled`. |
