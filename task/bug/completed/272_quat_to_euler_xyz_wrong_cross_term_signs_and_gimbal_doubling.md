# BUG-272: `Quat::to_euler_xyz` extracts pitch/roll/yaw via cross terms with the wrong sign, and its gimbal-lock branch doubles the wrong side of `atan2`

- **Severity:** High (silently wrong output for any genuine multi-axis rotation and for gimbal
  lock with nonzero roll/yaw -- not a panic or crash, but the function's advertised purpose,
  correct Euler-angle decomposition, was broken for the general case)
- **state:** Completed
- **Affects:** `Quat<E>::to_euler_xyz` (all instantiations, incl. `QuatF64`/`QuatF32`)
- **Component:** `module/math/ndarray_cg` (`src/quaternion/arithmetics.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`to_euler_xyz` derives pitch via `asin` and roll/yaw via `atan2` from the quaternion's `x,y,z,w`
components. Three defects, all in the same function:
1. The pitch, roll, and yaw cross terms each had a transcription-flipped sign: pitch used `w*y -
   z*x` (should be `w*y + z*x`), roll used `w*x + y*z` (should be `w*x - y*z`), yaw used `w*z +
   x*y` (should be `w*z - x*y`).
2. The gimbal-lock branch's (`pitch == +/-90 deg`) collapsed-yaw `atan2` denominator used `y*y +
   z*z` instead of `x*x + z*z`.
3. That same gimbal-lock branch computed `two * ( x * y + w * z ).atan2( .. )` -- Rust
   method-call precedence binds `.atan2` tighter than the leading `*`, so this doubles the
   *result* of `atan2`, not its first argument; `2 * atan2( n, d )` and `atan2( 2*n, d )` are
   different functions whenever `n != 0`.

Together, any genuinely multi-axis rotation (away from gimbal lock) or any gimbal-lock rotation
with nonzero roll and yaw returned the wrong Euler angles; only single-axis rotations and
exact-zero-roll/yaw gimbal lock happened to still read back correctly.

## Impact

**Who is affected:** any caller decomposing a quaternion into Euler XYZ angles via
`to_euler_xyz` for a rotation that is not purely single-axis (e.g. a UI inspector,
animation-retargeting, or debug-display code reading back a composed rotation).

**What breaks:** the returned `[roll, pitch, yaw]` does not represent the input rotation for any
genuine multi-axis case -- silently wrong data, not a panic, with no signal to the caller that
anything is off.

**Entity Scope:** `None` -- source-level formula defect, not entity directory instances.

## How Discovered

Assigned review of `module/math/ndarray_cg`'s quaternion files per this session's task.
`quaternion/from.rs`'s `Mat3::from_quat`/Shepperd's-method conversion was independently verified
correct (round-trips through the crate's own `Mat3::from_quat`), then used as ground truth to
hand-derive the correct `to_euler_xyz` formulas via matrix-product expansion (`R = Rx(roll) *
Ry(pitch) * Rz(yaw)`, matched against the standard quaternion-to-matrix entries). The derived
formulas disagreed with the existing code's signs on all three cross terms. Writing a genuinely
multi-axis round-trip regression test (`30 deg / 20 deg / 10 deg`, not previously exercised by
any existing test) first exposed the sign errors; extending that test to an exact gimbal-lock
case with nonzero roll/yaw (`30 deg / 90 deg / 20 deg`) then exposed the second, independent
doubling defect after the sign/denominator fix alone still failed that case.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p ndarray_cg --all-features test_to_euler_xyz_multi_axis_round_trip
```
**Expected** (fixed): 1 passed.

**Actual** (pre-fix, two separate real captures during this session):
- With all three defects reverted together (temporary direct-source-edit revert, real run): fails
  at the away-from-gimbal-lock assertion (`arithmetic.rs:350:3`,
  `assert_abs_diff_eq!(result, F64x3::from_array(input), epsilon = 1e-6)`):
  ```
  left  = Vector([0.5891131125151796, 0.2063351064307799, 0.3317567790593114])
  right = Vector([0.5235987755982988, 0.3490658503988659, 0.17453292519943295])
  ```
  (`right` is the input `30/20/10` degrees in radians.)
- With only the sign/denominator defects fixed (doubling defect not yet identified/fixed): fails
  at the gimbal-lock assertion for input `30/90/20` degrees:
  ```
  left  = Vector([0.0, 1.5707963267948966, 1.074764434326334])   // yaw = 61.586 deg
  right = Vector([0.0, 1.5707963267948966, 0.8726646259971648])  // yaw = 50 deg (correct)
  ```

## Root Cause

`to_euler_xyz` (pre-fix), abbreviated:
```rust
let sinp = two * ( w * y - z * x );              // WRONG sign -- should be `w * y + z * x`
let pitch = sinp.asin();
if ( sinp.abs() - one ).abs() < eps
{
  let yaw = two * ( x * y + w * z ).atan2( one - two * ( y * y + z * z ) );
  //        ^^^^^^ doubles the atan2 RESULT, not its numerator; denominator dims also wrong
  return [ E::zero(), pitch, wrap_pi( yaw ) ].into();
}
let mut roll = ( two * ( w * x + y * z ) ).atan2( one - two * ( x * x + y * y ) );  // WRONG sign
let mut yaw  = ( two * ( w * z + x * y ) ).atan2( one - two * ( y * y + z * z ) );  // WRONG sign
```
Deriving `R = Rx(roll) * Ry(pitch) * Rz(yaw)` explicitly (matching this crate's own
`from_euler_xyz` composition order, itself verified via the `multiply` Hamilton-product formula)
and comparing to the standard quaternion-to-rotation-matrix entries shows: `sin(pitch) = 2*(w*y +
z*x)`, `roll = atan2( 2*(w*x - y*z), 1 - 2*(x*x+y*y) )`, `yaw = atan2( 2*(w*z - x*y), 1 -
2*(y*y+z*z) )` -- each cross term's sign was flipped relative to this derivation. Independently,
the gimbal-lock branch's collapsed angle is `atan2( 2*(x*y+w*z), 1 - 2*(x*x+z*z) )` (verified by
hand for both `pitch = +90 deg` and `pitch = -90 deg`, confirming one formula covers both poles
without a sign branch); the pre-fix code had both the wrong denominator dimensions (`y*y+z*z`
instead of `x*x+z*z`) and, on top of that, multiplied the finished `atan2` angle by 2 instead of
doubling its numerator before the call -- two independent defects compounding in the same branch.

## Why Not Caught

The pre-existing `test_to_euler_xyz`/`test_to_euler_xyz_from_raw_quat` cases used only small
angles (1-3 degrees, where a flipped cross term is numerically tiny relative to those tests' loose
`epsilon = 1e-1`) or single-axis rotations (where the mismatched cross term multiplies an
always-zero component, and where gimbal-lock roll/yaw are both zero -- the one condition under
which `2 * atan2(n,d)` and `atan2(2*n,d)` coincide, since `n = 0` either way). One pre-existing
case (`test_to_euler_xyz_from_raw_quat`'s 4th case) additionally carried its own latent,
independent defect: its raw quat literal `[0.707, 0.0, 0.707, 0.0]` does not actually represent
its claimed `Ry(-90 deg)` (it has `w = 0`, i.e. it is a 180 degree rotation about `(1,0,1)/sqrt(2)`)
-- it only "passed" pre-fix because the buggy `sinp` formula coincidentally evaluated to `-1.0`
for that specific (wrong) quat too, another instance of the same masking pattern. No existing
test exercised a genuine multi-axis rotation or a gimbal-lock case with nonzero roll/yaw at tight
tolerance.

## Fix Applied (2026-08-17)

**`src/quaternion/arithmetics.rs`:** in `to_euler_xyz`, corrected the pitch/roll/yaw cross-term
signs, the gimbal-lock branch's denominator (`y*y+z*z` -> `x*x+z*z`), and parenthesized the
gimbal-lock branch's doubled numerator (`two * ( x*y + w*z ).atan2( .. )` ->
`( two * ( x*y + w*z ) ).atan2( .. )`).

**`tests/inc/quat_test/arithmetic.rs`:**
- Extended `test_to_euler_xyz_multi_axis_round_trip` (`test_kind: bug_reproducer(BUG-272)`) with
  a genuinely multi-axis case away from gimbal lock (checked by direct angle comparison, since the
  decomposition is unique there) and gimbal-lock cases at *both* poles (`pitch = +90 deg` and
  `pitch = -90 deg`) with nonzero roll and yaw -- each gimbal case checks the unambiguous
  roll/pitch directly, then re-composes the reported angles through `from_euler_xyz` again and
  asserts the round-tripped quaternion represents the same rotation as the original (`|dot| ~= 1`,
  since unit quaternions `p`/`-p` both encode the same rotation) rather than hardcoding a fragile
  hand-derived collapsed-angle literal.
- Corrected `test_to_euler_xyz_from_raw_quat`'s pre-existing 4th case: its raw quat literal
  `[0.707, 0.0, 0.707, 0.0]` (which never actually represented `Ry(-90 deg)`) was replaced with
  `[0.0, -FRAC_1_SQRT_2, 0.0, FRAC_1_SQRT_2]`, the quaternion `from_angle_y(-90 deg)` actually
  produces (expressed via the named constant, since the equivalent decimal literal trips
  clippy's `approx_constant`).

## Verification

`longrun`-detached, from the repo root:
- `cargo test -p ndarray_cg --all-features test_to_euler_xyz_multi_axis_round_trip` -- pre-fix
  (temporary direct-source-edit revert of all three defects together, real run): 1 failed, exact
  assertion output captured above (away-from-gimbal-lock case). Post-fix (restored): 1 passed.
- `cargo test -p ndarray_cg --all-features` (full scoped suite): 278 passed / 0 failed / 0
  ignored, plus 5 doctests passed / 2 ignored (pre-existing, unrelated).
- `cargo clippy -p ndarray_cg --all-targets --all-features -- -D warnings`: clean, exit 0.

## Generalized Version

**Broken assumption:** a matrix/quaternion-to-Euler decomposition formula with several
structurally similar cross terms (`w*y +/- z*x`, `w*x +/- y*z`, `w*z +/- x*y`) can be transcribed
by pattern-matching the "shape" of one correct/sibling term onto the others. Each sign must be
re-derived or checked independently against ground truth (a matrix-product expansion, or the
crate's own verified-correct forward conversion) -- copying the shape without re-deriving the
sign lets one transcription slip propagate silently across every term sharing the pattern.
Independently: `scalar * expr.method( .. )` in Rust silently binds the method call tighter than
the leading multiplication -- a formula that needs the multiplication applied *before* the call
(doubling a numerator, not a result) must parenthesize the multiplied expression explicitly, and
`atan2( 2*n, d ) != 2 * atan2( n, d )` in general. Both defects share a common test blind spot:
single-axis and small-angle inputs drive the erroneous term to (near) zero, so only a genuinely
multi-axis case, and a gimbal-lock case with nonzero angles on both sides, can distinguish a
correct decomposition from either of these wrong ones.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during this session's assigned review of `module/math/ndarray_cg`'s quaternion files (`quaternion.rs`, `quaternion/arithmetics.rs`, `quaternion/from.rs`), cross-checked against the crate's own verified-correct `Mat3::from_quat` and `from_euler_xyz`/`multiply` Hamilton-product convention. Root cause: `to_euler_xyz`'s pitch/roll/yaw cross-term signs were each transcription-flipped, its gimbal-lock branch's denominator used the wrong pair of squared components, and that branch separately doubled the *result* of `atan2` instead of its numerator (method-call precedence binds tighter than the leading `*`) -- three independent defects in the same function, none catchable by the pre-existing small-angle/single-axis test coverage. One pre-existing test case (`test_to_euler_xyz_from_raw_quat`'s 4th) was additionally found to carry an unrelated, latent data error (a raw quat literal that never actually matched its claimed Euler angle), masked the same way; corrected as part of this fix. Verified via 1 extended native unit test (confirmed fail pre-fix via full revert-and-rerun -- real panic, exact mismatched-vector assertion captured -- and pass post-fix) plus the full scoped suite (278 passed / 0 failed, 5 doctests) and clean clippy. Filed as BUG-272 after a fresh on-disk scan immediately before filing found that two concurrent session actors had already claimed BUG-270 and BUG-271 since an earlier scan in this same session (which had found 269 as the highest existing ID) -- this session's own original working assumption of BUG-270 was superseded before filing. |
