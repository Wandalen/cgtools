# BUG-464: `filters`' Gaussian Blur slider max (50.0) produces a far larger shader kernel than Box/Stack Blur's own max (80.0), risking render-loop stalls

- **Severity:** Medium (no crash under normal driver behavior, but a per-pixel, non-unrollable
  301-sample GLSL loop at the slider's high end is squarely in GPU-driver-timeout territory on
  weaker hardware)
- **state:** Verified
- **Affects:** `examples/minwebgl/filters`'s "Gaussian Blur" filter card's "Size" slider.
- **Component:** `examples/minwebgl/filters` (`src/ui_setup/filter_setup_advanced.rs`,
  `src/filters/blur.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Fix Task:** [508](../../verifying/508_register_filters_gaussian_blur_slider_max_fix_closes_bug464.md)

## Symptom

```rust
// pre-fix -- src/ui_setup/filter_setup_advanced.rs, blur_filters_setup
filter_setup_helpers::blur_filter_setup( filter_renderer, current_filter, "box-blur", "Box Blur", blur::Box, 80.0 );
filter_setup_helpers::blur_filter_setup( filter_renderer, current_filter, "gaussian-blur", "Gaussian Blur", blur::Gaussian, 50.0 );
filter_setup_helpers::blur_filter_setup( filter_renderer, current_filter, "stack-blur", "Stack Blur", blur::Stack, 80.0 );
```

```glsl
// src/filters/blur.rs, Blur<Gaussian>'s fragment shader
int kernel_size = u_sigma * 6 + 1; // u_sigma is fed directly from the "Size" slider's raw value
int half_size = kernel_size / 2;
for ( int i = -half_size; i <= half_size; i++ ) { /* dynamic, non-unrollable loop */ }
```

The "Size" slider's raw value is uploaded directly as `u_sigma`, and the shader derives a *dynamic*
loop width of `u_sigma * 6 + 1` from it. At the slider's pre-fix max of 50.0, that is a worst-case
`kernel_size` of `50*6+1 = 301` -- run twice per pixel (the two-pass separable blur) and un-
unrollable by the shader compiler because `u_sigma` is a uniform, not a compile-time constant. Box
Blur's max of 80.0 produces an 80-sample loop; Stack Blur's max of 80.0 produces up to
`2*80+1 = 161` samples -- so despite Gaussian's slider number *looking* smaller (50 vs. 80), its
actual worst-case shader cost was already far larger than either sibling filter's.

## Impact

**Who is affected:** Anyone dragging the Gaussian Blur "Size" slider toward its upper end.

**What breaks:** A 301-sample-wide dynamic fragment-shader loop, executed for every pixel in both
blur passes, is squarely in the range where weaker/mobile GPUs or a driver's own watchdog timer
(TDR on Windows, similar mechanisms elsewhere) can stall the render loop or kill the context
entirely -- a qualitatively different risk than Box/Stack's own worst case at less than half the
sample count.

**Entity Scope:** None -- a code-level defect (UI parameter tuning, not a logic error).

## How Discovered

Found during a repo-wide bug/UX-DX sweep of `examples/minwebgl/filters`, comparing each blur
variant's UI slider max against its own shader's actual loop-width formula rather than assuming
the slider numbers themselves were comparable across variants.

## Manual Reproduction / Verification

No dedicated automated MRE test was added -- this is a UI slider bound feeding a GLSL shader
uniform, with no native-testable pure-logic core, consistent with this sweep's granted exception
for example crates. Verified instead by:

1. Deriving the exact worst-case `kernel_size` for all three blur variants from their own shader
   source (`src/filters/blur.rs`): Gaussian pre-fix `6*50+1 = 301`; Gaussian post-fix
   `6*15+1 = 91`; Box `80` (direct `u_box_size`); Stack `2*80+1 = 161` (`u_radius` both directions).
2. `cargo check -p filters --target wasm32-unknown-unknown` -- clean, no errors.

**Verify Command:**
```bash
cd examples/minwebgl/filters && cargo check --target wasm32-unknown-unknown
```

## Root Cause

The Gaussian Blur slider's max was set as if the slider's raw value were directly comparable to
Box/Stack's own "Size"/"Radius" slider maxes, without accounting for the shader-side `*6`
multiplier that turns `u_sigma` into an actual loop width (`kernel_size = u_sigma * 6 + 1`) -- a
slider's raw numeric max is not a proxy for the actual shader cost when the uniform it feeds is
scaled by a shader-side formula before being used as a loop bound.

## Why Not Caught

No existing test or manual-testing checklist entry compared the *derived* shader loop width across
blur variants -- only the slider's own raw max was eyeballed, and 50.0 reads as "smaller and
therefore safer" than Box/Stack's 80.0 without cross-checking each variant's own formula.

## Fix Location

`examples/minwebgl/filters/src/ui_setup/filter_setup_advanced.rs`, `blur_filters_setup`: Gaussian
Blur's slider max lowered from `50.0` to `15.0`, capping worst-case `kernel_size` at
`15*6+1 = 91` -- in line with Stack Blur's own worst case (161) and a 3.3x reduction from the
pre-fix 301.

## Prevention

None added beyond the fix itself and the wasm32 compile check, per this sweep's exception for
example crates -- the `Fix(BUG-464)` source comment at the fix site explicitly states the
derived-loop-width comparison method, so a future slider-max change to any of the three blur
variants has the reasoning documented in place rather than requiring re-derivation from scratch.

## Pitfall

A slider's raw numeric max is not a proxy for the actual shader cost when the uniform it feeds is
scaled by a shader-side formula (here, `*6`) before being used as a loop bound -- compare the
*derived* loop width across filters/variants, not the slider max values themselves. A GLSL loop
bounded by a uniform (not a compile-time constant) cannot be unrolled by the shader compiler, so
its cost scales linearly and directly with whatever value the UI allows the user to drag it to.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during repo-wide bug/UX-DX sweep of `examples/minwebgl/filters`. |
| 2026-08-20 | fixed | Gaussian Blur slider max lowered 50.0 -> 15.0; documented with `Fix(BUG-464)`/`Root cause`/`Pitfall`. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 2/2

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Chosen value correctness (formula re-derivation + compile) | — | 🟢 | Adversarial pass: independently re-derived `kernel_size` for all three variants at both their pre-fix and post-fix maxes (see Manual Reproduction / Verification) to confirm 15.0 actually lands the Gaussian worst case near, not still far above, Box/Stack's own worst case, rather than trusting the finding's own suggested "~15" value without recomputing it. `cargo check -p filters --target wasm32-unknown-unknown` clean. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-464)`/`Root cause`/`Pitfall` 3-field format applied at the fix site, stating the derived-loop-width comparison explicitly. | — |

**Reproduced:** Confirmed via formula re-derivation from the actual shader source (not a live
GPU-timeout observation -- see Manual Reproduction / Verification for why an automated MRE was not
added). 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `examples/minwebgl/filters/src/ui_setup/filter_setup_advanced.rs` | `blur_filters_setup`: Gaussian Blur slider max `50.0` -> `15.0`, with `Fix(BUG-464)`/`Root cause`/`Pitfall` comment. |
