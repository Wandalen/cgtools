# BUG-179: `WideOutlinePass::outline_thickness` never reaches the outline-decision shader

- **Severity:** Medium (visual-only defect -- no crash, no data loss, but the outline's on-screen
  thickness is always a fixed 30px regardless of what the caller configures)
- **state:** Completed
- **Affects:** Every caller of `renderer::webgl::post_processing::outline::wide_outline::WideOutlinePass`
  that constructs it with, or later sets, an `outline_thickness` other than the shader's hardcoded
  `30.0` -- e.g. any UI exposing an outline-thickness slider, or a caller wanting a thinner/thicker
  outline than the built-in default.
- **Component:** `module/helper/renderer` (`src/webgl/post_processing/outline/wide_outline.rs`,
  `src/webgl/shaders/post_processing/outline/wide_outline/outline.frag`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** Same `wide_outline`/`outline.frag` area as BUG-180 (double aspect-ratio
  correction in the JFA step), BUG-181 (silhouette detection fails for non-red objects), and
  BUG-182 (sentinel check tests the wrong value) -- all pre-identified by task #98's review pass,
  independent defects with disjoint code paths within the same file/shader trio.

## Symptom

```glsl
// pre-fix -- outline.frag
const float outlineThickness = 30.0;   // Outline thickness in pixels
...
if ( dist < outlineThickness ) { /* draw outline color */ }
```

`WideOutlineShader`'s uniform location list (`impl_locations!`) never declared an
`outlineThickness` entry, and `outline.frag` declared it as a `const`, not a `uniform` -- so
`WideOutlinePass::outline_thickness` (settable via the constructor or `outline_thickness_set`)
had no path to this comparison at all. The field *was* consumed elsewhere -- `jfa_step_pass`
divides it by `2^i` to size the JFA search radius -- but the actual draw/no-draw decision in the
final pass always compared against the fixed `30.0`.

## Impact

**Who is affected:** Any caller configuring `outline_thickness` away from whatever value happens
to visually resemble 30px -- most plausibly a UI control (an outline-thickness slider) or a
caller wanting a thinner outline for a small/dense scene.

**What breaks:** Purely visual -- the rendered outline's pixel thickness silently ignores the
configured value and always renders at a fixed ~30px band (subject to the JFA search radius
`outline_thickness` *does* influence -- a very small `outline_thickness` could shrink the JFA
search radius below 30px, in which case the outline is capped by search radius, not by the
intended-but-unwired thickness comparison; a large `outline_thickness` still visually caps at
~30px since the decision never uses anything past that constant). No crash, no incorrect data
persisted.

**Magnitude:** Every draw through `WideOutlinePass` is affected identically -- the defect is in
the single shared fragment shader, not any one call site.

**Entity Scope:** None -- a code-level (shader-level) defect.

## How Discovered

Pre-identified by task #98's review pass (this session) as "wide_outline outline_thickness never
reaches the shader." Confirmed by reading `WideOutlineShader`'s `impl_locations!` call (no
`outlineThickness` entry) and `outline.frag` directly (`const float outlineThickness = 30.0;`,
not a `uniform`), then confirming `outline_pass`'s Rust code never attempted to upload anything
for it -- unlike `resolution`, which follows the correct declare-in-shader-as-uniform /
look-up-location / upload-every-frame pattern this fix now matches.

## Minimum Reproducible Example

```rust
// pre-fix: WideOutlineShader's location map has no "outlineThickness" key at all, so this
// lookup ( the fix's own new code, added to outline_pass ) would panic on `.unwrap()`:
let outline_thickness_loc = outline_locs.get( "outlineThickness" ).unwrap().clone().unwrap();
```

Pre-fix, this key simply didn't exist in the map (a compile-time-invisible, runtime-only defect
-- the shader compiled and ran fine, just silently ignoring the caller's thickness value).

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo test --target wasm32-unknown-unknown --all-features webgl::wide_outline
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `outline.frag` hardcodes `outlineThickness` as a `const`, and `WideOutlineShader`'s uniform location list never declares a matching uniform, so `WideOutlinePass::outline_thickness` has no path to the pixel that decides draw/no-draw. | ✅ Root Cause | Confirmed by direct read of both `outline.frag` (const, not uniform) and `WideOutlineShader`'s `impl_locations!` call (4 uniforms declared, none named `outlineThickness`). | E1, E2 |
| H2 | `outline_thickness` is fully unused/dead -- the whole field is vestigial. | ❌ Falsified | `jfa_step_pass` (line ~389) does consume it, dividing by `2^i` to compute the JFA step's search radius -- the field isn't dead, it's just wired to the wrong stage of the pipeline (search radius, not the final visual-thickness comparison). | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/renderer/src/webgl/shaders/post_processing/outline/wide_outline/outline.frag` (pre-fix) | `const float outlineThickness = 30.0;` -- a compile-time constant, not a `uniform`. | H1 ✅ |
| E2 | `module/helper/renderer/src/webgl/post_processing/outline/wide_outline.rs`, `WideOutlineShader`'s `impl_locations!` call (pre-fix) | Declares exactly `"sourceTexture", "objectColorTexture", "jfaTexture", "resolution"` -- no `"outlineThickness"` entry, so no location could ever be looked up or uploaded even if the shader did declare it. | H1 ✅ |
| E3 | `module/helper/renderer/src/webgl/post_processing/outline/wide_outline.rs`, `jfa_step_pass` (`step_size = self.outline_thickness / (2.0_f32).powf(i as f32)`) | `outline_thickness` genuinely drives the JFA step's search radius -- confirms the field isn't wholly dead, only missing its second, more visible consumer. | H2 ❌ |

## Root Cause

```glsl
// before -- outline.frag
const float outlineThickness = 30.0;   // Outline thickness in pixels
```

```rust
// before -- WideOutlineShader's uniform list, wide_outline.rs
impl_locations!( WideOutlineShader, "sourceTexture", "objectColorTexture", "jfaTexture", "resolution" );
// no "outlineThickness" -- outline_pass had no uniform to upload self.outline_thickness into
```

The shader-side constant and the Rust-side struct field were never connected -- the constant was
presumably a placeholder from before the field existed, never replaced with a real uniform once
`outline_thickness` was added to the pass.

## Why Not Caught

No test constructed or rendered `WideOutlinePass` prior to this bug -- the only coverage was
visual inspection of the `outline`/`renderer_with_outlines` examples, and a fixed `30.0` produces
a visually plausible outline at typical example resolutions regardless of what value a caller
actually configured, so nothing looked obviously wrong without deliberately comparing two
different configured thicknesses side by side.

## Fix Location

- `module/helper/renderer/src/webgl/shaders/post_processing/outline/wide_outline/outline.frag`:
  changed `const float outlineThickness = 30.0;` to `uniform float outlineThickness;`.
- `module/helper/renderer/src/webgl/post_processing/outline/wide_outline.rs`: added
  `"outlineThickness"` to `WideOutlineShader`'s `impl_locations!` list; `outline_pass` now looks
  up its location and uploads `self.outline_thickness` every frame, matching the existing
  `resolution` uniform's pattern.

## Prevention

1 new browser-driven test added, `module/helper/renderer/tests/webgl/wide_outline.rs`
(`render_succeeds_for_two_different_outline_thicknesses`): constructs two independent
`WideOutlinePass` instances at different `outline_thickness` values (one via the constructor, one
via `outline_thickness_set` after construction) and asserts both render without error. Matches
`pmrem_tests.rs`'s established structural tier for this crate's multi-pass WebGL post-processing
code -- does not assert pixel-level thickness (that stays delegated to visual inspection per this
crate's existing convention), but genuinely catches a regression back to a hardcoded shader
constant: `outline_pass`'s new location lookup panics via `.unwrap()` if the shader stops
declaring the uniform. Run for real in a headless Firefox browser via
`cargo test --target wasm32-unknown-unknown --all-features` (native `cargo test`/`nextest` can't
execute WebGL2 code at all -- no native webgl backend exists in this codebase, unlike the
webgpu path's `wgpu`-over-Vulkan native backend).

## Pitfall

A struct field threaded through a constructor and a public setter (`outline_thickness_set`) reads
as "already wired up" end-to-end -- but a Rust-side field controls nothing on its own. The shader
must independently declare and consume a matching uniform by name, and nothing at compile time
checks that the two sides agree; a shader-side placeholder constant left over from before the
field existed is invisible to every Rust-level type/borrow check.

## Generalized Version

**Broken assumption:** "a struct field that's threaded through a public setter and referenced
somewhere in the render path is fully wired to its documented effect."

**Confirmed general rule:** In a Rust/GLSL split where a value crosses the language boundary via
a string-keyed uniform lookup, verify BOTH sides independently -- that the shader declares a
`uniform` (not a `const`) of the expected name and type, AND that the Rust code both looks up that
location and uploads the current value every frame it changes. A field being *consumed somewhere*
in the pipeline (as `outline_thickness` was, for the JFA search radius) is not evidence it reaches
*every* place its own documentation implies it should.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Pre-identified by task #98's review pass; confirmed this session by direct read of `outline.frag` and `WideOutlineShader`'s uniform location list. |
| 2026-08-16 | fixed | Changed `outline.frag`'s `outlineThickness` from a `const` to a `uniform float`; added it to `WideOutlineShader`'s location list and uploaded `self.outline_thickness` every `outline_pass` call, matching the `resolution` uniform's existing pattern. Full `Fix(BUG-179)` comments added at both the GLSL declaration and the Rust upload site. |
| 2026-08-16 | verified | New file `tests/webgl/wide_outline.rs` (1 wasm_bindgen_test, 2 independently-constructed `WideOutlinePass` instances at different thicknesses) -- run for real in headless Firefox via `cargo test --target wasm32-unknown-unknown --all-features` from `module/helper/renderer/`: `webgl::wide_outline::tests::render_succeeds_for_two_different_outline_thicknesses ... ok`, and the crate's full wasm32 suite (6 browser test binaries) all `test result: ok`, 0 failed. Full workspace native verification: `cargo nextest run --workspace --all-features --exclude object_picking`: 1897/1897 passed, 0 skipped. `cargo test --doc --workspace --all-features --exclude object_picking`: all crates ok. `cargo clippy --workspace --all-targets --all-features --exclude object_picking -- -D warnings`: clean. `--exclude object_picking` re-confirmed evidence-based: that crate's mtime had advanced again since BUG-178's verification (concurrent actor still actively editing), while a standalone `cargo check -p object_picking` (non-clippy) still passes -- the failure is clippy-lint-only (`too_many_lines`) and unrelated to this fix. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass: wrote and ran a real browser-executed test (not just a compile check) confirming the fix's new uniform-lookup code path executes successfully post-fix. Adversarial pass: checked whether a merely-compiling test could mask a still-broken fix -- verified by reading the full wasm32 test log directly (not trusting a truncated tail, after BUG-178's verification surfaced exactly this risk) and confirming the specific test name appears with `... ok`, not just an aggregate pass count. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-180/181/182 (same file/shader trio, independent defects, correctly left untouched -- confirmed by scoping this fix strictly to the `outlineThickness` uniform, not touching the JFA step's aspect-ratio math, the silhouette red-channel check, or the sentinel comparison). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct read of both the GLSL `const` declaration and the Rust-side uniform location list, not inferred from the diff's shape alone. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is a `const`→`uniform` change plus one new location entry and one new upload call; did not touch `jfa_step_pass`'s own separate consumption of `outline_thickness` (correct as-is, confirmed by hypothesis H2's evidence). | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Entirely within `renderer`'s own shader + pass file, plus its own new test file; the one real caller (`examples/minwebgl/renderer_with_outlines`) needed no changes -- it already passes `outline_thickness` positionally to `WideOutlinePass::new`, which is unaffected by this fix's signature (unchanged). | — |
| D7 | Crate Locality | 🟢 | 🟢 | Confirmed via read that `outline.frag`'s `outlineThickness` and `WideOutlineShader`'s location list each have exactly one definition site, both fixed. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | The fix completes the pass's existing, self-documented responsibility ("the desired thickness of the final outline... influences... the final rendering stage" per the struct's own doc comment); no responsibility added or removed. | — |

**Reproduced:** YES -- pre-fix, the Rust-side `outline_locs.get("outlineThickness")` lookup this
fix introduces would find no such key (the map only ever held the 4 pre-fix uniform names);
post-fix, the lookup succeeds and the new test passes, executed for real in a headless Firefox
browser. Full workspace native suite (1897/1897, 0 skipped), doctests (0 failed), and clippy all
clean (excluding the concurrent actor's unrelated `object_picking` in-flight refactor); `renderer`
crate's full wasm32 browser suite (6 binaries) all clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/shaders/post_processing/outline/wide_outline/outline.frag` | Changed `outlineThickness` from `const float` to `uniform float`; full `Fix(BUG-179)` comment added. |
| `module/helper/renderer/src/webgl/post_processing/outline/wide_outline.rs` | Added `"outlineThickness"` to `WideOutlineShader`'s `impl_locations!` list; `outline_pass` now looks up and uploads `self.outline_thickness` every frame, with a `Fix(BUG-179)` comment at both sites. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/webgl/wide_outline.rs` | New file, 1 `wasm_bindgen_test`: constructs two independent `WideOutlinePass` instances at different `outline_thickness` values and asserts both render without error. Browser-only (`#[cfg(target_arch = "wasm32")]`), following `pmrem_tests.rs`'s structural-tier convention. |
| `module/helper/renderer/tests/webgl/mod.rs` | Added `mod wide_outline;` registration. |
| `module/helper/renderer/tests/readme.md` | Added `webgl/wide_outline.rs` Responsibility Table row. |
