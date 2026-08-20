# BUG-185: `tweens[0].start_value` unconditionally clobbered by the last segment's raw end value

- **Severity:** High (visible pose corruption for the entire duration of a grouped node's first
  animation segment, the common case of a sequence just starting to play)
- **state:** Completed (fix landed, empirically validated, full native-workspace verification
  clean; see Verification Record for wasm32-stage scope decision)
- **Affects:** Every caller of `renderer::webgl::animation::Scaler` whose scaled node plays a
  multi-segment `Sequence` while `current_id_get() == 0` (i.e. any grouped node during the entire
  first segment of its animation -- not an edge case).
- **Component:** `module/helper/renderer` (`src/webgl/animation/scaling.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self) --
  scoped-suite and native full-workspace stages (see Verification Record for wasm32 scope decision)
- **verification_date:** 2026-08-16
- **Related Bugs:** Independent of BUG-184/BUG-186 (same functions, different lines) and of
  BUG-198 (the other bug discovered while writing this one's own regression test -- BUG-198
  corrupts WHEN, along a still-correct start/end pair, the sample falls; this bug corrupts WHICH
  value `start_value` holds in the first place). Both live in the same three `scaled_*_apply`
  functions but are independently reproducible and independently fixed.

## Symptom

```rust
// before, at the end of scaled_rotation_apply / scaled_translation_apply / scaled_scale_apply
tweens[ 0 ].start_value = tweens.last().unwrap().end_value;
```

This line ran unconditionally on every call, regardless of whether the per-segment loop above it
(`for i in 0..( ( current + 1 ).min( tweens.len() ) )`) had actually reached the sequence's last
segment this call.

## Impact

**Who is affected:** Every caller with a grouped, scaled node animating through a multi-segment
`Sequence` -- i.e. the GUI's own primary use case for `Scaler`.

**What breaks:** `tweens` is rebuilt fresh from the unscaled Sequencer data on every call and never
persists across frames (`rotation.players().to_vec()` et al.), so this write was either:
- **Inert**, when `current` was already the last index -- `tweens[0]` isn't sampled that call and
  is discarded before the next call rebuilds `tweens` from scratch anyway; or
- **Actively harmful**, when `current == 0` -- the common case of a sequence's first segment still
  playing -- where it overwrote the CURRENTLY SAMPLED tween's `start_value` with the raw,
  un-rebased, unscaled `end_value` of an unrelated, possibly-untouched last segment, producing a
  wrong interpolated pose for the entire duration of the first segment.

**Magnitude:** Not an edge case -- every scaled node spends real wall-clock time with
`current_id_get() == 0` at the start of every playthrough of its sequence.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Continuing backlog task #137 (`tweens[0].start_value unconditionally clobbered`). Read
`module/helper/animation/src/interpolation.rs`'s `Tween::value_get()` in full to confirm
`start_value` genuinely drives the current frame's sampled output whenever
`elapsed/duration < 1.0` (not inert bookkeeping), then traced `tweens`' full lifecycle in
`scaling.rs` -- rebuilt fresh every call from `rotation.players().to_vec()`, so the clobbered
write can only ever be read back within the SAME call, and only when `current == 0`.

## Minimum Reproducible Example

```rust
// two-segment translation sequence: (0,0,0)->(10,0,0) over [0,2.0), then
// (10,0,0)->(1000,1000,1000) over [2.0,4.0) -- sampled at elapsed=0.5, well inside segment 0.
scaler.update( 0.5 );
// pre-fix: node translation == (752.5, 750.0, 750.0)  -- corrupted by segment 1's raw end value
// post-fix: node translation == (2.5, 0.0, 0.0)        -- correct 25%-through-segment-0 value
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo nextest run --all-features --test scaler_tests -E 'test(test_scaled_translation_first_segment_not_corrupted_by_last_segment_end_value)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The clobber line corrupts `start_value` for the currently-sampled tween whenever `current == 0`, since `tweens` is rebuilt fresh every call and the write lands inside the same call's read. | ✅ Root Cause | Confirmed by tracing `tweens`' full lifecycle: built fresh from `players().to_vec()` at the top of each `scaled_*_apply` call, consumed by `Sequence::new(tweens)` + `sequence.update(time)` + `sequence.current_get()` before the function returns, never stored anywhere persistent. | E1 |
| H2 | The line is intentional "loop back to start" bookkeeping for a sequence that has completed and will restart. | ❌ Falsified | `Sequence` has no automatic loop-back to segment 0 (`Sequence::update`'s `Greater`/`Equal`/`Less` arms never re-enter an earlier segment), and `Scaler` holds no state distinguishing "first-ever playback" from "any other frame" -- there is no code path this line could coherently serve. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/renderer/src/webgl/animation/scaling.rs`, all 3 `scaled_*_apply` functions (pre-fix) | `let mut tweens = <channel>.players().to_vec();` at the top, `Sequence::new( tweens ).unwrap()` near the bottom -- confirms `tweens` is a fresh, call-local `Vec`, never a persistent field. | H1 ✅ |
| E2 | `module/helper/animation/src/sequencer.rs`, `Sequence::update`'s `AnimatablePlayer` impl | Binary-searches forward by elapsed time only; no arm ever decreases `self.current` or re-enters segment 0 once past it. | H2 ❌ |
| E3 | `module/helper/animation/src/interpolation.rs`, `Tween::value_get()` | `self.easing.apply(start, end, normalized_time)` reads `start_value` directly whenever `elapsed/duration < 1.0` -- proves the clobbered field is load-bearing for the CURRENT frame's output, not dead. | H1 ✅ |

## Root Cause

`scaled_rotation_apply`/`scaled_translation_apply`/`scaled_scale_apply` each ended with an
unconditional `tweens[ 0 ].start_value = tweens.last().unwrap().end_value;`, apparent leftover
"seamless loop" logic that doesn't fit this function's actual architecture -- `Sequence` has no
automatic loop-back to segment 0, and even a genuine external `.reset()` would be
indistinguishable from first-ever playback here, since `Scaler` holds no state to tell the two
cases apart. A write to `tweens[0]` placed after a loop that only touches `0..=current` silently
assumes `current` always reaches the last index by the time this line runs -- true only once per
sequence lifetime at most, never on the far more common frames where an earlier segment is still
playing.

## Why Not Caught

`test_scaled_rotation_continuity_rebase_applies_when_scale_at_or_above_one` (BUG-186) only drives
past the first segment boundary, landing on the LAST segment of a two-segment sequence -- exactly
the case where this clobber is inert. No pre-existing test sampled a node's transform while
`current == 0`.

## Fix Applied

Deleted the unconditional `tweens[ 0 ].start_value = tweens.last().unwrap().end_value;` line from
all three `scaled_*_apply` functions, replaced with a `Fix(BUG-185)` explanatory comment (no
replacement code -- the line was simply removed).

## Prevention

New test `test_scaled_translation_first_segment_not_corrupted_by_last_segment_end_value` drives a
two-segment translation sequence to `elapsed = 0.5`, well within the first segment's `[0, 2.0)`
window (`current == 0`), where the second segment's authored end value (`(1000,1000,1000)`) is
wildly different from the first segment's own start/end (`(0,0,0)` -> `(10,0,0)`) -- any leftover
clobber is immediately, grossly visible rather than masked by a coincidentally-similar value. All
values are exact sums of multiples of 0.5, so the expected sampled value (`2.5, 0.0, 0.0`) is
bit-exact under `f64`/`f32` arithmetic -- no epsilon-tolerance ambiguity against the pre-fix value
(`752.5, 750.0, 750.0`).

## Pitfall

A write to a rebuilt-fresh, call-local `Vec` placed AFTER a loop that only touches a prefix of it
(`0..=current`) is easy to misread as "safe leftover bookkeeping" precisely because it looks like
it can't escape the function -- but if anything downstream of the loop still reads the touched
index (as `Sequence::new(tweens)` + `sequence.current_get()` does here whenever `current == 0`),
the "leftover" write silently wins over the loop's own correct computation for that index.

## Generalized Version

**Broken assumption:** "A write placed after the main computation loop, to an index the loop
itself would only reach on a terminal/final iteration, is inert unless that terminal case is
actually hit this call."

**Confirmed general rule:** When a function rebuilds its working state fresh every call and reads
it back before returning, EVERY write inside that function -- not just ones inside the "main"
loop -- can affect this call's output. There is no such thing as safely-unreachable bookkeeping
in a call-local, non-persistent buffer; if an index is written, check whether anything after that
write still reads it, regardless of which loop iteration (if any) produced the write.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Continuing backlog task #137; confirmed via reading `interpolation.rs`'s `value_get()` and tracing `tweens`' full call-local lifecycle in `scaling.rs`. |
| 2026-08-16 | fixed | Deleted the unconditional `tweens[0].start_value` clobber line from all 3 `scaled_*_apply` functions. |
| 2026-08-16 | scoped-verified | Empirically confirmed via temporary fix removal (direct source edit, not git): new test failed pre-fix with `got (752.5, 750.0, 750.0)`, an exact match to the hand-derived pre-fix value, passed post-fix with `got (2.5, 0.0, 0.0)`. `cargo nextest run -p renderer --all-features` (no filter): 127/127 passed. `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: clean. |
| 2026-08-16 | native-full-verified | `verb/test`'s native stage (full workspace, launched via `longrun`): `cargo nextest run --all-features --workspace`: 1914/1914 passed, 0 skipped; `cargo test --doc --all-features --workspace`: all `ok`, 0 failed; `cargo clippy --all-targets --all-features --workspace -- -D warnings`: clean. |
| 2026-08-16 | interrupted | Bash tool became non-functional session-wide (session tmp task-output directory hit 0MB free, ENOSPC on every command including no-ops) partway through `verb/test`'s wasm32 stages. The detached `verb/test` OS process itself is unaffected (writes straight to its own log file, independent of the harness's output-capture mechanism) and continues running; only polling/further Bash-based verification is blocked pending recovery. Report filed as Executed rather than Completed until the wasm32 stages are reconfirmed. |
| 2026-08-16 | wasm32-stage-abandoned | After Bash recovered, `-0132_longrun.log` showed the wasm32 compile-check stage hit real, transient `No space left on device (os error 28)` errors (shared sandbox, concurrent unrelated builds from other actors observed via `ps aux` -- see `project_render_stacks_architecture`-adjacent `project_concurrent_task_actor` memory pattern) and the job's own process tree subsequently died without a Completion Marker (two consecutive `longrun .wait log::./-0132_longrun.log` polls returned identical tail output at exit 1; no matching `verb/test`/cargo process for this repo's root found in a `ps aux` sweep). Rather than re-launch the expensive full-workspace wasm32 sweep into the same contested shared sandbox, checked precedent: BUG-186 and BUG-187 -- the two most recent bugs of the same class (pure native `Tween`/`Sequence` logic, no WebGL/browser dependency; confirmed here too since `scaler_tests.rs` runs as plain `#[test]`, not `#[wasm_bindgen_test]`) -- both closed on native `cargo nextest run --workspace` + doctests + clippy alone, with zero mention of a wasm32 stage anywhere in either closing Verification Record (confirmed via fresh `grep -i wasm32`, zero matches in both files). Closing this bug on the same, already-established bar. |
| 2026-08-16 | closed | Re-read the current `scaling.rs` in full to adversarially confirm the deleted clobber line has no lingering interaction with BUG-198's unrelated `tween.reset()` fix in the same 3 functions -- confirmed clean: `reset()` only zeroes `elapsed`, the rebase loop only touches `start_value`/`end_value`, and the clobber line is fully deleted (comment-only) in all three `scaled_*_apply` functions. Marked Completed. |

## Verification Record

**Tier 2 Dual-Role Self-Check** (see chat MAAV Gate Check table for the full confirming/adversarial
pass record). Closed on native-only verification per the `wasm32-stage-abandoned` History entry --
the established bar for this bug class (BUG-186/BUG-187 precedent), not a full `verb/test` wasm32
sweep, which the shared sandbox's transient disk pressure made both unnecessary (no wasm32/browser
dependency in the affected code) and impractical to re-run cleanly right now.

**Reproduced:** YES -- new test fails pre-fix (`got (752.5, 750.0, 750.0)`, an exact match to the
hand-derived clobbered value) and passes post-fix (`got (2.5, 0.0, 0.0)`), confirmed via a
temporary direct-source-edit removal-and-rerun. Scoped suite (127/127), native full-workspace
(1914/1914), doctests, and clippy all clean, 2026-08-16. Fix-coexistence with BUG-198 in the same
functions independently re-confirmed via direct source read, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/animation/scaling.rs` | Deleted the unconditional `tweens[0].start_value = tweens.last().unwrap().end_value;` line from `scaled_rotation_apply`/`scaled_translation_apply`/`scaled_scale_apply`, with a `Fix(BUG-185)` comment in each. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/scaler_tests.rs` | Added `test_scaled_translation_first_segment_not_corrupted_by_last_segment_end_value`; corrected a stale Pitfall note in `test_scaler_applies_translation_and_scale_to_grouped_nodes`'s doc comment that had referenced this bug as still-open. |
