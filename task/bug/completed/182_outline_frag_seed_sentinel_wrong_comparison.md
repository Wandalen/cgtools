# BUG-182: `outline.frag`'s JFA seed-validity check tests the wrong sentinel value

- **Severity:** Medium (visual-only defect -- no crash, no data loss, but can both draw a spurious
  outline at the wrong distance and drop a legitimate outline pixel sitting exactly on a `0.0`
  coordinate)
- **state:** Completed
- **Affects:** Every caller of `renderer::webgl::post_processing::outline::wide_outline::WideOutlinePass`
  -- the defect sits in the pass's own final draw step, not in any caller-supplied input.
- **Component:** `module/helper/renderer` (`src/webgl/shaders/post_processing/outline/wide_outline/outline.frag`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** Same `wide_outline` shader trio as BUG-179, BUG-180, and BUG-181 (all fixed,
  independent). Discovered while diagnosing this bug: BUG-193, a duplicate of BUG-181's
  `objectPresent` sentinel defect independently re-occurring earlier in this same `outline.frag`
  file -- fixed in the same edit pass, filed and verified separately.

## Symptom

```glsl
// pre-fix -- outline.frag
vec2 seedCoord = texture( jfaTexture, vUv ).xy;
// Check if a valid seed coordinate was found ( i.e., not the sentinel value -1.0 ).
if ( seedCoord.x != 0.0 && seedCoord.y != 0.0 )
```

The comment names the sentinel as `-1.0`, but the code checks `!= 0.0` -- an inequality against
zero, not a test against the actual sentinel. `jfa_init.frag`/`jfa_step.frag` write
`vec4(-1.0, -1.0, -1.0, 1.0)` for pixels with no seed found, and a real found seed is always a
non-negative UV coordinate (`vec4(vUv, 0.0, 1.0)`). `!= 0.0` is neither necessary nor sufficient
for that sentinel: `-1.0 != 0.0` is true, so the real sentinel incorrectly passes as "valid"; and
a legitimate seed coordinate landing exactly on `0.0` on either axis (`0.0 != 0.0` is false) is
incorrectly rejected as if it were the sentinel.

## Impact

**Who is affected:** Every caller of `WideOutlinePass` -- this is the pass's own internal
draw-decision logic, not something a caller's input can avoid.

**What breaks:** Two independent failure modes from the same wrong comparison:
1. A pixel that legitimately found no nearby seed ( the true `(-1,-1)` sentinel ) incorrectly
   enters the "valid seed" branch and computes `distance( vUv * resolution, seedCoord * resolution )`
   against the bogus `(-1,-1)` position. In practice this distance is at least the screen's own
   diagonal in pixels, which exceeds any realistic `outlineThickness`, so the branch still falls
   through to drawing the background color -- visually masked in typical scenes, but silently
   relying on the outline thickness never being configured absurdly large rather than on the
   check being correct.
2. A pixel whose real nearest-seed coordinate happens to land exactly on `0.0` on either axis is
   incorrectly treated as "no seed found" and drawn as far background instead of being
   distance-tested against `outlineThickness` -- a genuine (if narrow) case of a dropped outline
   pixel, most plausible for silhouettes touching the UV origin edge.

**Magnitude:** Failure mode 1 is masked in ordinary configurations; failure mode 2 is a real,
reproducible defect whenever a found seed coordinate is exactly `0.0` on an axis.

**Entity Scope:** None -- a code-level (shader-level) defect.

## How Discovered

Pre-identified by task #98's review pass (this session) as "outline.frag's sentinel check tests
wrong value." Confirmed by reading `outline.frag`'s seed-validity check together with its own
comment (naming the sentinel as `-1.0`) and cross-referencing `jfa_init.frag`/`jfa_step.frag`'s
actual sentinel-write value (`vec4(-1.0, -1.0, -1.0, 1.0)`, confirmed independently while
diagnosing BUG-181 in the same shader trio) -- the comparison operator (`!= 0.0`) and comparison
target (`0.0`) both diverge from what the comment itself already names as correct.

## Minimum Reproducible Example

```glsl
// pre-fix: a legitimately-found seed coordinate landing exactly on the UV origin's x-axis
vec2 seedCoord = vec2( 0.0, 0.42 );          // a real, found seed -- not the sentinel
if ( seedCoord.x != 0.0 && seedCoord.y != 0.0 )  // 0.0 != 0.0 is false -- incorrectly rejected
// falls through to "far background" even though a real seed was found nearby
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo nextest run --all-features webgl::outline_seed_sentinel
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The check should test against the actual `-1.0` sentinel via a sign comparison (`>= 0.0`), matching the sentinel the code's own comment already names -- `!= 0.0` is simply the wrong operator/operand for that intent. | ✅ Root Cause | Confirmed directly: `jfa_init.frag`/`jfa_step.frag` write `(-1,-1,-1,1)` for no-seed pixels and non-negative UV coordinates otherwise; `!= 0.0` neither excludes the former nor admits every instance of the latter. | E1, E2 |
| H2 | `!= 0.0` is an intentional "not the default/uninitialized value" idiom and is functionally equivalent to a sentinel check for this texture's actual value range. | ❌ Falsified | The value range is `{-1.0} ∪ [0.0, 1.0]` (sentinel or non-negative UV), not `{0.0} ∪ (0.0, ...]` -- `0.0` is itself a legitimate non-sentinel value here, so an inequality-with-zero test is not equivalent to a sentinel check in this value space. | E1, E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/renderer/src/webgl/shaders/post_processing/outline/wide_outline/jfa_init.frag` | Writes `vec4(-1.0, -1.0, -1.0, 1.0)` for no-seed pixels and `vec4(vUv, 0.0, 1.0)` (always non-negative) for found-seed pixels -- confirms the real sentinel is `-1.0`, and `0.0` is an ordinary in-range UV value, not a marker. | H1 ✅, H2 ❌ |
| E2 | `module/helper/renderer/src/webgl/shaders/post_processing/outline/wide_outline/outline.frag`'s own comment on the check (`// ... i.e., not the sentinel value -1.0`) | The code's own comment already names the correct sentinel (`-1.0`), directly contradicting the `!= 0.0` comparison actually written beneath it. | H1 ✅ |

## Root Cause

```glsl
// before
if ( seedCoord.x != 0.0 && seedCoord.y != 0.0 )
```

The check compares against `0.0` with `!=` when the real discriminant is sign against `-1.0`. The
value space this texture actually produces is a negative sentinel or a non-negative UV coordinate
-- an inequality-with-zero test is neither necessary (rejects legitimate `0.0`-valued coordinates)
nor sufficient (admits the real `-1.0` sentinel) for that space.

## Why Not Caught

No test exercised this check prior to this bug. Failure mode 1 (the sentinel incorrectly passing
as valid) is visually self-masking in ordinary scenes, since the resulting bogus distance almost
always exceeds `outlineThickness` and still falls through to the same background-color result a
correct rejection would have produced -- so the wrong branch was taken but the *visible* output
usually matched what a correct check would have produced anyway, hiding the defect from casual
visual inspection. Failure mode 2 requires a found seed coordinate landing exactly on `0.0`, a
narrow (UV-origin-edge) case unlikely to appear in ad hoc manual testing.

## Fix Location

`module/helper/renderer/src/webgl/shaders/post_processing/outline/wide_outline/outline.frag`:
changed the comparison from `seedCoord.x != 0.0 && seedCoord.y != 0.0` to `seedCoord.x >= 0.0 &&
seedCoord.y >= 0.0`, with a `Fix(BUG-182)` comment explaining the actual sentinel-vs-real-coordinate
contract this now correctly tests against.

## Prevention

3 new native unit tests added, `module/helper/renderer/tests/webgl/outline_seed_sentinel.rs`.
Following this session's established convention for GLSL logic with no CPU-execution path
(GLSL ES 3.00 is outside naga's `glsl-in` front end, per `shader_validation_tests.rs`'s own scope
note), `seed_is_valid` is a line-for-line Rust port of the fixed shader check. Asserts: the real
`(-1,-1)` sentinel is rejected, a legitimately-found seed sitting exactly on `0.0` on either axis
is accepted (the case the pre-fix `!= 0.0` check would have wrongly rejected), and an ordinary
positive seed coordinate is unaffected. Additionally, re-ran the existing BUG-179 `wide_outline`
browser test (`cargo test --target wasm32-unknown-unknown --all-features webgl::wide_outline`)
after this edit, confirming the hand-edited GLSL still compiles and links successfully in a real
WebGL2 context.

## Pitfall

An inequality-with-zero test (`!= 0.0`) can look like a generic "not the default/empty value"
guard, but it silently assumes zero is both the only sentinel value and never itself a legitimate
value -- neither holds here: the real sentinel is `-1.0`, and `0.0` is an ordinary, reachable
coordinate. When a comment already names the actual sentinel, that's a direct signal to verify
the code beneath it tests against that same value, not a proxy for it.

## Generalized Version

**Broken assumption:** "checking `!= 0.0` is a safe, general way to test whether a value is 'the
default/unset case' versus 'a real value.'"

**Confirmed general rule:** A sentinel check must compare against the sentinel's actual value, not
against a conventionally-common placeholder like `0.0` -- especially when `0.0` is itself inside
the type's legitimate value range. When a comment already documents the sentinel value, treat any
divergence between the comment and the code's actual comparison target as a concrete signal to
re-derive the check from the producer's real write contract, not from convention.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Pre-identified by task #98's review pass; confirmed this session by reading `outline.frag`'s seed-validity check against its own comment and `jfa_init.frag`/`jfa_step.frag`'s actual sentinel-write contract (already established while diagnosing BUG-181 in the same shader trio). |
| 2026-08-16 | fixed | Changed `outline.frag`'s comparison from `seedCoord.x != 0.0 && seedCoord.y != 0.0` to `seedCoord.x >= 0.0 && seedCoord.y >= 0.0`, matching the actual sentinel-vs-real-coordinate contract. Full `Fix(BUG-182)` comment added at the fix site. |
| 2026-08-16 | verified | New file `tests/webgl/outline_seed_sentinel.rs` (3 native `#[test]` functions: sentinel rejected, zero-coordinate seed accepted, ordinary seed accepted) -- `cargo nextest run --all-features webgl::outline_seed_sentinel` from `module/helper/renderer/`: 3/3 passed. Re-ran the existing BUG-179 wasm32 browser test (`webgl::wide_outline`) to confirm the hand-edited GLSL still compiles/links/runs: 1/1 passed. `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: clean. Full workspace: `cargo nextest run --workspace --all-features --exclude object_picking`: 1906/1906 passed, 0 skipped (up from 1902 -- this bug's 3 new tests plus BUG-193's 1 new test, fixed and verified in the same pass). `cargo test --doc --workspace --all-features --exclude object_picking`: all crates ok. `cargo clippy --workspace --all-targets --all-features --exclude object_picking -- -D warnings`: clean. `--exclude object_picking` re-confirmed evidence-based: working tree still dirty from the concurrent actor's own unrelated in-progress work (`examples/minwebgl/object_picking/{Cargo.toml,src/main.rs}`); `cargo check -p object_picking` (non-clippy) still passes clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming: traced the actual sentinel-write contract from `jfa_init.frag`/`jfa_step.frag` directly rather than trusting the `!= 0.0` comparison's shape. Adversarial: checked whether the fix's `>= 0.0` could itself misclassify anything -- no code path in this shader trio ever writes a negative non-sentinel coordinate, so sign is a complete discriminant, not merely an improvement. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-179/BUG-180/BUG-181 (independent, same file trio, already fixed) and BUG-193 (found and fixed in the same edit pass, filed separately). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by a direct read of the sentinel-write contract and the check's own contradicting comment, not inferred from the diff's shape alone. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is a single comparison-operator and operand change on one `if` condition; no other logic in the file touched. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Entirely within `renderer`'s own shader file and its own new test file. | — |
| D7 | Crate Locality | 🟢 | 🟢 | The seed-validity check has exactly one definition site, fixed there. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix completes the pass's own documented responsibility (distinguish a found seed from no-seed) without adding or removing scope. | — |

**Reproduced:** YES -- `seed_is_valid([0.0, 0.3])` (the pre-fix formula's blind spot: `0.0 != 0.0`
is false) now correctly returns `true` post-fix (`0.0 >= 0.0` is true), and
`seed_is_valid([-1.0, -1.0])` (the true sentinel, which pre-fix incorrectly passed as valid) now
correctly returns `false`, encoded as `outline_seed_sentinel.rs`'s executable regression tests
(3/3 passing). Existing BUG-179 browser test re-run to confirm the hand-edited GLSL still
compiles/links/runs in a real WebGL2 context (1/1 passing). Full workspace native suite
(1906/1906, 0 skipped), doctests (0 failed), and clippy all clean (excluding the concurrent
actor's unrelated `object_picking` in-flight refactor), 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/shaders/post_processing/outline/wide_outline/outline.frag` | Changed the seed-validity check from `seedCoord.x != 0.0 && seedCoord.y != 0.0` to `seedCoord.x >= 0.0 && seedCoord.y >= 0.0`; full `Fix(BUG-182)` comment added explaining the sentinel-vs-real-coordinate contract. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/webgl/outline_seed_sentinel.rs` | New file, 3 native `#[test]` functions: background sentinel rejected, zero-coordinate seed accepted, ordinary positive seed accepted. |
| `module/helper/renderer/tests/webgl/mod.rs` | Added `mod outline_seed_sentinel;` registration. |
| `module/helper/renderer/tests/readme.md` | Added `webgl/outline_seed_sentinel.rs` Responsibility Table row. |
