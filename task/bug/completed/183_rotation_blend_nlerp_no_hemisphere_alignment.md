# BUG-183: `Blender::rotation_blend` NLERP sum has no hemisphere alignment

- **Severity:** Medium (visual-only defect -- no crash, but blending two or more animation
  clips' rotations can produce a visibly wrong or near-degenerate result whenever the clips'
  current quaternions land in opposite hemispheres)
- **state:** Completed
- **Affects:** Every caller of `renderer::webgl::animation::Blender` blending two or more
  weighted animation clips that both drive the same node's rotation channel.
- **Component:** `module/helper/renderer` (`src/webgl/animation/blending.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-16
- **Related Bugs:** Sibling of BUG-194 (`Quat::slerp` in `ndarray_cg` had the same "hemisphere
  matters" defect class in a different function, found while investigating this bug). Its own
  regression test was blocked from actually exercising the fix by BUG-195 (test-fixture prefix
  mismatch in `blender_tests.rs`) until that was fixed first. Fixing this bug's test surfaced a
  second, independent defect in the same function, BUG-196 (accumulator seeded from identity
  instead of zero).

## Symptom

```rust
// pre-fix
let mut rotation = QuatF32::default();
for ( mut r, w ) in values
{
  if rotation.dot( &r ) < 0.0    // <- this check did not exist pre-fix
  {
    r = r * -1.0;
  }
  rotation += r * w;
}
node.borrow_mut().rotation_set( rotation.normalize() );
```

Pre-fix, the loop had no hemisphere check at all -- it read `rotation += r * w;` unconditionally.
A quaternion `q` and its negation `-q` represent the identical rotation, but naive addition does
not respect that equivalence: summing two quaternions whose dot product is negative cancels
components instead of blending them, producing a result that walks the *long* way around between
the two rotations instead of the short way (up to 180 degrees off), or, in near-opposite cases,
a near-zero-magnitude sum that normalizes to an arbitrary/unstable direction.

## Impact

**Who is affected:** Any caller blending two or more weighted animation clips via `Blender`
where the clips' *current* rotation values (as sampled by `Sequence::current_get()` at whatever
point each clip's own playback has reached) happen to be more than 90 degrees apart as raw
quaternions -- common whenever independent clips are not phase-locked to each other.

**What breaks:** The blended rotation applied to the node is visibly wrong: instead of
interpolating along the shorter rotational path between the two clips' current orientations, it
interpolates along the longer path, or produces a near-degenerate result when the two
orientations are close to exactly opposite.

**Magnitude:** Every frame the affected clips remain in opposite hemispheres relative to each
other -- not intermittent, but also not present for every combination of clips (only those whose
current values happen to diverge by more than 90 degrees).

**Entity Scope:** None -- a code-level defect.

## How Discovered

Pre-identified in the task backlog (task #135) as part of a systematic animation-subsystem review
following the outline-shader bug cluster (BUG-178 through BUG-193). Confirmed by reading
`rotation_blend`'s accumulation loop directly and recognizing the missing hemisphere check against
the same "`q` and `-q` are the same rotation" principle already documented for `Quat::slerp`
elsewhere in the workspace.

## Minimum Reproducible Example

```rust
// two clips' current rotations: 0 degrees and 270 degrees about the same axis
// ( a negative-dot-product pair, since 270 degrees is closer to -90 than to +90 )
let q_a = QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] );
let q_b = QuatF64::from_axis_angle( z_axis, 270.0_f64.to_radians() );
// blender.set(&nodes) pre-fix: walks the long path (135 deg), not the short path (-45 deg)
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo nextest run --all-features --test blender_tests test_blender_rotation_blend_aligns_hemisphere_across_clips
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `rotation_blend`'s accumulation loop lacks a hemisphere check, unlike `Quat::slerp` (BUG-194) which has one but discards its own correction. | ✅ Root Cause | Confirmed by reading the pre-fix loop body directly: no `dot`/sign check anywhere before the `+=`. | E1 |
| H2 | The bug is purely theoretical -- no realistic caller ever blends clips whose current values diverge by more than 90 degrees. | ❌ Falsified | `Blender` is explicitly designed to blend *independently playing* clips (each with its own `Sequencer`/`Sequence` state, never phase-locked to each other) -- nothing in the API prevents or makes unlikely two clips' current values landing in opposite hemispheres. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/renderer/src/webgl/animation/blending.rs` (`rotation_blend`, pre-fix) | `rotation += r * w;` inside the loop, with no preceding sign/dot check. | H1 ✅ |
| E2 | `module/helper/renderer/src/webgl/animation/blending.rs` (`Blender::weighted_animations`) | Each weighted animation owns its own independent `Sequencer`, advanced independently via `Blender::update`'s per-entry `animation.update(delta_time)` -- nothing couples two clips' playback phase together. | H2 ❌ |

## Root Cause

```rust
// before
let mut rotation = QuatF32::default();
for ( r, w ) in values
{
  rotation += r * w;
}
```

The loop treated quaternion blending as an ordinary vector weighted sum, which is only valid when
every summed quaternion already lies in the same hemisphere as the running total. Nothing in the
loop established or maintained that invariant.

## Why Not Caught

No pre-existing test in `blender_tests.rs` asserted on the actual blended rotation *value* --
every existing test only checked that `blender.set()` didn't panic, or that weight bookkeeping
round-tripped through `weights_get`/`weights_get_mut`. Writing the first value-asserting test for
this function (this bug's own regression test) was blocked from even reaching this code path by
BUG-195 (wrong channel-name prefix in the test fixture) until that was fixed first -- so the
defect had zero effective test coverage despite the file containing 20 tests.

## Fix Location

`module/helper/renderer/src/webgl/animation/blending.rs`, `rotation_blend`: added
`if rotation.dot( &r ) < 0.0 { r *= -1.0; }` before each accumulation, with a `Fix(BUG-183)`
comment. Applied in the same edit as BUG-196's accumulator-seed fix, since both live in the same
loop and BUG-196 was only discovered while validating this fix.

## Prevention

New test `test_blender_rotation_blend_aligns_hemisphere_across_clips` added to
`module/helper/renderer/tests/blender_tests.rs`: blends two rotation clips whose (unadvanced,
`Pending`-state) current values are 0 and 270 degrees about the same axis -- a
negative-dot-product pair -- and asserts the blended result matches the short-path (-45 degree)
blend rather than the long-path (135 degree) blend a naive sum would produce. Since `Blender`
stores its clips in an `FxHashMap` (iteration order not guaranteed), the assertions compare via
`|dot(got, expected)|` rather than direct component equality, so the test is correct regardless
of which clip is visited first.

## Pitfall

A "sum then normalize" quaternion blend is only correct once every term has been aligned to a
common hemisphere first -- this must be re-derived for every new accumulation site, since Rust's
type system gives no signal that `Add`/`AddAssign` on a quaternion type needs this precondition
the way it doesn't for an ordinary vector. Writing the *value-asserting* test for a blend function
is what actually catches this class of bug; a test that only checks "does not panic" or "weights
round-trip" gives false confidence.

## Generalized Version

**Broken assumption:** "if a function compiles and its existing tests pass, its core arithmetic is
correct."

**Confirmed general rule:** For any function whose existing tests only check side-channel
properties (no panic, unrelated getter round-trips) rather than the function's own primary
output, treat that output as effectively untested regardless of the file's test count -- write a
value-asserting test before trusting the arithmetic.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Pre-identified in the backlog as task #135. Investigated after closing BUG-182/BUG-193; found the hemisphere-check gap by reading `rotation_blend` directly. |
| 2026-08-16 | fixed | Added the `dot`-then-flip hemisphere check to `rotation_blend`'s accumulation loop. Applied together with BUG-196's accumulator-seed fix in the same loop, discovered while writing this bug's own test. |
| 2026-08-16 | verified | Writing this bug's test first required fixing BUG-195 (test fixture used wrong channel-name prefix constants, silently making the channel lookup return `None`). After both BUG-195 and BUG-196 were fixed, `cargo nextest run -p renderer --test blender_tests --all-features`: 20/20 passed (including this bug's new test). `cargo nextest run -p renderer --test scaler_tests --all-features`: 8/8 passed (BUG-195's sibling fix, no regressions). `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: clean. Full workspace: `cargo nextest run --workspace --all-features --exclude object_picking`: 1909/1909 passed, 0 skipped. `cargo test --doc --workspace --all-features --exclude object_picking`: all crates ok. `cargo clippy --workspace --all-targets --all-features --exclude object_picking -- -D warnings`: clean. `--exclude object_picking` re-confirmed evidence-based: working tree still dirty from the concurrent actor's own unrelated in-progress work; `cargo check -p object_picking` passes clean standalone. No shader files touched by this fix, so no wasm32 GLSL-compile re-check was applicable. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming: test hand-derives both the pre-fix (135 deg) and post-fix (-45 deg) expected outputs before asserting, confirming the test actually discriminates. Adversarial: checked whether the `FxHashMap` iteration-order nondeterminism could make the test flaky -- assertions use `|dot(...)|`, which is invariant to which clip is visited first (verified by hand-deriving both orderings). | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-194 (sibling hemisphere-alignment defect), BUG-195 (blocked this bug's own test), BUG-196 (found via this bug's test). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct reading of the pre-fix loop body; not assumed from the task's pre-existing one-line description alone. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is the hemisphere-check addition only; the accumulator-seed change is scoped and reported separately as BUG-196 despite living in the same edit. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Entirely within `renderer`'s own `blending.rs` and `blender_tests.rs`. | — |
| D7 | Crate Locality | 🟢 | 🟢 | `rotation_blend` has exactly one definition site, fixed there. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix completes the function's own documented responsibility (blend rotation values) without adding or removing scope. | — |

**Reproduced:** YES -- `test_blender_rotation_blend_aligns_hemisphere_across_clips` fails pre-fix
(observed: result skewed heavily toward one input, `|dot|` with the expected short-path blend
well below the 0.999 threshold) and passes post-fix. Full workspace native suite (1909/1909, 0
skipped), doctests (0 failed), and clippy all clean (excluding the concurrent actor's unrelated
`object_picking` in-flight refactor), 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/animation/blending.rs` | Added a hemisphere-alignment check (`if rotation.dot( &r ) < 0.0 { r *= -1.0; }`) to `rotation_blend`'s accumulation loop, with a `Fix(BUG-183)` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/blender_tests.rs` | Added `test_blender_rotation_blend_aligns_hemisphere_across_clips`; also fixed the file's channel-name prefix constants as part of BUG-195 (necessary prerequisite for this test to exercise real code). |
