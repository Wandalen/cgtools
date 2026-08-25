# BUG-132: `Animation::update` discards frame-timer overshoot beyond the first crossing

- **Severity:** Medium (silently under-advances sprite animation under a large `dt` — no
  panic, no compile error, just a wrong `current_frame`)
- **state:** Completed
- **Affects:** Any caller of `Animation::update(dt)` where a single call's `dt` can span more
  than one `frame_duration` (e.g. a stalled render loop, a background tab resuming, or any
  frame-rate hitch)
- **Component:** `module/helper/tiles_tools` (`src/ecs/components.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — second bug filed for this crate this session; independent of BUG-131
  (different module, different mechanism)

## Symptom

```rust
let mut anim = Animation::new( 4, 0.25 ); // 4 frames, 0.25s per frame
anim.update( 0.1 );  // 0.10s elapsed
anim.update( 0.2 );  // 0.30s elapsed -> frame 1
anim.update( 0.75 ); // 1.05s elapsed total = 4.2 frame-durations since start

// Wrong (pre-fix):
anim.current_frame == 2  // only ever advances by 1 frame per update() call

// Correct (post-fix):
anim.current_frame == 0  // 4 full frames elapsed -> looped back, 0.05s into frame 0
```

## Impact

**Who is affected:** Any caller of `Animation::update` whose `dt` is not guaranteed to stay
well under `frame_duration` — the exact case a stalled/variable-rate render loop, a browser tab
resuming from background suspension, or any single large frame-time hitch produces.

**What breaks:** `current_frame` and `frame_timer` under-advance relative to real elapsed time:
each `update()` call consumes at most one `frame_duration` regardless of how many actually
elapsed, and discards ("`frame_timer = 0.0`") whatever overshoot remained instead of carrying it
into the next frame — so a sprite animation silently falls further and further behind real time
the more frame-time hitches it accumulates.

**Magnitude:** Not a crash — a silently wrong `u32`/`f32` pair consumed directly by rendering
code with no error signal. The animation eventually "catches up" only because subsequent normal
`update()` calls each still only advance by one frame at a time, so a large-enough deficit is
permanent until manually resynchronized.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

Task #66, a targeted code review of `tiles_tools` under the standing bug-hunt mandate. The
reviewing agent flagged `Animation::update`'s single `if` (not `while`/modulo) around the
frame-duration threshold. Independently confirmed by direct reading of
`src/ecs/components.rs` lines 494-522, and by finding that the crate's own existing
`test_animation_component` (`tests/integration/ecs_tests.rs`) already exercised the exact
multi-crossing scenario — its own inline comment already derives the *correct* expected value
("0.05s into the next cycle = frame 0") immediately before pinning the *actual*, buggy value
("`current_frame, 2`") with a comment acknowledging the discrepancy ("Actually frame 2 due to
animation timing").

## Minimum Reproducible Example

```bash
cd module/helper/tiles_tools && cargo test --test integration_tests --features enabled,integration test_animation_component 2>&1 | tail -10
```

**Expected** (post-fix):
```
test integration::ecs_tests::test_animation_component ... ok
```

**Actual** (pre-fix — confirmed by temporarily reverting `update()`'s `while` loop back to a
single `if` with the unconditional `frame_timer = 0.0` reset, then restoring the fix immediately
after capturing the failure):
```
assertion `left == right` failed
  left: 2
 right: 0
```

**Verify Command** (≤3 lines, standalone):
```bash
cd module/helper/tiles_tools && cargo test --test integration_tests --features enabled,integration test_animation_component
# 1 passed = fixed; 1 failed (left: 2, right: 0) = bug present
```

**Known MRE limitation (check 205):** none — `Animation::update` is pure, synchronous,
dependency-free arithmetic; runs as an ordinary native `cargo test` against the real crate
directly.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `update`'s single `if` consumes at most one `frame_duration` per call and unconditionally resets `frame_timer` to `0.0`, discarding overshoot instead of carrying it forward. | ✅ Root Cause | Direct read of `src/ecs/components.rs` lines 502-521 pre-fix: `if self.frame_timer >= self.frame_duration { self.frame_timer = 0.0; self.current_frame += 1; ... }` — a single conditional, not a loop or modulo. | E1 |
| H2 | `frame_timer` itself is never correctly accumulated (the bug is in the `+= dt` line, not the threshold check). | ❌ Falsified | `self.frame_timer += dt;` is unconditional and correct on every call, pre- and post-fix — confirmed by reading the line immediately above the `if`/`while` block, unchanged by the fix. | E1 |
| H3 | The bug only manifests for non-looping animations (the `playing = false` branch). | ❌ Falsified | The MRE's animation has `looping: true` (the `new()` default) and the bug still manifests — the defect is in the crossing-detection itself, independent of the looping/non-looping branch reached afterward. | E2, E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/ecs/components.rs:494-522`, pre-fix | Single `if self.frame_timer >= self.frame_duration { self.frame_timer = 0.0; self.current_frame += 1; ... }` — one crossing consumed per call, overshoot discarded via the unconditional reset. | H1 ✅, H2 ❌ |
| E2 | `tests/integration/ecs_tests.rs:221-225`, pre-fix | Test's own comments: `// This means 4.2 frames total, which loops to frame 0 + remainder` then `// After 4 full frames (1.0s) we loop back, 0.05s into the next cycle = frame 0` — immediately followed by `assert_eq!(anim.current_frame, 2); // Actually frame 2 due to animation timing`. The test author already derived the correct value and pinned the wrong one instead. | H1 ✅, H3 ❌ |
| E3 | MRE run, reverted code | `left: 2, right: 0` — captured failure exactly matches the buggy value E2's comment predicted, confirming the defect reproduces in the `looping: true` (default) case. | H1 ✅, H3 ❌ |

## Root Cause

```
update(dt):
  frame_timer += dt
  if frame_timer >= frame_duration:        // <- single check, not a loop
    frame_timer = 0.0                      // <- discards overshoot unconditionally
    current_frame += 1                     // <- advances exactly one frame, regardless of dt size
```

A `dt` spanning N `frame_duration`s should advance `current_frame` by N and carry the remainder
into `frame_timer` — but the single `if` can only ever detect and consume the first crossing,
after which it unconditionally zeros `frame_timer`, throwing away however much of `dt` remained
beyond that first `frame_duration`.

## Why Not Caught

The crate's own existing test already constructed the exact multi-crossing scenario (a `dt` of
`0.75` against a `frame_duration` of `0.25`, i.e. 3 crossings in one call) — but its final
assertion was written against the buggy tool's actual output rather than the value the test's
own preceding comments had already derived as correct, so it passed by pinning the defect
instead of catching it.

## Fix Location

`module/helper/tiles_tools/src/ecs/components.rs`, `Animation::update`:

```rust
// before
if self.frame_timer >= self.frame_duration
{
  self.frame_timer = 0.0;
  self.current_frame += 1;
  ...
}

// after
while self.frame_duration > 0.0 && self.frame_timer >= self.frame_duration
{
  self.frame_timer -= self.frame_duration;
  self.current_frame += 1;
  ...
  // (non-looping completion branch additionally sets frame_timer = 0.0; break;)
}
```

The `frame_duration > 0.0` guard is a necessary consequence of switching from `if` to `while`,
not a scope expansion: `new()` never validated `frame_duration`, and the original single-`if`
form was harmless for a non-positive value (bounded to exactly +1 frame per call) — an
unguarded `while` would turn that same input into an infinite loop.

## Prevention

Converted the pre-existing `test_animation_component` into the formal `bug_reproducer(BUG-132)`
— it already built the exact repro scenario; only its final pinned value needed correcting.

**Pitfall:** silently wrong only when a single `update` call spans more than one
`frame_duration` (a large `dt`, e.g. after a stalled render loop) — any test or usage calling
`update` every frame with `dt << frame_duration` never exposes it, since a single crossing per
call is indistinguishable from the correct behavior in that regime.

## Generalized Version

**Broken assumption:** "the state-machine step function is called often enough that its input
delta never exceeds one internal state transition." Silently false whenever the caller's timing
is not guaranteed uniform — variable frame rate, background-tab suspension, GC pauses, or any
producer whose `dt` can spike.

**Confirmed general rule:** any accumulator-vs-threshold state transition driven by an external,
possibly-bursty delta (`timer += dt; if timer >= threshold { timer = 0; advance() }`) must
consume the threshold in a loop (or via `%`/`/`), not a single `if` — a single-shot check is only
ever correct if the caller can independently guarantee the delta is bounded below the threshold,
a guarantee that should be enforced or documented, never assumed silently.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered via task #66's targeted code review of `tiles_tools`; confirmed by direct read of `update()` and by finding the pre-existing test's own comments already derived the correct value it failed to assert. |
| 2026-08-16 | fixed | `if` converted to a `frame_duration > 0.0`-guarded `while`, subtracting (not resetting) `frame_timer` per crossing; non-looping completion branch explicitly zeros `frame_timer` and breaks to match the original stop-transition semantics. |
| 2026-08-16 | verified | Converted `test_animation_component` into `bug_reproducer(BUG-132)`; confirmed it fails against the reverted pre-fix code with the exact predicted wrong value (`left: 2, right: 0`) and passes against the fix; full crate suite (233 tests incl. doctests) + `cargo clippy --all-targets --features enabled,integration -- -D warnings` clean. |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-16 earlier same day, this verifier 2026-08-16). Independently re-read `Animation::update` (confirmed the `frame_duration > 0.0`-guarded `while` loop genuinely present, subtracting not resetting `frame_timer`, 3-field comment intact) and `test_animation_component` (non-tautological: asserts `current_frame == 0`, the corrected value, not the previously-pinned buggy `2`). Fresh `cargo nextest run --all-features` via `longrun` (crate-wide, covering BUG-131 through BUG-137 together): 251/251 passed. `cargo clippy --all-features --all-targets -- -D warnings`: clean. `**Related Bugs:** None` confirmed accurate. MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-131 through BUG-137 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass reused the pre-existing test's own comment-derived value; adversarial pass required actually observing the FAIL against the reverted pre-fix `if`, not trusting the test's comment — closed via revert-test-restore, captured failure text (`left: 2, right: 0`) matched exactly. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Second bug for `tiles_tools` this session; independent of BUG-131 (different file, different mechanism) — no cross-ref needed. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass challenged whether the bug was in `frame_timer`'s accumulation itself (H2) or scoped to only the non-looping branch (H3) — both falsified by direct source read and by the MRE reproducing under the default `looping: true` config. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Checked whether the `frame_duration > 0.0` guard changes any existing caller's behavior for valid (positive) durations — confirmed no, guard only affects the previously-unreachable-safely non-positive case. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `tiles_tools` `src/ecs/components.rs` + `tests/integration/ecs_tests.rs` + this bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to `update()`'s body; no public API/signature change. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface added — `update()`'s existing contract is now actually honored. | — |

**Reproduced:** YES — reverting `update()` to its exact pre-fix single-`if` form and running
`cargo test --test integration_tests --features enabled,integration test_animation_component`
produced the exact predicted wrong value (`left: 2, right: 0`); restoring the fix returned the
full suite to 233/233 passing (including doctests) plus a clean
`cargo clippy --all-targets --features enabled,integration -- -D warnings`, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/src/ecs/components.rs` | `Animation::update`: single `if` converted to a `frame_duration > 0.0`-guarded `while`, subtracting `frame_duration` per crossing instead of resetting to `0.0`. `Fix(BUG-132)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/tests/integration/ecs_tests.rs` | `test_animation_component` converted to `bug_reproducer(BUG-132)` (5-section doc comment added); final assertion corrected from the pinned buggy value (`2`) to the correct value (`0`) already derived by the test's own preceding comment. |
