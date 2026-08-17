# BUG-239: `Transform::to_mat3()`'s Y-skew (`skew[1]`) term has an inverted sign, diverging from the SVG backend's own skew convention

- **Severity:** Low (latent — zero in-tree callers currently set `Transform::skew` to a
  non-default value — but a live public-API defect in code consumed by all 3 GPU backends,
  natively testable with no GPU context)
- **state:** Completed
- **Affects:** Any current or future caller that sets `Transform::skew[1]` (Y-skew) to a
  non-zero value and renders through `adapter-native`, `adapter-webgl`, or `adapter-webgpu`
  (`to_mat3()` is consumed at 8 call sites in `webgl.rs`, 1 in `native.rs`, 1 in `webgpu.rs`).
- **Component:** `module/helper/tilemap_renderer` (`src/types.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Related Bugs:** Found during this session's `tilemap_renderer` crate scout (task #173).
  Checked against the crate's 5 existing bugs (BUG-153, 204, 209, 210, 211) — none touch
  `to_mat3()`, `skew`, or the GPU-backend transform-matrix construction path; unrelated, distinct
  defect.

## Symptom

```rust
// pre-fix -- src/types.rs, Transform::to_mat3()
let m00 = ( cos_r + sin_r * sky ) * sx;
let m10 = ( sin_r - cos_r * sky ) * sx;
let m01 = ( cos_r * skx - sin_r ) * sy;
let m11 = ( sin_r * skx + cos_r ) * sy;
```

With `rotation = 0`, `scale = [1,1]`, `skew = [0, π/4]` (Y-skew only), the x-basis column
(`m00`, `m10`) evaluated to `(1.0, -1.0)` — but the real SVG `skewY(45°)` matrix
(`x'=x, y'=y+x*tan(a)`) applied to the same unit point `(1,0)` gives `(1.0, +1.0)`. The sign on
the `sky` (`skew[1].tan()`) term was inverted; `skx` (`skew[0]`, "skewX") had no such error.

## Impact

**Who is affected:** Any caller that sets `Transform::skew[1]` to a non-zero value and renders
through any GPU backend (native/webgl/webgpu) — currently none, confirmed via exhaustive
workspace-wide grep (`module/`, `examples/`): every real `Transform` construction that sets
`skew` at all (`tilemap_scene/src/compile/frame.rs` x3, `tilemap_scene/src/compile/viewport.rs`
x1) hardcodes `skew: [0.0, 0.0]`.

**What breaks:** the exact same `Transform` value would render Y-skew mirrored on one GPU
backend relative to the SVG backend — `skew[1]`'s own doc comment and `svg.rs`'s
`transform_to_svg_local` (which passes `skew[1]` straight into a real SVG `skewY()` op with no
sign flip) both define `skew[1]` against the unnegated real-SVG convention; `to_mat3()` diverged
from that convention silently.

**Magnitude:** 1 function (`Transform::to_mat3`), single shared root cause; 10 call sites across
3 backend files all inherit the same fix with no changes of their own (they consume the returned
`[f32; 9]` opaquely).

**Entity Scope:** None — a code-level defect.

## How Discovered

This session's `tilemap_renderer` crate scout (task #173), reading `src/types.rs` in full.
`to_mat3()`'s hand-derived combined rotation/scale/skew formula had no existing test exercising
`skew` at all (every pre-existing `to_mat3_*` test in `types_test.rs` uses
`..Default::default()`). Cross-checked the formula against `svg.rs`'s `transform_to_svg_local` /
`transform_to_svg_static` (already-existing, unambiguous code that emits real SVG
`skewX()`/`skewY()` strings) as independent ground truth, then confirmed the sign error via
isolated single-axis numeric evaluation (rotation=0, scale=1, only one `skew` field nonzero at a
time — this removes all composition-order ambiguity, after two earlier hand-algebra attempts at
the combined formula produced self-contradictory results and were discarded).

## Minimum Reproducible Example

```rust
// module/helper/tilemap_renderer/src/types.rs -- Transform::to_mat3, pre-fix
let t = Transform { skew : [ 0.0, core::f32::consts::FRAC_PI_4 ], ..Default::default() };
let m = t.to_mat3();
// pre-fix:  m[1] == -0.99999994  (mirrored -- wrong sign)
// post-fix: m[1] == 1.0          (matches real SVG skewY(45deg) applied to (1,0): y'=y+x*tan(a)=1)
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/tilemap_renderer && cargo nextest run --all-features --test types_test -E 'test(to_mat3_skew_y_matches_svg_skew_y_convention) + test(to_mat3_skew_x_matches_svg_skew_x_convention)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `to_mat3()`'s `m00`/`m10` formula has an inverted sign on the `sky` (`skew[1]`) term relative to the real SVG `skewY(a)` matrix. | ✅ Root Cause | Isolated single-point check (rotation=0, scale=1, `skew=[0, π/4]`): code's `m10` = `-0.99999994`; real SVG `skewY(45°)` on `(1,0)` gives `y'=+0.99999994`. Opposite sign. | E1, E3 |
| H2 | `to_mat3()`'s `m01`/`m11` formula (`skx` / `skew[0]`) is already correct and needs no change. | ✅ Confirmed | Same isolated method with `skew=[π/4, 0]` (only `skew[0]` nonzero): code's y-basis column already matches real SVG `skewX(45°)` applied to `(0,1)` (`x'=+1`). | E2, E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/tilemap_renderer/src/types.rs`, pre-fix `to_mat3()` (direct read) + isolated numeric check (`skew=[0,π/4]`, rotation=0, scale=1) | `m10` evaluates to `-0.99999994`, opposite sign from real SVG `skewY(45°)` applied to `(1,0)`. | H1 ✅ |
| E2 | Same file/method, isolated numeric check (`skew=[π/4,0]`, rotation=0, scale=1) | `m01`/`m11`-derived y-basis column already matches real SVG `skewX(45°)` applied to `(0,1)`. | H2 ✅ |
| E3 | `module/helper/tilemap_renderer/src/adapters/svg.rs`, `transform_to_svg_local` (direct read, lines 423-456) | Passes `t.skew[0]`/`t.skew[1]` straight into real SVG `skewX()`/`skewY()` ops with no sign flip — establishes the unnegated ground-truth convention `to_mat3()` (also Y-up, no viewport flip) must match. | H1 ✅, H2 ✅ |

## Root Cause

`to_mat3()`'s x-basis column (`m00`, `m10`) was computed as `(cos_r + sin_r*sky, sin_r -
cos_r*sky)` — the `sky` (`skew[1].tan()`, "skewY") term's sign is inverted relative to the real
SVG `skewY(a)` matrix (`x'=x, y'=y+x*tan(a)`) that `Transform::skew`'s own doc comment and
`svg.rs`'s `transform_to_svg_local` both define `skew[1]` against. `skx`'s (`skew[0]`, "skewX")
formula in `m01`/`m11` had no corresponding error.

## Why Not Caught

No test in `types_test.rs` ever set `skew` to a non-default value — every pre-existing
`to_mat3_*` test relies on `..Default::default()`, which zeroes `skew` — and an exhaustive
workspace-wide grep (`module/`, `examples/`) confirmed zero real callers anywhere set
`Transform::skew` to a non-default value either, so this path was unexercised by both tests and
every real caller.

## Fix Location

`module/helper/tilemap_renderer/src/types.rs`, `Transform::to_mat3()`: flipped the `sky`
operators in `m00`/`m10` (`cos_r + sin_r * sky` → `cos_r - sin_r * sky`; `sin_r - cos_r * sky` →
`sin_r + cos_r * sky`). `m01`/`m11` (the `skx` formula) is unchanged.

## Prevention

2 new tests added, `module/helper/tilemap_renderer/tests/types_test.rs`:
`to_mat3_skew_y_matches_svg_skew_y_convention` (isolates `skew[1]`, asserts the x-basis column
matches real SVG `skewY(45°)` applied to `(1,0)`) and `to_mat3_skew_x_matches_svg_skew_x_convention`
(isolates `skew[0]`, regression guard proving the already-correct path stayed undisturbed by the
fix, asserts the y-basis column matches real SVG `skewX(45°)` applied to `(0,1)`). All 5
pre-existing `to_mat3_*` tests re-verified passing unchanged.

## Pitfall

`skew[0]` and `skew[1]` are NOT symmetric in a skewX/skewY matrix (skewX shifts x by y's amount;
skewY shifts y by x's amount) — a wrong sign on one field can hide indefinitely behind passing
tests that only ever exercise the other fields or the identity/rotation/scale-only cases,
especially when zero real callers exercise the field either. Verify each input's sign against an
independent, authoritative single-axis case (isolate one field, zero the rest, compare against a
ground-truth source), not just the combined matrix's overall shape — hand-algebra on the full
composed formula converged on self-contradictory answers twice before the isolated method
resolved it cleanly.

## Generalized Version

**Broken assumption:** "a hand-derived combined transform matrix that looks structurally
plausible (correct shape, right terms present) has each individual term's sign correct too."

**Confirmed general rule:** Each independent input to a hand-derived combined transform matrix
must be verified against an authoritative single-axis case (all other inputs at their
identity/zero value) before trusting the combined formula — structural plausibility and per-term
sign correctness are independent properties, and nothing catches the latter unless every field is
exercised in isolation at least once, whether by a test or by deliberate manual derivation.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed | Found via this session's `tilemap_renderer` crate scout (task #173) while reading `src/types.rs`; cross-checked against `svg.rs`'s already-correct `transform_to_svg_local` skew semantics as ground truth. |
| 2026-08-17 | fixed | `m00`/`m10`'s `sky` sign flipped; `m01`/`m11` (`skx`) left unchanged. 2 new regression tests added. |
| 2026-08-17 | verified | `cargo nextest run -p tilemap_renderer --all-features` (via `verb/test_only pkg::tilemap_renderer`, `longrun`-detached, log `module/-0012_longrun.log`): 150/150 passed, 0 skipped, including both new tests. `cargo clippy -p tilemap_renderer --all-targets --all-features -- -D warnings` (`longrun`-detached, log `module/-0013_longrun.log`): clean, exit 0. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | MRE isolates the exact single-axis case (`skew[1]` alone, rotation=0, scale=1) that exposes the sign inversion, distinct from every pre-existing `to_mat3` test (identity/translation/scale/rotation-only, none touching skew). | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Checked against the crate's 5 existing bugs (BUG-153/204/209/210/211) and 4 open tasks (#114/#198/#218/#221) — none touch `to_mat3`, `skew`, or the transform-matrix path; correctly filed as a new, unrelated ID. | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by an isolated single-point numeric check cross-validated against `svg.rs`'s independent, already-correct skew semantics (`transform_to_svg_local`), not merely a plausible-looking algebraic re-derivation — two earlier hand-algebra attempts at the combined formula produced self-contradictory results and were explicitly discarded in favor of the isolated numeric method. | — |
| D5 | Execution Scope | — | 🟢 | Fix confined to the 2 lines computing `m00`/`m10`; `m01`/`m11` (`skx` path) and every other field (rotation/scale/position/depth) deliberately left untouched and re-confirmed via the sibling skewX regression test plus all 5 pre-existing `to_mat3_*` tests still passing unchanged. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `tilemap_renderer` (`src/types.rs` + `tests/types_test.rs`); `to_mat3()`'s 10 call sites in `webgl.rs`/`native.rs`/`webgpu.rs` consume the returned `[f32; 9]` opaquely — no downstream crate changes needed. | — |

**Reproduced:** YES — pre-fix, `to_mat3_skew_y_matches_svg_skew_y_convention` fails (`m[1]` =
`-0.99999994`, not the asserted `+1.0` within tolerance); post-fix, both new tests pass.
2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/src/types.rs` | `Transform::to_mat3()`: `m00`/`m10`'s `sky` sign flipped (`Fix(BUG-239)` comment block). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/tests/types_test.rs` | Added `to_mat3_skew_y_matches_svg_skew_y_convention`, `to_mat3_skew_x_matches_svg_skew_x_convention`. |
