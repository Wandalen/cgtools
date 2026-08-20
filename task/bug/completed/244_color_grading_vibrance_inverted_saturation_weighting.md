# BUG-244: `adjust_vibrance`'s saturation-push weight grows WITH existing saturation instead of
against it -- boosts already-vivid colors (skin tones) harder than dull ones

- **Severity:** Low (visual/cosmetic color-grading parameter produces a weaker "smart" effect than
  documented -- no panic, no crash, no data corruption; the shader still runs and still visibly
  adjusts saturation, just not weighted the way its own documented contract promises)
- **state:** Completed
- **Affects:** `color_grading.frag`'s `adjust_vibrance()`, every invocation with `vibrance != 0.0`
  (parameter default is `0.0`/neutral, so this only fires when a caller actively dials vibrance)
- **Component:** `module/helper/renderer` (`src/webgl/shaders/post_processing/color_grading.frag`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`color_grading.frag`'s own header doc comment states the contract explicitly:

> **Vibrance**: Smart saturation affecting less-saturated colors more. Preserves skin tones better
> than uniform saturation.

The implementation did the opposite: a color's saturation push grew LARGER the MORE saturated it
already was, and shrank toward zero for already-near-gray colors -- so already-vivid colors
(including skin tones, which are moderately-to-highly saturated, not neutral) were pushed harder
than dull, desaturated ones.

## Impact

**Who is affected:** Any consumer of `ColorGradingParams::vibrance` with a non-zero value (default
is `0.0`, neutral). The `outline`/`narrow_outline`/color-grading example scenes and any embedding
application dialing this slider.

**What breaks:** The headline benefit of a "vibrance" control over plain "saturation" -- protecting
already-saturated content (skin tones, foliage, sky) from oversaturating while still making dull
background elements pop -- did not hold. Quantitatively (see `## Root Cause`): a color with a raw
channel spread of `0.10` gained roughly 3.45x more relative spread than a color with spread `0.80`
under the FIXED formula; under the pre-fix formula the relationship was inverted (the
already-saturated color gained the larger relative boost, confirmed via the temporary
revert-and-rerun in `## Verification`).

**Entity Scope:** `None` -- source-level shader logic defect, not entity directory instances.

## How Discovered

During this session's `renderer` crate scout (task #174), a `general-purpose` subagent fork
dispatched to review the post-processing subsystem (read-only, no fixes) flagged `adjust_vibrance`
as inconsistent with its own doc comment. Independently re-derived by hand-evaluating the formula
against two concrete colors of differing existing saturation (both in absolute pixel-shift terms
and in normalized-saturation-delta terms) before accepting the finding -- both metrics agreed the
already-saturated color received the proportionally larger push, confirming the bug rather than a
misreading of the doc comment's intent (independently cross-checked against Adobe's own published
definition of the Vibrance control, which uses this same "less saturated colors get a bigger boost
than already-saturated ones" language).

## Minimum Reproducible Example

GLSL ES 3.00 has no native/offline execution path in this crate (`shader_validation_tests.rs`'s
own scope note: naga's `glsl-in` front end parses desktop GLSL, not the ES profile these `.frag`
files use), so no GPU-context MRE is practical here either -- the defect is fully captured by a
line-for-line Rust port of the shader function, mirroring this crate's own `white_balance.rs`
(BUG-178) precedent. See `tests/webgl/vibrance.rs`.

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p renderer --test tests vibrance
```
**Expected** (fixed): all 5 tests pass. **Actual** (pre-fix, confirmed via temporary direct-edit
revert-and-rerun of the Rust port): 2 fail --
`low_saturation_color_gets_a_larger_relative_boost_than_high_saturation_color` (low-saturation
color's relative spread growth was NOT larger than the high-saturation color's -- the pre-fix
formula gives the opposite ranking) and `fully_saturated_color_is_unaffected_by_positive_vibrance`
(a fully saturated color was NOT left unaffected -- the pre-fix formula still pushes it, since its
weight is `mx - average`, which is near its MAXIMUM for a fully saturated primary, not zero).

## Root Cause

`adjust_vibrance` (pre-fix):
```glsl
float average = ( color.r + color.g + color.b ) / 3.0;
float mx = max( max( color.r, color.g ), color.b );
float amt = ( mx - average ) * ( -vibrance * 3.0 );
return mix( color, vec3( mx ), amt );
```
`( mx - average )` is `0` for a gray color and grows toward `2/3` for a fully saturated primary
(e.g. pure red `(1,0,0)`) -- it is an INCREASING function of the color's own existing saturation.
Since `amt`'s magnitude directly scales the `mix` push toward/away from `vec3( mx )` (the fully
desaturated "gray at max-channel brightness" target), and `mix`'s extrapolation strength is
proportional to `amt`, a LARGER `amt` for an already-saturated color means a LARGER push for that
color -- backwards from "affects less-saturated colors more", which requires the weight to
DECREASE as existing saturation increases.

## Why Not Caught

No test exercised `adjust_vibrance`'s *relative* boost strength across colors of differing
existing saturation prior to this bug. The existing `color_grading_tests.rs` only covers
`ColorGradingParams`'s `Default`/`Clone` derives, not any shader math (matching the pattern already
established for BUG-178's `apply_white_balance` in this same file). The bug also produces no crash
and no obviously-wrong-looking image on casual inspection -- a stronger-than-intended push on
already-vivid colors still visually reads as "the image got more vibrant," just not distributed
the way the documented contract promises; nothing about it looks broken without comparing relative
boost strength across colors of different starting saturation directly.

## Fix Applied (2026-08-17)

**`src/webgl/shaders/post_processing/color_grading.frag`:** replaced the `( mx - average )` weight
with the complement of a normalized HSV-style saturation:
```glsl
float mx = max( max( color.r, color.g ), color.b );
float mn = min( min( color.r, color.g ), color.b );
float sat = ( mx - mn ) / max( mx, 0.0001 );
float amt = ( 1.0 - sat ) * ( -vibrance * 3.0 );
return mix( color, vec3( mx ), amt );
```
`sat` is `0` for gray, `1` for a fully saturated color (one channel at `0`); `( 1.0 - sat )` is
therefore `1` at zero existing saturation (maximal boost headroom) and `0` at full existing
saturation (fully protected) -- the correct decreasing-with-saturation direction. The `max( mx,
0.0001 )` guard avoids a `0.0 / 0.0` (`NaN`) for pure black, matching this codebase's established
`.max( epsilon )` divide-by-zero-guard convention (BUG-233/236/237). The `-vibrance * 3.0`
sign/scale convention and the final `mix( color, vec3( mx ), amt )` blend are unchanged, so
positive vibrance still saturates and negative still desaturates -- only the weighting direction
changed. `average` is no longer used and was removed.

**`tests/webgl/vibrance.rs`** (new file, registered in `tests/webgl/mod.rs`): a line-for-line Rust
port of the fixed function (`white_balance.rs`/BUG-178 precedent) plus 5 native `#[test]`
functions: the two that specifically discriminate the bug
(`low_saturation_color_gets_a_larger_relative_boost_than_high_saturation_color`,
`fully_saturated_color_is_unaffected_by_positive_vibrance`) plus 3 general-behavior sanity checks
(gray always unaffected, positive vibrance increases spread, negative decreases it) that hold
under both the pre-fix and post-fix formulas and exist to pin the parts of the contract that were
never broken.

## Verification

`longrun`-detached, from repo root:
- `cargo test -p renderer --test tests vibrance` -- pre-fix (temporary direct-source-edit revert
  of the Rust port only, shader `.frag` itself left fixed throughout): 3 passed, 2 failed (the two
  discriminating tests, as designed). Post-fix (port restored): 5 passed, 0 failed.
- `verb/test_only pkg::renderer` (full scoped suite, post-fix): **143 tests run: 143 passed, 0
  skipped** (27s) -- up from 138 (this bug's 5 new tests), including the real GPU-backed
  `native_render_test.rs::opaque_path_renders_lit_quad`.
- `cargo clippy -p renderer --all-features --all-targets -- -D warnings`: exit 0, clean.

## Generalized Version

**Broken assumption:** a quantity that correlates with "how much of property X a value already
has" is safe to use directly as a boost weight without checking whether the desired relationship
is increasing or decreasing in that quantity. Here, `( mx - average )` genuinely does measure
"how saturated is this color" -- the mistake was using it as-is (increasing weight) for a feature
whose entire documented purpose is the opposite (protect the already-saturated, boost the
under-saturated). Whenever a "smart"/adaptive adjustment is documented as protecting or
de-emphasizing colors/values that already have more of some property, explicitly verify the
weight term is a DECREASING function of that property -- an increasing one is the natural,
easy-to-write mistake when the underlying proxy measure itself happens to increase with the
property being protected.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found by a scouting-fork review of `renderer`'s post-processing subsystem during task #174, independently re-derived by hand-evaluating two concrete colors of differing existing saturation under the formula before acceptance. Root cause: `adjust_vibrance`'s `( mx - average )` weight term increases with existing saturation instead of decreasing, so already-vivid colors got pushed harder than dull ones -- backwards from the file's own documented "affects less-saturated colors more" contract. Fixed via a normalized HSV-style saturation complement `( 1.0 - ( mx - mn ) / mx )`. Verified via a line-for-line Rust port (no GLSL ES execution path exists in this crate) with 5 new native unit tests (2 confirmed to fail pre-fix / pass post-fix via temporary revert-and-rerun) plus the full 143/143 scoped suite and clean clippy. Closed same-session (Tier 2 Dual-Role Self-Check). |
