# BUG-119: `Quat::from(Mat3)` writes the trace-derived `w` term into the `x` slot, cyclically shifting all four quaternion components

- **Severity:** High
- **state:** Completed
- **Affects:** Any caller of `Quat<E>::from(mat3_instance)` (rotation-matrix-to-quaternion conversion) — confirmed concretely for a 90° rotation about the Z axis, and reachable internally via `Mat4::decompose()`'s final step (see BUG-250)
- **Component:** `module/math/ndarray_cg` (`src/quaternion/from.rs::{impl From<Mat3<E,Descriptor>> for Quat<E>}`)
- **repo_identity:** self
- **Filed:** 2026-08-15
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** [BUG-250](../completed/250_decompose_scale_resquared_into_rotation_matrix.md) — `decompose()`'s final step calls this same conversion; both bugs compound on any real `decompose()` call but have distinct, unrelated root causes (BUG-250 is a wrong operator on the rotation-matrix columns computed *before* this conversion runs; this bug is a component-array reorder inside the conversion itself), fixed together under the same task #52 targeted math review

## Symptom

```bash
# Rotation matrix for exactly 90 deg about Z: r11=0,r12=-1,r13=0 / r21=1,r22=0,r23=0 / r31=0,r32=0,r33=1
# Expected quaternion (axis=[0,0,1], angle=90deg): x=0, y=0, z=0.70711, w=0.70711

# Wrong (pre-fix) -- w-term value lands in the x slot, z slot stays 0
quat = [ x: 0.70711, y: 0.0, z: 0.0, w: 0.70711 ]   # rotation info in x, not z -- wrong axis entirely

# Correct (post-fix)
quat = [ x: 0.0, y: 0.0, z: 0.70711, w: 0.70711 ]   # matches hand-derived expectation
```

## Impact

**Who is affected:** Any caller of `Quat::from(mat3)` — the only rotation-matrix-to-quaternion
conversion in this crate. Reached directly by any code converting a `Mat3` rotation to a
quaternion, and indirectly by every `Mat4::decompose()` call (its very last step, see BUG-250)
and by `Mat4::decompose_generic`-style consumers built on top of it.

**What breaks:** Silent, no error, no panic — produces a normalized, plausible-looking quaternion
that represents the *wrong rotation* (rotation about the wrong combination of axes) for any input
matrix whose three off-diagonal-derived terms (`n1`,`n2`,`n3`) are not all equal. The only case
where the bug is invisible is the identity matrix (all `n1=n2=n3=0`, `n0=4`, so every component
except `w` is `0` regardless of ordering).

**Magnitude:** Every non-identity rotation converted through this path — this is the sole `Mat3 →
Quat` conversion in the crate, so there is no unaffected alternative path.

**Entity Scope:** None — a code-level math defect, not an operational-entity concern.

## How Discovered

Task #52, a targeted math/geometry code review of core crates dispatched under the standing
bug-hunt mandate. The reviewing agent flagged the array literal `[half*n0.sqrt(), half*n1.sqrt()*
.., half*n2.sqrt()*.., half*n3.sqrt()*..]` as inconsistent with the crate's `[x,y,z,w]` quaternion
storage convention. Independently re-verified before filing via a full from-scratch derivation of
the standard rotation-matrix-to-quaternion formula (Shepperd's method), cross-checked term-by-term
against this crate's own, already-correct reverse conversion `Mat3::from_quat`
(`d2/mat3x3/general.rs:224-247`) and against the storage order used by `from_angle_x/y/z`
(`quaternion/arithmetics.rs:220-249`, each of which stores its sine term first and cosine/`w` term
last) and `from_axis_angle` (`quaternion/arithmetics.rs:26-37`, storing `[x,y,z,w]` explicitly).

```bash
$ grep -n "Self::from( \[" module/math/ndarray_cg/src/quaternion/arithmetics.rs
224:      Self::from( [ s, E::zero(), E::zero(), c ] )    # from_angle_x: sin->x slot, cos->w slot (last)
236:      Self::from( [ E::zero(), s, E::zero(), c ] )    # from_angle_y: sin->y slot, cos->w slot (last)
248:      Self::from( [ E::zero(), E::zero(), s, c ] )    # from_angle_z: sin->z slot, cos->w slot (last)
```

## Minimum Reproducible Example

```bash
rm -rf /tmp/mre119 && mkdir -p /tmp/mre119/src
cat > /tmp/mre119/Cargo.toml <<'EOF'
[package]
name = "mre119"
version = "0.1.0"
edition = "2021"

[dependencies]
ndarray_cg = { path = "/home/user1/pro/lib/yrd_gamedev/cgtools/module/math/ndarray_cg" }
EOF
cat > /tmp/mre119/src/main.rs <<'EOF'
use ndarray_cg::{ Mat3, Quat, mat::DescriptorOrderColumnMajor };

fn main()
{
  // Exactly 90 deg rotation about Z, written directly (no from_axis_angle dependency):
  // r11=cos90=0, r12=-sin90=-1, r21=sin90=1, r22=cos90=0, r33=1, rest 0.
  let m = Mat3::< f64, DescriptorOrderColumnMajor >::from_column_major
  (
    [
      0.0, 1.0, 0.0,
      -1.0, 0.0, 0.0,
      0.0, 0.0, 1.0,
    ]
  );

  let q : Quat< f64 > = m.into();
  println!( "quat = [ x: {:.5}, y: {:.5}, z: {:.5}, w: {:.5} ]", q.x(), q.y(), q.z(), q.w() );
}
EOF
cd /tmp/mre119 && cargo run 2>&1 | tail -1
```

**Expected** (axis=[0,0,1], angle=90deg -> half-angle sin/cos both 0.70711, on the z and w slots):
```
quat = [ x: 0.00000, y: 0.00000, z: 0.70711, w: 0.70711 ]
```

**Actual** (pre-fix):
```
quat = [ x: 0.70711, y: 0.00000, z: 0.00000, w: 0.70711 ]
```

**Verify Command** (≤3 lines, standalone):
```bash
cd /tmp/mre119 && cargo run 2>&1 | tail -1
# x==0.00000 and z==0.70711 = fixed; x==0.70711 and z==0.00000 = bug present
```
**What:** Violates the crate's own `[x,y,z,w]` quaternion storage convention (confirmed by
`from_angle_x/y/z`/`from_axis_angle` above) — the conversion returns a normalized quaternion
representing a rotation about the wrong axis combination.

**Known MRE limitation (check 205):** `ndarray_cg` is this workspace's own crate; the MRE
path-depends on it locally rather than a registry version, mirroring BUG-116/BUG-250's own
documented exception. The 90°-about-Z matrix is written out from exact closed-form trig values
(`0`, `±1`), so there is no floating-point or registry-version ambiguity this local dependency
could be hiding.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The final array literal in `Quat::from(Mat3)` (`from.rs`, pre-fix) assembles `[n0, n1, n2, n3]`-derived terms directly into storage-slot order, but `n0` is the `w`-proportional (trace) term while `n1`/`n2`/`n3` are the `x`/`y`/`z`-proportional terms respectively — a direct positional write therefore shifts every component one slot: `w`'s value lands in `x`, `x`'s in `y`, `y`'s in `z`, `z`'s in `w`. | ✅ Root Cause | Standard Shepperd's-method algebra confirms `n0 ∝ w²`, `n1 ∝ x²`, `n2 ∝ y²`, `n3 ∝ z²`; the crate's own `[x,y,z,w]` convention is independently confirmed by `from_angle_x/y/z` and `from_axis_angle` (E2). MRE's 90°-about-Z case shows exactly this shift: `w`'s value (0.70711) appears in the `x` slot instead of `z`. | E1, E2, E3 |
| H2 | The sign terms (`(r32-r23).signum()` etc.) are attached to the wrong `n` term, independent of overall array ordering. | ❌ Falsified | Each sign term is already paired with the `n` term that shares its axis under the standard derivation (`n1`↔`(r32-r23)` for `x`, `n2`↔`(r13-r31)` for `y`, `n3`↔`(r21-r12)` for `z`) — re-deriving each pairing from the rotation-matrix-to-quaternion identity confirms all three pairings were already correct; only the *slot* each pairing was written into was wrong. | E1 |
| H3 | `Mat3::from_quat` (the reverse, quaternion-to-matrix conversion used as this bug's cross-check oracle) has a matching or compensating defect, making the "expected" value used for comparison unreliable. | ❌ Falsified | Direct read of `d2/mat3x3/general.rs:224-247` confirms the standard, textbook quaternion-to-rotation-matrix formula (`x2=x+x`, `1-(yy+zz)` diagonal, etc.) with no `[x,y,z,w]` ordering irregularities — safe to use as an independent oracle, which is exactly how this bug's expected values were cross-checked. | E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/math/ndarray_cg/src/quaternion/from.rs:69-96` (pre-fix array literal at what are now lines 90-96, post-fix) | `n0..n3` computed from the matrix trace/diagonal (standard Shepperd's-method terms); pre-fix, the array was built as `[half*n0.sqrt(), half*n1.sqrt()*sign, half*n2.sqrt()*sign, half*n3.sqrt()*sign]` — `n0` (the `w`-term) written first, into the `x` slot. | H1 ✅ |
| E2 | `module/math/ndarray_cg/src/quaternion/arithmetics.rs:220-249` (`from_angle_x/y/z`), `:26-37` (`from_axis_angle`) | All four constructors store the sine/axis term(s) first and the cosine/scalar term (`w`) last — `[x,y,z,w]` — confirming the crate-wide storage convention independently of this bug's own code. | H1 ✅ |
| E3 | `/tmp/mre119` run, pre-fix vs. post-fix, 90°-about-Z matrix | Pre-fix: `x: 0.70711` (should be `0`), `z: 0.00000` (should be `0.70711`) — `w`'s computed value (`n0`-derived) appears in the `x` slot; the true `z`-component's value (`n3`-derived) appears in the `w` slot instead. Post-fix: matches hand-derived expectation exactly. | H1 ✅ |
| E4 | `module/math/ndarray_cg/src/d2/mat3x3/general.rs:224-247` (`Mat3::from_quat`) | Standard textbook quaternion→matrix formula, `[x,y,z,w]`-consistent throughout (`x2=quat.x()+quat.x()`, diagonal `1-(yy+zz)`, etc.) — confirmed safe as this bug's independent cross-check oracle. | H3 ❌ |

## Root Cause

```
n0 = 1 + r11 + r22 + r33     (∝ 4w², trace-derived "w term")     (from.rs:69)
n1 = 1 + r11 - r22 - r33     (∝ 4x², "x term")                    (from.rs:70)
n2 = 1 - r11 + r22 - r33     (∝ 4y², "y term")                    (from.rs:71)
n3 = 1 - r11 - r22 + r33     (∝ 4z², "z term")                    (from.rs:72)

q = [ n0-derived, n1-derived, n2-derived, n3-derived ]   (pre-fix, from.rs:90-96)
  = [   w value ,   x value ,   y value ,   z value    ]   (by the algebra above)
  ↓ stored positionally into a [x, y, z, w]-convention array
  = [ x_slot=w_value, y_slot=x_value, z_slot=y_value, w_slot=z_value ]   ✗ cyclic shift
```

The derivation names its four intermediate terms `n0..n3` in the order that's algebraically
convenient to compute (trace/`w` term first), but this crate's `Quat` stores components in
`[x,y,z,w]` order — confirmed by every other quaternion constructor in the crate (`from_angle_x/
y/z`, `from_axis_angle`; see `## Evidence Table` E2). Building the final array directly from
`n0..n3` in computation order, without remapping each term to the storage slot its *own* algebraic
identity corresponds to, silently writes `w`'s value into the `x` slot, `x`'s into `y`, `y`'s into
`z`, and `z`'s into `w` — a cyclic shift, not a random scramble, which is why the identity-matrix
case (all off-diagonal terms zero) never exposed it.

## Why Not Caught

No test exercises `Quat::from(Mat3)` (equivalently `Mat3::into()`) at all —
`tests/inc/quat_test/general.rs` has coverage for `Quat::from(&[E])`'s slice-length validation
(TASK-014) but nothing constructing a `Mat3` and converting it to a `Quat`. No round-trip test
exists comparing `Mat3::from_quat(q)` (already correct) against `Quat::from(Mat3::from_quat(q))`
recovering `q`, which would have caught any non-identity-preserving conversion in either
direction.

## Fix Location

`module/math/ndarray_cg/src/quaternion/from.rs`, inside `impl From<Mat3<E,Descriptor>> for
Quat<E>`. Reordered the final array literal so each `n`-derived term lands in the storage slot its
own algebraic identity corresponds to (`n1`→`x`, `n2`→`y`, `n3`→`z`, `n0`→`w`) instead of
positional/computation order.

```rust
// before
let q =
[
  half * n0.sqrt(),
  half * n1.sqrt() * ( r32 - r23 ).signum(),
  half * n2.sqrt() * ( r13 - r31 ).signum(),
  half * n3.sqrt() * ( r21 - r12 ).signum()
];

// after
let q =
[
  half * n1.sqrt() * ( r32 - r23 ).signum(),
  half * n2.sqrt() * ( r13 - r31 ).signum(),
  half * n3.sqrt() * ( r21 - r12 ).signum(),
  half * n0.sqrt()
];
```

## Prevention

Added `test_from_mat3_recovers_known_axis_angle_rotation` (and a
`test_from_mat3_round_trips_through_from_quat_generic` round-trip counterpart, `_row_major`/
`_column_major` instantiations) to `tests/inc/quat_test/general.rs`: the first hand-derives the
expected quaternion for a 90°-about-Z rotation matrix (the same closed-form fixture as this
bug's MRE) and asserts an exact component-by-component match — this would fail immediately under
the pre-fix cyclic shift; the second builds a matrix via the already-correct `Mat3::from_quat`
from a generic non-axis-aligned quaternion and confirms converting it back recovers the original
quaternion (up to sign, since `q` and `-q` represent the same rotation).

**Pitfall:** when a derivation names its intermediate terms in the order they're *computed*
(trace term first, for algebraic convenience), that order can silently diverge from the order the
target type actually *stores* its components in — always map each intermediate back to its named
component (`x`, `y`, `z`, or `w`) before assembling the final array, rather than assuming
computation order matches storage order.

## Generalized Version

**Broken assumption:** "A tuple of intermediate values computed in a convenient derivation order
can be assembled directly, in that same order, into a struct/array whose fields have a different,
independently-defined canonical order" — false whenever the two orders diverge and nothing enforces
they stay aligned.

**Confirmed general rule:** any conversion function that (a) derives N named intermediate
quantities via a formula whose natural computation order is NOT explicitly tied to the target
type's field/slot order, and (b) assembles them into that target type via a positional literal
(not named-field construction), is at risk of exactly this defect. The detection invariant: for
any such conversion, either construct the target via named fields/setters (compile-time-checked
by name, not position) or add a round-trip test against an independently-verified reverse
conversion (as this bug's own oracle, `Mat3::from_quat`, made possible here).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-15 | filed | Discovered via task #52's targeted math/geometry code review; root cause independently confirmed via a full from-scratch re-derivation of the rotation-matrix-to-quaternion formula, cross-checked term-by-term against the crate's own correct `Mat3::from_quat` and against `from_angle_x/y/z`/`from_axis_angle`'s storage convention, before filing. |
| 2026-08-15 | fixed | Reordered the final `q` array literal in `Quat::from(Mat3)` from `[n0,n1,n2,n3]`-positional to `[n1,n2,n3,n0]` (matching the `[x,y,z,w]` storage convention); 3-field `Fix(BUG-119)`/`Root cause`/`Pitfall` comment added at the fix site. |
| 2026-08-15 | verified | Added `test_from_mat3_recovers_known_axis_angle_rotation` and `test_from_mat3_round_trips_through_from_quat_generic` (row-major + column-major instantiations) to `tests/inc/quat_test/general.rs`. Narrow suite and full workspace verification recorded in BUG-121's own closing History entry (all four math bugs verified together as one gate — see `task/bug/readme.md`). |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-15, this verifier 2026-08-16). Independently re-read `Quat::from(Mat3)` (confirmed the final array genuinely reordered to `[n1,n2,n3,n0]`-derived slots, matching `[x,y,z,w]` storage; 3-field comment intact) and `test_from_mat3_recovers_known_axis_angle_rotation` (non-tautological: hand-derives the expected quaternion for a known 90°-about-Z rotation matrix and asserts an exact component match). Fresh `cargo nextest run -p ndarray_cg --all-features` via `longrun`: 272/272 passed. `cargo clippy -p ndarray_cg --all-features --all-targets -- -D warnings`: clean. Corrected the stale `**Related Bugs:**` cross-reference (`../verified/118_...` → `../completed/118_...`). MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-250/119/120/121 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections + `Refs: src/`/`Refs: tests/` present — confirmed by direct re-read of the full file. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass hand-derived the 90°-about-Z expected values; adversarial pass independently re-verified the closed-form trig substitution (`cos90=0, sin90=1`) against the general `n0..n3` formulas term-by-term rather than trusting the first pass's arithmetic. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Adversarial pass confirmed BUG-250's file carries the reciprocal `**Related Bugs:**` line forward to this file. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass re-confirmed the H2 (sign-term pairing) alternative was genuinely checked and falsified, not assumed — each `signum()` argument pairing was re-derived independently against the standard identity. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Adversarial pass checked whether `Mat3::from_quat` (the reverse conversion, used as oracle) needed any change — confirmed no, it was already correct and is the reason it could serve as the cross-check oracle in the first place. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `ndarray_cg`'s own `src/`/`tests/` and this bug-tracking file touched — no cross-crate scope creep. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix is local to the final array-literal assembly inside `Quat::from(Mat3)`; no shared helper needed changing. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix does not add any new responsibility to the conversion — it corrects the storage-slot mapping within the function's existing, documented contract. | — |

**Reproduced:** YES — `/tmp/mre119` pre-fix: `quat = [ x: 0.70711, y: 0.00000, z: 0.00000, w: 0.70711 ]` vs. expected `z`-slot value, 2026-08-15

## Refs: src/

| File | Change |
|------|--------|
| `module/math/ndarray_cg/src/quaternion/from.rs` | `impl From<Mat3<E,Descriptor>> for Quat<E>`: reordered the final `q` array literal from `[n0,n1,n2,n3]`-positional to `[n1,n2,n3,n0]`, matching the `[x,y,z,w]` storage convention. `Fix(BUG-119)`/`Root cause`/`Pitfall` 3-field comment added at the fix site. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/math/ndarray_cg/tests/inc/quat_test/general.rs` | Added `test_from_mat3_recovers_known_axis_angle_rotation` (`bug_reproducer(BUG-119)`, closed-form 90°-about-Z fixture) and `test_from_mat3_round_trips_through_from_quat_generic` (`_row_major`/`_column_major` instantiations, round-trip via `Mat3::from_quat`), each with a 5-section doc comment. |
