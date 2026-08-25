# BUG-445: `Mat3::look_at`/`look_to_rh`/`look_at_rh` produce a `NaN` basis when `dir`/`up` are (numerically) parallel

- **Severity:** High (silently produces a fully-`NaN` rotation/view matrix for an ordinary,
  frequently-hit camera orientation -- top-down or bottom-up, with the conventional world-up hint --
  not just an adversarial edge case)
- **state:** Completed
- **Affects:** Any caller of `d2::rotation::Rotation::look_at` (impl for `Mat3<E,Descriptor>`), or
  `d2::mat3x3h::transformation::look_to_rh`/`look_at_rh`, with a view/look direction parallel or
  antiparallel to the supplied "up" hint -- most commonly a straight-down or straight-up camera paired
  with the default world-up `(0,1,0)`.
- **Component:** `module/math/ndarray_cg` (`src/d2/rotation.rs`, `src/d2/mat3x3h/transformation.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Same defect *class* as BUG-272 (`Quat::to_euler_xyz`'s missing `asin` clamp) and
  BUG-446/447 (this same sweep) -- an unguarded floating-point edge case in a geometric construction --
  but a distinct mechanism (a zero cross product from parallel vectors, not an out-of-domain
  `acos`/`asin` input) and a distinct call site; filed separately, no shared root cause beyond "defensive
  floating-point guards were missing."

## Symptom

```rust
// pre-fix
let dir = Vector::< f32, 3 >::from_array( [ 0.0, -1.0, 0.0 ] ); // looking straight down
let up  = Vector::< f32, 3 >::from_array( [ 0.0,  1.0, 0.0 ] ); // conventional world-up
let rotation = Mat3::< f32, _ >::look_at( &dir, &up );
// every component of `rotation` is NaN
```

`look_at`/`look_to_rh` derive the basis's `x` axis as `normalized( cross( z, up ) )`, with `z =
normalized( dir )`. When `dir` and `up` are (numerically) parallel or antiparallel, `cross( z, up )` is
the exact zero vector, and `normalized()` on a zero vector divides `0.0 / 0.0`, i.e. `NaN` -- which then
propagates through `y = cross( x, z )` into the entire returned matrix.

## Impact

**Who is affected:** Any caller constructing a look-at/view basis for a camera (or any other
directional frame) whose direction happens to be parallel to its own up hint. A top-down or bottom-up
camera view -- an ordinary, non-exotic orientation -- with the default world-up `(0,1,0)` hits this
exactly, not merely under adversarial input.

**What breaks:** The entire returned `Mat3`/`Mat4` becomes `NaN` in every component -- any downstream
consumer (rendering, physics, further transform composition) silently receives garbage with no error
signal; `NaN` does not panic or clamp, it propagates.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX-DX discovery sweep of `module/math/ndarray_cg` and
`module/math/mdmath_core`, using BUG-272's already-fixed `to_euler_xyz` asin-clamp as the reference
pattern for "unguarded floating-point domain edge in a geometric construction." `look_at`/`look_to_rh`'s
`normalized( cross( z, up ) )` step was audited for the same class of defect and found to have no
degeneracy guard at all (unlike `between_vectors`, a sibling function in the same file, which already
handled its own analogous antiparallel-input case via a helper-axis fallback).

## Minimum Reproducible Example

```rust
// module/math/ndarray_cg/tests/inc/d2_test/rotation_test.rs
let dir = Vector::< f32, 3 >::from_array( [ 0.0, -1.0, 0.0 ] );
let up  = Vector::< f32, 3 >::from_array( [ 0.0,  1.0, 0.0 ] );
let rotation = Mat3::< f32, DescriptorOrderRowMajor >::look_at( &dir, &up );
// pre-fix: every component of `rotation` is NaN
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/math/ndarray_cg && cargo nextest run -E 'test(test_look_at_parallel_up_no_nan) or test(test_look_to_rh_parallel_up_no_nan)'
```

## Root Cause

A right-handed look-at/view basis derives its `x` axis as `normalized( cross( z, up ) )` with no guard
for `dir`/`up` degeneracy. When `z` (the normalized view direction) and `up` are numerically parallel or
antiparallel, `cross( z, up )` is the exact zero vector; `normalized()` on a zero-magnitude vector
divides `0.0 / 0.0`, silently producing `NaN`, which then propagates into `y = cross( x, z )` and the
entire assembled matrix. `look_at_rh` has no basis-construction logic of its own -- it computes `dir =
center - eye` and delegates to `look_to_rh`, so it is covered transitively by the same fix.

## Why Not Caught

Neither `look_at`'s nor `look_to_rh`'s test coverage exercised a parallel `dir`/`up` pair before this
task -- `look_to_rh`/`look_at_rh` had no dedicated tests at all beyond `rot`/`scale`/`translation`, and
`look_at`'s existing tests used non-degenerate direction/up pairs. The sibling function
`between_vectors` (same file as `look_at`) already had a fallback for its own analogous antiparallel
case, but that fix was never generalized to `look_at`/`look_to_rh`'s independent, structurally identical
degeneracy.

## Fix Location

`module/math/ndarray_cg/src/d2/rotation.rs` (`Rotation::look_at` impl for `Mat3<E,Descriptor>`) and
`module/math/ndarray_cg/src/d2/mat3x3h/transformation.rs` (`look_to_rh`): both now guard with `mag(
cross( z, up ) ) < 1e-6` and, when triggered, derive `x` via `normalized( cross( z, non_parallel_hint(
z ) ) )` instead -- `non_parallel_hint` (shared with `between_vectors`'s own existing fallback) picks a
helper axis guaranteed not parallel to `z`. `look_at_rh` required no direct change; it is fixed
transitively through `look_to_rh`.

## Prevention

Three new tests: `test_look_at_parallel_up_no_nan_row_major`/`_column_major` (generic over both
`Mat3` descriptor orderings, `d2_test/rotation_test.rs`) and `test_look_to_rh_parallel_up_no_nan`
(`mat3x3h_test/transformation_test.rs`), all using the exact top-down-camera reproduction above and
asserting the resulting matrix exactly equals the deterministic `non_parallel_hint`-derived basis.
`NaN` is unequal to everything including itself, so a plain `assert_eq!` against the expected matrix is
sufficient to prove the fix without a separate per-element `is_nan()` check.

## Pitfall

Any `normalized( cross( a, b ) )` basis construction needs an explicit guard for `a`/`b` being
(numerically) parallel -- the zero cross product itself is silent (no panic, no early `NaN`), so the
defect only surfaces once the degenerate basis is actually used, far from the construction site. When a
sibling function in the same file already solves this exact degeneracy (here, `between_vectors`'s
existing fallback), check whether its fix generalizes to other unguarded call sites in the same module
before treating each as an isolated case.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during repo-wide bug/UX-DX discovery sweep; fix and tests implemented same session. |
| 2026-08-20 | fixed | Guard + `non_parallel_hint` fallback applied to `look_at` and `look_to_rh`; `look_at_rh` fixed transitively. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: reproducer inputs empirically verified (scratchpad `rustc` probe) to hit the exact `cross(z,up)==0` degeneracy pre-fix. Adversarial pass: reasoned through the pre-fix code path by inspection (guard did not exist) to confirm the test would have failed with `NaN != expected` before the fix landed; `cargo nextest run -p mdmath_core -p ndarray_cg --no-fail-fast` -- 395/395 pass post-fix. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-445)`/`Root cause`/`Pitfall` 3-field format applied at both call sites (`rotation.rs`, `mat3x3h/transformation.rs`); 5-section test doc comment (`bug_reproducer(BUG-445)`) on all 3 new tests. | — |
| D3 | Scope containment | — | 🟢 | Changes confined to `module/math/ndarray_cg/src/d2/rotation.rs`, `src/d2/mat3x3h/transformation.rs`, and their own test files -- within the assigned edit scope. `cargo clippy -p mdmath_core -p ndarray_cg --all-targets --all-features -- -D warnings` clean. | — |

**Reproduced:** YES -- the top-down-camera input (`dir=(0,-1,0)`, `up=(0,1,0)`) drives `cross(z,up)` to
the exact zero vector pre-fix, verified algebraically and via a standalone `rustc` probe; post-fix the
same input produces the deterministic `non_parallel_hint`-derived basis, confirmed by all 3 new tests
passing. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/math/ndarray_cg/src/d2/rotation.rs` | `Rotation::look_at` (impl for `Mat3<E,Descriptor>`): added `mag(cross(z,up)) < 1e-6` guard with `non_parallel_hint` fallback; `Fix(BUG-445)`/`Root cause`/`Pitfall` comment. |
| `module/math/ndarray_cg/src/d2/mat3x3h/transformation.rs` | `look_to_rh`: same guard/fallback; added `# Panics` doc section for the pre-existing `E::from(1.0e-6).unwrap()` call (required by `clippy::missing_panics_doc` once the crate was actually linted). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/math/ndarray_cg/tests/inc/d2_test/rotation_test.rs` | Added `test_look_at_parallel_up_no_nan_generic` (+ `_row_major`/`_column_major` wrappers). |
| `module/math/ndarray_cg/tests/inc/mat3x3h_test/transformation_test.rs` | Added `test_look_to_rh_parallel_up_no_nan`; added `look_to_rh` to the file's top-level import. |
