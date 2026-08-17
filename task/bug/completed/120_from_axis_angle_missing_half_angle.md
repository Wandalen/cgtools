# BUG-120: `Quat::from_axis_angle` uses the full angle instead of the half angle, producing a rotation twice the requested amount

- **Severity:** High
- **state:** Completed
- **Affects:** Any caller of `Quat<E>::from_axis_angle(axis, angle)` for any `angle != 0` — confirmed concretely against the sibling constructors `from_angle_x`/`from_angle_y`/`from_angle_z`, which correctly halve. Confirmed REAL, ACTIVE downstream impact (not merely theoretical): `module/helper/renderer/src/webgl/animation/scaling.rs`'s `Scaler` component calls `QuatF64::from_axis_angle(axis, angle_scaled)` (line 180) to reconstruct a partial-rotation delta from an angle already extracted (as a FULL angle, `2.0 * w.acos()`) by its own correctly-implemented sibling `quat_to_axis_angle` — pre-fix, this made every non-1.0 rotation-scale factor apply DOUBLE the intended scaled angle (e.g. a caller requesting `scale=0.5`, intending half the original rotation, silently got the FULL original rotation instead). `module/helper/renderer`'s own `scaler_tests.rs`/`blender_tests.rs` exercise this call path but assert only on unrelated fields (scale/weight getters, not the resulting rotation value), so they neither caught this nor need updating for the fix.
- **Component:** `module/math/ndarray_cg` (`src/quaternion/arithmetics.rs::from_axis_angle`); confirmed reachable production consumer: `module/helper/renderer` (`src/webgl/animation/scaling.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-15
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — independent root cause from BUG-118/BUG-119, filed under the same task #52 targeted math review

## Symptom

```bash
# from_axis_angle( [0,0,1], 90 deg in radians = pi/2 )
# Expected: a quaternion representing a 90 deg rotation about Z (half-angle = 45 deg)
# Actual: a quaternion representing a 180 deg rotation about Z (uses the full 90 deg as if
# it were already the half-angle, i.e. sin/cos of pi/2 instead of pi/4)

# Wrong (pre-fix)
quat = [ x: 0.00000, y: 0.00000, z: 1.00000, w: 0.00000 ]   # this is a 180 deg rotation

# Correct (post-fix)
quat = [ x: 0.00000, y: 0.00000, z: 0.70711, w: 0.70711 ]   # this is the requested 90 deg rotation
```

## Impact

**Who is affected:** Any caller of `Quat::from_axis_angle(axis, angle)` — the crate's only
general (non-axis-aligned) axis-angle quaternion constructor.

**What breaks:** Silent, no error. The returned quaternion is always normalized and represents a
*valid* rotation — just not the one requested. For a requested angle `θ`, the actual rotation
applied is `2θ` instead of `θ` (e.g. requesting 90° yields 180°; requesting 180° yields a full
360°, i.e. the identity — an especially confusing case where "rotate by a half turn" silently
becomes "don't rotate at all"). The sibling constructors `from_angle_x`, `from_angle_y`,
`from_angle_z` do NOT have this defect — they correctly halve — so `from_axis_angle([1,0,0],
θ)` and `from_angle_x(θ)` (which the crate's own doc comments describe as equivalent, one general
and one axis-specialized) silently disagreed for every non-zero `θ`.

**Magnitude:** Every non-zero-angle call to `from_axis_angle` — no unaffected subset.

**Entity Scope:** None — a code-level math defect, not an operational-entity concern.

## How Discovered

Task #52, a targeted math/geometry code review of core crates dispatched under the standing
bug-hunt mandate. The reviewing agent flagged `from_axis_angle`'s direct `angle.sin_cos()` call as
inconsistent with the standard axis-angle-to-quaternion formula (`q = (axis·sin(θ/2), cos(θ/2))`)
and with this crate's own sibling constructors. Independently confirmed before filing by reading
`from_angle_x`/`from_angle_y`/`from_angle_z` (`arithmetics.rs:220-249`), all three of which compute
`let two = E::one() + E::one(); let (s,c) = (x / two).sin_cos();` — `from_axis_angle` was the only
one of the four angle-based constructors missing this halving step.

```bash
$ grep -n "sin_cos()" module/math/ndarray_cg/src/quaternion/arithmetics.rs
30:        let ( s, c ) = angle.sin_cos();                 # from_axis_angle -- pre-fix, no halving
223:      let ( s, c ) = ( x / two ).sin_cos();             # from_angle_x -- halves
235:      let ( s, c ) = ( y / two ).sin_cos();             # from_angle_y -- halves
247:      let ( s, c ) = ( z / two ).sin_cos();             # from_angle_z -- halves
```

## Minimum Reproducible Example

```bash
rm -rf /tmp/mre120 && mkdir -p /tmp/mre120/src
cat > /tmp/mre120/Cargo.toml <<'EOF'
[package]
name = "mre120"
version = "0.1.0"
edition = "2021"

[dependencies]
ndarray_cg = { path = "/home/user1/pro/lib/yrd_gamedev/cgtools/module/math/ndarray_cg" }
EOF
cat > /tmp/mre120/src/main.rs <<'EOF'
use ndarray_cg::QuatF64;
use std::f64::consts::FRAC_PI_2;

fn main()
{
  let from_axis = QuatF64::from_axis_angle( [ 0.0, 0.0, 1.0 ], FRAC_PI_2 );
  let from_z = QuatF64::from_angle_z( FRAC_PI_2 );

  println!( "from_axis_angle( [0,0,1], 90deg ) = [ {:.5}, {:.5}, {:.5}, {:.5} ]",
    from_axis.x(), from_axis.y(), from_axis.z(), from_axis.w() );
  println!( "from_angle_z( 90deg )             = [ {:.5}, {:.5}, {:.5}, {:.5} ]",
    from_z.x(), from_z.y(), from_z.z(), from_z.w() );
}
EOF
cd /tmp/mre120 && cargo run 2>&1 | tail -2
```

**Expected** (the general and axis-specialized constructors agree for the same axis/angle):
```
from_axis_angle( [0,0,1], 90deg ) = [ 0.00000, 0.00000, 0.70711, 0.70711 ]
from_angle_z( 90deg )             = [ 0.00000, 0.00000, 0.70711, 0.70711 ]
```

**Actual** (pre-fix — `from_axis_angle` disagrees, representing double the requested rotation):
```
from_axis_angle( [0,0,1], 90deg ) = [ 0.00000, 0.00000, 1.00000, 0.00000 ]
from_angle_z( 90deg )             = [ 0.00000, 0.00000, 0.70711, 0.70711 ]
```

**Verify Command** (≤3 lines, standalone):
```bash
cd /tmp/mre120 && cargo run 2>&1 | tail -2
# both lines equal = fixed; first line differs from second = bug present
```
**What:** Violates the expected equivalence between the general `from_axis_angle` constructor and
the axis-specialized `from_angle_x/y/z` constructors for the same rotation — `from_axis_angle`
applies twice the requested angle.

**Known MRE limitation (check 205):** `ndarray_cg` is this workspace's own crate; the MRE
path-depends on it locally rather than a registry version, mirroring BUG-116/118/119's own
documented exception. The comparison is against another function in the same crate
(`from_angle_z`), already independently confirmed correct, so there is no external ambiguity this
local dependency could be hiding.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `from_axis_angle` (pre-fix) computes `angle.sin_cos()` directly instead of halving the angle first, unlike the standard axis-angle-to-quaternion formula and unlike its own sibling constructors. | ✅ Root Cause | Direct read of `arithmetics.rs:26-37` (pre-fix) confirms no halving; the standard formula `q=(axis·sin(θ/2),cos(θ/2))` requires it. MRE shows `from_axis_angle` and `from_angle_z` disagree for the identical axis/angle, with `from_axis_angle`'s result matching what `from_angle_z(2θ)` would produce. | E1, E2, E3 |
| H2 | `from_angle_x`/`from_angle_y`/`from_angle_z` are themselves wrong (over-halving, or some other defect), and `from_axis_angle` is the correct one — the MRE's "expected" values are backwards. | ❌ Falsified | These three constructors are exercised by pre-existing, already-passing tests (`tests/inc/quat_test/arithmetic.rs::test_from_angle_x/y/z`) that assert their output against independently-known correct quaternion values — confirmed correct prior to and independent of this bug's own investigation. | E4 |
| H3 | The axis vector itself is the problem — e.g. `from_axis_angle` expects a pre-scaled or pre-halved axis rather than a unit axis, so the API contract (not the angle handling) is what's mismatched. | ❌ Falsified | The doc comment (`arithmetics.rs:17-21`, unchanged) specifies `axis` as "The normalized 3D vector representing the axis of rotation" and `angle` as "The angle of rotation in radians" — a plain angle, no half-angle caveat in the documented contract; the function's own body applies `s`/`c` from `sin_cos()` directly to the *axis* components (`x = axis[0]*s`, etc.), not to any pre-scaling of the axis — isolating the defect to the missing angle-halving step alone. | E1 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/math/ndarray_cg/src/quaternion/arithmetics.rs:17-37` (doc comment + body, pre-fix) | Documented contract takes a plain angle in radians with no half-angle caveat; body (pre-fix) computed `angle.sin_cos()` with no halving step anywhere before or after. | H1 ✅, H3 ❌ |
| E2 | `module/math/ndarray_cg/src/quaternion/arithmetics.rs:220-249` (`from_angle_x/y/z`) | All three sibling constructors compute `let two = E::one()+E::one(); (x/two).sin_cos()` — confirms the crate's own established convention requires halving, which `from_axis_angle` alone omitted. | H1 ✅ |
| E3 | `/tmp/mre120` run, pre-fix vs. post-fix | Pre-fix: `from_axis_angle([0,0,1],90°) = [0,0,1,0]` (a 180° rotation) vs. `from_angle_z(90°) = [0,0,0.70711,0.70711]` (correct 90°) — disagree. Post-fix: identical. | H1 ✅ |
| E4 | `module/math/ndarray_cg/tests/inc/quat_test/arithmetic.rs::test_from_angle_x/y/z` (pre-existing, already passing) | Independently asserts `from_angle_x/y/z`'s output against known-correct quaternion component values — confirms these three are not themselves the defective party. | H2 ❌ |

## Root Cause

```
Standard formula:  q = ( axis * sin(angle/2), cos(angle/2) )

from_axis_angle (pre-fix, arithmetics.rs:30):
  let ( s, c ) = angle.sin_cos()                    # uses angle directly, not angle/2  ✗

from_angle_z (arithmetics.rs:246-247, always correct):
  let two = E::one() + E::one();
  let ( s, c ) = ( z / two ).sin_cos()               # correctly halves first  ✓
```

`from_axis_angle` implements the general axis-angle-to-quaternion formula, which requires the
HALF angle (`sin(θ/2)`, `cos(θ/2)`) — a standard property of how quaternions double-cover the
rotation group (a quaternion `q` and its negation `-q` represent the same rotation, and composing
two half-angle rotations via quaternion multiplication yields the full-angle rotation). The
function computed `sin`/`cos` of the full angle directly, with no halving step anywhere in the
body — unlike every one of its three sibling constructors (`from_angle_x/y/z`), each of which
explicitly halves via `let two = E::one() + E::one(); (angle / two).sin_cos()`. The omission
causes every call to `from_axis_angle` to produce a quaternion representing double the requested
rotation angle.

## Why Not Caught

No test exercises `from_axis_angle` at all — `tests/inc/quat_test/arithmetic.rs` has
`test_from_angle_x`, `test_from_angle_y`, `test_from_angle_z` (each asserting known-correct
component values) but no `test_from_axis_angle` counterpart, and no cross-check test asserting
that `from_axis_angle(unit_axis, θ)` agrees with the corresponding `from_angle_{x,y,z}(θ)` for an
axis-aligned case — which would have caught this immediately, since the two code paths would
disagree for any non-zero angle.

## Fix Location

`module/math/ndarray_cg/src/quaternion/arithmetics.rs:26-37`, inside `from_axis_angle`. Added the
same halving step already used by `from_angle_x/y/z`, applied before `sin_cos()`.

```rust
// before
pub fn from_axis_angle< T >( axis : T, angle : E ) -> Self
where
  T : VectorIter< E, 3 >
{
    let ( s, c ) = angle.sin_cos();
    ...

// after
pub fn from_axis_angle< T >( axis : T, angle : E ) -> Self
where
  T : VectorIter< E, 3 >
{
    let two = E::one() + E::one();
    let ( s, c ) = ( angle / two ).sin_cos();
    ...
```

## Prevention

Added `test_from_axis_angle_matches_axis_aligned_from_angle_z` (and `_x`/`_y` counterparts) to
`tests/inc/quat_test/arithmetic.rs`: for each of the three axis-aligned unit axes, asserts
`from_axis_angle(axis, θ)` equals the corresponding `from_angle_{x,y,z}(θ)` via
`assert_abs_diff_eq!`, for a non-trivial `θ` (90°) where a missing-halving defect and a correct
implementation produce visibly different results (the previously-noted 180°-becomes-identity edge
case at `θ=360°` would NOT have reliably caught this, since both wrap to equivalent rotations —
90° was chosen specifically to avoid that blind spot).

**Pitfall:** sibling constructors that should share an invariant (here: "always half-angle the
input" for any quaternion built from an angle) can drift independently when each is implemented
as its own free-standing function instead of being built on one shared half-angle helper —
cross-check new constructors against already-correct siblings covering the same class of input,
not just against the formula in isolation.

## Generalized Version

**Broken assumption:** "A function implementing a general-case formula and a sibling function
implementing a specialized case of the same formula will independently apply the same
transformation to their shared input (here: angle) correctly, just because they're documented as
computing 'the same kind of thing'" — false whenever the two are implemented as fully independent
function bodies with no shared helper enforcing the invariant.

**Confirmed general rule:** whenever a codebase has both a general constructor and one or more
axis/case-specialized constructors for the same underlying mathematical object, and the
specialized ones share a transformation step (here: angle-halving) that the general one's formula
also requires, the two are at risk of silently diverging unless (a) the general one is implemented
in terms of the specialized ones (or vice versa), or (b) a cross-agreement test pins the two
together. The detection invariant: for any such general/specialized pair, assert their outputs
agree for an input the specialized case can also represent (e.g. an axis-aligned axis for a
general axis-angle formula).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-15 | filed | Discovered via task #52's targeted math/geometry code review; confirmed by direct comparison against the already-correct `from_angle_x/y/z` siblings before filing. |
| 2026-08-15 | fixed | Added the missing `let two = E::one() + E::one(); (angle / two).sin_cos()` halving step to `from_axis_angle`, matching the sibling constructors' pattern; 3-field `Fix(BUG-120)`/`Root cause`/`Pitfall` comment added at the fix site. |
| 2026-08-15 | verified | Added `test_from_axis_angle_matches_axis_aligned_from_angle_x/y/z` to `tests/inc/quat_test/arithmetic.rs`. Narrow suite and full workspace verification recorded in BUG-121's own closing History entry (all four math bugs verified together as one gate — see `task/bug/readme.md`). |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-15, this verifier 2026-08-16). Independently re-read `from_axis_angle` (confirmed the halving step `(angle / two).sin_cos()` genuinely present, matching the already-correct `from_angle_x/y/z` siblings; 3-field comment intact) and `test_from_axis_angle_matches_axis_aligned_from_angle_z` (non-tautological: asserts `from_axis_angle` output equals the independently-correct `from_angle_z` sibling's output for the same angle). Independently confirmed the file's own disclosed test-coverage limitation for the `renderer` consumer: re-read `scaler_tests.rs::test_grouped_nodes_independence` directly — it asserts only `scale_get(...).y()` (the scale factor), never the resulting rotation value from the affected `from_axis_angle` call path, so it genuinely doesn't (and can't) catch this class of regression; disclosure is accurate. Fresh `cargo nextest run -p ndarray_cg --all-features` via `longrun`: 272/272 passed; `cargo nextest run -p renderer --all-features` via `longrun`: 79/79 passed (structural regression check only, per the above — not a behavioral proof for the scaling path). `cargo clippy -p ndarray_cg -p renderer --all-features --all-targets -- -D warnings`: clean. MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-118/119/120/121 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections + `Refs: src/`/`Refs: tests/` present — confirmed by direct re-read of the full file. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass hand-derived the 180°-vs-90° expected values; adversarial pass re-checked the `θ=360°` edge case is deliberately NOT used as the primary MRE fixture, since it would falsely appear to pass under both correct and buggy code (both wrap to the identity). | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Adversarial pass confirmed this file correctly declares no `**Related Bugs:**` (root cause is genuinely independent of BUG-118/119/121 — different function, no shared code path). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass re-confirmed H3 (axis-scaling contract mismatch) was genuinely checked against the doc comment's own wording, not assumed away. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Adversarial pass checked whether `from_euler_xyz` (which composes `from_angle_x/y/z`) is affected — it isn't, since it doesn't call `from_axis_angle` at all (confirmed via direct read of `arithmetics.rs:259+`). | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `ndarray_cg`'s own `src/`/`tests/` and this bug-tracking file touched — no cross-crate scope creep. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix is local to `from_axis_angle`'s own body; no shared helper needed changing (though `## Prevention`/`## Generalized Version` note a shared-helper refactor as a possible future improvement, not required for correctness). | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix does not add any new responsibility to `from_axis_angle` — it corrects the angle-handling within the function's existing, documented contract. | — |

**Reproduced:** YES — `/tmp/mre120` pre-fix: `from_axis_angle([0,0,1],90°) = [0,0,1,0]` vs. `from_angle_z(90°) = [0,0,0.70711,0.70711]`, 2026-08-15

## Refs: src/

| File | Change |
|------|--------|
| `module/math/ndarray_cg/src/quaternion/arithmetics.rs` | `from_axis_angle`: added `let two = E::one() + E::one(); let ( s, c ) = ( angle / two ).sin_cos();`, replacing the un-halved `angle.sin_cos()`. `Fix(BUG-120)`/`Root cause`/`Pitfall` 3-field comment added at the fix site. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/math/ndarray_cg/tests/inc/quat_test/arithmetic.rs` | Added `test_from_axis_angle_matches_axis_aligned_from_angle_x/y/z` (`bug_reproducer(BUG-120)`, cross-agreement fixtures at `θ=90°`), each with a 5-section doc comment. |
