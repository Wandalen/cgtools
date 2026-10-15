# BUG-194: `Quat::slerp` computes hemisphere correction but discards it

- **Severity:** Medium (visual-only defect -- no crash, but any slerp between two quaternions
  whose dot product is negative interpolates along the long rotational path instead of the short
  one)
- **state:** Completed
- **Affects:** Every caller of `ndarray_cg::Quat::slerp` interpolating between two quaternions
  more than 90 degrees apart as raw 4-vectors.
- **Component:** `module/math/ndarray_cg` (`src/quaternion/arithmetics.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-16
- **Related Bugs:** Sibling of BUG-183 (`Blender::rotation_blend` had the identical "hemisphere
  matters for quaternion math" defect class, in an NLERP sum rather than SLERP, discovered in the
  same investigation pass).

## Symptom

```rust
// before
pub fn slerp( &self, other : &Self, t : E ) -> Self
{
  let dot = self.dot( other );
  let q2 = if dot < E::zero() { -*other } else { *other };
  let dot = dot.abs();
  // ... proceeds to use `other` (the ORIGINAL, unflipped argument) below, not `q2`
  ...
}
```

The function correctly computed `q2` as the hemisphere-corrected copy of `other`, and correctly
used the absolute value of `dot` for the interpolation-factor math -- but the actual
interpolation at the end of the function referenced the original `other` parameter instead of the
locally-computed, sign-corrected `q2`, silently discarding the correction it had just computed.

## Impact

**Who is affected:** Any caller of `Quat::slerp` (directly, or transitively through any higher-
level animation/rotation code built on it) interpolating between two quaternions whose dot
product is negative.

**What breaks:** The interpolated rotation walks the long way around the rotational path instead
of the short way -- up to 180 degrees off from the correct SLERP result.

**Magnitude:** Deterministic whenever the two input quaternions are in opposite hemispheres; not
triggered otherwise (dot product >= 0 inputs were already correct, since `q2 == other` in that
branch).

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found while investigating BUG-183 (`Blender::rotation_blend`'s missing hemisphere check) -- read
`Quat::slerp` as a reference for "what does correct hemisphere handling look like elsewhere in
this workspace," and found it had already computed the correction locally but never used it.

## Minimum Reproducible Example

```rust
let q1 = QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] );                       // identity
let q2 = QuatF64::from_axis_angle( z_axis, 270.0_f64.to_radians() );    // dot(q1,q2) < 0
let result = q1.slerp( &q2, 0.5 );
// pre-fix: result is the midpoint of the LONG path (135 deg), not the short path (-45 deg)
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/math/ndarray_cg && cargo nextest run --all-features test_slerp_negative_dot_product_takes_short_path
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `slerp` computes `q2` as the hemisphere-corrected copy but the final interpolation expression still references `other` (the uncorrected original) instead of `q2`. | ✅ Root Cause | Confirmed by reading the full function body: `q2` is bound and never read again after the `dot`/abs computation. | E1 |
| H2 | The bug only manifests for exact antipodal quaternions (dot == -1), a case rare enough to ignore. | ❌ Falsified | The `dot < 0` branch is the entire negative half of the domain, not a single exact point -- any pair more than 90 degrees apart as raw 4-vectors triggers it. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/math/ndarray_cg/src/quaternion/arithmetics.rs` (`slerp`, pre-fix) | `let q2 = if dot < E::zero() { -*other } else { *other };` followed later by an interpolation expression using `other`, not `q2`. | H1 ✅ |
| E2 | `module/math/ndarray_cg/src/quaternion/arithmetics.rs` (`slerp`, pre-fix) | The `dot < E::zero()` condition is a half-space of the input domain, not a measure-zero point. | H2 ❌ |

## Root Cause

The function's own local hemisphere-corrected variable (`q2`) was computed but never substituted
into the interpolation formula that used the original, uncorrected parameter instead -- a
copy/paste or refactor slip where the corrected binding was introduced but the usage sites below
it were not updated to reference it.

## Why Not Caught

No pre-existing test in the `ndarray_cg` quaternion test suite exercised a negative-dot-product
input pair for `slerp` -- all existing SLERP tests used inputs already in the same hemisphere, so
`q2 == other` held in every tested case and the discarded correction was never observed.

## Fix Location

`module/math/ndarray_cg/src/quaternion/arithmetics.rs`, `slerp`: changed the interpolation
expression to use `q2` instead of `other`, with a `Fix(BUG-194)` comment.

## Prevention

New test `test_slerp_negative_dot_product_takes_short_path` added to
`module/math/ndarray_cg/tests/inc/quat_test/general.rs`: slerps between two quaternions 270
degrees apart about the same axis (negative dot product) at `t = 0.5`, and asserts the result
matches the short-path (-45 degree) midpoint rather than the long-path (135 degree) midpoint a
discarded correction would produce.

## Pitfall

A locally-computed "corrected" variable that shadows or parallels an original parameter is a
silent trap if any usage site below it is not updated to reference the correction -- the compiler
gives no warning, since the original parameter is still a perfectly valid expression in scope.
Grep for every usage of the *original* parameter name after introducing a corrected copy, not just
the corrected variable's own definition site.

## Generalized Version

**Broken assumption:** "since the correction variable is computed and named clearly, it must be
the one actually used downstream."

**Confirmed general rule:** After introducing a corrected/derived local variable meant to replace
uses of an original parameter within the same function, grep every remaining reference to the
original parameter's name in that function body to confirm none were missed.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Found while investigating BUG-183; read `Quat::slerp` as a "what does correct hemisphere handling look like" reference and found the discarded `q2` correction. |
| 2026-08-16 | fixed | Changed the interpolation expression to reference `q2` instead of `other`. |
| 2026-08-16 | verified | `cargo nextest run -p ndarray_cg test_slerp_negative_dot_product_takes_short_path --all-features`: 1/1 passed. `cargo clippy -p ndarray_cg --all-targets --all-features -- -D warnings`: clean (after removing two unnecessary-parens lint hits in the new test's own local variable bindings). Full workspace: `cargo nextest run --workspace --all-features --exclude object_picking`: 1909/1909 passed. `cargo test --doc --workspace --all-features --exclude object_picking`: all crates ok. `cargo clippy --workspace --all-targets --all-features --exclude object_picking -- -D warnings`: clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming: test hand-derives both pre-fix (135 deg) and post-fix (-45 deg) expected outputs. Adversarial: checked whether `t=0.5` specifically was needed to discriminate, or whether the bug is visible at any `t` -- confirmed the discarded-correction bug is visible at every `t != 0`, `t=0.5` chosen only for a clean expected-value calculation. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-183 (sibling defect class, found via this investigation). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct reading of the pre-fix function body, not assumed. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is the single-token `other` -> `q2` substitution only. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Entirely within `ndarray_cg`'s own `arithmetics.rs` and its test suite. | — |
| D7 | Crate Locality | 🟢 | 🟢 | `slerp` has exactly one definition site, fixed there. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix completes the function's own already-intended behavior (its own local `q2` binding proves the correction was intended), no scope change. | — |

**Reproduced:** YES -- `test_slerp_negative_dot_product_takes_short_path` fails pre-fix (observed
result matching the long-path 135-degree midpoint, not the expected short-path -45-degree
midpoint) and passes post-fix. Full workspace native suite (1909/1909, 0 skipped), doctests, and
clippy all clean (excluding the concurrent actor's unrelated `object_picking` in-flight refactor),
2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/math/ndarray_cg/src/quaternion/arithmetics.rs` | `slerp`'s interpolation expression now uses the hemisphere-corrected `q2` instead of the original `other`, with a `Fix(BUG-194)` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/math/ndarray_cg/tests/inc/quat_test/general.rs` | Added `test_slerp_negative_dot_product_takes_short_path`. |
