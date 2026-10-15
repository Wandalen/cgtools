# BUG-178: `color_grading.frag`'s white-balance `tint` shifts the wrong direction

- **Severity:** Medium (visual-only defect -- no crash, no data loss, but every caller passing a
  non-zero `tint` gets the exact opposite of the documented and intended color shift)
- **state:** Completed
- **Affects:** Every user of `renderer::webgl::post_processing::ColorGradingPass` that sets
  `ColorGradingParams.tint` away from `0.0` -- a positive ("magenta") value visibly shifted the
  image toward green, and a negative ("green") value shifted it toward magenta.
- **Component:** `module/helper/renderer` (`src/webgl/shaders/post_processing/color_grading.frag`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** None -- isolated shader-math sign error, unrelated to any other open or closed
  bug in this crate. Testing approach follows the precedent established by BUG-158
  (`join_tangent.rs`'s Rust port of a GLSL formula), since GLSL ES shaders have no CPU-executable
  path in this crate (`shader_validation_tests.rs` covers WGSL via `naga` only).

## Symptom

```glsl
// pre-fix -- color_grading.frag, apply_white_balance
vec3 apply_white_balance( vec3 color, float temperature, float tint )
{
  vec3 t = vec3( 1.0 );
  t.r += 0.2 * temperature - 0.1 * tint;
  t.b -= 0.2 * temperature + 0.1 * tint;
  return color * t;
}
```

`ColorGradingParams::tint`'s own doc comment states "Positive: magenta tint - Negative: green
tint," but a positive `tint` here *subtracts* from `t.r` and *adds* to (via `-=`, increasing the
subtraction of) `t.b`'s multiplier -- lowering both red and blue relative to green, which is a
shift *toward* green, not magenta. Negative `tint` does the opposite -- toward magenta, not green.

## Impact

**Who is affected:** Any caller of `ColorGradingPass` (the crate's standard post-processing color
grading stage) that exposes `tint` to a user or drives it programmatically -- e.g. a color-grading
UI slider, or a preset/LUT-style parameter bank. `temperature`'s own sign convention was
independently traced and confirmed correct; the defect is scoped strictly to `tint`.

**What breaks:** Purely visual -- the rendered frame's color cast is inverted relative to what the
`tint` parameter's own documented contract and any UI label built against it would lead a user to
expect. No crash, no panic, no incorrect data persisted -- the shader always produces *a* valid
color, just the wrong one.

**Magnitude:** Every draw through `ColorGradingPass` with `tint != 0.0` is affected identically,
since the defect is in the single shared fragment shader, not any one call site. `tint` defaults
to `0.0` (neutral) per `ColorGradingParams`'s `Default` impl, so the bug is latent until a caller
actively dials in a non-zero value.

**Entity Scope:** None -- a code-level (shader-level) defect.

## How Discovered

Found during this session's continued bug-fixing sweep of `module/helper/renderer`, while
reviewing `post_processing/color_grading.rs` and its shader after closing BUG-176/BUG-177 in
sibling `webgpu` code. Confirmed by hand-tracing both parameters' sign relationships against their
own doc-commented intent: `temperature` (warm/cool) is *supposed* to move red and blue in
*opposite* directions -- confirmed correct pre-fix. `tint` (magenta/green) is *supposed* to move
red and blue in the *same* direction -- confirmed backwards pre-fix, since it reused
`temperature`'s opposing-sign pattern instead of its own.

## Minimum Reproducible Example

```rust
// module/helper/renderer/tests/webgl/white_balance.rs -- pre-fix, this assertion fails
let neutral = [ 0.5, 0.5, 0.5 ];
let result = apply_white_balance( neutral, /* temperature */ 0.0, /* tint */ 1.0 );
// pre-fix: result[0] < neutral[0] && result[2] < neutral[2] -- shifted toward green
// post-fix: result[0] > neutral[0] && result[2] > neutral[2] -- shifted toward magenta, as documented
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo nextest run -p renderer --all-features webgl::white_balance
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `apply_white_balance` applies `tint`'s contribution with the same opposing-channel sign pattern used for `temperature`, but magenta/green requires red and blue to move *together*, so the tint term's sign is backwards on one (or both) channels. | ✅ Root Cause | Confirmed by hand-deriving both channels' net sign for `tint = +1, temperature = 0`: `t.r` decreases, `t.b` decreases -- both down, i.e. toward green, opposite of the documented "positive = magenta." | E1, E2 |
| H2 | `temperature`'s own sign convention is also wrong, and the bug is broader than just `tint`. | ❌ Falsified | Hand-derived `temperature`'s isolated effect (`tint = 0`): positive `temperature` raises `t.r` and lowers `t.b` -- red up, blue down, which is warm/orange as documented. Confirmed correct; fix scoped to `tint` only. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/renderer/src/webgl/shaders/post_processing/color_grading.frag` (pre-fix, `apply_white_balance`) | `t.r += 0.2*temperature - 0.1*tint; t.b -= 0.2*temperature + 0.1*tint;` -- tint's coefficient is subtracted on `t.r` and added (under the `-=`) on `t.b`, an opposing-channel pattern. | H1 ✅ |
| E2 | `module/helper/renderer/src/webgl/post_processing/color_grading.rs`, `ColorGradingParams::tint` doc comment | "White balance tint adjustment (-1.0 to 1.0, 0.0 is neutral) - Positive: magenta tint - Negative: green tint" -- states the intended direction the pre-fix formula contradicts. | H1 ✅ |
| E3 | Hand-derivation of `apply_white_balance( color, temperature, 0.0 )` | Isolating `temperature`'s own contribution confirms `t.r`/`t.b` move in opposition (warm raises red, lowers blue) exactly as its own inline comment states -- correct, unaffected by the fix. | H2 ❌ |

## Root Cause

```glsl
// before -- tint's sign opposes across r/b, copying temperature's (correct, but different) pattern
t.r += 0.2 * temperature - 0.1 * tint;
t.b -= 0.2 * temperature + 0.1 * tint;
```

Magenta requires red AND blue boosted *together* (same sign as `tint`); green requires both
suppressed together. `temperature`'s warm/cool instead requires red and blue to move in
*opposition*. The pre-fix code applied the opposition pattern to both parameters, which is only
correct for `temperature` -- `tint`'s contribution needed a matching, not opposing, sign on `t.r`
and `t.b`.

## Why Not Caught

No test exercised `apply_white_balance`'s tint direction prior to this bug. `color_grading_tests.rs`
only covers `ColorGradingParams`'s `Default`/`Clone` derive behavior, and GLSL ES 3.00 shaders have
no CPU-executable or `naga`-validatable path in this crate (`shader_validation_tests.rs`'s own
documented scope covers WGSL only), so a sign inversion visible only at shader-execution time had
no automated check that could have caught it.

## Fix Location

`module/helper/renderer/src/webgl/shaders/post_processing/color_grading.frag`,
`apply_white_balance`: changed `tint`'s sign to match on both channels (`t.r += ... + 0.1*tint`,
`t.b -= ... - 0.1*tint`), so positive `tint` boosts red and blue together (magenta) and negative
`tint` suppresses both together (green), leaving `temperature`'s own independent, correctly
opposing sign relationship unchanged.

## Prevention

3 new tests added, `module/helper/renderer/tests/webgl/white_balance.rs` (a line-for-line Rust
port of the fixed GLSL formula, per the BUG-158 precedent for GLSL-only shader math with no
CPU-side twin in `src/`): `positive_tint_shifts_toward_magenta_not_green`,
`negative_tint_shifts_toward_green_not_magenta` (both assert the documented red/blue direction and
that green itself is untouched by tint), and `temperature_direction_unaffected_by_the_tint_fix`
(regression guard confirming the fix didn't disturb `temperature`'s own, separately-verified-correct
sign relationship).

## Pitfall

Two parameters sharing a channel-adjustment formula can each need a *different* sign relationship
between the channels they touch. Copying one parameter's sign pattern onto a second parameter
without independently checking that second parameter's own intended channel relationship is exactly
how this bug was introduced -- `temperature`'s opposing-channel pattern was correct for
*temperature*, but wrong once reused verbatim for `tint`.

## Generalized Version

**Broken assumption:** "two color-grading parameters adjusting the same two channels can share one
sign pattern, since they're structurally similar-looking formulas."

**Confirmed general rule:** When two parameters share a formula shape (same channels, same
operator structure), each parameter's sign relationship between those channels must be derived
independently from its own documented semantics -- structural similarity between two formulas is
not evidence they share the same sign convention.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Found during continued review of `module/helper/renderer`'s post-processing shaders; confirmed via hand-derivation of both `tint`'s and `temperature`'s isolated channel effects against `ColorGradingParams`'s own doc comment. |
| 2026-08-16 | fixed | Flipped `tint`'s sign on both `t.r`/`t.b` lines in `apply_white_balance` so both channels move together for `tint`, while leaving `temperature`'s own (already-correct) opposing sign untouched; full `Fix(BUG-178)` comment block added directly above the changed lines. |
| 2026-08-16 | verified | New file `tests/webgl/white_balance.rs` (Rust port of the fixed formula, 3 tests) -- all pass (`cargo nextest run -p renderer --all-features webgl::white_balance`: 3/3). Full workspace verification: `cargo nextest run --workspace --all-features --exclude object_picking`: 1897/1897 passed, 0 skipped (includes the 3 new tests). `cargo test --doc --workspace --all-features --exclude object_picking`: all crates `test result: ok`, 0 failed. `cargo clippy --workspace --all-targets --all-features --exclude object_picking -- -D warnings`: clean, zero warnings. `--exclude object_picking` applied because a concurrent actor's in-flight refactor of that unrelated example (`git status`/mtime confirmed: both files modified minutes before this run, well after this fix's own files) tripped clippy's `too_many_lines` threshold -- not a regression from this fix. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote 3 differential tests against a direct Rust port of the fixed formula, asserting the documented magenta/green direction plus a temperature-independence regression guard. Adversarial pass re-checked whether the `--exclude object_picking` workaround was evidence-based rather than habitual: `git status --porcelain` + `stat` mtimes confirmed `object_picking/{src/main.rs,Cargo.toml}` were modified by a different actor at 10:42, after this fix's own files (10:30) and after the verification launch itself (10:39) -- genuinely a live concurrent edit, not a rationalization. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-158 (same GLSL-has-no-CPU-path testing pattern, disjoint shader/crate) -- correctly cited as a precedent, not a duplicate. No other open or closed bug touches `color_grading.frag`. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by independent hand-derivation of both `tint`'s and `temperature`'s isolated channel effects against the parameter's own doc comment, not inferred from the diff's shape alone. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is a 2-line sign change plus explanatory comment; no unrelated refactor attempted. Did not touch any other `apply_*` function in the same shader file (`apply_tonal_adjustments`, `apply_filmic_curve`, `adjust_vibrance`, `adjust_saturation`) since none were implicated by this bug's own hypothesis or evidence. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Entirely within `renderer`'s own shader file plus its own new test file; no downstream call sites needed updating (`tint` is uploaded to the shader uniform unmodified by `color_grading.rs`, confirmed by direct read). | — |
| D7 | Crate Locality | 🟢 | 🟢 | Confirmed via read that `apply_white_balance` has exactly one definition site, already fixed. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | The fix corrects the function's existing, self-documented responsibility (apply white-balance adjustment matching each parameter's own documented direction); no responsibility added or removed. | — |

**Reproduced:** YES -- pre-fix, the Rust port of `apply_white_balance( [0.5,0.5,0.5], 0.0, 1.0 )`
decreases red and blue (toward green); post-fix, it increases both (toward magenta), matching the
documented contract. All 3 new tests pass post-fix. Full workspace suite (1897/1897, 0 skipped, +3
new), doctests (0 failed across every crate), and clippy all clean (excluding the concurrent
actor's unrelated `object_picking` in-flight refactor), 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/shaders/post_processing/color_grading.frag` | `apply_white_balance`: flipped `tint`'s sign on both `t.r += ...` and `t.b -= ...` lines so both channels move together for `tint` (magenta/green) while `temperature`'s independent opposing-sign relationship (warm/cool) is unchanged; full `Fix(BUG-178)` comment block added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/webgl/white_balance.rs` | New file, 3 tests: a Rust port of the fixed `apply_white_balance` formula plus tests asserting the documented magenta/green tint direction and temperature's unaffected warm/cool direction. |
| `module/helper/renderer/tests/webgl/mod.rs` | Added `mod white_balance;` registration. |
| `module/helper/renderer/tests/readme.md` | Added `webgl/white_balance.rs` Responsibility Table row. |
