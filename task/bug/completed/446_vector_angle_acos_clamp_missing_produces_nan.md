# BUG-446: `vector::angle` feeds an unclamped ratio to `.acos()`, producing `NaN` for a vector's angle with itself or its own negation

- **Severity:** High (breaks on inputs as ordinary as `angle(a, a)` or `angle(a, -a)` -- not just
  adversarial or extreme vectors -- for any float type where the rounding happens to land outside
  `[-1, 1]`)
- **state:** Completed
- **Affects:** Any caller of `mdmath_core::vector::angle` where `dot(a,b) / ( mag(a) * mag(b) )` rounds
  to a value fractionally outside `[-1, 1]` -- confirmed concretely for `a = b = [1.0, 0.0, 1.0]` (an
  ordinary, non-adversarial `f32` input) and its exact negation.
- **Component:** `module/math/mdmath_core` (`src/vector/arithmetics.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Same defect *class* as BUG-272 (`Quat::to_euler_xyz`'s `asin` clamp, the explicit
  reference pattern for this fix) and BUG-445/447 (this same sweep). Distinct call site and distinct
  regression discovered while fixing this one: the naive `max`/`min` clamp idiom used at BUG-272 turned
  out to silently launder a *genuine* `NaN` input (from a zero-magnitude vector) into a bogus finite
  angle -- see Root Cause and History below; not present in BUG-272/447's own call sites since neither
  has an equivalent "this input is legitimately, intentionally `NaN`" contract to preserve.

## Symptom

```rust
// pre-fix
let a : [ f32 ; 3 ] = [ 1.0, 0.0, 1.0 ];
let self_angle = vector::angle( &a, &a ); // NaN, not 0.0
let neg_a : [ f32 ; 3 ] = [ -1.0, 0.0, -1.0 ];
let opposite_angle = vector::angle( &a, &neg_a ); // NaN, not PI
```

`angle(a,b)` computes `cos_theta = dot(a,b) / ( mag(a) * mag(b) )` and passes it directly to
`.acos()`. `cos_theta` is mathematically bounded to `[-1,1]`, but `mag(a) = dot(a,a).sqrt()` is itself
rounded, and squaring it back inside the denominator does not always exactly reproduce `dot(a,a)` -- so
the ratio can land fractionally outside `[-1,1]` even for simple, exact-integer-valued inputs.
`.acos()` on an out-of-range input is documented to return `NaN`.

## Impact

**Who is affected:** Any caller computing the angle between a vector and itself, or a vector and its
own exact negation -- both common, non-adversarial operations (e.g. degenerate-input guards elsewhere
in the codebase that compare a vector against itself).

**What breaks:** `angle()` silently returns `NaN` instead of the mathematically correct `0.0`/`PI`, with
no error signal -- propagates into any downstream computation exactly like BUG-445/447.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during the same repo-wide discovery sweep as BUG-445/447, using BUG-272's `to_euler_xyz` asin
clamp as the reference pattern for "unclamped ratio into an inverse-trig function." `vector::angle`'s
`cos_theta` was audited against the same class of defect and found unclamped.

## Minimum Reproducible Example

```rust
// module/math/mdmath_core/tests/inc/arithmetics.rs
let vec_a = [ 1.0, 0.0, 1.0 ];
let self_angle : f32 = vector::angle( &vec_a, &vec_a );
// pre-fix: NaN (dot(a,a)/(mag(a)*mag(a)) rounds to 1.000000119...f32, .acos() of that is NaN)
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/math/mdmath_core && cargo nextest run -E 'test(test_angle_self_and_negation_no_nan) or test(test_angle)'
```

## Root Cause

`cos_theta = dot(a,b) / ( mag(a) * mag(b) )` is mathematically bounded to `[-1,1]`, but `mag(a) =
dot(a,a).sqrt()` rounds, and squaring a rounded square root does not always exactly reproduce the
original value -- so `cos_theta` can round fractionally outside `[-1,1]` even for simple, exact-integer
inputs (empirically confirmed: `a=b=[1.0,0.0,1.0]` rounds `dot(a,a)/(mag(a)*mag(a))` to
`1.000000119..._f32`, strictly greater than `1.0`). `.acos()` on an out-of-range input returns `NaN`.

**A second, initially-introduced defect surfaced while fixing this one:** the first fix attempt used
`cos_theta.max(-E::one()).min(E::one())`, mirroring BUG-272's `sinp.max(-one).min(one)` pattern
verbatim. `f32`/`f64`'s `max`/`min` follow IEEE `maxNum`/`minNum` semantics -- "if one operand is `NaN`,
return the other" -- so `NaN.max(-1.0).min(1.0)` evaluates to `-1.0`, not `NaN`. This silently broke the
*pre-existing* `test_angle` test, which intentionally asserts `angle(a, zero_vector)` is `NaN` (a
zero-magnitude vector has no defined direction, so `NaN` is the documented, correct answer, not a bug --
see `normalize`'s own "Zero-magnitude input" doc note, BUG-448). The `max`/`min` clamp silently
"rescued" that genuine `0.0/0.0` `NaN` into a bogus finite `cos_theta` of `-1.0`, producing `PI` instead
of the intentional `NaN`. Fixed by using `.clamp(-E::one(), E::one())` instead: `num_traits::Float`'s
`clamp` (like `f32`/`f64`'s own inherent `clamp`) is implemented as `if x < min { min } else if x > max
{ max } else { x }` -- both comparisons are `false` for a `NaN` input, so it falls through to `else { x
}` and returns `NaN` unchanged, while still correctly rescuing genuine, finite, rounding-induced
out-of-range values. Empirically confirmed via a standalone `rustc` probe before committing the fix.

## Why Not Caught

The pre-existing `test_angle` only exercised orthogonal vectors (`cos_theta = 0.0` exactly, no rounding
involved) and one deliberately-`NaN` zero-vector case -- never a case where `cos_theta` itself rounds
outside `[-1,1]`, which is exactly what happens for a vector's angle with itself or its exact negation.
The `max`/`min`-vs-`clamp` NaN-handling difference was itself only caught because the fix was verified
against the *full* existing test suite (`--no-fail-fast`), not just the new reproducer in isolation --
running only the new test would have shown a false PASS while silently regressing `test_angle`.

## Fix Location

`module/math/mdmath_core/src/vector/arithmetics.rs::angle`: `cos_theta.max(-E::one()).min(E::one())`
replaced with `cos_theta.clamp(-E::one(), E::one())` before the `.acos()` call.

## Prevention

`test_angle_self_and_negation_no_nan` (`mdmath_core/tests/inc/arithmetics.rs`) asserts `angle(a,a)` and
`angle(a,-a)` are not `NaN` and equal `0.0`/`PI` respectively, using the empirically-confirmed
`a=[1.0,0.0,1.0]` input. The pre-existing `test_angle`'s zero-vector `NaN`-expectation assertion itself
now doubles as a regression guard against the `max`/`min`-vs-`clamp` NaN-laundering defect -- it is run
as part of every verification pass and would fail again if the clamp were ever reverted to a `max`/`min`
chain.

## Pitfall

`x.max(lo).min(hi)` and `x.clamp(lo,hi)` are **not** interchangeable when `x` may legitimately be `NaN`
-- `max`/`min` follow IEEE `maxNum`/`minNum` semantics and silently discard `NaN` (return the other,
non-`NaN` operand), while `clamp` preserves it (returns `x` unchanged whenever both `x < min` and `x >
max` are false, which includes `NaN`). Prefer `clamp` for any defensive pre-`acos`/`asin`/`sqrt` rescue
where the input could legitimately be `NaN` from an upstream `0/0` or negative-domain case that must
stay `NaN` -- do not copy the `max`/`min` idiom from a reference call site without checking whether
*that* site has the same "input may be genuinely, intentionally NaN" contract this one does.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during repo-wide bug/UX-DX discovery sweep. |
| 2026-08-20 | fixed | First attempt (`max`/`min`, mirroring BUG-272) regressed the pre-existing `test_angle` zero-vector case; corrected to `.clamp()`, verified NaN-preserving via a standalone `rustc` probe before re-applying. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | 🔴 | 🟢 | Confirming pass: `.clamp()`'s NaN-preserving behavior verified via standalone `rustc` probe (`nan.clamp(-1,1)` -> `NaN`) before committing the fix, and `num_traits::Float::clamp`'s source (`crate::clamp`, an `if/else` chain with no NaN-clearing branch) read directly to confirm the generic path matches. Adversarial pass: first fix attempt (`max`/`min`) actually shipped, ran against the full suite, and was caught failing `test_angle` -- a real regression, not a hypothetical one -- then corrected. `cargo nextest run -p mdmath_core -p ndarray_cg --no-fail-fast` -- 395/395 pass post-correction. | Replaced `max`/`min` clamp with `.clamp()`. |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-446)`/`Root cause`/`Pitfall` 3-field format applied at the call site, including the max/min-vs-clamp pitfall discovered mid-fix; 5-section test doc comment (`bug_reproducer(BUG-446)`) on the new test. | — |
| D3 | Scope containment | — | 🟢 | Change confined to a single line in `module/math/mdmath_core/src/vector/arithmetics.rs::angle` plus its own test file. `cargo clippy -p mdmath_core -p ndarray_cg --all-targets --all-features -- -D warnings` clean. | — |

**Reproduced:** YES -- `a=b=[1.0,0.0,1.0]` (`f32`) drives `cos_theta` to `1.000000119..._f32` pre-fix,
`.acos()` of which is `NaN`; post-fix the clamp rescues it to exactly `1.0` and `.acos(1.0) = 0.0`.
Confirmed via standalone `rustc` probe and the passing test. The intermediate `max`/`min` regression
against `test_angle`'s zero-vector case was also directly reproduced (test failed with "Angle
calculation failed for zero vector") before being corrected. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/math/mdmath_core/src/vector/arithmetics.rs` | `angle`: `cos_theta.max(-E::one()).min(E::one())` -> `cos_theta.clamp(-E::one(), E::one())`; `Fix(BUG-446)`/`Root cause`/`Pitfall` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/math/mdmath_core/tests/inc/arithmetics.rs` | Added `test_angle_self_and_negation_no_nan`. |
