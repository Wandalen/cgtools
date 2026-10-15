# BUG-331: `skeletal_animation`'s camera near/far derivation is V-shaped in `exponent`, collapsing `far <= near` for an ordinary scene scale (including its own bundled asset) and panicking `Camera::new`

- **Severity:** High (panics the demo at startup for its own bundled asset)
- **state:** Completed
- **Affects:** `examples/minwebgl/skeletal_animation/src/main.rs`
- **Component:** examples/minwebgl/skeletal_animation
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`near_far_from_exponent`'s unguarded `far` formula
(`(near * 100.0f32.powi(exponent.abs()) / 100.0).max(near * 10.0)`) collapses to `far <= near` for
`exponent in [-1, 0, 1]` -- e.g. `far == near` at `exponent == -1` and `1`, and `far < near` at
`exponent == 0`. That band covers scene bounding-box diagonals in `[0.5, 4.0)`, an ordinary size
range for a normalized glTF asset, including this demo's own bundled `bug_bunny.glb`.
`Camera::new` rejects `far <= near` outright, panicking the demo's own
`.expect("camera parameters are valid")` at startup.

## Impact

**Who is affected:** every user of this demo -- it panics before rendering a single frame for its
own bundled scene.

**What breaks:** the whole demo fails to start; `Camera::new`'s `far > near` precondition is
violated by the near/far values derived from the bundled asset's own bounding-box diagonal.

**Entity Scope:** `None` -- confined to this crate's own camera-setup derivation.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184), sharing the same defect class independently found in sibling crates
`animation_amplitude_change` (BUG-320) and `pbr_lighting` (BUG-332) -- three genuinely different
formulas, each collapsing `far <= near` for a different input band, filed as three separate
cross-referenced bugs (matching the BUG-307/308 precedent) rather than combined under one ID.
Independently verified by the orchestrating session by evaluating the formula across
`exponent in -10..=10` and confirming the `[-1, 0, 1]` collapse band.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p skeletal_animation test_far_always_exceeds_near_across_exponent_range
```
**Expected** (fixed): `far > near` holds for every `exponent in -10..=10`. **Actual** (pre-fix):
`far <= near` at `exponent in [-1, 0, 1]`, `Camera::new` panics for the bundled scene's actual
scale.

## Root Cause

`100.0f32.powi(exponent.abs()) / 100.0` is V-shaped in `exponent` -- `.abs()` makes it shrink
toward `exponent == 0` from both sides, reaching its minimum of `1.0/100.0` exactly there --
instead of monotonically increasing with scene size, so the multiplier applied to `near` can
collapse well below the margin `far > near` requires.

## Why Not Caught

No test exercised `near_far_from_exponent` across a representative `exponent` range or against the
bundled asset's own bounding-box diagonal -- the panic only surfaces at runtime, on the very first
frame.

## Fix Applied (2026-08-18)

Added a floor (`.max(near * 10.0)`) to the `far` formula, guaranteeing `far` is always at least
10x `near` regardless of where the V-shaped multiplier collapses to. Added an inline
`#[cfg(test)]` module in `main.rs` (native unit tests, no `tests/` directory needed for this
pure-function fix): `test_far_always_exceeds_near_across_exponent_range` sweeps
`exponent in -10..=10` asserting `far > near` and both finite/positive throughout;
`test_pre_fix_formula_was_broken_for_exponents_negative_one_zero_and_one` pins the exact pre-fix
collapse values, confirming the bug was real and not a hypothetical edge case.

## Verification

- **Pre-fix (RED):** reverted the `far` formula to its unguarded (no `.max` floor) form; new tests
  failed, reproducing `far <= near` across `exponent in [-1, 0, 1]`.
- **Post-fix (GREEN):** `cargo test -p animation_blending -p skeletal_animation -p pbr_lighting -p character_control --no-fail-fast`
  (run together with sibling BUG-320/332 and this crate's own BUG-339 lil_gui fix) -- 12 tests
  passed, 0 failed; `cargo check --target wasm32-unknown-unknown` and
  `cargo clippy --all-targets --all-features -- -D warnings` (native + wasm32) all clean.

## Generalized Version

A derived camera-projection parameter (near/far, FOV) that depends on a scene's own runtime scale
must be validated across its full expected input range, not just spot-checked at one scale --
a formula that is V-shaped, non-monotonic, or otherwise collapses a required invariant (`far >
near`) at a specific input band needs an explicit floor/ceiling against its sibling value, rather
than trusting the formula's general shape to hold everywhere.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-XXX-A` placeholder marker (disambiguated from sibling findings in the same fork's other crates) since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-331 after a fresh on-disk collision scan. Related: BUG-320 (`animation_amplitude_change`), BUG-332 (`pbr_lighting`) -- same defect class, independently derived, different formulas. |
