# BUG-332: `pbr_lighting`'s camera near/far derivation collapses to `far == near` exactly at `exponent == 0`, panicking `Camera::new` for an ordinary scene scale

- **Severity:** High (panics the demo at startup for an ordinary scene scale)
- **state:** Completed
- **Affects:** `examples/minwebgl/pbr_lighting/src/main.rs`
- **Component:** examples/minwebgl/pbr_lighting
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`near_far_from_exponent`'s unguarded `far` formula (`near * 100.0f32.powi(exponent.abs())`)
collapses to `far == near` at `exponent == 0` (`100.0f32.powi(0) == 1.0`), which `Camera::new`
rejects (requires `far > near`), panicking the whole demo for any scene whose bounding-box
diagonal falls in `[1.0, 2.0)` -- an ordinary size for a normalized glTF asset.

## Impact

**Who is affected:** every user of this demo whose loaded scene's bounding-box diagonal falls in
the `[1.0, 2.0)` band.

**What breaks:** the whole demo fails to start; `Camera::new`'s `far > near` precondition is
violated exactly at `exponent == 0`, a real, reachable point in the formula's own domain.

**Entity Scope:** `None` -- confined to this crate's own camera-setup derivation.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184), sharing the same defect class independently found in sibling crates
`animation_amplitude_change` (BUG-320) and `skeletal_animation` (BUG-331) -- three genuinely
different formulas, each collapsing `far <= near` for a different input band, filed as three
separate cross-referenced bugs (matching the BUG-307/308 precedent) rather than combined under one
ID. Independently verified by the orchestrating session by evaluating the formula at
`exponent == 0` directly.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p pbr_lighting test_pre_fix_formula_was_degenerate_at_exponent_zero
```
**Expected** (fixed): `far > near` holds for every `exponent in -10..=10`, including `exponent ==
0`. **Actual** (pre-fix): `far == near == 10.0` exactly at `exponent == 0`, `Camera::new` panics.

## Root Cause

`100.0f32.powi(exponent.abs())` is V-shaped in `exponent` -- `.abs()` makes it shrink toward
`exponent == 0` from both sides, reaching exactly `1.0` there -- instead of monotonically
increasing with scene size, so the multiplier applied to `near` collapses to a no-op (`far ==
near`) precisely at that reachable point.

## Why Not Caught

No test exercised `near_far_from_exponent` across a representative `exponent` range including 0 --
the panic only surfaces at runtime, on the very first frame, for scenes at exactly that scale.

## Fix Applied (2026-08-18)

Added a floor (`.max(near * 10.0)`) to the `far` formula, guaranteeing `far` is always at least
10x `near` regardless of where the V-shaped multiplier collapses to (including its exact `1.0`
value at `exponent == 0`). Added an inline `#[cfg(test)]` module in `main.rs` (native unit tests,
no `tests/` directory needed for this pure-function fix):
`test_far_always_exceeds_near_across_exponent_range` sweeps `exponent in -10..=10` asserting `far
> near` throughout; `test_pre_fix_formula_was_degenerate_at_exponent_zero` pins the exact pre-fix
`far == near == 10.0` collapse, confirming the bug was real and not a hypothetical edge case.

## Verification

- **Pre-fix (RED):** reverted the `far` formula to its unguarded (no `.max` floor) form; new tests
  failed, reproducing `far == near` at `exponent == 0`.
- **Post-fix (GREEN):** `cargo test -p animation_blending -p skeletal_animation -p pbr_lighting -p character_control --no-fail-fast`
  (run together with sibling BUG-320/331 and this crate's own BUG-339 lil_gui fix) -- 12 tests
  passed, 0 failed; `cargo check --target wasm32-unknown-unknown` and
  `cargo clippy --all-targets --all-features -- -D warnings` (native + wasm32) all clean.

## Generalized Version

A derived camera-projection parameter (near/far, FOV) that depends on a scene's own runtime scale
must be validated across its full expected input range, including boundary/identity points like
`exponent == 0` where a power function's exponent-of-absolute-value shape can degenerate to a
no-op multiplier -- always floor/ceiling such a derived value against its sibling rather than
trusting the formula's shape to hold across the whole domain.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-XXX-B` placeholder marker (disambiguated from sibling findings in the same fork's other crates) since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-332 after a fresh on-disk collision scan. Related: BUG-320 (`animation_amplitude_change`), BUG-331 (`skeletal_animation`) -- same defect class, independently derived, different formulas. |
