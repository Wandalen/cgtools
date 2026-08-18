# BUG-313: `sprite_animation` example's frame-index modulus uses `sprite_count - 1` instead of `sprite_count`, permanently skipping the sprite sheet's last frame

- **Severity:** Medium (active, visually-wrong behavior -- not latent -- but confined to 1
  non-critical example/demo crate, not library code)
- **state:** Verified
- **Affects:** the displayed animation frame in the `sprite_animation` example, which cycles
  through only 63 of the sprite sheet's 64 frames, forever
- **Component:** `examples/minwebgl/sprite_animation`
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/ (self)
- **verification_date:** 2026-08-18

## Symptom

`update_and_draw`'s per-frame closure computed `let amount = sprite_sheet.amount as f32 -
1.0;` (`63.0` for the 64-frame `rock.png` sheet, `sprite_sheet.amount = 64`), then used this
single `amount` value for two different purposes: `let frame = ( step / amount ).floor();`
(a pacing divisor -- how many `step` units each frame is held) and
`gl.vertex_attrib1f( 0, frame % amount )` (the wraparound modulus -- which frame index range
is valid). The second use is wrong: `x % 63.0` can only ever produce values in `[0, 63.0)`,
so frame index `63` (the sheet's 64th and last frame) can never be produced, no matter how
long the animation runs.

## Impact

**Who is affected:** anyone running (or visually inspecting the rendered output of) the
`sprite_animation` example.

**What breaks:** the animation loops through frames `0` to `62` forever, immediately
wrapping back to `0` at the point where frame `63` should display. One frame out of 64
(1.56% of the sheet) never renders, for the entire lifetime of the program. Purely cosmetic:
nothing panics, no data is corrupted, and it does not affect any library API consumed
elsewhere.

**Entity Scope:** `None` -- source-level call-site defect, not entity directory instances.

## How Discovered

Found during this session's workspace-wide bug-hunt pass, `examples/` review stage
(originally noted for follow-up while triaging pending Stage 2 items alongside BUG-311/312).
Reading `update_and_draw`'s frame-index computation in full and re-deriving the valid output
range of `frame % amount` by hand (`amount = 63.0`, so the modulus range is `[0, 63.0)`,
excluding the value `63.0` itself) against the sheet's actual frame count (`64`, `sprite_sheet.amount`)
showed the two numbers must differ by design (a modulus that includes every valid index of an
N-element sequence must equal `N`, not `N - 1`) but the code used the same variable, sourced
from `N - 1`, for both.

## Minimum Reproducible Example

**Verify Command** (inline unit test -- this crate is a `fn main()`-only binary with no
`tests/` directory; the buggy computation was pure local arithmetic with no library
dependency, so it was extracted into a small private `fn` and tested inline per this repo's
own `rulebook.md` § Test placement -- private-helper tests live in `#[cfg(test)] mod tests`
inside the source file):
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools/examples/minwebgl/sprite_animation
cargo test --bins
```
**Expected** (fixed): `test tests::test_sprite_frame_index_reaches_last_frame ... ok` -- 1
passed, 0 failed.

**Actual** (the raw pre-fix expression's real behavior): the test picks the exact `step`
value that makes `frame == 63.0` via closed-form arithmetic (`step = 63.0 * hold_ticks`,
where `hold_ticks = 63.0`), then asserts the fixed call (`sprite_frame_index( step,
hold_ticks, sprite_count )`, `sprite_count = 64.0`) reaches index `63` while the pre-fix
buggy call (passing `hold_ticks` as the modulus too, replicating the original single-`amount`
expression) evaluates to `0` at that same `step` -- both assertions executed and passed,
empirically confirming the exact frame at which the pre-fix code silently wraps early.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The modulus base must equal `sprite_count` for the full index range to be reachable | ✅ Root Cause | `x % N` produces values in `[0, N)` only; using `N - 1` excludes the value `N - 1` itself, the last valid index of an `N`-element sequence | E1 |
| H2 | The pre-fix code used one variable (`amount = count - 1`) for two unrelated roles: pacing divisor and wraparound modulus | ✅ Verified | Pre-fix: `let frame = ( step / amount ).floor(); frame % amount` -- both operations reference the same `amount` | E2 |
| H3 | This is a real, always-reproducing defect, not a rare edge case | ✅ Verified | The excluded index (`63`) is reached once per full cycle of the animation -- with `hold_ticks = 63.0` and `frame_rate = 24.0`, a full cycle is ~168 ticks, and the skip recurs every cycle indefinitely | E3 |
| H4 | Nothing in this crate could have caught this mechanically -- no test harness exists | ✅ Verified | `find examples/minwebgl/sprite_animation -iname '*test*'` returns nothing pre-fix | E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `examples/minwebgl/sprite_animation/src/main.rs` (pre-fix line 44, `frame % amount`) | `amount = 63.0`; `frame % 63.0` is mathematically confined to `[0, 63.0)`, never `63.0` | H1 |
| E2 | `examples/minwebgl/sprite_animation/src/main.rs` (pre-fix lines 38-44) | `let amount = sprite_sheet.amount as f32 - 1.0;` (line 38), then both `step / amount` (line 42) and `frame % amount` (line 44) reference it | H2 |
| E3 | `examples/minwebgl/sprite_animation/src/main.rs:26-30` (`sprite_sheet` literal) | `sprites_in_row: 8, sprite_width: 128, sprite_height: 128, amount: 64` -- confirms the sheet genuinely has 64 frames, index `63` is a real, valid frame being skipped, not an out-of-range value | H3 |
| E4 | Terminal output (this section, `find` command) | Empty output -- no `tests/` directory or test file anywhere in the crate, pre-fix | H4 |

## Root Cause

```
sprite_sheet.amount = 64  (true frame count)
  |
  |  examples/minwebgl/sprite_animation/src/main.rs (pre-fix)
  |
  +-- let amount = sprite_sheet.amount as f32 - 1.0;   // 63.0 -- ONE variable, TWO roles
        |
        +-- let frame = ( step / amount ).floor();      // role 1: pacing divisor (fine as 63.0)
        +-- gl.vertex_attrib1f( 0, frame % amount );     // role 2: wraparound modulus (WRONG -- must be 64.0)
```
`amount` (`63.0`) is a reasonable choice for "how many `step` units to hold each frame," but
an incorrect choice for "how many distinct frame indices exist" -- the modulus that must
equal the true sheet size (`64.0`) to make every index in `[0, 64)` reachable, including the
last one (`63`).

## Why Not Caught

`sprite_animation` had no `tests/` directory or test file (confirmed via `find`, E4) -- it is
a `fn main()`-only WebGL demo binary, verified only by running it in a browser and watching
the animation. A 1-frame-out-of-64 skip in a smoothly-looping animation produces no crash, no
visible glitch (the sheet still animates continuously, just one frame short each cycle), and
is easy to miss without frame-by-frame comparison against the source sprite sheet.

## Fix Location

`examples/minwebgl/sprite_animation/src/main.rs:35-47` (pre-fix), now split into
`hold_ticks`/`sprite_count` plus an extracted `sprite_frame_index()` function
(`main.rs:69-73`):

```rust
// Before:
let mut step = 0.0;
let frame_rate = 24.0;
let amount = sprite_sheet.amount as f32 - 1.0;

move | _ |
{
  let frame = ( step / amount ).floor();
  gl.vertex_attrib1f( 0, frame % amount );
  ...
}

// After:
let mut step = 0.0;
let frame_rate = 24.0;
let hold_ticks = sprite_sheet.amount as f32 - 1.0;
let sprite_count = sprite_sheet.amount as f32;

move | _ |
{
  gl.vertex_attrib1f( 0, sprite_frame_index( step, hold_ticks, sprite_count ) );
  ...
}

fn sprite_frame_index( step : f32, hold_ticks : f32, sprite_count : f32 ) -> f32
{
  let frame = ( step / hold_ticks ).floor();
  frame % sprite_count
}
```
Source comment (`Fix(BUG-313)`/`Root cause`/`Pitfall`) added immediately above the call site
(`main.rs:43-48`).

**`examples/minwebgl/sprite_animation/src/main.rs:76-136`** (new `#[cfg(test)] mod tests`,
inline per this repo's own `rulebook.md` § Test placement -- the buggy logic was a private
helper with no library to test against, unlike BUG-311/312's example-crate findings):
`test_sprite_frame_index_reaches_last_frame` asserts the fixed `sprite_frame_index` call
reaches index `63` at the exact `step` where the pre-fix expression (replicated by passing
`hold_ticks` as the modulus) evaluates to `0` instead.

## Prevention

Detection pattern for this exact shape (a single variable derived as `count - 1` used as both
a divisor and a modulus base in the same closure) is not reliably grep-able -- it is a
semantic conflation, not a syntactic one. The general principle: whenever a value is computed
as `N - 1` and used as a modulus/range bound, check whether the modulus is meant to cover all
`N` valid indices (in which case it must be `N`, not `N - 1`) or is genuinely bounding a
"maximum index" context (where `N - 1` is correct, e.g. `array[i.min(len - 1)]`).

**Pitfall:** a divisor controlling *pacing* (how often something advances) and a modulus
controlling *range* (which values are valid) are different quantities even when they start
from the same source count -- collapsing them into one shared variable silently breaks the
range as soon as the pacing choice (`count - 1`, chosen for its own unrelated reason) diverges
from the true range size (`count`).

## Generalized Version

**Broken assumption:** a single variable can correctly serve as both a rate-of-advance
divisor and a wraparound-range modulus, because both are "derived from the frame count."

Fails whenever:
1. A value is used as the divisor in a `(x / v).floor()` pacing expression, AND
2. The SAME value is also used as the modulus base in a `% v` wraparound expression, AND
3. The value is not exactly equal to the true size of the range being wrapped (`count`, not
   `count - 1` or any other derived quantity)

**Detection invariant:**
```
for every `frame % modulus_base` wraparound expression over an N-element sequence:
  modulus_base must equal N exactly, independent of any other divisor used to compute `frame`
```
First instance of this specific "one variable, two roles" modulus defect in this workspace
this session -- distinct from BUG-311 (a degrees/radians unit confusion) and BUG-312 (a
spurious caller-side scaling factor); this one is a range-size/pacing-divisor conflation, a
different failure shape from either angle-related finding.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Found during this session's workspace-wide bug-hunt task, `examples/` review stage, by re-deriving the valid output range of `frame % amount` against the sheet's true frame count |
| 2026-08-18 | fix_applied | Split `amount` into `hold_ticks` (unchanged pacing divisor) and `sprite_count` (true wraparound range); extracted `sprite_frame_index()` |
| 2026-08-18 | verified | `tests::test_sprite_frame_index_reaches_last_frame` (bug_reproducer) passes; native and wasm32 clippy (`-D warnings`) clean |

## Refs: src/

- `examples/minwebgl/sprite_animation/src/main.rs` — split `amount` into `hold_ticks`/`sprite_count`, extracted `sprite_frame_index()`, added inline `#[cfg(test)] mod tests`

## Refs: tests/

- `examples/minwebgl/sprite_animation/src/main.rs` — added `tests::test_sprite_frame_index_reaches_last_frame` (bug_reproducer, inline)

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | Adversarial pass mechanically confirmed all 12+2 headers present (`grep -n "^## "`) and re-read each body for substantive, non-generic content -- none found thin | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Adversarial pass re-ran the documented verify command fresh (exit 0, 1 passed, matching "Expected"); separately re-derived the closed-form arithmetic by hand (`step=3969.0` -> `frame=63.0` -> `correct=63.0`, `buggy=0.0`) and confirmed it matches the "Actual" section's claims exactly | — |
| D3 | Cross-Reference Integrity | — | 🟢 | `grep -rln "BUG-313" --include=*.rs --include=*.md .`: exactly 2 files (report + `main.rs`, which holds both the fix comment and the test backreference since fix+test are co-located in this crate), matching both `## Refs:` entries | — |
| D4 | Root Cause Quality | 🟠 | 🟢 | Adversarial pass discovered mid-gate that a concurrent actor had committed this session's working tree (commit `254b7812`, own unrelated work bundled in) -- re-verified all cited pre-fix line numbers against `git show 254b7812` instead of a live diff: E1 (line 44) and E3 (lines 26-30) accurate, but caught 3 real citation defects: E2's stated range "38-42" excluded line 44 referenced in its own description; Fix Location's "34-51" didn't match the actual pre-fix hunk range; Fix Location's "76-133" didn't match `mod tests`'s actual closing line (136) | Corrected E2 to "38-44", Fix Location's pre-fix range to "35-47" (matching the diff hunk exactly), and the `mod tests` range to "76-136" |
| D5 | Execution Scope | — | 🟢 | `git show 254b7812 -- main.rs` re-read in full: this bug's own diff hunk is exactly the intended change (comment block, `hold_ticks`/`sprite_count` split, extracted fn, `mod tests`) -- no scope creep, independent of whatever else the bundling commit contains | — |
| D6 | Crate Scope Unity | — | 🟢 | `grep -rn "amount.*- 1\.0\|\.amount as f32 - 1" examples/` (excluding `sprite_animation` itself) re-confirmed zero sibling instances of this exact pattern elsewhere -- correctly scoped as one isolated report | — |
| D7 | Crate Locality | — | 🟢 | `git status --porcelain` on all touched paths re-checked: only the report file itself remains untracked (created after the concurrent actor's commit); the fix/test file was already absorbed into their commit intact, content-verified via D4's `git show`. Live highest ID re-verified via unbounded `find`: 313 (this report itself) -- no collision | — |
| D8 | Crate Single Responsibility | — | 🟢 | Re-read the full current file (136 lines) end to end for any other latent defect near the fix -- none found; fix stayed scoped to exactly the one reported modulus defect | — |
| **Total** | | — | 🟢 | 0 open | 3/3 |

**Reproduced:** YES — `tests::test_sprite_frame_index_reaches_last_frame` exit 0 (1 passed),
re-run fresh with the exact documented command. `cargo clippy --all-targets --all-features
-- -D warnings` clean both natively and for `--target wasm32-unknown-unknown` (the crate's
actual runtime target). Note: this session's concurrent actor committed the working tree
mid-gate (commit `254b7812`, unrelated bundled work) -- this bug's own fix/test content was
verified intact and unaltered via `git show`, consistent with this repo's previously
documented "sudden clean status = committed, not lost" pattern.
