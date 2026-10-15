# BUG-193: `outline.frag` duplicates BUG-181's silhouette-detection threshold defect

- **Severity:** Medium (visual-only defect -- no crash, no data loss, but any object whose color
  isn't sufficiently red draws the plain background/source color over its own pixels instead of
  its actual rendered appearance)
- **state:** Completed
- **Affects:** Every caller of `renderer::webgl::post_processing::outline::wide_outline::WideOutlinePass`
  where the `object_color_texture` input holds a non-red object color (pure green/blue/cyan, or
  even ordinary black).
- **Component:** `module/helper/renderer` (`src/webgl/shaders/post_processing/outline/wide_outline/outline.frag`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** Duplicate of BUG-181's `jfa_init.frag` defect, independently re-occurring in
  `outline.frag`'s own object-vs-background branch -- same GBuffer `OBJECT_COLOR` contract, same
  fix shape, discovered while diagnosing BUG-182 (same file, adjacent check) and fixed in the same
  edit pass. Also related to BUG-179/BUG-180 (same `wide_outline` shader trio, both independent,
  already fixed).

## Symptom

```glsl
// pre-fix -- outline.frag
float objectColorR = texture( objectColorTexture, vUv ).r;
if ( objectColorR > 0.01 ) // If the pixel is part of the object silhouette
```

`outline.frag` independently re-implements the same "is this pixel part of an object" check that
`jfa_init.frag` had (BUG-181), deciding whether to draw the pixel with its own rendered color or
fall through to the JFA outline-distance path. Like BUG-181, this reads the `OBJECT_COLOR` GBuffer
attachment, which `gbuffer.rs`'s `GBuffer::render` clears to `(-1, -1, -1, 1)` and `gbuffer.frag`
writes the caller's arbitrary `objectColor` uniform to verbatim -- so `> 0.01` only matches objects
whose red channel happens to be close to `1.0`, true only by coincidence of the one real caller
(`renderer_with_outlines`) currently hardcoding every object to red.

## Impact

**Who is affected:** Any caller supplying an `object_color_texture` where object pixels' red
channel is `<= 0.01` -- pure green, pure blue, cyan, or plain black objects, all common,
legitimate colors. Identical affected population to BUG-181.

**What breaks:** Purely visual -- for an affected-colored object, this branch's condition is
false, so the pixel falls through to the JFA distance-based outline/background logic instead of
drawing `texture( sourceTexture, vUv )` (the object's own rendered appearance). In practice this
means the object's own interior pixels get treated as candidate outline/background pixels rather
than the object's actual rendered color -- a visibly wrong result distinct from BUG-181's
"missing outline" symptom (BUG-181's `jfa_init.frag` defect stops the object from ever seeding
the JFA at all; this defect is a second, independent check in the pass's final draw step).

**Magnitude:** Every affected-colored object is affected identically, every frame, for as long as
the caller supplies that color -- not intermittent or frame-dependent. Compounds with BUG-181
whenever both defects are unfixed simultaneously (they were fixed together in this session).

**Entity Scope:** None -- a code-level (shader-level) defect.

## How Discovered

Not part of any pre-identified bug list. Surfaced while diagnosing BUG-182 (`outline.frag`'s
seed-sentinel check, a few lines below this one in the same file): reading the full file to
scope BUG-182's fix revealed this earlier `objectColorR > 0.01` check is the same defect pattern
already fixed under BUG-181 in `jfa_init.frag`, independently duplicated here. Confirmed via the
same GBuffer clear/write evidence already established for BUG-181 (`gbuffer.rs`'s clear call,
`gbuffer.frag`'s write) -- no new investigation needed, since the contract is identical.

## Minimum Reproducible Example

```glsl
// pre-fix: a pure-green object color, e.g. objectColor = vec4(0.0, 1.0, 0.0, 1.0)
float objectColorR = texture( objectColorTexture, vUv ).r;  // samples 0.0 for this object's pixels
if ( objectColorR > 0.01 )  // 0.0 > 0.01 is false -- object's own pixel falls through to JFA path
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo nextest run --all-features webgl::jfa_silhouette::outline_frag
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `outline.frag`'s `> 0.01` check is the same defect as BUG-181, against the same GBuffer contract, independently duplicated rather than shared code. | ✅ Root Cause | Confirmed by reading `outline.frag`'s check text (`objectColorR > 0.01`) against the already-established `gbuffer.rs`/`gbuffer.frag` clear/write contract from BUG-181's investigation -- identical value space, identical wrong threshold. | E1, E2, E3 |
| H2 | `outline.frag`'s check is intentionally different from `jfa_init.frag`'s (e.g. a deliberately looser/tighter threshold for this pass's own purposes), so fixing BUG-181 alone was sufficient. | ❌ Falsified | The two checks are textually near-identical (`objectPresent`/`objectColorR`, `.r`, `> 0.01`) with no accompanying comment or logic suggesting an intentional difference -- both read the same texture under the same contract for the same purpose (distinguish object pixel from background). | E1, E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/renderer/src/webgl/post_processing/gbuffer.rs` (`GBuffer::render`, clear calls) | `gl.clear_bufferfv_with_f32_array( gl::COLOR, 4, [ -1.0, -1.0, -1.0, 1.0 ].as_slice() )` -- the `OBJECT_COLOR` attachment ( index 4 ) clears to a negative RGB sentinel, the same contract `outline.frag` reads from. | H1 ✅ |
| E2 | `module/helper/renderer/src/webgl/shaders/post_processing/gbuffer.frag` | `FragObjectColor = objectColor;` -- writes the real, arbitrary per-draw `objectColor` uniform verbatim to every object pixel, the same value `outline.frag`'s check samples. | H1 ✅, H2 ❌ |
| E3 | `module/helper/renderer/src/webgl/shaders/post_processing/outline/wide_outline/outline.frag` (pre-fix) | `float objectColorR = texture( objectColorTexture, vUv ).r; if ( objectColorR > 0.01 )` -- textually the same threshold-on-red-channel pattern as BUG-181's pre-fix `jfa_init.frag` check. | H1 ✅ |

## Root Cause

```glsl
// before
float objectColorR = texture( objectColorTexture, vUv ).r;
if ( objectColorR > 0.01 )
```

Same root cause as BUG-181: the check assumed "object present" looks like "red channel near
1.0," an assumption that happens to hold only for the one existing caller
(`renderer_with_outlines`'s red-hardcoded `object_colors_generate`), not for the actual
clear/write contract. This is a second, independent occurrence of that exact reasoning error in
a different file, not a regression of BUG-181's fix.

## Why Not Caught

Same as BUG-181: the one real caller hardcodes every object's color to red, so both copies of the
defect were invisible until a caller used a non-red object color. This copy specifically survived
even after BUG-181's fix landed, because the two checks live in different files
(`jfa_init.frag` vs. `outline.frag`) with no shared code path -- fixing one does not fix the
other, and nothing short of re-reading the sibling file for the same pattern would surface it.

## Fix Location

`module/helper/renderer/src/webgl/shaders/post_processing/outline/wide_outline/outline.frag`:
changed the comparison from `objectColorR > 0.01` to `objectColorR >= 0.0`, with a `Fix(BUG-193)`
comment cross-referencing BUG-181's identical fix and explaining the shared clear-value/write-value
contract.

## Prevention

1 new native unit test added directly to the existing `module/helper/renderer/tests/webgl/jfa_silhouette.rs`
(BUG-181's test file), reusing its `object_is_present` port function -- widened from private to
`pub( crate )` -- rather than duplicating an identical formula in a new file or new assertions,
per this codebase's Anti-Duplication convention: `outline.frag`'s check is textually identical to
`jfa_init.frag`'s post-fix check, so one shared, tested model covers both call sites. The new test
(`outline_frag_object_presence_check_matches_jfa_init_frag`) exercises the same three cases
(black object present, sentinel excluded, red object present) explicitly documented as covering
`outline.frag`'s call site. Additionally, re-ran the existing BUG-179 `wide_outline` browser test
(`cargo test --target wasm32-unknown-unknown --all-features webgl::wide_outline`) after this edit
(applied in the same pass as BUG-182's fix to the same file), confirming the hand-edited GLSL
still compiles and links successfully in a real WebGL2 context.

## Pitfall

Fixing one occurrence of a defect pattern does not fix every occurrence -- when a bug's root
cause is a specific wrong-comparison idiom (here, a magnitude threshold standing in for a sign
check), the same idiom can be independently duplicated elsewhere in the same shader trio without
any shared code path connecting the two. After fixing an instance, searching for the same literal
pattern (`> 0.01` against this texture) elsewhere in sibling files is what surfaced this one.

## Generalized Version

**Broken assumption:** "this bug was isolated to the one file/check I found it in."

**Confirmed general rule:** When a shader (or any code without shared subroutines) reimplements
the same conceptual check in more than one place, fixing one occurrence does not fix the others --
after root-causing a defect pattern, grep sibling files in the same subsystem for the same
literal comparison before considering the bug closed.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered while diagnosing BUG-182 in the same file; confirmed via the GBuffer clear/write contract already established for BUG-181. Allocated bug ID 193 (`task/readme.md`'s `highest_id` was 192; confirmed 190-193 unused via grep/find before allocating). |
| 2026-08-16 | fixed | Changed `outline.frag`'s comparison from `objectColorR > 0.01` to `objectColorR >= 0.0`, in the same edit pass as BUG-182's fix to the same file. Full `Fix(BUG-193)` comment added at the fix site, cross-referencing BUG-181. |
| 2026-08-16 | verified | Added `outline_frag_object_presence_check_matches_jfa_init_frag` to the existing `tests/webgl/jfa_silhouette.rs` (widened `object_is_present` to `pub( crate )`, updated the file's header doc comment to cover both call sites) -- `cargo nextest run --all-features webgl::jfa_silhouette` from `module/helper/renderer/`: 4/4 passed (1 pre-existing BUG-181 file plus this new test). Re-ran the existing BUG-179 wasm32 browser test (`webgl::wide_outline`) to confirm the hand-edited GLSL still compiles/links/runs: 1/1 passed. `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: clean. Full workspace: `cargo nextest run --workspace --all-features --exclude object_picking`: 1906/1906 passed, 0 skipped (up from 1902 -- this bug's 1 new test plus BUG-182's 3 new tests, fixed and verified in the same pass). `cargo test --doc --workspace --all-features --exclude object_picking`: all crates ok. `cargo clippy --workspace --all-targets --all-features --exclude object_picking -- -D warnings`: clean. `--exclude object_picking` re-confirmed evidence-based: working tree still dirty from the concurrent actor's own unrelated in-progress work; `cargo check -p object_picking` (non-clippy) still passes clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming: reused BUG-181's already-verified clear/write contract evidence rather than re-deriving it, since the contract is identical. Adversarial: checked whether `outline.frag`'s check might have a subtly different purpose justifying a different threshold (e.g. anti-aliased edge blending) -- no such logic exists in the surrounding code; the branch is a plain binary object/background split, same as `jfa_init.frag`'s. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-181 (the original occurrence of this exact defect pattern, already fixed) and BUG-182 (same file, adjacent check, fixed in the same edit pass, filed separately). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct comparison of `outline.frag`'s pre-fix text against BUG-181's already-established root cause, not assumed by pattern-name alone. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is a single comparison-operator and threshold change; no other logic in the file touched. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Entirely within `renderer`'s own shader file and its existing (BUG-181) test file, extended rather than duplicated. | — |
| D7 | Crate Locality | 🟢 | 🟢 | This specific check has exactly one definition site (`outline.frag`'s own object-vs-background branch), fixed there -- distinct from `jfa_init.frag`'s separate definition site, already fixed under BUG-181. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix completes the pass's own documented responsibility (draw the object's real color for its own pixels) without adding or removing scope. | — |

**Reproduced:** YES -- `object_is_present(0.0)` (the pre-fix formula's blind spot: `0.0 > 0.01`
is false) now correctly returns `true` post-fix (`0.0 >= 0.0` is true), encoded as
`jfa_silhouette.rs`'s new `outline_frag_object_presence_check_matches_jfa_init_frag` regression
test (passing, part of 4/4 in that file). Existing BUG-179 browser test re-run to confirm the
hand-edited GLSL still compiles/links/runs in a real WebGL2 context (1/1 passing). Full workspace
native suite (1906/1906, 0 skipped), doctests (0 failed), and clippy all clean (excluding the
concurrent actor's unrelated `object_picking` in-flight refactor), 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/shaders/post_processing/outline/wide_outline/outline.frag` | Changed the object-presence check from `objectColorR > 0.01` to `objectColorR >= 0.0`; full `Fix(BUG-193)` comment added cross-referencing BUG-181's identical fix. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/webgl/jfa_silhouette.rs` | Widened `object_is_present` from private to `pub( crate )`; added `outline_frag_object_presence_check_matches_jfa_init_frag` test; updated header doc comment to cover both `jfa_init.frag` (BUG-181) and `outline.frag` (BUG-193) call sites. |
| `module/helper/renderer/tests/readme.md` | Updated `webgl/jfa_silhouette.rs` Responsibility Table row to note both call sites are covered. |
