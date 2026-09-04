# BUG-196: `rotation_blend` NLERP accumulator seeded from identity, not zero

- **Severity:** Medium (visual-only defect -- no crash, but corrupts the blended rotation result
  for every call, including the single-clip case, by mixing in an unweighted "stay at identity"
  term)
- **state:** Completed
- **Affects:** Every caller of `renderer::webgl::animation::Blender` blending one or more
  weighted animation clips that drive a node's rotation channel.
- **Component:** `module/helper/renderer` (`src/webgl/animation/blending.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-16
- **Related Bugs:** Found while validating BUG-183's own fix (same function, same accumulation
  loop) -- BUG-183's new test failed a second time even after BUG-195 (the test-fixture prefix
  mismatch that was blocking it) was fixed, which is what led to finding this bug. Both fixes were
  applied together in the same edit to `rotation_blend`, but are reported as two separate defects
  since they are independent root causes.

## Symptom

```rust
// before
let mut rotation = QuatF32::default();   // == [0, 0, 0, 1] -- IDENTITY, not zero
for ( r, w ) in values
{
  rotation += r * w;
}
node.borrow_mut().rotation_set( rotation.normalize() );
```

`QuatF32::default()` is `Quat`'s custom (non-derived) `Default` impl, which deliberately returns
the identity quaternion `[0,0,0,1]` -- appropriate for "a node with no rotation applied," but
wrong as the seed of a weighted-sum-then-normalize accumulator, which needs the vector space's
additive zero (`[0,0,0,0]`) to avoid injecting an extra, unweighted term into the sum.

## Impact

**Who is affected:** Every caller of `Blender::rotation_blend` -- i.e., every call to
`Blender::set` for a node with at least one weighted rotation clip, regardless of how many clips
are blended or what their weights are.

**What breaks:** The accumulator starts already "one identity quaternion" ahead of a correct zero-
seeded sum. Even the simplest case -- a single clip at full weight 1.0 -- no longer normalizes
back to exactly that clip's own rotation; it normalizes to a blend BETWEEN that rotation and
identity, silently pulling every blended rotation toward identity by an amount depending on how
far the true blended rotation is from identity in the first place.

**Magnitude:** Present on every single call, not intermittent -- this is a strictly worse defect
than BUG-183 in terms of frequency (BUG-183 only manifests when clips diverge by more than 90
degrees; this manifests unconditionally, for any nonzero rotation).

**Entity Scope:** None -- a code-level defect.

## How Discovered

While validating BUG-183's new regression test after BUG-195 was fixed (unblocking the test's
channel lookup), the test failed a SECOND time with a result inconsistent with either the pre- or
post-BUG-183-fix expected values (`got = [0,0,-0.187,0.982]` vs. expected `[0,0,-0.383,0.924]`).
Hand-recomputing the NLERP loop with the accumulator explicitly seeded from `[0,0,0,1]` (rather
than assuming `[0,0,0,0]`) reproduced the observed wrong output almost exactly, confirming the
accumulator's actual starting value was the culprit. Reading `Quat::default()`'s real
implementation (`module/math/ndarray_cg/src/quaternion.rs`) confirmed it returns identity, not
zero, settling the root cause.

## Minimum Reproducible Example

```rust
// single clip, full weight, non-identity rotation
let q = QuatF64::from_axis_angle( z_axis, 90.0_f64.to_radians() );
blender.add( "anim", single_clip_sequencer_for( q ), F64x3::new( 0.0, 1.0, 0.0 ) );
blender.set( &nodes );
// pre-fix: node.rotation_get() != q (pulled toward identity), despite weight 1.0 and one clip
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo nextest run --all-features --test blender_tests test_blender_rotation_blend_aligns_hemisphere_across_clips
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `Quat::default()` returns identity `[0,0,0,1]`, not the additive zero `[0,0,0,0]` a weighted-sum accumulator needs, and `rotation_blend` used it as the accumulator seed. | ✅ Root Cause | Confirmed by reading `Quat::default()`'s actual (custom, non-derived) implementation directly. | E1 |
| H2 | The wrong-by-hand-recomputation result could instead be explained by an error in BUG-183's hemisphere-check fix itself, not a separate accumulator-seed defect. | ❌ Falsified | Hand-recomputed the loop twice: once assuming a zero-seeded accumulator (did not match observed output), once assuming an identity-seeded accumulator (matched observed output almost exactly) -- isolating the discrepancy to the seed value, not the hemisphere-check logic (which was independently verified correct by the same recomputation). | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/math/ndarray_cg/src/quaternion.rs` (`impl Default for Quat<E>`) | `Quat(Vector([E::zero(), E::zero(), E::zero(), E::one()]))` -- identity, not zero. | H1 ✅ |
| E2 | Hand-derived recomputation (session scratch work, not committed) | Identity-seeded recomputation matches the observed wrong test output; zero-seeded recomputation matches the originally-derived expected output. | H2 ❌ |

## Root Cause

`rotation_blend` reused the same `let mut X = Default::default();` accumulator-seeding pattern
used by `translation_blend`/`scale_blend` (both `F32x3`, whose `Default` correctly IS the additive
zero), without accounting for `Quat`'s `Default` deliberately diverging from that pattern (it
returns identity, appropriate for "no rotation applied," not zero).

## Why Not Caught

Identical root cause to BUG-183's "why not caught": no pre-existing test asserted on the actual
blended rotation value, so an accumulator seed that silently biases every result toward identity
produced no observable test failure. The defect was only exposed once BUG-195 (test-fixture
prefix mismatch) was fixed, which is what let a real value-asserting test reach this code for the
first time this session.

## Fix Location

`module/helper/renderer/src/webgl/animation/blending.rs`, `rotation_blend`: replaced the
`QuatF32::default()`-seeded accumulator with one seeded from the first entry itself
(`let mut rotation = first_r * first_w;`), using `values.into_iter().next()` (via an `if let
Some(...) = ... else { ... }` pattern) to explicitly handle the empty-values case (falls back to
`QuatF32::default()` for "no clips -> identity," preserving the pre-existing empty-input
behavior). This sidesteps the question of what a "zero rotation" would even mean, and needs no
hemisphere check for the first entry since there is nothing yet to align against. Comment
documents this as `Fix(BUG-196)`, layered alongside BUG-183's `Fix(BUG-183)` comment in the same
loop.

## Prevention

BUG-183's own regression test (`test_blender_rotation_blend_aligns_hemisphere_across_clips`,
`blender_tests.rs`) incidentally also covers this bug: its expected values were hand-re-derived
under the CORRECT (first-entry-seeded) formula after this bug was found, and the test would fail
under the old identity-seeded formula even independent of the hemisphere-check fix (verified by
hand-computing both formulas separately during root-causing). No separate test was added, since
one test already exercises both defects in the same call.

## Pitfall

`Default::default()` is not a safe universal "zero" seed for an accumulator pattern shared across
multiple types -- it must be re-checked per type, since a type's `Default` impl is free to mean
"the identity/neutral element for its own semantics" (as `Quat` deliberately does, for "no rotation
applied") rather than "the additive zero of its underlying representation." Copy-pasting an
accumulator pattern from one blend function (`translation_blend`, using `F32x3`) to a sibling
function using a different type (`rotation_blend`, using `QuatF32`) silently carries this
assumption across without re-verifying it holds for the new type.

## Generalized Version

**Broken assumption:** "`Default::default()` means the additive/neutral zero for any type used in
a weighted-sum accumulator, since it did for the last type I used this pattern with."

**Confirmed general rule:** Before reusing an accumulator-seeding pattern across sibling functions
operating on different types, check each type's actual `Default` implementation directly rather
than assuming semantic consistency with a previously-verified type -- especially for types (like
quaternions) where "identity" and "zero" are both meaningful but different values.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Found while validating BUG-183's fix, after BUG-195 unblocked the regression test's channel lookup and a second, unexpected failure appeared. |
| 2026-08-16 | fixed | Changed `rotation_blend`'s accumulator to seed from the first entry (scaled by its own weight) instead of `QuatF32::default()`, applied in the same edit as BUG-183's hemisphere-check fix. |
| 2026-08-16 | verified | Hand-re-derived BUG-183's test's expected values under the corrected formula and confirmed they matched the originally-derived values exactly (no test changes needed). `cargo nextest run -p renderer --test blender_tests --all-features`: 20/20 passed. Full workspace: `cargo nextest run --workspace --all-features --exclude object_picking`: 1909/1909 passed. `cargo clippy --workspace --all-targets --all-features --exclude object_picking -- -D warnings`: clean (after fixing a `manual_assign_op` hit on `r *= -1.0` introduced by this same edit). |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming: hand-recomputation under the identity-seeded formula reproduced the observed wrong output. Adversarial: attempted to attribute the discrepancy to BUG-183's hemisphere-check fix instead -- ruled out by recomputing the hemisphere logic separately and confirming it matched expectations on its own. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-183 (same function/loop, fixed together), BUG-195 (unblocked the test that surfaced this). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct reading of `Quat::default()`'s real implementation, not assumed. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is the accumulator-seed change only; BUG-183's hemisphere-check addition reported and scoped separately despite the same edit. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Entirely within `renderer`'s own `blending.rs`. | — |
| D7 | Crate Locality | 🟢 | 🟢 | `rotation_blend` has exactly one definition site, fixed there. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix completes the function's own documented responsibility (blend rotation values correctly) without adding or removing scope. | — |

**Reproduced:** YES -- observed wrong output (`[0,0,-0.187,0.982]`) matched a hand-recomputation
under the identity-seeded formula; post-fix output matches the hand-derived correct expected value
(`|dot| > 0.999` against `[0,0,-0.383,0.924]`). Full workspace native suite (1909/1909, 0 skipped),
doctests, and clippy all clean (excluding the concurrent actor's unrelated `object_picking`
in-flight refactor), 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/animation/blending.rs` | `rotation_blend`'s accumulator now seeds from the first entry (scaled by its own weight) instead of `QuatF32::default()`, with a `Fix(BUG-196)` comment; empty-input case explicitly falls back to identity via an `if let ... else` pattern. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/blender_tests.rs` | No separate test added -- BUG-183's `test_blender_rotation_blend_aligns_hemisphere_across_clips` covers this defect too; its expected values were re-derived under the corrected formula and confirmed to match the original derivation exactly. |
