# BUG-447: `Quat::from(Mat3)`'s Shepperd's-method terms can round marginally negative, `.sqrt()`-ing to `NaN`

- **Severity:** Medium (requires a near-degenerate or tiny-angle rotation matrix to trigger the
  rounding; a common case for smoothly-interpolated/accumulated transforms, but not every input)
- **state:** Completed
- **Affects:** Any caller converting a `Mat3` to a `Quat` (`impl From<Mat3<E,Descriptor>> for
  Quat<E>`) where the matrix, though a valid rotation, is only *approximately* orthonormal, or where
  one of Shepperd's method's four trace-derived terms is mathematically at or extremely near zero
  (e.g. a tiny-angle rotation about a single axis).
- **Component:** `module/math/ndarray_cg` (`src/quaternion/from.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Same defect *class* as BUG-272 (`Quat::to_euler_xyz`'s `asin` clamp, the explicit
  reference pattern for this fix) and BUG-445/446 (this same sweep) -- an algebraic non-negativity
  identity that only holds for *exact* inputs, not rounded floating-point ones. Distinct call site
  (`.sqrt()` rather than `.acos()`/`.asin()`) and, unlike BUG-446, this fix's single-sided
  `.max(E::zero())` clamp was confirmed NOT to share BUG-446's NaN-laundering regression -- see Root
  Cause.

## Symptom

```rust
// pre-fix -- a rotation matrix for a very small (0.0006 rad) rotation about the Y axis
let m = Mat3::< f32, _ >::from_row_major([
  0.999_999_8,        0.0, 0.000_599_999_97,
  0.0,                1.0, 0.0,
  -0.000_599_999_97,  0.0, 0.999_999_8,
]);
let q : Quat< f32 > = m.into();
// pre-fix: q.x() (and potentially other components) is NaN
```

Shepperd's method derives four terms `n0..n3` (each algebraically `1 +/- r11 +/- r22 +/- r33`) that are
non-negative *only* when the input matrix is exactly orthonormal. For a tiny-angle rotation, `n1`
(proportional to `x²` for a Y-axis-only rotation, which should be exactly `0`) rounds to a marginally
negative `f32` value; `.sqrt()` of a negative input silently returns `NaN`, which then propagates into
the corresponding quaternion component via `half * n1.sqrt() * signum(...)`.

## Impact

**Who is affected:** Any caller converting an approximately-but-not-exactly-orthonormal matrix to a
quaternion (e.g. one accumulated from repeated transform composition), or any matrix representing a
tiny-angle rotation about a single axis, where the corresponding `n*` term is mathematically at or near
zero.

**What breaks:** One or more quaternion components silently become `NaN`, propagating into any
downstream use exactly like BUG-445/446.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during the same repo-wide discovery sweep as BUG-445/446, auditing every `.sqrt()`/`.acos()`/
`.asin()` call reachable from a computed (not directly caller-supplied) intermediate value against
BUG-272's reference pattern. Shepperd's method's four `n0..n3` terms fit exactly: each is an algebraic
identity relying on exact orthonormality, computed from caller-supplied matrix components that need not
be exactly orthonormal.

## Minimum Reproducible Example

```rust
// module/math/ndarray_cg/tests/inc/quat_test/general.rs
let m = Mat3::< f32, DescriptorOrderColumnMajor >::from_row_major([
  0.999_999_8,        0.0, 0.000_599_999_97,
  0.0,                1.0, 0.0,
  -0.000_599_999_97,  0.0, 0.999_999_8,
]);
let got : Quat< f32 > = m.into();
// pre-fix: one or more of got.x()/y()/z()/w() is NaN
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/math/ndarray_cg && cargo nextest run -E 'test(test_from_mat3_tiny_rotation_no_nan)'
```

## Root Cause

Each of Shepperd's method's four terms (`n0 = 1+r11+r22+r33`, `n1 = 1+r11-r22-r33`, `n2 =
1-r11+r22-r33`, `n3 = 1-r11-r22+r33`) is guaranteed non-negative only when the input `Mat3` is *exactly*
orthonormal. Floating-point rounding -- or a caller-supplied matrix that is only approximately
orthonormal -- can drive one term marginally negative; `.sqrt()` of a negative input is documented to
return `NaN`, which propagates into every component of the resulting quaternion via the `half *
n*.sqrt()` products.

**Confirmed independent of BUG-446's NaN-laundering regression:** the fix uses a single-sided
`n*.max(E::zero())`, not a two-sided `max`/`min` chain. Unlike `vector::angle`'s zero-vector input,
there is no legitimate "this input should stay `NaN`" contract for `n0..n3` -- they are sums/differences
of finite matrix components, so they can only be `NaN` if the *input matrix itself* already contained
`NaN` (a garbage-in-garbage-out case, not a meaningful degenerate-but-valid input the way a zero-length
vector is for `angle`). `NaN.max(0.0)` does evaluate to `0.0` (same IEEE `maxNum` semantics as BUG-446),
but there is no pre-existing test or documented contract expecting `Quat::from(Mat3)` to produce `NaN`
for any valid (even non-orthonormal) input, so this is not a regression -- confirmed by grepping the
full `quat_test`/`d2_test`/`mat3x3h_test` suites for any `is_nan`-based expectation before applying the
fix.

## Why Not Caught

The pre-existing `quat_test/general.rs` `from_mat3`/round-trip tests used well-conditioned, exactly (or
near-exactly, well within rounding tolerance) orthonormal matrices with no term near the zero boundary
-- never a tiny-angle single-axis rotation where one `n*` term itself is the thing being driven negative
by rounding.

## Fix Location

`module/math/ndarray_cg/src/quaternion/from.rs` (`impl From<Mat3<E,Descriptor>> for Quat<E>`): each of
`n0`, `n1`, `n2`, `n3` clamped via `.max(E::zero())` immediately after computation, before any is used
in a `.sqrt()` call.

## Prevention

`test_from_mat3_tiny_rotation_no_nan` (`ndarray_cg/tests/inc/quat_test/general.rs`) uses the
empirically-confirmed 0.0006-rad Y-axis rotation matrix above (values chosen via a targeted numerical
search, confirmed consistent across both debug and `-O` release `rustc` profiles), asserting no
component of the resulting quaternion is `NaN`, plus an exact-value assertion that `x` is precisely
`0.0` (provable exactly: `n1` clamps to `0.0`, `sqrt(0.0)` is exactly `0.0`, and `0.0` times any finite
signum is exactly `0.0` -- no rounding drift possible).

## Pitfall

An algebraic identity that guarantees non-negativity only for *exact* inputs (here: an exactly
orthonormal rotation matrix) does not carry that guarantee into finite-precision floating point --
always clamp before `.sqrt()`/`.acos()`/`.asin()` when the domain constraint is a mathematical property
of exact inputs, not a syntactic property of the formula itself. Unlike BUG-446's `.acos()` clamp,
confirm whether the specific input can *legitimately* be `NaN` before choosing between a NaN-clearing
`max`/`min` chain and a NaN-preserving `.clamp()` -- the two are not interchangeable, and the correct
choice depends on the call site's own contract, not on copying whichever pattern a reference bug used.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during repo-wide bug/UX-DX discovery sweep. |
| 2026-08-20 | fixed | `.max(E::zero())` clamp applied to all four Shepperd's-method terms. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: reproduction matrix values found via a targeted numerical search (scratchpad `rustc` probe) confirming `n1` rounds negative pre-fix, consistent across debug and `-O` release profiles. Adversarial pass: audited whether the single-sided `max(0.0)` clamp could launder a legitimate-NaN input the way BUG-446's two-sided clamp did -- confirmed no such contract exists for this call site (grepped all sibling test files for any `is_nan`-based expectation on `Quat::from(Mat3)`; none found). `cargo nextest run -p mdmath_core -p ndarray_cg --no-fail-fast` -- 395/395 pass. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-447)`/`Root cause`/`Pitfall` 3-field format already present at the call site; 5-section test doc comment (`bug_reproducer(BUG-447)`) added to the new test; 4 `clippy::unreadable_literal` findings on the reproduction matrix's literals fixed with digit-group separators. | Added digit separators to matrix literals. |
| D3 | Scope containment | — | 🟢 | Test-only change (source fix pre-existed from an earlier pass in this same task); confined to `module/math/ndarray_cg/tests/inc/quat_test/general.rs`. `cargo clippy -p mdmath_core -p ndarray_cg --all-targets --all-features -- -D warnings` clean. | — |

**Reproduced:** YES -- the 0.0006-rad Y-axis rotation matrix drives `n1` to a marginally negative `f32`
value pre-fix (confirmed via standalone `rustc` probe), `.sqrt()` of which is `NaN`; post-fix the clamp
rescues `n1` to exactly `0.0`, and the resulting quaternion's `x` component is exactly `0.0` with no
`NaN` in any component. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/math/ndarray_cg/src/quaternion/from.rs` | `impl From<Mat3<E,Descriptor>> for Quat<E>`: `n0`..`n3` each clamped via `.max(E::zero())` before `.sqrt()`; `Fix(BUG-447)`/`Root cause`/`Pitfall` comment (pre-existing from an earlier pass in this task). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/math/ndarray_cg/tests/inc/quat_test/general.rs` | Added `test_from_mat3_tiny_rotation_no_nan`. |
