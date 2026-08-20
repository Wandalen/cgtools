# BUG-333: `outline`'s JFA step size pre-multiplies only the x-axis by aspect ratio before uploading, double-applying the aspect correction the shader already does per-axis, stretching the outline on non-square canvases

- **Severity:** Medium (visible rendering distortion on non-square canvases, not a crash)
- **state:** Completed
- **Affects:** `examples/minwebgl/outline/src/main.rs`
- **Component:** examples/minwebgl/outline
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`jfa_step_size(t, i)` pre-multiplied only its x component by `viewport.0 as f32 / viewport.1 as
f32` before uploading to `u_step_size`. `jfa_step.frag`'s `offset` calculation already divides by
`u_resolution` per-axis to convert a pixel-space jump into normalized UV space -- which alone
correctly compensates for a non-square canvas, exactly as this crate's sibling "production" shader
(`module/helper/renderer/.../wide_outline/jfa_step.frag`, already fixed under BUG-180) documents.
The caller's extra x-only pre-multiplication double-applied the aspect-ratio correction on top of
that per-axis division.

## Impact

**Who is affected:** any user viewing this demo on a non-square canvas.

**What breaks:** the JFA search radius -- and therefore the rendered outline -- is stretched wider
than tall instead of uniform in all directions, a visible distortion whose severity scales with
how far the canvas's aspect ratio departs from square.

**Entity Scope:** `None` -- confined to this crate's own JFA step-size computation (a documented
sibling of the already-fixed BUG-180 in the production renderer's own copy of this shader).

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184), by recognizing this crate's `jfa_step.frag` as a near-identical sibling of the renderer's
own JFA shader (already documented in this file's own header comment) and checking it against the
same double-aspect-correction pattern already fixed there under BUG-180, rather than assuming an
independent copy is independently correct. Independently verified by the orchestrating session:
`jfa_step.frag`'s `offset` line already divides by `u_resolution` per component, confirming the
caller's separate x-only aspect pre-multiplication was redundant and distorting.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p outline test_jfa_step_size_is_symmetric_across_passes_and_time
```
**Expected** (fixed): `jfa_step_size(t, i)` returns equal x/y components for every `(t, i)`.
**Actual** (pre-fix): the x component was scaled by `viewport.0/viewport.1`, diverging from y on
any non-square canvas.

## Root Cause

The caller pre-multiplied only the x component of the step size by the canvas's aspect ratio,
duplicating the aspect-ratio correction `jfa_step.frag`'s own per-axis division by `u_resolution`
already performs -- a documented pattern this crate's own "production" sibling shader had already
been fixed for under BUG-180, but this copy was not cross-checked against that fix.

## Why Not Caught

No test exercised `jfa_step_size`'s x/y symmetry, and the demo still renders a plausible-looking
outline on a square canvas (the default/common case) either way -- the distortion is only visible
on a non-square canvas, and nothing cross-referenced this crate's shader against its already-fixed
sibling in the renderer module.

## Fix Applied (2026-08-18)

Removed the x-only aspect-ratio pre-multiplication from `jfa_step_size`, making both components
equal to the same raw pixel-space jump distance -- letting `jfa_step.frag`'s existing per-axis
`u_resolution` division be the sole source of aspect-ratio correction, matching the already-fixed
production shader. Added 2 tests in the crate's existing `#[cfg(test)]` module:
`test_jfa_step_size_is_symmetric_across_passes_and_time` sweeps `i`/`t` asserting x == y;
`test_jfa_step_shader_uses_full_step_size_vector_not_x_only` (`include_str!` on `jfa_step.frag`)
asserts the shader multiplies by the full `u_step_size` vector, not a broadcast `.x` component.

## Verification

- **Pre-fix (RED):** reverted `jfa_step_size` to its x-only aspect-multiplied form; new test
  failed (x/y asymmetry reproduced).
- **Post-fix (GREEN):** `cargo test -p outline` -- both new tests pass;
  `cargo check --target wasm32-unknown-unknown -p outline` and
  `cargo clippy --all-targets --all-features -p outline -- -D warnings` both clean.

## Generalized Version

When a shader already documents itself as a near-identical sibling of another shader elsewhere in
the codebase, and that sibling has a known, already-fixed bug (here, BUG-180's double
aspect-ratio-correction pattern), check the copy for the exact same defect class rather than
assuming duplication implies independence -- a documented "mirror this shader's fixes" comment is
a maintenance obligation, not just informational.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-XXX-C` placeholder marker (disambiguated from sibling findings in the same fork's other crates) since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-333 after a fresh on-disk collision scan. Related: BUG-180, the already-fixed instance of this same defect class in `module/helper/renderer`'s production copy of this shader. |
