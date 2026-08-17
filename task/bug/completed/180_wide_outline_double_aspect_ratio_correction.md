# BUG-180: `WideOutlinePass`'s JFA step double-applies aspect-ratio correction, stretching outlines on non-square canvases

- **Severity:** Medium (visual-only defect -- no crash, no data loss, but the outline's search
  radius, and therefore its rendered shape, is anisotropic -- wider than tall -- on any
  non-square canvas)
- **state:** Completed
- **Affects:** Every caller of `renderer::webgl::post_processing::outline::wide_outline::WideOutlinePass`
  rendered to a non-square canvas (i.e. essentially every real caller -- browser canvases are
  rarely exactly square).
- **Component:** `module/helper/renderer` (`src/webgl/post_processing/outline/wide_outline.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** Same `wide_outline` pass as BUG-179 (`outlineThickness` uniform never wired
  up) -- an independent defect in the same file, already fixed. Also candidate BUG-181
  (`jfa_init.frag`'s silhouette detection only checks the red channel) and BUG-182 (`outline.frag`'s
  sentinel check tests the wrong value) in the same shader trio, not yet fixed. A structurally
  identical `aspect_ratio` pattern also exists in the self-contained walkthrough example
  `examples/minwebgl/outline/src/main.rs` (its own `jfa_step.frag` uses a different, `.x`-only
  step-size scheme, so it is NOT necessarily the identical defect and was deliberately left
  untouched here as out of scope -- see Prevention).

## Symptom

```rust
// pre-fix -- wide_outline.rs, jfa_step_pass
let aspect_ratio = self.width as f32 / self.height as f32;
let step_size =  self.outline_thickness / ( 2.0_f32 ).powf( i as f32 );
let step_size = [ step_size * aspect_ratio, step_size ];
```

`jfa_step_pass` scaled the JFA step's horizontal jump distance (`stepSize.x`) by
`width / height` before uploading it as the `stepSize` uniform. But `jfa_step.frag` already
converts `stepSize` from pixels to normalized UV space *per axis*
(`ceil( dir * stepSize ) / resolution`, dividing the x component by `resolution.x` and the y
component by `resolution.y` independently) -- which alone already produces a uniform real-pixel
jump on a non-square canvas, with no extra correction needed. Applying both meant the real
per-axis pixel jump worked out to `step_size * aspect_ratio` horizontally vs. just `step_size`
vertically.

## Impact

**Who is affected:** Any caller rendering `WideOutlinePass` to a non-square canvas -- effectively
every real caller, since browser canvases are almost never exactly square.

**What breaks:** Purely visual -- the JFA search radius (and therefore the rendered outline
shape) is stretched horizontally relative to vertically by a factor of `width / height`. On a
typical 16:9 canvas (`aspect_ratio ≈ 1.78`), the outline extends ~78% farther from the silhouette
horizontally than vertically -- an elliptical distortion of what should be a uniform ("circular")
outline band. No crash, no incorrect persisted data.

**Magnitude:** Every draw through `WideOutlinePass` on a non-square canvas is affected identically
-- the defect is in the single shared step-size computation, not any one call site. Invisible only
on an exactly-square canvas (`aspect_ratio == 1.0`, where the erroneous scaling factor is `1.0`
and has no effect).

**Entity Scope:** None -- a code-level defect.

## How Discovered

Pre-identified by task #98's review pass (this session) as "wide_outline double aspect-ratio
correction in JFA step." Confirmed by reading `jfa_step_pass`'s Rust-side step-size computation
alongside `jfa_step.frag`'s own per-axis `/ resolution` conversion, then algebraically deriving
the real per-axis pixel jump both formulas produce (see Evidence Table) -- the two divisions
compound rather than being alternatives.

## Minimum Reproducible Example

```python
# pre-fix, 1920x1080 canvas, outline_thickness = 64.0, step i = 0:
aspect = 1920.0 / 1080.0          # 1.7778
step_size_x = 64.0 * aspect       # 113.78 (Rust-side, pre-fix)
step_size_y = 64.0
# jfa_step.frag: offset = ceil(dir * step_size) / resolution ; real pixel jump = offset * resolution
real_jump_x = ceil(1.0 * step_size_x) / 1920.0 * 1920.0   # = 114.0
real_jump_y = ceil(1.0 * step_size_y) / 1080.0 * 1080.0   # = 64.0
# 114.0 != 64.0 -- anisotropic; post-fix both equal 64.0 (see Evidence Table for i=0,1,2)
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo nextest run --all-features webgl::jfa_step_size
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `jfa_step_pass`'s `* aspect_ratio` scaling double-applies the aspect-ratio correction that `jfa_step.frag`'s own per-axis `/ resolution` division already performs, making the real per-axis pixel jump unequal on a non-square canvas. | ✅ Root Cause | Confirmed algebraically: with the pre-fix formula, real pixel jump = `(step_size, step_size)` scaled to `(step_size * aspect_ratio, step_size)`; with the fix, both terms reduce to exactly `step_size` regardless of resolution. | E1, E2, E3 |
| H2 | The `/ resolution` division in `jfa_step.frag` is itself wrong and needs the Rust-side `aspect_ratio` scaling to compensate for some other omission. | ❌ Falsified | The shader's per-axis `/ resolution` is exactly the standard pixel-to-UV conversion (each axis divided by its own resolution component) -- algebraically sufficient on its own, confirmed by the fixed formula producing an exactly isotropic result at every checked step/resolution combination. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/renderer/src/webgl/post_processing/outline/wide_outline.rs`, `jfa_step_pass` (pre-fix) | `let step_size = [ step_size * aspect_ratio, step_size ];` -- horizontal component scaled, vertical left alone. | H1 ✅ |
| E2 | `module/helper/renderer/src/webgl/shaders/post_processing/outline/wide_outline/jfa_step.frag` (unchanged) | `vec2 offset = ceil( vec2( x, y ) * stepSize ) / resolution;` -- a full component-wise division by `resolution`, already correcting per axis. | H1 ✅ |
| E3 | Hand-derived for a 1920x1080 canvas, `outline_thickness = 64.0`: pre-fix real pixel jump = `(114.0, 64.0)`, `(57.0, 32.0)`, `(29.0, 16.0)` for steps 0-2 (ratio exactly `1920/1080` each time); post-fix = `(64.0, 64.0)`, `(32.0, 32.0)`, `(16.0, 16.0)`. | Confirms both H1 and falsifies H2 -- removing the Rust-side scaling alone produces an exact isotropic result at every step. | H1 ✅, H2 ❌ |

## Root Cause

```rust
// before
let aspect_ratio = self.width as f32 / self.height as f32;
let step_size =  self.outline_thickness / ( 2.0_f32 ).powf( i as f32 );
let step_size = [ step_size * aspect_ratio, step_size ];
```

`stepSize` is meant to be a plain pixel distance, uniform in both directions -- `jfa_step.frag`'s
own `/ resolution` (a component-wise division by a vec2) already converts that uniform pixel
distance into the correct, generally-different, per-axis UV-space offset for a non-square canvas.
Scaling `stepSize.x` by `width / height` *before* that division applies the same correction a
second time, in the wrong place.

## Why Not Caught

No test exercised `jfa_step_pass` prior to this bug (it is private and GL-embedded, with no
existing native or browser test touching the JFA step passes specifically), and the resulting
distortion is a subtle elliptical stretch rather than a crash or a missing/wrong pixel -- easy to
overlook on visual inspection of a demo scene, especially since JFA-generated outlines already
have a slightly soft/approximate edge and most demo scenes lack sharp geometric reference lines
that would make a ~78% horizontal-vs-vertical thickness difference obvious without a deliberate
side-by-side comparison.

## Fix Location

`module/helper/renderer/src/webgl/post_processing/outline/wide_outline.rs`, `jfa_step_pass`:
removed the `aspect_ratio` computation and its use; `step_size` is now `[ step_size, step_size ]`
-- the same pixel distance in both components, relying entirely on the shader's own per-axis
`/ resolution` division for correct non-square-canvas normalization.

## Prevention

2 new native unit tests added, `module/helper/renderer/tests/webgl/jfa_step_size.rs`. `jfa_step_pass`
is private and GL-embedded (no pure function to extract without adding new public API surface
purely for testability), so -- following this crate's own `white_balance.rs` precedent (BUG-178)
for logic with no test-reachable execution path -- the test file is a line-for-line Rust port of
the fixed Rust-side step-size computation plus the (unchanged, already-correct) shader-side
conversion, composed exactly as production code composes them. Asserts the real per-axis pixel
jump is equal on a non-square (1920x1080) canvas across 3 JFA steps, and matches the configured
`outline_thickness` exactly on a second, non-16:9 (800x600) canvas at step 0 -- ruling out a
coincidental cancellation specific to one aspect ratio. The pre-fix formula would have failed the
first assertion by a factor of `width / height` at every step. Not pursued: fixing the
structurally similar `aspect_ratio` pattern in `examples/minwebgl/outline/src/main.rs` -- that
example's own `jfa_step.frag` uses a different `.x`-only step-size scheme (`ceil( dir *
stepSize.x ) / resolution`, ignoring `.y` entirely), so it is not confirmed to be the identical
defect and would need its own independent diagnosis; flagged as candidate future bug material
rather than fixed speculatively alongside this one.

## Pitfall

When a value is converted from pixel space to normalized UV space via a component-wise
`/ resolution` on a non-square texture, that division has ALREADY corrected for aspect ratio per
axis -- pre-scaling the value by `width / height` first does not compensate for anything still
missing, it double-corrects. "Divide by resolution" and "multiply by aspect ratio" are not two
complementary halves of the same correction; on a non-square texture, the first alone is already
the complete correction.

## Generalized Version

**Broken assumption:** "if a value ends up looking wrong on a non-square canvas, the fix is to
scale it by the canvas's aspect ratio somewhere in the pipeline."

**Confirmed general rule:** Before adding an aspect-ratio scaling factor anywhere, check whether a
downstream per-axis division (by resolution, by viewport size, by any vec2 with independently
different x/y components) already performs that exact correction. A per-axis division by a
non-square vec2 IS an aspect-ratio correction; layering an explicit multiplicative correction on
top of it — rather than instead of a missing one — doubles the effect instead of fixing it.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Pre-identified by task #98's review pass; confirmed this session by direct read of `jfa_step_pass` and `jfa_step.frag`, and hand-derivation of the real per-axis pixel jump both pre- and post-fix. |
| 2026-08-16 | fixed | Removed `jfa_step_pass`'s `aspect_ratio` computation and its `* aspect_ratio` scaling of `step_size.x`; both `stepSize` components now upload the same pixel distance. Full `Fix(BUG-180)` comment added at the fix site explaining the double-correction and citing the shader-side division it was compounding with. |
| 2026-08-16 | verified | New file `tests/webgl/jfa_step_size.rs` (2 native `#[test]` functions: isotropy across 3 JFA steps on a 1920x1080 canvas, exact-match-to-configured-thickness on an 800x600 canvas) -- `cargo nextest run --all-features webgl::jfa_step_size` from `module/helper/renderer/`: 2/2 passed. `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: clean. `cargo check --target wasm32-unknown-unknown --tests --all-features` (from `module/helper/renderer/`): clean -- confirms the fix doesn't break the crate's existing wasm32 browser test suite. Full workspace: `cargo nextest run --workspace --all-features --exclude object_picking`: 1899/1899 passed, 0 skipped (up from 1897 -- the 2 new tests). `cargo test --doc --workspace --all-features --exclude object_picking`: all crates ok. `cargo clippy --workspace --all-targets --all-features --exclude object_picking -- -D warnings`: clean. `--exclude object_picking` re-confirmed evidence-based: `object_picking`'s working tree remains dirty from the concurrent actor's own in-progress, unrelated work (`Cargo.toml`/`src/main.rs` modified, a new untracked `Trunk.toml`), while a standalone `cargo check -p object_picking` (non-clippy) still passes clean -- the exclusion remains clippy-lint-only and unrelated to this fix. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming: hand-derived the pre-fix vs post-fix real pixel jump numerically for 3 JFA steps on a 1920x1080 canvas, then encoded the same derivation as an executable native test. Adversarial: checked whether the derivation could be an artifact of one specific aspect ratio -- added a second test on an unrelated 800x600 (4:3) canvas, confirming the isotropy result generalizes rather than being a coincidence of the 16:9 numbers used in the MRE. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-179 (independent, already fixed, same file), and against the structurally similar but not-confirmed-identical pattern in `examples/minwebgl/outline/src/main.rs` -- deliberately left unfixed and documented as out of scope rather than silently ignored or speculatively changed. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct reads of both the Rust-side computation and the shader-side division, plus an explicit algebraic derivation with concrete numbers, not inferred from the diff's shape alone. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is exactly the 3-line step-size computation; no GL call structure, framebuffer logic, or uniform wiring touched. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Entirely within `renderer`'s own pass file and its own new test file; no caller signature changed. | — |
| D7 | Crate Locality | 🟢 | 🟢 | `jfa_step_pass`'s step-size computation has exactly one definition site, fixed there. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix restores the pass's own documented responsibility (uniform outline thickness) without adding or removing scope. | — |

**Reproduced:** YES -- hand-derived real per-axis pixel jump for a 1920x1080 canvas at
`outline_thickness = 64.0`: pre-fix `(114.0, 64.0)`, `(57.0, 32.0)`, `(29.0, 16.0)` for JFA steps
0-2 (each pair's ratio exactly `1920/1080`); post-fix `(64.0, 64.0)`, `(32.0, 32.0)`, `(16.0,
16.0)` at the same steps -- encoded as `jfa_step_size.rs`'s executable regression tests (2/2
passing). Full workspace native suite (1899/1899, 0 skipped), doctests (0 failed), and clippy all
clean (excluding the concurrent actor's unrelated `object_picking` in-flight refactor); wasm32
compile check for `renderer` (including its existing browser test suite) clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/post_processing/outline/wide_outline.rs` | Removed `jfa_step_pass`'s `aspect_ratio` computation and its `* aspect_ratio` scaling of `step_size.x`; both `stepSize` components now upload the same pixel distance, with a `Fix(BUG-180)` comment explaining the double-correction. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/webgl/jfa_step_size.rs` | New file, 2 native `#[test]` functions: real per-axis JFA pixel jump is isotropic on a 1920x1080 canvas across 3 steps, and matches `outline_thickness` exactly on an 800x600 canvas at step 0. |
| `module/helper/renderer/tests/webgl/mod.rs` | Added `mod jfa_step_size;` registration. |
| `module/helper/renderer/tests/readme.md` | Added `webgl/jfa_step_size.rs` Responsibility Table row. |
