# BUG-286: `palette_cosine_preview`'s default render is flat grayscale, contradicting its own readme's documented "canonical rainbow" description

- **Severity:** Low (cosmetic/documentation-parity defect -- the underlying `palette_cosine` function
  computes correctly; only the bundled demo's default-parameter rendering fails to demonstrate the
  chunk's own advertised purpose)
- **state:** Completed
- **Affects:** `palette_cosine_preview` (`shader/palette_cosine/palette_cosine.wgsl`); the structural
  test asserting its generated call site (`shader_chunks_preview_core/tests/preview_bundle_test.rs`)
- **Component:** `shader/palette_cosine` + `module/shader/shader_chunks_preview_core`
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`shader/palette_cosine/readme.md`'s own Visualization section explicitly documents the chunk's
`preview.png` as rendered through "the canonical rainbow parameterization (`a = b = vec3f(0.5)`,
`c = vec3f(1.0)`, `d = vec3f(0.0, 0.33, 0.67)`)," and states outright that "the phase vector `d` is
still what spreads the three channels a third of a cycle apart." The actual default-rendered image
was flat grayscale -- every pixel's R, G, and B channels held the identical value, at every point in
the frame.

## Impact

**Who is affected:** anyone viewing `shader/palette_cosine/preview.png` (spot-checked while
regenerating all 46 preview images as part of the `transient-bubbling-crystal` shader-preview plan's
closing verification step) or running `sch preview palette_cosine`/`sch render palette_cosine` with
default parameters -- the one bundled chunk whose entire purpose is demonstrating multi-channel color
separation instead demonstrated the opposite.

**What breaks:** documentation/artifact parity (the committed readme text and the generated image it
embeds directly contradict each other) and the demo's own didactic purpose -- a "cosine color
palette" chunk whose default preview cannot show a palette.

**Entity Scope:** `None` -- WGSL chunk source defect, not entity directory instances.

## How Discovered

While executing the `transient-bubbling-crystal.md` shader-preview plan's Section 6 verification step
("spot-checking 3-4 regenerated `preview.png` files... by reading the image directly to confirm each
looks like its documented description"), read `shader/palette_cosine/preview.png` after regenerating
it and found grayscale vertical stripes instead of the rainbow the readme describes -- one of exactly
the checks that verification step was designed to catch.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
./target/debug/shader_chunks_render render palette_cosine out::/tmp/palette_check.png
```
**Expected** (fixed): prints only `preview_scale = 8` under `parameters:` (no more tunable
base/amplitude/frequency/phase_r/phase_g/phase_b to report); the written PNG shows a repeating
rainbow gradient.
**Actual** (pre-fix): prints `base = 0.5`, `amplitude = 0.5`, `frequency = 2.05`, `phase_r = 0.5`,
`phase_g = 0.5`, `phase_b = 0.5` -- all three phases identical -- and the written PNG is flat
grayscale.

## Root Cause

`palette_cosine_preview` threaded its documented-fixed "canonical rainbow parameterization" through
as six independent tunable WGSL arguments (`base`, `amplitude`, `frequency`, `phase_r`, `phase_g`,
`phase_b`) instead of hardcoding them, each declared via its own `//@ param: ... range(min, max)`
line. `shader_chunks_params_core`'s range inference always defaults an argument to its own declared
range's midpoint, independently of every other argument -- there is no manifest syntax for expressing
"these N parameters' defaults must differ from each other." Since `phase_r`/`phase_g`/`phase_b` were
each independently declared `range(0.0, 1.0)`, all three defaulted to the identical midpoint `0.5`.
Because the three RGB output channels differ from each other *only* via the phase vector `d`
(`palette_cosine`'s own doc comment: "d = per-channel phase"), an identical `d` on all three channels
collapses the whole palette to a single shared value at every `t` -- grayscale, not a rainbow.

## Why Not Caught

The existing structural test (`preview_bundle_test.rs::vec3_value_chunk_gets_a_synthesized_harness`)
only asserted the generated WGSL called `palette_cosine_preview` with the correct variable names --
it exercises the `Vec3`-shape bundle-building/code-generation path, never renders a frame, and never
inspects a pixel. A semantically-monochrome-by-default demo passes it cleanly. No other test in the
crate family rendered this chunk and inspected its actual output color.

## Fix Applied (2026-08-17)

**`shader/palette_cosine/palette_cosine.wgsl`:** `palette_cosine_preview` now takes only `p: vec2f`
and hardcodes the readme's own documented canonical values directly in its body
(`palette_cosine(p.x, vec3f(0.5), vec3f(0.5), vec3f(1.0), vec3f(0.0, 0.33, 0.67))`), matching every
sibling bespoke-demo chunk's established convention (`rot2_preview`, `glow_preview`, etc.) of baking
fixed compositional constants directly into the wrapper body rather than exposing them as
independently-defaulted sliders. The 6 now-unused `//@ param:` manifest lines were removed along with
the corresponding function arguments; the `//@ export:` manifest line was updated to match the new
`fn palette_cosine_preview(p: vec2f) -> vec3f` signature.

**`shader/palette_cosine/readme.md`:** the `export` table row updated to the new (parameterless)
`palette_cosine_preview` signature; the Visualization prose was already accurate to the intended
fixed values and needed no change.

**`shader_chunks_preview_core/tests/preview_bundle_test.rs`:** `vec3_value_chunk_gets_a_synthesized_harness`'s
expected generated-call-site string updated from the old 6-argument form to `palette_cosine_preview( p )`.

**New regression test** (`shader_chunks_render/tests/render_cli_test.rs`):
`palette_cosine_default_render_shows_distinct_channels_not_flat_grayscale` -- renders the chunk for
real on the headless GPU, decodes the written PNG via the `image` crate (already a regular
`shader_chunks_render` dependency, no new dependency added), and asserts the maximum per-pixel
R/G/B channel spread across the whole frame exceeds a threshold (40 of 255) that flat grayscale can
never cross, so the test fails on an actual pixel-color assertion rather than a structural string
match.

**Also regenerated:** `preview.png` for all 46 chunks touched by the (already-implemented, pre-dating
this bug) `transient-bubbling-crystal` shader-preview plan, whose images had gone stale relative to
their current `.wgsl` source (see Generalized Version) -- `palette_cosine/preview.png` specifically
now shows the genuine repeating rainbow, visually confirmed.

## Verification

`longrun`-detached, from repo root. Revert-and-rerun proof used a scratchpad copy of the fixed
`palette_cosine.wgsl` plus `git show HEAD:<path>` to temporarily restore pristine content -- never
`git stash`, per this session's standing practice. `preview_bundle_test.rs` and `render_cli_test.rs`
were left in their fixed (new-expectation) state throughout, since only the WGSL source needed
reverting to exercise both tests' RED state.

- **Pre-fix (RED):** `cargo test -p shader_chunks_render --test render_cli_test -- palette_cosine_default_render_shows_distinct_channels_not_flat_grayscale`
  and `cargo test -p shader_chunks_preview_core --test preview_bundle_test -- vec3_value_chunk_gets_a_synthesized_harness`
  against the temporarily-restored pristine source: both failed (`0 passed; 1 failed` each) --
  the new test on an assert_eq-style pixel-spread panic, the existing test on the stale
  code-occurrence string -- confirming the bug before any fix existed.
- **Post-fix (GREEN):** same two targeted commands both passed, then the full scoped suite --
  `cargo nextest run -p shader_chunks_core -p shader_chunks_preview_core -p shader_chunks_preview
  -p shader_chunks_render --all-features` (106/106 passed) + `cargo clippy` on the same 4 crates
  `--all-targets --all-features -- -D warnings` (zero warnings/errors).
- **Full-tree confirmation:** re-ran the same combined native (14 crates) + wasm32
  (`shader_chunks_preview_web`) sweep used to close task #178, to confirm the whole `module/shader`
  tree is still clean after this additional chunk-source change.

## Generalized Version

A parameter-range-inference system that always defaults independently per-argument (to that
argument's own range midpoint, with no cross-argument relationship) cannot correctly serve a demo
whose entire visual point depends on several structurally-identical, identically-ranged parameters
differing *from each other* -- as opposed to each merely being independently "reasonable" on its own.
Every other bundled chunk with 3+ declared `//@ param:` lines was swept for this same shape
(distinct-range parameters combining into a demo, e.g. the SDF operators' circle/box offsets) and
found not vulnerable -- their parameters are each independently fine at any point in their own
(mutually distinct) ranges, with no "must differ from a sibling parameter" requirement.
`palette_cosine` was the sole instance of the vulnerable shape. The general lesson: when a manifest
line declares N parameters sharing an identical name pattern and range, and a chunk's own
documentation asserts a specific fixed relationship between them, prefer hardcoding that relationship
in the WGSL body (as every sibling bespoke-demo chunk already does for its own fixed compositional
constants) over exposing it as N independently-defaulted sliders -- there is currently no manifest
syntax for an explicit non-midpoint default, and adding one was considered and rejected here as
disproportionate to a single-chunk defect (a real feature addition touching the parser and multiple
consumer call sites, not a narrow fix).

Separately: this investigation's own preview-image regeneration pass found that `preview.png` had
gone stale (mtime older than its chunk's `.wgsl`) for all 46 chunks touched by the shader-preview
plan, plus 3 of the 4 natively-previewable chunks (`fbm3`, `gradient_noise`, `value_noise` --
`hash21` was still current) -- none had been regenerated since the plan's per-chunk WGSL/readme work
landed. Regenerating a derived image artifact after its source changes is easy to silently skip since
nothing enforces it mechanically (no test reads `preview.png`'s content); a periodic mtime check
(`wgsl mtime > preview.png mtime`) is a cheap way to catch this class of drift in the future without
needing per-chunk visual review every time.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found while executing the `transient-bubbling-crystal.md` shader-preview plan's own Section 6 verification step (spot-checking regenerated `preview.png` files against their documented descriptions) -- investigation of the plan's staleness first found the plan's entire original scope (harness `ValueFnKind` widening, all 46 per-chunk `_preview` exports, readme updates, tests, algorithm docs) had already been implemented by another actor between the plan's authoring (2026-08-15) and commit `0e713a83` (2026-08-16) plus follow-on commits; only `preview.png` regeneration (46 files, all stale) remained, and regenerating them surfaced this genuine rendering defect. Root cause: `palette_cosine_preview` exposed its documented-fixed canonical rainbow parameterization as 6 independently-defaulted tunable arguments; per-argument range-midpoint defaulting collapsed the 3 phase channels to an identical value, rendering flat grayscale instead of the readme's documented rainbow. Fixed by hardcoding the canonical values directly in the WGSL body, matching every sibling bespoke-demo chunk's convention. Verified via a new pixel-content regression test (real GPU render + PNG decode, not a string match), confirmed failing against a temporarily-restored pristine source (scratchpad copy + `git show HEAD:<path>`, no `git stash`) then passing post-fix, plus the full 4-crate suite (106/106) and clean clippy, plus a full 15-crate `module/shader` tree re-sweep. `task/readme.md`'s `highest_id` stood at 285 at filing time, confirmed via a fresh on-disk scan immediately before filing. |
