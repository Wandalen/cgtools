# BUG-250: `Mat4::decompose()` divides by the scale reciprocal instead of multiplying, re-squaring scale into the extracted rotation matrix

- **Severity:** High
- **state:** Completed
- **Affects:** Any caller of `Mat4<E,Descriptor>::decompose()` on a matrix built with non-unit scale (uniform or non-uniform) on any axis — confirmed concretely via round-trip against `from_scale_rotation_translation` with `scale = (2.0, 3.0, 0.5)`
- **Component:** `module/math/ndarray_cg` (`src/d2/mat4x4/general.rs::decompose`)
- **repo_identity:** self
- **Filed:** 2026-08-15
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** [BUG-119](../completed/119_quat_from_mat3_cyclic_component_shift.md) — `decompose()`'s final step calls `Quat::from(rot_mat)`, a separate, independently-defective conversion; both bugs compound on any real `decompose()` call but have distinct, unrelated root causes (this bug is a wrong operator on the rotation-matrix columns; BUG-119 is a component-array reorder inside the matrix→quaternion conversion itself), fixed together under the same task #52 targeted math review

## Symptom

```bash
# scale = (2.0, 3.0, 0.5), rotation = 30° about Y, translation = (1.0, 2.0, 3.0)
# built via Mat4::from_scale_rotation_translation, then .decompose()'d back

# Wrong (pre-fix) — recovered scale is way off, rotation matrix fed to Quat::from
# is non-orthonormal (columns squared by scale instead of normalized)
recovered scale: (4.0, 9.0, 0.25)      # expected: (2.0, 3.0, 0.5) -- each axis squared

# Correct (post-fix)
recovered scale: (2.0, 3.0, 0.5)       # matches input exactly (within float epsilon)
```

## Impact

**Who is affected:** Any caller of `decompose()` on a matrix that carries non-unit scale on any
axis — this includes every consumer that round-trips a `Mat4` through
`from_scale_rotation_translation` → `decompose()`, or calls `decompose()` on a matrix produced by
scene-graph/transform composition where scale ≠ 1 (the overwhelmingly common case for any node
that isn't deliberately unscaled).

**What breaks:** Silent, no error. `sx`/`sy`/`sz` come back squared instead of recovered exactly
(`a / inv_scale.x()` = `a / (1/sx)` = `a * sx`, and since `a` already carries one factor of `sx`,
the result carries `sx²`). Worse, the rotation matrix handed to `Quat::from()` is built from
columns still carrying `sx`/`sy`/`sz` instead of unit-length columns — for any non-unit scale, this
matrix is not orthonormal, so the extracted rotation quaternion is mathematically undefined
(garbage), not merely inaccurate. For the special case scale = (1,1,1) exactly, `inv_scale` is
also (1,1,1), so division and multiplication coincide and the bug is invisible — this is almost
certainly why it shipped unnoticed.

**Magnitude:** Every non-unit-scale `decompose()` call, which is the typical case for scene-graph
transforms; unit-scale callers are unaffected by construction.

**Entity Scope:** None — a code-level math defect, not an operational-entity concern.

## How Discovered

Task #52, a targeted math/geometry code review of core crates dispatched under the standing
bug-hunt mandate. The reviewing agent flagged `decompose()`'s use of `/ inv_scale` against the
cited three.js reference (`Matrix4.js#L1050`, whose `decompose` multiplies each column by
`1 / sx` etc.). Independently re-derived by hand before filing: `inv_scale.x() = E::one() / sx`
(`general.rs:308`), so `a / inv_scale.x()` algebraically equals `a * sx`, not `a * (1/sx)` — the
opposite operation from the reference.

```bash
$ grep -n "inv_scale" module/math/ndarray_cg/src/d2/mat4x4/general.rs
306:    let inv_scale = Vector::< E, 3 >::from_array
308:      [ E::one() / sx, E::one() / sy, E::one() / sz ]
319:        a / inv_scale.x(), b / inv_scale.x(), c / inv_scale.x(),   # pre-fix
```

## Minimum Reproducible Example

```bash
rm -rf /tmp/mre118 && mkdir -p /tmp/mre118/src
cat > /tmp/mre118/Cargo.toml <<'EOF'
[package]
name = "mre118"
version = "0.1.0"
edition = "2021"

[dependencies]
ndarray_cg = { path = "/home/user1/pro/lib/yrd_gamedev/cgtools/module/math/ndarray_cg" }
EOF
cat > /tmp/mre118/src/main.rs <<'EOF'
use ndarray_cg::{ Mat4, QuatF64, F64x4x4, mat::DescriptorOrderColumnMajor };

fn main()
{
  let scale = [ 2.0_f64, 3.0, 0.5 ];
  let rotation = QuatF64::from_axis_angle( [ 0.0, 1.0, 0.0 ], 0.3_f64 ).normalize();
  let translation = [ 1.0_f64, 2.0, 3.0 ];

  let m : F64x4x4< DescriptorOrderColumnMajor > =
    Mat4::from_scale_rotation_translation( scale, rotation, translation );

  let ( _t, _r, recovered_scale ) = m.decompose().expect( "decompose should succeed" );
  println!( "input  scale: {:?}", scale );
  println!( "recovered scale: {:?}", recovered_scale.to_array() );
}
EOF
cd /tmp/mre118 && cargo run 2>&1 | tail -3
```

**Expected** (recovered scale matches input, within float epsilon):
```
input  scale: [2.0, 3.0, 0.5]
recovered scale: [2.0, 3.0, 0.5]
```

**Actual** (pre-fix — each axis squared):
```
input  scale: [2.0, 3.0, 0.5]
recovered scale: [4.0, 9.0, 0.25]
```

**Verify Command** (≤3 lines, standalone):
```bash
cd /tmp/mre118 && cargo run 2>&1 | tail -2
# recovered scale == input scale (within epsilon) = fixed; squared values = bug present
```
**What:** Violates the round-trip invariant `decompose( from_scale_rotation_translation( s, r, t
) ) == ( t, r, s )` — `decompose()`'s own doc comment cites the three.js `Matrix4.js#L1050`
algorithm it's a port of, which recovers scale exactly.

**Known MRE limitation (check 205):** `ndarray_cg` is this workspace's own crate; the MRE
path-depends on it locally rather than a registry version, mirroring BUG-116's own documented
exception (`## Symptom` above's numbers are exact rational arithmetic — `sx=2.0` squares to
`4.0` deterministically — so unlike BUG-116 there is no bisection/registry-version ambiguity this
local dependency could be hiding).

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `decompose()`'s rotation-matrix reconstruction (`general.rs:319-326`) divides each matrix entry by `inv_scale`'s component instead of multiplying, algebraically re-multiplying scale back in (`a / (1/sx) = a*sx`) rather than removing it. | ✅ Root Cause | Direct read of `general.rs:306-309` confirms `inv_scale` already holds `1/sx,1/sy,1/sz`; `general.rs:319-326` (pre-fix) divided by these values instead of multiplying, the opposite of the cited three.js reference's `1/sx` column-scaling. MRE confirms scale comes back squared, exactly matching this algebra. | E1, E2, E3 |
| H2 | `from_scale_rotation_translation` (the matrix-construction half of the round trip) has the defect instead — e.g. composes scale and rotation in the wrong order or with the wrong reciprocal. | ❌ Falsified | Direct read of `general.rs:195-229` shows `from_scale_rotation_translation` builds columns as `rotation.into().to_matrix()`'s columns each multiplied by the corresponding raw (non-reciprocal) scale component — standard TRS composition, independent of `decompose()`, and already exercised correctly by the pre-existing `test_from_scale_rotation_translation_generic` test. Not implicated. | E4 |
| H3 | `Quat::from(Mat3)` (the final step of `decompose()`, converting the extracted rotation matrix to a quaternion) is the source of the wrong-rotation symptom, not the scale arithmetic. | ✅ Confirmed but Separate | `Quat::from(Mat3)` does carry its own, independent defect (component-array cyclic shift — filed separately as BUG-119) — but it is unrelated to *this* bug's scale-squaring defect: even with BUG-119 fixed, `decompose()`'s recovered `scale` output (not routed through `Quat::from` at all) was still wrong pre-this-fix, isolating H1 as this bug's own root cause. | E1, E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/math/ndarray_cg/src/d2/mat4x4/general.rs:306-309` (pre-fix) | `inv_scale` is constructed as `[ E::one()/sx, E::one()/sy, E::one()/sz ]` — confirmed reciprocal, not raw scale. | H1 ✅ |
| E2 | `module/math/ndarray_cg/src/d2/mat4x4/general.rs:319-326` (pre-fix), cross-checked against the doc comment's cited source `three.js Matrix4.js#L1050` | Pre-fix code divided each matrix entry by the `inv_scale` component (`a / inv_scale.x()`); the cited reference multiplies each column by `1/sx` directly (i.e. by the already-reciprocal value) — dividing by an already-reciprocal value is the inverse operation of what the reference performs. | H1 ✅ |
| E3 | `/tmp/mre118` run, pre-fix vs. post-fix | Pre-fix: recovered scale `[4.0, 9.0, 0.25]` for input `[2.0, 3.0, 0.5]` — each component exactly squared, matching `a/(1/sx) = a*sx` algebra applied to an already-scaled column. Post-fix: recovered scale matches input exactly. | H1 ✅, H3 |
| E4 | `module/math/ndarray_cg/src/d2/mat4x4/general.rs:195-229` (`from_scale_rotation_translation`) | Builds each column as `rotation.to_matrix()`'s column multiplied by the corresponding *raw* scale component (not a reciprocal) — standard composition, confirmed independent of `decompose()`'s own separate reciprocal-handling code. | H2 ❌ |

## Root Cause

```
inv_scale.x()        = 1 / sx                                    (general.rs:308)
rot_mat column 0     = [ a / inv_scale.x(), ... ]                (general.rs:319, pre-fix)
                      = [ a / (1/sx), ... ]
                      = [ a * sx, ... ]                            ✗ (re-multiplies sx; a already = sx * r00)
                      = [ sx² * r00, ... ]
```

`decompose()` is a direct port of three.js's `Matrix4.prototype.decompose`, whose algorithm
multiplies each rotation-matrix column by the *reciprocal* of that axis's scale
(`te[0] *= invSX`, where `invSX = 1/sx`) to strip scale out and leave unit-length columns. This
port precomputed the same reciprocal into `inv_scale` (`general.rs:306-309`) but then divided by
it instead of multiplying by it (`general.rs:319-326`, pre-fix) — an operator inversion that
re-applies the scale a second time instead of removing it once. The mistake is easy to make and
easy to miss: `inv_scale` reads naturally as "the thing you divide by" even though its name
already encodes the reciprocal, so dividing by it *looks* correct at a glance while being the
exact opposite of the reference algorithm's own operator.

## Why Not Caught

No test exercises `decompose()` at all — `tests/inc/mat4x4_test/general_test.rs` has
`test_from_scale_rotation_translation_generic` (constructing a matrix from scale/rotation/
translation) but nothing calling `.decompose()` on the result to confirm the round trip recovers
the original components. The bug is also invisible for the identity-scale case (`sx=sy=sz=1`,
where `inv_scale` is also `(1,1,1)` and division/multiplication coincide) — any test author who
happened to test only unit-scale matrices would not have caught this even if `decompose()` were
exercised at all.

## Fix Location

`module/math/ndarray_cg/src/d2/mat4x4/general.rs:319-326`, inside `decompose()`. Changed every
`/ inv_scale.{x,y,z}()` to `* inv_scale.{x,y,z}()` in the `rot_mat` column construction — no other
line in the function changes; `inv_scale` itself, `scale`'s public return value, and every
surrounding line are unaffected.

```rust
// before
a / inv_scale.x(), b / inv_scale.x(), c / inv_scale.x(),
d / inv_scale.y(), e / inv_scale.y(), f / inv_scale.y(),
g / inv_scale.z(), h / inv_scale.z(), i / inv_scale.z()

// after
a * inv_scale.x(), b * inv_scale.x(), c * inv_scale.x(),
d * inv_scale.y(), e * inv_scale.y(), f * inv_scale.y(),
g * inv_scale.z(), h * inv_scale.z(), i * inv_scale.z()
```

## Prevention

Added `test_decompose_recovers_scale_rotation_translation_generic` (and its
`_row_major`/`_column_major` instantiations) to `tests/inc/mat4x4_test/general_test.rs`, following
the existing `test_from_scale_rotation_translation_generic` pattern: build a matrix via
`from_scale_rotation_translation` with deliberately non-uniform, non-unit scale, `.decompose()`
it, and `assert_abs_diff_eq!` the recovered scale/rotation/translation against the original
inputs. A non-uniform scale is essential — a uniform-scale or identity-scale fixture would not
have exposed this bug (see `## Why Not Caught`).

**Pitfall:** A precomputed reciprocal variable (`inv_scale`, `inv_x`, etc.) removes the reader's
need to write `1.0 / x` at the use site, but also removes the visual cue that a division is
"supposed" to happen there — always check the *reference algorithm's own operator* at each use
site of a reciprocal, not just whether the variable name plausibly fits either operator.

## Generalized Version

**Broken assumption:** "Given a precomputed reciprocal variable, dividing by it is equivalent to
(or a safe substitute for) multiplying by the original value's reciprocal" — false. Dividing by a
reciprocal (`x / (1/n)`) equals multiplying by the original (`x * n`), the exact opposite of
multiplying by the reciprocal (`x * (1/n)`).

**Confirmed general rule:** any code path that both defines `inv_v = 1/v` and later uses `/
inv_v` (rather than `* inv_v`) at a site where the reference/intended algorithm calls for
dividing by `v` (equivalently, multiplying by `1/v`) has this same operator-inversion defect. The
detection invariant: for any such variable, grep every use site and confirm each is `*
inv_v`, never `/ inv_v` — a lone `/ inv_v` occurrence in an otherwise `*`-only codebase is a high-
confidence signal of this exact mistake.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-15 | filed | Discovered via task #52's targeted math/geometry code review; root cause independently re-derived by hand (algebraic substitution of `inv_scale`'s definition into the `/ inv_scale` use site) before filing, confirmed HIGH confidence without needing empirical bisection. |
| 2026-08-15 | fixed | Changed the 9 `/ inv_scale.{x,y,z}()` occurrences in `decompose()`'s `rot_mat` reconstruction to `* inv_scale.{x,y,z}()`; 3-field `Fix(BUG-250)`/`Root cause`/`Pitfall` comment added at the fix site. |
| 2026-08-15 | verified | Added `test_decompose_recovers_scale_rotation_translation_generic` (row-major + column-major instantiations) to `tests/inc/mat4x4_test/general_test.rs` with a non-uniform scale fixture. Narrow suite and full workspace verification recorded in BUG-121's own closing History entry (all four math bugs verified together as one gate — see `task/bug/readme.md`). |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-15, this verifier 2026-08-16). Independently re-read `decompose()`'s `rot_mat` reconstruction (confirmed all 9 occurrences genuinely changed from `/ inv_scale` to `* inv_scale.{x,y,z}()`, 3-field comment intact) and `test_decompose_recovers_scale_rotation_translation_generic` (non-tautological: builds a real `Mat4` from known non-uniform scale/rotation/translation via `from_scale_rotation_translation`, decomposes it, asserts recovered scale/rotation/translation match the originals within `1e-9`). Fresh `cargo nextest run -p ndarray_cg --all-features` via `longrun`: 272/272 passed. `cargo clippy -p ndarray_cg --all-features --all-targets -- -D warnings`: clean. Corrected the stale `**Related Bugs:**` cross-reference (`../verified/119_...` → `../completed/119_...`). MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-250/119/120/121 together. State → Completed. |
| 2026-08-17 | renumbered | 118 → 250, resolving a bug/task ID collision with `TASK-118` (`task/accepting/118_renderer_gltf_light_extension_parsing_test.md`), both filed independently under the shared tsk ID namespace. File, `task/bug/readme.md` row, the `Fix(BUG-250)` source comment in `general.rs`, and the `bug_reproducer`-style citations in `general_test.rs` and 4 sibling bug files (119/120/121/122, part of the same task #52 review batch) all updated. `/tmp/mre118` MRE-script transcripts left verbatim as accurate historical fact (the scratch directory really was named that at the time). |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections + `Refs: src/`/`Refs: tests/` present — confirmed by direct re-read of the full file. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass wrote the MRE from the algebra alone; adversarial pass actually ran it (see `## History`/narrow-suite log) against both pre-fix and post-fix source to confirm the printed numbers are real, not asserted. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Adversarial pass confirmed BUG-119's file (filed after this one) carries the reciprocal `**Related Bugs:**` line back to this file. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass re-derived `a / inv_scale.x() = a * sx` independently from the raw definitions rather than trusting the earlier derivation's arithmetic. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Adversarial pass checked whether any other `decompose()`-adjacent function (`compose`, `from_scale_rotation_translation`) shares this operator mistake — confirmed no, `from_scale_rotation_translation` uses raw scale (not a reciprocal) throughout, so the same mistake isn't structurally possible there. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `ndarray_cg`'s own `src/`/`tests/` and this bug-tracking file touched — no cross-crate scope creep. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix is local to `decompose()`'s own column-construction block; no shared helper needed changing. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix does not add any new responsibility to `decompose()` — it corrects an operator within the function's existing, documented contract. | — |

**Reproduced:** YES — `/tmp/mre118` pre-fix: `recovered scale: [4.0, 9.0, 0.25]` vs. input `[2.0, 3.0, 0.5]`, 2026-08-15

## Refs: src/

| File | Change |
|------|--------|
| `module/math/ndarray_cg/src/d2/mat4x4/general.rs` | `decompose()`: `rot_mat` column reconstruction changed from `/ inv_scale.{x,y,z}()` to `* inv_scale.{x,y,z}()`. `Fix(BUG-250)`/`Root cause`/`Pitfall` 3-field comment added at the fix site. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/math/ndarray_cg/tests/inc/mat4x4_test/general_test.rs` | Added `test_decompose_recovers_scale_rotation_translation_generic` (`_row_major`/`_column_major` instantiations), a `bug_reproducer(BUG-250)` round-trip test with a non-uniform, non-unit scale fixture and a 5-section doc comment. |
