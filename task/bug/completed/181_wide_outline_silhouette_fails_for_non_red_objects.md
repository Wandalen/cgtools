# BUG-181: `jfa_init.frag`'s silhouette detection fails for non-red objects

- **Severity:** Medium (visual-only defect -- no crash, no data loss, but any object whose color
  isn't sufficiently red silently receives no outline at all)
- **state:** Completed
- **Affects:** Every caller of `renderer::webgl::post_processing::outline::wide_outline::WideOutlinePass`
  where the `object_color_texture` input holds a non-red object color (pure green/blue/cyan, or
  even ordinary black).
- **Component:** `module/helper/renderer` (`src/webgl/shaders/post_processing/outline/wide_outline/jfa_init.frag`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** Same `wide_outline` shader trio as BUG-179 and BUG-180 (both fixed,
  independent). Also candidate BUG-182 (`outline.frag`'s own sentinel check, same `-1.0` sentinel
  convention, not yet fixed -- diagnosing that bug will build directly on this one's confirmation
  of the GBuffer's `( -1, -1, -1, 1 )` clear-value contract).

## Symptom

```glsl
// pre-fix -- jfa_init.frag
float objectPresent = texture( objectColorTexture, vUv ).r;
if ( objectPresent > 0.01 ) // If pixel is part of the object ( check > 0.0 for robustness )
```

`jfa_init.frag` decided whether a pixel belonged to an object by checking if its red channel
exceeded `0.01`. But `gbuffer.rs`'s `GBuffer::render` clears the `OBJECT_COLOR` attachment to
`( -1, -1, -1, 1 )` before drawing, and `gbuffer.frag` writes the real, caller-supplied
`objectColor` uniform to every rasterized object pixel (`FragObjectColor = objectColor;`) -- so
the true set of possible values is "the negative sentinel" or "any real, non-negative object
color," not "near 0" vs. "near 1". The `> 0.01` threshold only matched objects whose red channel
happened to be close to 1.0 -- true only by coincidence of the one caller that existed at the
time (`renderer_with_outlines`, which hardcodes every object to red via `object_colors_generate`)
-- so any object with a different color would have had its silhouette silently dropped.

## Impact

**Who is affected:** Any caller supplying an `object_color_texture` where object pixels' red
channel is `<= 0.01` -- pure green, pure blue, cyan, or plain black objects, all common,
legitimate colors.

**What breaks:** Purely visual -- an affected object is treated as pure background throughout the
whole JFA pipeline: no seed pixels are ever written for it in `jfa_init_pass`, so the JFA step
passes never propagate a "nearest seed" distance for its silhouette, and `outline_pass` never
draws an outline around it. The object still renders normally in the underlying scene ( this pass
only adds the outline overlay ), so the failure mode is a silently *missing* outline, not a
crash or corrupted render.

**Magnitude:** Every affected-colored object is affected identically, every frame, for as long as
the caller supplies that color -- not intermittent or frame-dependent.

**Entity Scope:** None -- a code-level (shader-level) defect.

## How Discovered

Pre-identified by task #98's review pass (this session) as "wide_outline silhouette detection
fails for non-red objects." Confirmed by reading `jfa_init.frag`'s check together with
`gbuffer.rs`'s `GBuffer::render` clear call (`gl.clear_bufferfv_with_f32_array( gl::COLOR, 4,
[ -1.0, -1.0, -1.0, 1.0 ].as_slice() )` for the `OBJECT_COLOR` attachment) and `gbuffer.frag`'s
write (`FragObjectColor = objectColor;`) -- confirming the actual value space is a negative
sentinel vs. an arbitrary non-negative color, not "near 0 vs. near 1" as the `> 0.01` threshold
assumed. Cross-checked against the one real caller, `renderer_with_outlines`'s
`object_colors_generate`, which currently hardcodes every object's color to `(1.0, 0.0, 0.0, 1.0)`
-- explaining why the defect has been invisible so far despite the function's name and per-index
signature implying it's meant to produce distinct per-object colors.

## Minimum Reproducible Example

```glsl
// pre-fix: a pure-green object color, e.g. objectColor = vec4(0.0, 1.0, 0.0, 1.0)
float objectPresent = texture( objectColorTexture, vUv ).r;  // samples 0.0 for this object's pixels
if ( objectPresent > 0.01 )  // 0.0 > 0.01 is false -- treated as background, no outline seeded
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo nextest run --all-features webgl::jfa_silhouette
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The `> 0.01` red-channel threshold assumes object pixels are red-dominant, but the actual data model is "negative sentinel vs. any non-negative color" -- any object color with `r <= 0.01` is misclassified as background. | ✅ Root Cause | Confirmed by reading the GBuffer's clear value (`-1,-1,-1,1`) and write path (`FragObjectColor = objectColor`) directly -- the only two value classes are the negative sentinel and whatever color the caller supplies, with no assumption that the caller's color is red-dominant. | E1, E2, E3 |
| H2 | `objectColorTexture` always legitimately holds red (or near-red) colors for objects, so the `> 0.01` threshold is correct as documented ("object_pass renders object pixels as white"). | ❌ Falsified | `gbuffer.frag` writes an arbitrary caller-supplied `objectColor` uniform, not a fixed white/red value -- the stale comment describing "white (r=1.0)" doesn't match what the code actually writes; `object_colors_generate`'s name and per-index signature further show the intent is per-object distinct colors, only currently stubbed to always return red. | E2, E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/renderer/src/webgl/post_processing/gbuffer.rs` (`GBuffer::render`, clear calls) | `gl.clear_bufferfv_with_f32_array( gl::COLOR, 4, [ -1.0, -1.0, -1.0, 1.0 ].as_slice() )` -- the `OBJECT_COLOR` attachment ( index 4 ) clears to a negative RGB sentinel, not `(0,0,0,0)`. | H1 ✅ |
| E2 | `module/helper/renderer/src/webgl/shaders/post_processing/gbuffer.frag` | `FragObjectColor = objectColor;` -- writes the real, arbitrary per-draw `objectColor` uniform verbatim to every object pixel, not a fixed white/red constant. | H1 ✅, H2 ❌ |
| E3 | `examples/minwebgl/renderer_with_outlines/src/main.rs`, `object_colors_generate` | Hardcodes every object's color to `(1.0, 0.0, 0.0, 1.0)` regardless of its `_` index parameter -- explains why the bug has been invisible to date (the one real caller always happens to supply red), while its own signature ( per-object-index mapping ) signals the intended design is per-object distinct colors. | H2 ❌ |

## Root Cause

```glsl
// before
float objectPresent = texture( objectColorTexture, vUv ).r;
if ( objectPresent > 0.01 )
```

The check assumed "object present" looks like "red channel near 1.0" -- an assumption that
happened to hold for the one caller that existed, but doesn't follow from the actual contract
(`GBuffer::render`'s negative-sentinel clear vs. `gbuffer.frag`'s arbitrary-color write). The
correct discriminant is sign, not magnitude: any non-negative red-channel value can only come
from a real object color (colors in this codebase are always non-negative), never from the
`-1.0` sentinel.

## Why Not Caught

No test exercised the silhouette check prior to this bug, and the one real caller
(`renderer_with_outlines`) hardcodes every object's color to red via `object_colors_generate` (see
E3) -- itself plausibly an unfinished stub, given its name and per-index signature imply
per-object distinct colors that it doesn't currently produce -- so the defect had no way to
surface until a caller actually supplied a non-red object color.

## Fix Location

`module/helper/renderer/src/webgl/shaders/post_processing/outline/wide_outline/jfa_init.frag`:
changed the comparison from `objectPresent > 0.01` to `objectColorR >= 0.0`, with a `Fix(BUG-181)`
comment explaining the actual clear-value/write-value contract this now correctly tests against.

## Prevention

3 new native unit tests added, `module/helper/renderer/tests/webgl/jfa_silhouette.rs`. Following
this crate's `white_balance.rs` (BUG-178) precedent for GLSL logic with no CPU-execution path
(GLSL ES 3.00 is outside naga's `glsl-in` front end, per `shader_validation_tests.rs`'s own scope
note), `object_is_present` is a line-for-line Rust port of the fixed shader check. Asserts: a
black (`r = 0.0`) object is detected as present (the case the pre-fix `> 0.01` threshold would
have missed), the actual `-1.0` sentinel is correctly excluded, and the original red (`r = 1.0`)
case still works (no regression for the one existing caller). Additionally, re-ran the existing
BUG-179 `wide_outline` browser test (`cargo test --target wasm32-unknown-unknown --all-features
webgl::wide_outline`) after this edit, confirming the hand-edited GLSL still compiles and links
successfully in a real WebGL2 context -- this crate has no offline GLSL syntax validation, so a
real shader-compile pass is the only way to catch a manual-edit typo here.

## Pitfall

A magnitude threshold (`> 0.01`) silently encodes an assumption about which channel and which
value range "presence" looks like. When the actual discriminant is a sign difference against a
sentinel (a non-negative real color vs. a negative marker), a sign check (`>= 0.0`) is what the
data model actually guarantees -- a magnitude threshold picked to fit one caller's current values
will silently break for any other legitimate value in between the threshold and the sentinel.

## Generalized Version

**Broken assumption:** "the one caller I can see today defines what 'valid input' looks like, so
a threshold that happens to separate today's inputs from the clear/background value is a correct
general check."

**Confirmed general rule:** When a check distinguishes "real data" from "not yet written /
background," trace the actual producer (the clear call) and the actual writer (the draw call)
directly, and encode the check against the invariant they jointly guarantee -- not against
whichever specific values the currently-known caller(s) happen to produce. A single caller's
current behavior is not the contract.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Pre-identified by task #98's review pass; confirmed this session by reading `jfa_init.frag`, `gbuffer.rs`'s clear call, `gbuffer.frag`'s write path, and `renderer_with_outlines`'s `object_colors_generate` caller. |
| 2026-08-16 | fixed | Changed `jfa_init.frag`'s comparison from `objectPresent > 0.01` to `objectColorR >= 0.0`, matching the actual negative-sentinel-vs-real-color contract established by `GBuffer::render`'s clear value and `gbuffer.frag`'s write. Full `Fix(BUG-181)` comment added at the fix site. |
| 2026-08-16 | verified | New file `tests/webgl/jfa_silhouette.rs` (3 native `#[test]` functions: black object detected, sentinel excluded, red object still detected) -- `cargo nextest run --all-features webgl::jfa_silhouette` from `module/helper/renderer/`: 3/3 passed. Re-ran the existing BUG-179 wasm32 browser test (`webgl::wide_outline`) to confirm the hand-edited GLSL still compiles/links/runs: 1/1 passed. `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: clean. Full workspace: `cargo nextest run --workspace --all-features --exclude object_picking`: 1902/1902 passed, 0 skipped (up from 1899 -- the 3 new tests). `cargo test --doc --workspace --all-features --exclude object_picking`: all crates ok. `cargo clippy --workspace --all-targets --all-features --exclude object_picking -- -D warnings`: clean. `--exclude object_picking` re-confirmed evidence-based: working tree still dirty from the concurrent actor's own unrelated in-progress work; `cargo check -p object_picking` (non-clippy) still passes clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming: traced the actual clear-value and write-path contract directly in source rather than trusting the shader's own (stale) comment. Adversarial: checked whether the fix could itself be wrong by asking "could a legitimate object color ever be negative?" -- no color-producing code path in this codebase constructs a negative RGB component; the `-1.0` sentinel is deliberately out of the normal color range specifically so it can't collide. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-179/BUG-180 (independent, same file trio, already fixed) and BUG-182 (same `-1.0` sentinel convention, not yet fixed -- explicitly left for its own separate diagnosis). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct reads of the clear call, the write path, and the one real caller, not inferred from the diff's shape alone. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is a single comparison-operator and threshold change; no other logic in the file touched. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Entirely within `renderer`'s own shader file and its own new test file; the one real caller needed no changes since it already always supplies red (unaffected either way). | — |
| D7 | Crate Locality | 🟢 | 🟢 | The silhouette check has exactly one definition site, fixed there. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix completes the pass's own documented responsibility (mark object silhouette seeds) without adding or removing scope. | — |

**Reproduced:** YES -- `object_is_present(0.0)` (the pre-fix formula's blind spot: `0.0 > 0.01` is
false) now correctly returns `true` post-fix (`0.0 >= 0.0` is true), encoded as
`jfa_silhouette.rs`'s executable regression tests (3/3 passing). Existing BUG-179 browser test
re-run to confirm the hand-edited GLSL still compiles/links/runs in a real WebGL2 context (1/1
passing). Full workspace native suite (1902/1902, 0 skipped), doctests (0 failed), and clippy all
clean (excluding the concurrent actor's unrelated `object_picking` in-flight refactor), 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/shaders/post_processing/outline/wide_outline/jfa_init.frag` | Changed the silhouette check from `objectPresent > 0.01` to `objectColorR >= 0.0`; full `Fix(BUG-181)` comment added explaining the clear-value/write-value contract. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/webgl/jfa_silhouette.rs` | New file, 3 native `#[test]` functions: black object detected as present, `-1.0` sentinel excluded, red object still detected (no regression). |
| `module/helper/renderer/tests/webgl/mod.rs` | Added `mod jfa_silhouette;` registration. |
| `module/helper/renderer/tests/readme.md` | Added `webgl/jfa_silhouette.rs` Responsibility Table row. |
