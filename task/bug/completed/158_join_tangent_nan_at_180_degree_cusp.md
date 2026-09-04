# BUG-158: The join/body tangent computation shared by 5 `d2` shaders produces `NaN` at a ~180-degree cusp

- **Severity:** High (silent visual corruption, not a crash -- `gl_Position` becomes `NaN` for
  the affected joint/segment, propagating garbage or discarded geometry into the rendered line
  with no error signal; reachable via ordinary API usage any time a line's path folds back on
  itself, not an adversarial/malformed-input edge case)
- **state:** Completed
- **Affects:** `join_miter.vert`, `join_bevel.vert`, `join_round.vert`, `body.vert`,
  `body_terminal.vert` -- every `d2` line-rendering shader that computes a join/segment tangent
  from 3 consecutive path points, for any 3 consecutive points forming a ~180-degree cusp
  (the outgoing direction from the middle point is ~exactly opposite the incoming direction)
- **Component:** `module/helper/line_tools` (`src/d2/shaders/*.vert`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None (independent of BUG-154, filed in an earlier `line_tools` review batch
  covering unrelated doc-comment accuracy).

## Symptom

```rust
// Rust port of the shared tangent formula (see "Minimum Reproducible Example" -- this crate has
// no shader-execution test harness, so the regression is captured via a line-for-line Rust
// mirror of the GLSL rather than a live render).
let point_a = [ 0.0, 0.0 ];
let point_b = [ 1.0, 0.0 ];
let point_c = [ -1.0, 0.0 ]; // path reverses ~180 degrees at point_b -- a cusp

// Pre-fix formula: normalize( normalize(pointC-pointB) + normalize(pointB-pointA) )
// -> normalize( (-1,0) + (1,0) ) = normalize( (0,0) ) = (NaN, NaN)
```

## Impact

**Who is affected:** Any consumer of `d2::Line` whose path data can contain 3 consecutive points
forming (or nearly forming) a ~180-degree cusp -- plausible for hand-drawn input, animated paths
that reverse direction, or path-simplification/generation code that doesn't explicitly forbid
zero-width folds. All 5 `d2` vertex shaders share the identical vulnerable formula (copy-pasted,
no shared GLSL header in this crate), so both segment bodies (`body.vert`/`body_terminal.vert`)
and all 3 join styles (`join_miter.vert`/`join_bevel.vert`/`join_round.vert`) are affected.

**What breaks:** `tangent` feeds directly into `normal`, which every one of the 5 shaders uses to
compute `sigma` (bend direction), `offsetPoint`/`intersectionPoint` (corner placement), and in
`join_round.vert`'s case the final vertex position directly (`point = p2 + tangent * point.x +
normal * point.y`). A `NaN` `tangent` therefore corrupts `gl_Position` for every vertex of the
affected joint (or, for `body.vert`/`body_terminal.vert`, the affected segment's variable
corner) -- typically manifesting as a discarded or wildly out-of-bounds primitive, silently,
with no panic and no error signal.

**Magnitude:** Silent -- IEEE-754 NaN propagates through every subsequent arithmetic operation
and comparison (`normal.x - normToAB.x < 1e-6`-style guards elsewhere in `body.vert`/
`body_terminal.vert` cannot rescue it: any comparison against NaN is defined to be `false`, so
execution falls through to the branch that further propagates the NaN rather than one that
short-circuits it).

**Entity Scope:** None -- a code-level defect, not an operational-entity concern.

## How Discovered

Flagged by a background investigation (task #106) covering `pitfall/006_parallel_segment_
division_by_zero.md`'s previously-unresolved "explicit guard... was not conclusively identified"
note. Independently re-verified by hand-recomputing the NaN mechanism with concrete coordinates
and reading both `join_miter.vert` and `join_bevel.vert` directly. During fix implementation, a
direct grep (`grep -n "tangent = normalize( normalize" src/d2/shaders/*.vert`) found the
identical vulnerable line in 3 additional files (`join_round.vert`, `body.vert`,
`body_terminal.vert`) beyond the 2 originally scoped by task #106 -- broadening this bug's fix
scope to all 5 sites rather than treating the other 3 as separate bugs, since they share one
root cause (one formula, copy-pasted 5 times).

## Minimum Reproducible Example

```bash
cd module/helper/line_tools && cargo test -p line_tools --test tests webgl::join_tangent::guarded_tangent_stays_finite_at_a_cusp_bug_158 2>&1 | tail -10
```

**Expected** (post-fix):
```
test webgl::join_tangent::guarded_tangent_stays_finite_at_a_cusp_bug_158 ... ok
```

**Actual** (pre-fix -- confirmed via in-place revert-test-restore against the guard itself):
```
thread 'webgl::join_tangent::guarded_tangent_stays_finite_at_a_cusp_bug_158' panicked at module/helper/line_tools/tests/webgl/join_tangent.rs:76:3:
guarded_tangent must never produce NaN/inf at a cusp, got [NaN, NaN]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 14 filtered out
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/line_tools && cargo test -p line_tools --test tests webgl::join_tangent::
# 4 "ok" = fixed; any NaN/inf assertion failure = bug present
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The shared `tangent = normalize( normalize(pointC-pointB) + normalize(pointB-pointA) )` formula produces `NaN` when the two summed unit vectors are exactly (or near-exactly) opposite, and this is reachable from ordinary path data with no CPU-side guard. | ✅ Root Cause | Hand-recomputation with concrete cusp coordinates confirmed `normalize((0,0))` is `NaN` in IEEE-754 (0/0); grep of `line.rs`/`lib.rs` found only near-coincident-point `EPSILON=1e-8` distance guards, never a direction/collinearity check. | E1, E2, E3 |
| H2 | The bug is isolated to `join_miter.vert` (task #106's original scope). | ❌ Rejected | `grep -n "tangent = normalize( normalize" src/d2/shaders/*.vert` returned 5 matches, not 1 -- `join_bevel.vert`, `join_round.vert`, `body.vert`, `body_terminal.vert` all carry the byte-for-byte identical vulnerable line. | E4 |
| H3 | `body.vert`'s existing near-parallel guard (`abs(normal.x - normToAB.x) < 1e-6 && ...`, line 95 pre-fix) already rescues the NaN case by routing around the bad `lineIntersection` call. | ❌ Falsified | IEEE-754 defines every comparison against `NaN` (including `<`) as `false` -- `abs(NaN - x) < 1e-6` is `false` regardless of `x`, so the guard's condition fails and execution falls into the `else` branch, which calls `lineIntersection` with the NaN `normal` directly, propagating rather than rescuing it. | E5 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | Hand computation | `pointA=(0,0)`, `pointB=(1,0)`, `pointC=(-1,0)`: `dirIn=(1,0)`, `dirOut=(-1,0)`, sum=`(0,0)`, `normalize((0,0))` = `(0/0, 0/0)` = `(NaN, NaN)` per IEEE-754. | H1 ✅ |
| E2 | `src/line.rs:180-181`, `src/lib.rs:101,133` (unedited) | Only near-coincident-point checks exist (`EPSILON=1e-8` distance comparisons) -- no angle/direction/collinearity guard anywhere in the CPU-side API. | H1 ✅ |
| E3 | `tests/webgl/join_tangent.rs::guarded_tangent_stays_finite_at_a_cusp_bug_158` (real run) | Captured real pre-fix failure: `guarded_tangent must never produce NaN/inf at a cusp, got [NaN, NaN]`. | H1 ✅ |
| E4 | `grep -n "tangent = normalize( normalize" src/d2/shaders/*.vert` (pre-fix) | 5 matches: `join_round.vert:46`, `join_bevel.vert:42`, `body.vert:59`, `join_miter.vert:42`, `body_terminal.vert:42`. | H2 ❌ |
| E5 | `src/d2/shaders/body_terminal.vert:97` (pre-fix, unedited structure) / `body.vert:95` | `if( abs( normal.x - normToAB.x ) < 1e-6 && ... )` -- a NaN `normal` makes both operands of `<` involve NaN, so the comparison is `false` per IEEE-754, routing into the `else` branch (`lineIntersection(p1, normal, ...)`) which propagates the NaN into `intersectionPoint`/`offsetPoint`/`gl_Position` instead of avoiding it. | H3 ❌ |

## Root Cause

```
5 x .vert files (pre-fix, byte-for-byte identical line)
  vec2 tangent = normalize( normalize( pointC - pointB ) + normalize( pointB - pointA ) );
  // No guard: if the two normalized directions are opposite, their sum is (0,0) and
  // normalize((0,0)) is NaN (0/0) in GLSL -- propagates into normal/sigma/offsetPoint/gl_Position.
```

One formula, copy-pasted into 5 shader files (this crate has no shared GLSL header/include
mechanism), all missing the same guard.

## Why Not Caught

No CPU-side twin of this formula exists anywhere in `src/` to unit-test, and this crate has no
shader-execution test harness (`tests/webgl/*.rs` exercises the CPU-side `Line` API only, never
a live GPU/WebGL context) -- so the NaN was reachable only via live rendering with a cusp in the
input path, which no existing test constructs. `pitfall/006`'s own pre-existing doc had already
flagged this as an open, unconfirmed risk ("an explicit guard... was not conclusively
identified... Confirm against current shader source"), but its "In Scope" also named only
`join_miter.vert`, missing the other 4 files sharing the identical defect.

## Fix Location

All 5 files in `module/helper/line_tools/src/d2/shaders/`: `join_miter.vert`, `join_bevel.vert`,
`join_round.vert`, `body.vert`, `body_terminal.vert`.

```glsl
// before (all 5 files)
vec2 tangent = normalize( normalize( pointC - pointB ) + normalize( pointB - pointA ) );
vec2 normal = vec2( -tangent.y, tangent.x );

// after (all 5 files; body.vert uses p0/p1/p2 in place of pointA/pointB/pointC)
vec2 dirIn = normalize( pointB - pointA );
vec2 dirOut = normalize( pointC - pointB );
vec2 tangentSum = dirOut + dirIn;
vec2 tangent = dot( tangentSum, tangentSum ) > 1e-12 ? normalize( tangentSum ) : dirIn;
vec2 normal = vec2( -tangent.y, tangent.x );
```

Outside the degenerate branch this is the exact same sequence of operations as the original
formula (`dirOut + dirIn` is the same sum, same operands, just named) -- zero behavior change for
any non-cusp input, confirmed by `guarded_tangent_matches_unguarded_formula_for_an_ordinary_bend`
and `guarded_tangent_uses_the_real_formula_for_a_near_cusp_not_just_an_exact_one`.

## Prevention

Added `tests/webgl/join_tangent.rs` (new file, `bug_reproducer(BUG-158)`): a line-for-line Rust
port of the guarded formula plus 4 tests -- exact-cusp NaN-free (2 independent cusp
orientations), ordinary-bend bit-for-bit equivalence with the unguarded formula, and a
near-but-not-exact cusp confirming the guard's `1e-12` threshold doesn't misfire on a merely-sharp
non-degenerate bend.

## Pitfall

A `vec2` sum whose length can legitimately reach exactly zero must never be fed to `normalize`
without a guard -- GLSL has no defined "safe normalize" and silently produces `NaN`, not a panic
or a zero vector. When the same formula is duplicated across multiple files with no shared
header (as in this crate), a NaN-producing bug found in one file is a strong signal to grep for
the identical text in sibling files before scoping the fix narrowly -- 3 of this bug's 5 affected
files were found only via that grep, not via the original investigation that flagged the bug.

## Generalized Version

**Broken assumption:** "the two segment directions at a joint always have a well-defined average
direction." False for a ~180-degree cusp -- geometrically the miter/bevel/round join direction is
genuinely undefined at an exact reversal, so *some* fallback must be chosen; the pre-fix code
implicitly assumed this case never occurs rather than choosing one.

**Confirmed general rule:** before trusting a `normalize()` call in shader code (or any
division-by-length operation) is safe, check whether its input's length can reach exactly zero
under valid (not just malformed) input -- and if the code exists in more than one file with no
shared header, grep for the exact formula text across the whole shader directory before assuming
a single-file fix is complete.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Flagged by task #106's investigation of `pitfall/006`; independently re-verified via hand-recomputation with concrete coordinates before filing. Grep during fix implementation broadened scope from 2 to 5 affected files. |
| 2026-08-16 | fixed | Added a squared-length guard (`dot(tangentSum,tangentSum) > 1e-12`) before the final `normalize` in all 5 `.vert` files, falling back to `dirIn` (already unit-length) when the sum collapses. Updated `docs/pitfall/006_parallel_segment_division_by_zero.md` to record the resolution and corrected file scope. |
| 2026-08-16 | verified | Added `tests/webgl/join_tangent.rs` (4 tests) via in-place revert-test-restore against the Rust port's own guard: captured the real pre-fix NaN failure, restored, confirmed passing. Full crate suite (93 tests) + `cargo clippy -p line_tools --all-targets --all-features -- -D warnings` clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote the Rust port line-for-line against the actual GLSL guard and verified it passes; adversarial pass performed a real in-place revert-test-restore on the Rust port's own guard, capturing the actual `[NaN, NaN]` failure before restoring, plus a separate line-by-line transcription check confirming the Rust port matches the GLSL exactly (dirIn/dirOut/tangentSum/threshold/branches all in the same order). | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Independent of BUG-154 (unrelated doc-comment fix in the same crate, different code path) -- no cross-dependency. `pitfall/006` doc updated to reflect resolution and corrected 5-file scope. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by hand-recomputation with concrete coordinates plus a direct IEEE-754-based falsification of the "existing guard rescues it" hypothesis (H3). | — |
| D5 | Execution Scope | 🟢 | 🟢 | Scope was independently re-verified via `grep` rather than trusting the originally-filed task's 2-file scope -- found and fixed all 5 real sites, not just the 2 hypothesized ones; post-fix grep confirms zero remaining unguarded instances. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `line_tools` src (5 `.vert` files) + test + doc + bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix is a 3-line insertion + 1-line change per file, identical shape at all 5 sites; no signature/API change. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public Rust surface (shader files aren't part of the crate's public API); existing documented-but-unresolved pitfall now actually resolved. | — |

**Reproduced:** YES -- `guarded_tangent_stays_finite_at_a_cusp_bug_158` was confirmed to fail with
the exact predicted `[NaN, NaN]` when the Rust port's own guard was temporarily reverted to the
unguarded formula; restoring the guard returns the test to passing. Full crate suite (93 tests)
+ `cargo clippy -p line_tools --all-targets --all-features -- -D warnings` clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/line_tools/src/d2/shaders/join_miter.vert` | Guarded `tangent` computation (full `Fix(BUG-158)` root cause/pitfall comment). |
| `module/helper/line_tools/src/d2/shaders/join_bevel.vert` | Guarded `tangent` computation (cross-references join_miter.vert). |
| `module/helper/line_tools/src/d2/shaders/join_round.vert` | Guarded `tangent` computation (cross-references join_miter.vert). |
| `module/helper/line_tools/src/d2/shaders/body.vert` | Guarded `tangent` computation, `p0`/`p1`/`p2` naming (cross-references join_miter.vert). |
| `module/helper/line_tools/src/d2/shaders/body_terminal.vert` | Guarded `tangent` computation (cross-references join_miter.vert). |
| `module/helper/line_tools/docs/pitfall/006_parallel_segment_division_by_zero.md` | Updated to record the resolution and corrected 5-file scope (was 1-file). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/line_tools/tests/webgl/join_tangent.rs` | New file: Rust port of the guarded formula + 4 tests (`bug_reproducer(BUG-158)`). |
| `module/helper/line_tools/tests/webgl/mod.rs` | Registered `mod join_tangent;`. |
| `module/helper/line_tools/tests/webgl/readme.md` | Added Responsibility Table row for `join_tangent.rs`. |
