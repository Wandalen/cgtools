# BUG-298: `Quat::invert()` returns the bare conjugate unconditionally, silently wrong for any non-unit-length quaternion

- **Severity:** Medium (currently zero reachable call sites anywhere in the workspace for
  `invert`/`devide`/`Div`/`DivAssign` on `Quat` -- a latent defect, not an active regression;
  would be High if any caller existed, since the wrong result is silent -- no panic, no error --
  and scales with the divisor's squared magnitude)
- **state:** Verified
- **Affects:** `Quat<E>::invert()`, and every caller reached through it: `devide()`,
  `device_mut()`, `Div for Quat<E>`, `DivAssign for Quat<E>` (the Quat/Quat forms only -- the
  Quat/scalar `Div<E>`/`DivAssign<E>` forms do not call `invert` and are unaffected)
- **Component:** `module/math/ndarray_cg` (`src/quaternion/arithmetics.rs`,
  `src/quaternion/operator/div.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/ (self)
- **verification_date:** 2026-08-18
- **Fix Task:** [357](../../verifying/357_fix_quat_invert_wrong_for_non_unit_quaternions_bug298.md) (filed via `bug_promote` skill/PROC12, 2026-08-18; Readiness Verification Gate PASS 8/8; blocked at 🔬 Verifying by this sandbox's same-actor `tsk .verify_pass` guard)

## Symptom

```bash
# pre-fix: dividing by a non-unit quaternion, then multiplying back by the same divisor,
# does not recover the original value -- it comes back scaled by the divisor's |q|^2
$ cargo test -p ndarray_cg --all-features test_devide_non_unit_round_trip
thread '...test_devide_non_unit_round_trip' panicked at tests/inc/quat_test/arithmetic.rs:99:3:
assert_abs_diff_eq!(reconstructed, q1, epsilon = 1e-9)

    left  = Quat(Vector([135.0, 270.0, 405.0, 540.0]))   # wrong -- (q1/q2)*q2, non-unit q2
    right = Quat(Vector([1.0, 2.0, 3.0, 4.0]))            # correct -- should equal q1

# post-fix (same test): 1 passed
```

## Impact

**Who is affected:** any current or future caller of `Quat::invert()`, `devide()`,
`device_mut()`, or the `Div`/`DivAssign` operators between two `Quat` values, whenever the
right-hand-side quaternion is not already unit-length. Currently zero such callers exist
anywhere in the workspace (confirmed by grep below), so this is a latent defect rather than an
active regression -- but the public API accepts any `Quat<E>` value with no unit-length
enforcement, so any future caller (e.g. blending/extrapolating raw, non-normalized quaternion
deltas) would hit this silently.

**What breaks:** silent wrong output, not a panic or error -- `a.devide(&b)` for a non-unit `b`
returns a value scaled away from the true quotient by a factor related to `b`'s squared
magnitude, and nothing signals the caller that the precondition was violated.

**Magnitude:** every call where the divisor is not unit-length; unit-length divisors are
unaffected (the formula reduces to the pre-fix behavior exactly in that case, so `test_devide`,
the pre-existing test that only exercises normalized operands, keeps passing before and after).

**Entity Scope:** `None` -- source-level formula defect, not entity directory instances.

## How Discovered

Assigned review of `module/math` + `module/min` per this session's workspace-wide bug-hunt task.
Reading `quaternion/arithmetics.rs` end to end, `invert()`'s own doc comment ("Inverts the
unit-length quaternion, which is equivalent to its conjugate") named an explicit precondition
that its signature (`pub fn invert( &self ) -> Self`, taking any `Quat<E>`) does nothing to
enforce or check. Tracing callers found `devide()`/`Div`/`DivAssign` all route through it
unconditionally. Writing a round-trip test with deliberately non-unit operands (`(a/b)*b == a`,
the defining property of division) reproduced the doc comment's implied precondition violation
immediately.

```bash
$ grep -rn "\.invert()\|\.devide(\|device_mut(" --include=*.rs /home/user1/pro/lib/yrd_gamedev/cgtools \
    | grep -v "/quaternion/arithmetics.rs\|/quaternion/operator/div.rs\|/quaternion/operator.rs"
# (no output -- zero external callers of any of these four APIs anywhere in the workspace)
```

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p ndarray_cg --all-features test_devide_non_unit_round_trip
```
**What:** violates the defining algebraic property of division for a group action --
`(a / b) * b == a` for any nonzero `b` -- using two deliberately non-unit-length operands.

**Expected** (fixed): `test result: ok. 1 passed`.

**Actual** (pre-fix, real capture via temporary revert-and-rerun of the fix, this session):
```
thread '...test_devide_non_unit_round_trip' panicked at tests/inc/quat_test/arithmetic.rs:99:3:
assert_abs_diff_eq!(reconstructed, q1, epsilon = 1e-9)
    left  = Quat(Vector([135.0, 270.0, 405.0, 540.0]))
    right = Quat(Vector([1.0, 2.0, 3.0, 4.0]))
test result: FAILED. 0 passed; 1 failed
```
(`135.0 == 1.0 * 135.0`; `135` is exactly `mag2()` of the divisor `q2 = [-5,1,3,10]`, i.e.
`25+1+9+100`, confirming the reconstructed value is `q1` scaled by the divisor's squared
magnitude rather than recovered exactly.)

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `invert()` is only correct for unit-length quaternions because it returns the bare conjugate with no magnitude normalization | ✅ Root Cause | `arithmetics.rs:224-227`: `invert()` body is exactly `self.conjugate()`, no division by `mag2()` | E1, E2, E3 |
| H2 | `devide`/`Div`/`DivAssign` inherit the defect because they call `invert()` directly with no independent normalization step | ✅ Verified | `arithmetics.rs:147-150` (`devide`) and `operator/div.rs:25-28` (`Div`), `operator/div.rs:47-50` (`DivAssign`) all call `.invert()` on the RHS with no guard | E2, E4 |
| H3 | The bug is unreachable in current production code because no caller exists yet | ✅ Verified | Workspace-wide grep for `.invert()`/`.devide(`/`device_mut(` outside the defining files returns zero matches | E5 |
| H4 | The unit-quaternion case (as used by `test_devide`, the only pre-existing division test) is unaffected by this defect | ✅ Verified | `mag2()` of a unit quaternion is `1` by definition, so `conjugate() / 1 == conjugate()` -- pre-fix and post-fix formulas coincide exactly for unit operands | E6 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/quaternion/arithmetics.rs:221` | Doc comment: "Inverts the unit-length quaternion, which is equivalent to its conjugate" -- names the precondition explicitly | H1 ✅ |
| E2 | `src/quaternion/arithmetics.rs:224-227` | `pub fn invert( &self ) -> Self { self.conjugate() }` -- no magnitude term anywhere in the body | H1 ✅, H2 ✅ (symptom) |
| E3 | Terminal output (Symptom / MRE sections above) | `(q1.devide(&q2)) * q2 == Quat([135.0, 270.0, 405.0, 540.0])` instead of `q1 == Quat([1.0, 2.0, 3.0, 4.0])`; `135 == q2.mag2()` exactly | H1 ✅ (demonstrates) |
| E4 | `src/quaternion/operator/div.rs:25-28`, `:47-50` | `Div for Quat<E>::div` and `DivAssign for Quat<E>::div_assign` both call `self.devide(&rhs)` / `*self / rhs`, no independent handling | H2 ✅ |
| E5 | Terminal output (How Discovered section above) | Workspace-wide grep for `.invert()`/`.devide(`/`device_mut(` outside the four defining files: zero matches | H3 ✅ |
| E6 | `src/quaternion/arithmetics.rs:78-90` | `mag2()`/`mag()` are pre-existing public methods on `Quat<E>`, confirming a magnitude term was available but unused by `invert()` | H1 ✅ (symptom), H4 ✅ |

## Root Cause

```
Quat::devide( a, b )        -> a * b.invert()                     (arithmetics.rs:147-150)
Div for Quat / DivAssign    -> route to devide()                  (operator/div.rs:25-28,47-50)
Quat::invert( q )           -> q.conjugate()                      (arithmetics.rs:224-227)  ✗
                              should be: q.conjugate() / q.mag2()
```
The general formula for a quaternion's multiplicative inverse is `q⁻¹ = conjugate(q) / |q|²`;
it reduces to the bare conjugate only in the special case `|q|² = 1` (unit-length). `invert()`
implemented only the special-case shortcut, unconditionally, so every caller reached through it
-- `devide`, `Div`, `DivAssign` for `Quat`/`Quat` -- silently inherited the narrower assumption
for any divisor that is not already normalized.

## Why Not Caught

The only pre-existing division test, `test_devide` (`tests/inc/quat_test/arithmetic.rs:34-53`),
calls `.normalize()` on both operands before dividing. For a unit quaternion `mag2() == 1`, so
`conjugate()` and the true inverse `conjugate()/mag2()` are numerically identical -- the defect
is invisible under that test's own inputs. No test exercised division (or `invert()` directly)
with a non-unit operand, and no test checked the defining algebraic property of division,
`(a / b) * b == a`, which is exactly the property this bug violates.

## Fix Location

`src/quaternion/arithmetics.rs:221-227`:

```rust
// Before:
/// Inverts the unit-length quaternion, which is equivalent to its conjugate.
#[ inline ]
#[ must_use ]
pub fn invert( &self ) -> Self
{
  self.conjugate()
}

// After:
/// Inverts the quaternion, producing its multiplicative inverse. Reduces to the conjugate
/// for a unit-length quaternion ( `mag2() == 1` ), and is the general formula otherwise.
#[ inline ]
#[ must_use ]
pub fn invert( &self ) -> Self
{
  self.conjugate() / self.mag2()
}
```

No change needed in `devide`/`device_mut`/`Div`/`DivAssign` (`operator/div.rs`) -- they become
correct automatically once `invert()` itself is correct, since all four already route through it.

## Prevention

Add (done, see MRE) a division round-trip test using deliberately non-unit operands, asserting
`(a / b) * b == a` -- the defining property of division -- rather than only ever testing with
pre-normalized inputs. Detection command for the general pattern (any `invert`/`inverse`-named
method whose doc comment states a precondition the signature does not enforce):
```bash
grep -B2 "pub fn invert\|pub fn inverse" src/**/*.rs | grep -i "unit-length\|normalized\|assumes"
```

**Pitfall:** A function whose doc comment names a precondition ("unit-length") but whose
signature accepts any value of the type provides no compile-time or run-time signal when that
precondition is violated -- every caller reached through a general-purpose operator overload
(here, `Div`/`DivAssign`) silently inherits the narrower assumption with no local indication
anything is wrong. Prefer implementing the operation correctly for the general case when doing
so costs no more than the special-case shortcut, rather than documenting an unchecked
precondition.

## Generalized Version

**Broken assumption:** `conjugate(q) == inverse(q)` -- only true when `|q|² = 1`.

Fails for any `Quat::invert()`/`devide()`/`Div`/`DivAssign` (Quat/Quat) call when:
1. The right-hand operand (the value being inverted / divided by) is not unit-length, AND
2. No caller-side normalization is applied before the call

**Detection invariant:**
```
for all nonzero q: (a.devide(&q)) * q == a   (within floating-point epsilon)
```

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Found during this session's workspace-wide bug-hunt task, assigned review of `module/math`/`module/min` |
| 2026-08-18 | fix_applied | `src/quaternion/arithmetics.rs:224-227`: `invert()` changed from `self.conjugate()` to `self.conjugate() / self.mag2()` |
| 2026-08-18 | verified | `test_devide_non_unit_round_trip` (bug_reproducer) passes; full `ndarray_cg` scoped suite (281 passed) and clippy (`-D warnings`) clean |
| 2026-08-18 | promoted to fix task | Linked to [Task 357](../../verifying/357_fix_quat_invert_wrong_for_non_unit_quaternions_bug298.md) via the `bug_promote` skill (PROC12) — formal task-system registration of this bug's already-applied, already-verified fix. Task 357 reached its own Readiness Verification Gate PASS 8/8 (8-dimension Tier 2 Dual-Role Self-Check) and is blocked on `tsk .verify_pass`'s same-actor guard (identical to this bug's own filing/verifying actor), same standing pattern as this backlog's other same-actor-blocked tasks (e.g. 254, 358). |

## Refs: src/

- `src/quaternion/arithmetics.rs` — `invert()` changed to `self.conjugate() / self.mag2()`

## Refs: tests/

- `tests/inc/quat_test/arithmetic.rs` — added `test_devide_non_unit_round_trip` (bug_reproducer)

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | — | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | MRE uses an in-repo `cargo test` command, not literal `/tmp/mreNNN/` paths — deliberate, precedented local adaptation for a math-crate defect (matches BUG-272/BUG-120's own already-verified shape in this same crate), not an oversight | — |
| D3 | Cross-Reference Integrity | 🟠 | 🟢 | Adversarial pass caught missing `## Refs:` sections + FI027 backreferences | Added `Refs: src/`+`Refs: tests/` sections and backreference comments in both files; re-verified via `grep -rn 'BUG-298' src/ tests/` |
| D4 | Root Cause Quality | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 0 open | 1/1 |

**Reproduced:** YES — exit 0 (`test_devide_non_unit_round_trip` ... ok), 2026-08-18. Full `ndarray_cg` scoped suite (281 passed / 0 failed) and `cargo clippy -p ndarray_cg --all-targets --all-features -- -D warnings` (clean) also re-confirmed post-fix.
