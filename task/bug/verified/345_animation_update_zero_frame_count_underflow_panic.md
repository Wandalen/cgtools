# BUG-345: `Animation::update` panics with `u32` underflow when a non-looping animation has `frame_count == 0`

- **Severity:** Medium (requires a specific combination -- `frame_count == 0` AND `looping ==
  false` AND at least one full `update` cycle -- but every field on `Animation` is `pub` and
  `Animation::new` performs no validation, so the triggering state is trivially reachable via the
  public API, and the result is an unconditional panic, not a soft failure)
- **state:** Verified
- **Affects:** `tiles_tools::ecs::components::Animation::update` (`src/ecs/components.rs`) -- the
  non-looping branch of the frame-advance loop, when `frame_count == 0`
- **Component:** `module/helper/tiles_tools` (`src/ecs/components.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/

## Symptom

```bash
# Actual (pre-fix): frame_count = 0, looping = false, update(0.2) with frame_duration = 0.1.
$ cargo test -p tiles_tools --all-features --test integration_tests -- integration::ecs_tests::test_animation_zero_frame_count_non_looping_does_not_panic
thread 'integration::ecs_tests::test_animation_zero_frame_count_non_looping_does_not_panic' panicked at module/helper/tiles_tools/src/ecs/components.rs:539:32:
attempt to subtract with overflow
test result: FAILED. 1 failed

# Expected (fixed): current_frame saturates at 0, playing is set false, no panic.
$ cargo test -p tiles_tools --all-features --test integration_tests -- integration::ecs_tests::test_animation_zero_frame_count_non_looping_does_not_panic
test integration::ecs_tests::test_animation_zero_frame_count_non_looping_does_not_panic ... ok
test result: ok. 1 passed
```

## Impact

**Who is affected:** any caller that constructs an `Animation` with `frame_count: 0` and
`looping: false`, then calls `update` with a `dt` large enough to cross one `frame_duration`
threshold -- e.g. a placeholder/uninitialized sprite entity before its real frame data loads, or
an asset pipeline that produces a zero-frame animation asset for a currently-empty animation slot.

**What breaks:** `update` panics unconditionally on the first frame-boundary crossing, crashing
whatever system drives the per-frame ECS tick for every entity processed in that tick, not just
the offending one (a single panicking system call typically aborts the whole tick in this crate's
synchronous ECS update loop).

**Entity Scope:** `None` -- source-level arithmetic defect, not entity directory instances.

## How Discovered

During the same systematic bug-hunt pass over `tiles_tools`'s ECS module that found BUG-344,
`Animation::update`'s non-looping branch (`src/ecs/components.rs`, inside the frame-advance
`while` loop added by the pre-existing BUG-132 fix) was inspected for the same
clamp-after-arithmetic and unchecked-subtraction shape. The non-looping branch's
`self.current_frame = self.frame_count - 1;` has no guard for `frame_count == 0`, and
`Animation::new`'s doc comment documents no lower bound on `frame_count`. Direct construction of
a zero-frame, non-looping `Animation` and a call to `update` with `dt` exceeding one
`frame_duration` confirmed the subtraction underflows and panics.

## Minimum Reproducible Example

**Verify Command** (run from repo root; ≤3 lines):
```bash
cargo test -p tiles_tools --all-features --test integration_tests -- integration::ecs_tests::test_animation_zero_frame_count_non_looping_does_not_panic
```
**Known MRE limitation (check 203/205):** this MRE runs the crate's own reproducer test in place
rather than a synthetic `/tmp/mreNNN/` fixture -- the bug is pure in-workspace ECS logic
(`Animation::update`'s arithmetic) with no external dependencies, filesystem state, or live
GL/GPU context to isolate, so a standalone fixture would only re-implement the same struct and
call already captured by the reproducer test below. Precedent: BUG-132.

**What:** `Animation::new(0, 0.1)` (zero frames, one-second-scale duration), `.looping = false`,
then `.update(0.2)` (one full `frame_duration` crossing plus margin) -- the frame-advance loop
increments `current_frame` to `1`, finds `current_frame (1) >= frame_count (0)`, takes the
non-looping branch, and computes `self.frame_count - 1` == `0u32 - 1`, which underflows.

**Expected** (fixed): 1 passed -- `current_frame == 0` (saturating-sub floors at 0), `playing ==
false`.

**Actual** (pre-fix, directly observed via temporary revert-and-rerun of this fix): 1 failed --
panics `attempt to subtract with overflow` at `src/ecs/components.rs:539:32` (pre-fix line
numbering, at the un-guarded `self.frame_count - 1`).

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The non-looping branch of `Animation::update` can underflow `u32` when `frame_count == 0` | ✅ Verified | `src/ecs/components.rs` (pre-fix, inside the BUG-132 `while` loop's `else` branch): `self.current_frame = self.frame_count - 1;` -- no `frame_count == 0` guard | E1, E2 |
| H2 | The triggering state (`frame_count: 0, looping: false`) is reachable via public API alone | ✅ Root Cause | `Animation`'s fields are all `pub` (`src/ecs/components.rs:473-487`); `Animation::new(frame_count, frame_duration)` performs no validation on `frame_count` and always sets `looping: true` -- a caller must explicitly flip `looping` to `false` afterward, which `test_animation_zero_frame_count_non_looping_does_not_panic` does | E3 |
| H3 | This defect is distinct from, and does not collide with, the pre-existing BUG-132 fix in the same function | ✅ Verified | BUG-132's fix (`src/ecs/components.rs:516-525` comment, `526` the `while` guard itself) addresses overshoot-handling (converting a single `if` to a `while` plus a `frame_duration > 0.0` guard); BUG-345's defect is the separate `frame_count - 1` subtraction nested inside that loop's non-looping `else` branch -- different lines, different failure mode (infinite loop vs. underflow panic) | E4 |
| H4 | No existing test exercises `Animation::update` with `frame_count == 0` | ✅ Verified | `test_animation_component` (`tests/integration/ecs_tests.rs`, pre-existing, the BUG-132 regression test) constructs `Animation::new(4, 0.1)` -- never `frame_count: 0` | E5 |
| H5 | The fix does not change `update`'s behavior for any ordinary (`frame_count >= 1`) non-looping animation | ✅ Verified | `test_animation_component` re-run post-fix: unchanged pass -- `saturating_sub(1)` is identical to `- 1` for any `frame_count >= 1` | E6 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/ecs/components.rs` (pre-fix, non-looping `else` branch inside the BUG-132 `while` loop) | `self.current_frame = self.frame_count - 1;` -- unguarded subtraction | H1 ✅ |
| E2 | Terminal output (this report, MRE section) | `Animation::new(0, 0.1)` with `looping = false`, `.update(0.2)` panics `attempt to subtract with overflow` at line 539 (pre-fix numbering) | H1 ✅ |
| E3 | `src/ecs/components.rs:473-487, 491-504` | `pub struct Animation { pub current_frame: u32, pub frame_count: u32, ... pub looping: bool, ... }`; `new()` sets `looping: true` unconditionally, no `frame_count` validation | H2 ✅ |
| E4 | `src/ecs/components.rs:516-525` (unchanged comment) vs. `539-552` (this fix's comment + line) | BUG-132's fix comment documents the `while`/overshoot concern; BUG-345's fix comment (added directly below, same `else` branch) documents the separate `frame_count - 1` underflow concern -- non-overlapping line ranges, non-overlapping failure modes | H3 ✅ |
| E5 | `tests/integration/ecs_tests.rs`, `test_animation_component` (pre-existing, unmodified) | `Animation::new(4, 0.1)` -- `frame_count` is always `>= 1` in this test | H4 ✅ |
| E6 | Terminal output (`cargo test -p tiles_tools --all-features --test integration_tests -- integration::ecs_tests::test_animation_component`, post-fix) | `test result: ok. 1 passed` | H5 ✅ |

## Root Cause

```
while self.frame_duration > 0.0 && self.frame_timer >= self.frame_duration   // BUG-132's guard
{
  self.frame_timer -= self.frame_duration;
  self.current_frame += 1;

  if self.current_frame >= self.frame_count
  {
    if self.looping { self.current_frame = 0; }
    else
    {
      self.current_frame = self.frame_count - 1;
                                              ^^^ underflows when frame_count == 0
      self.playing = false;
      self.frame_timer = 0.0;
      break;
    }
  }
}
```
The non-looping branch's intent is "clamp to the last valid frame index" -- for any
`frame_count >= 1`, `frame_count - 1` is exactly that last valid index. For `frame_count == 0`
there is no valid frame index at all, and the expression underflows instead of producing a
sensible fallback (e.g. `0`).

## Why Not Caught

`test_animation_component`, the only existing test exercising the non-looping branch (added as
part of the BUG-132 fix), constructs `Animation::new(4, 0.1)` -- always at least one frame. Every
other `Animation`-constructing call site in the crate's own test suite likewise uses a positive
`frame_count`. `Animation::new` has no validation and no doc-comment lower bound on
`frame_count`, so a zero-frame animation is a silently-permitted, never-exercised state.

## Fix Location

**`src/ecs/components.rs:507-559`** (the `else` branch inside `update`'s `while` loop,
`before`/`after` at the underflowing line, `539-552` post-fix):

```rust
// Before:
else
{
  self.current_frame = self.frame_count - 1;
  self.playing = false;
  self.frame_timer = 0.0;
  break;
}

// After:
else
{
  // Fix(BUG-345): `self.frame_count - 1` underflowed (panic:
  // "attempt to subtract with overflow") when `frame_count == 0` --
  // `Animation::new` documents no lower bound on `frame_count`, and
  // the non-looping branch unconditionally subtracted 1 regardless.
  // [...full 3-field comment at src/ecs/components.rs:539-551...]
  self.current_frame = self.frame_count.saturating_sub( 1 );
  self.playing = false;
  self.frame_timer = 0.0;
  break;
}
```
Source comment (`Fix(BUG-345)`/`Root cause`/`Pitfall`) added inside the non-looping `else`
branch, immediately above the corrected line. This fix is nested inside the same `while` loop as
the pre-existing BUG-132 fix (`src/ecs/components.rs:516-526`) but modifies a different line
(`552`, not `526`) and addresses a different failure mode -- see H3/E4 above for the
non-collision confirmation.

## Prevention

Detection command for the general pattern (an unguarded `- 1` on a struct field, inside this
crate's ECS components):
```bash
grep -n "self\.[a-z_]* - 1\b" src/ecs/components.rs
```
Run against the fixed file, this finds no remaining un-guarded matches for this specific line
(now `saturating_sub(1)`, which the regex's literal `- 1` does not match) -- a starting point for
review, not a precise or general-purpose detector; it would not catch the same defect written as
`frame_count.wrapping_sub(1)` or via a temporary variable.

**Pitfall:** `count - 1` on any caller-controlled, unvalidated `u32`/unsigned count is only safe
when the type's own invariants already guarantee `count >= 1` -- they do not here (`frame_count`
is `pub` with no validating constructor, and the looping branch's `frame_count == 0` case never
reaches this line, so the `>= 1` assumption held silently until a zero-frame, non-looping
animation actually reached it). Use `saturating_sub(1)` (or an explicit `== 0` guard) instead.

## Generalized Version

**Broken assumption:** `some_count - 1` computes "the last valid index" and is safe whenever the
surrounding code conceptually assumes `some_count` represents a non-empty collection.

Fails whenever:
1. `some_count` is an unsigned (or otherwise non-overflow-checked in release builds) integer, AND
2. Nothing in the type system or a preceding runtime check actually enforces `some_count >= 1`, AND
3. A code path exists that reaches the subtraction with `some_count == 0`

**Detection invariant:**
```
for every `some_count - 1` expression on an unsigned integer type meant to represent
"the last valid index of a collection of size some_count":
  the subtraction must be `saturating_sub(1)` (or preceded by an explicit `== 0` early return/guard),
  unless `some_count >= 1` is provably enforced by the type itself (e.g. a NonZeroU32,
  or a validating constructor with no direct-field-write escape hatch)
```
Single confirmed instance in this crate for this exact shape (the `while` loop's non-looping
branch is the only "last valid frame index" computation in `Animation::update`; BUG-132's own fix
in the same loop is a distinct concern -- overshoot/looping logic, not an off-by-one index
computation). Not a duplicate of BUG-132 (that bug was about `frame_timer` overshoot discarding
excess time on a large `dt`, fixed by converting `if` to `while`; this bug is about the
`frame_count - 1` arithmetic itself underflowing when `frame_count == 0`, a concern BUG-132's fix
did not touch or introduce) -- confirmed via `git log`-independent direct reading of both fix
comments in place (E4). Dedup search:
`grep -rli "frame_count.*- 1\|animation.*underflow\|zero.*frame_count" task/bug/` found no prior
filing referencing this specific subtraction.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Found during the same systematic bug-hunt pass over `tiles_tools`'s ECS module that found BUG-344; root-caused by inspecting the non-looping branch of `Animation::update`'s frame-advance loop for unguarded arithmetic on a `pub`, unvalidated field |
| 2026-08-18 | fix_applied | Changed `self.frame_count - 1` to `self.frame_count.saturating_sub(1)` inside the non-looping branch (`src/ecs/components.rs:552`), directly below the pre-existing BUG-132 fix in the same `while` loop. Confirmed no collision (different line, different failure mode). Reproducer test confirmed FAIL pre-fix (`attempt to subtract with overflow` panic) and PASS post-fix; full `ecs_tests` module (24 tests, including the pre-existing BUG-132 regression test `test_animation_component`) and scoped clippy (`cargo clippy -p tiles_tools --all-targets --all-features -- -D warnings`) both clean |
| 2026-08-18 | VERIFY Gate | Reproducer test `integration::ecs_tests::test_animation_zero_frame_count_non_looping_does_not_panic` confirmed passing (`cargo test -p tiles_tools --all-features --test integration_tests -- integration::ecs_tests::test_animation_zero_frame_count_non_looping_does_not_panic`: 1 passed; 0 failed) against current source; fix in `src/ecs/components.rs` confirmed present at line 552 (`self.current_frame = self.frame_count.saturating_sub( 1 );`), with the 3-field `Fix(BUG-345)`/`Root cause`/`Pitfall` comment at lines 539-551 matching the report's claimed After block. state: Unverified -> Verified |
| 2026-08-18 | re-verified | Independent second Tier 2 Dual-Role Self-Check (separate session, task-scoped to BUG-343/344/345 specifically). Re-confirmed source fix (`saturating_sub` at `src/ecs/components.rs:552`, matching root cause exactly) and reproducer test (`Animation::new(0, 0.1)`, `looping=false`, `update(0.2)` -- genuinely underflows pre-fix, clamps to 0 post-fix) directly; full-crate `cargo nextest run -p tiles_tools --all-features` (detached via `longrun`) 272/272 passed. Adversarial pass caught an MRE portability defect (check 203/205) the prior pass's D2 row missed: Verify Command hardcoded `cd /home/user1/pro/lib/yrd_gamedev/cgtools`, an absolute per-user path -- fixed by removing it and adding the `**Known MRE limitation**` disclosure (BUG-132 precedent). See `## Verification Record`. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Completeness (101,102,103,104,108) | — | 🟢 | — | — |
| D2 | MRE Validity & Reproducibility (203,204,205,206) | 🔴 | 🟢 | Non-`/tmp/`-path MRE is a valid exception (crate-internal test, precedent BUG-227), but Verify Command still hardcoded `cd /home/user1/pro/lib/yrd_gamedev/cgtools` — an absolute per-user path (checklist 203/205), missed by the first pass above | Removed the hardcoded path; added the `**Known MRE limitation (check 203/205)**` disclosure (BUG-132 precedent). Re-confirmed via `cargo nextest run -p tiles_tools --all-features` (detached via `longrun`): 272/272 passed, including this test |
| D3 | Cross-Reference Integrity (301-306) | — | 🟢 | Evidence Table Hypothesis column used bare H-IDs (H1-H5) without state symbols (checklist 304) | Added `✅` annotation to all 6 Evidence Table rows, matching BUG-114 precedent |
| D4 | Root Cause Quality (401,402,403) | — | 🟢 | — | — |
| D5 | Execution Scope (107) | — | 🟢 | — | — |
| D6 | Crate Scope Unity (501) | — | 🟢 | — | — |
| D7 | Crate Locality (502) | — | 🟢 | — | — |
| D8 | Crate Single Responsibility (503) | — | 🟢 | — | — |
| **Total** | | — | 🟢 | 2 issues (1 per pass) | 2 fixes |

**Reproduced:** YES — `cargo test -p tiles_tools --all-features --test integration_tests -- integration::ecs_tests::test_animation_zero_frame_count_non_looping_does_not_panic`, exit 0, 2026-08-18. Independently re-confirmed via full-crate `cargo nextest run -p tiles_tools --all-features` (detached via `longrun`): 272/272 passed, including this test.

## Refs: src/

- `src/ecs/components.rs` — changed `Animation::update`'s non-looping-branch subtraction to `saturating_sub(1)`, nested inside the same `while` loop as the pre-existing BUG-132 fix but on a different line, addressing a distinct failure mode

## Refs: tests/

- `tests/integration/ecs_tests.rs` — new reproducer test `test_animation_zero_frame_count_non_looping_does_not_panic`: `Animation::new(0, 0.1)` with `looping = false`, `.update(0.2)` must not panic and must leave `current_frame == 0`, `playing == false`
