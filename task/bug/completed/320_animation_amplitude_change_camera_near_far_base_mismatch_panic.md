# BUG-320: `animation_amplitude_change`'s camera near/far derivation mixes a base-2 bit-layout scale with a base-10 power function, collapsing `far <= near` and panicking `Camera::new` for ordinary scene scales

- **Severity:** High (panics the demo at startup for its own bundled asset)
- **state:** Completed
- **Affects:** `examples/minwebgl/animation_amplitude_change/src/main.rs`
- **Component:** examples/minwebgl/animation_amplitude_change
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`camera_setup`'s `near`/`far` were derived from a scale value read out of the raw IEEE-754 bit
layout of `diagonal` (a base-2 quantity by construction), then fed into a base-10 power function
(`10.0f32.powf(magnitude)`-style derivation). The base mismatch, combined with a `far` formula not
monotonically greater than `near` across its own domain, collapsed to `far <= near` for an
ordinary scene scale -- including this crate's own bundled `multi_animation.glb` -- which
`Camera::new` rejects outright (`far` must be strictly greater than `near`), panicking the demo's
own `.expect("camera parameters are valid")` at startup.

## Impact

**Who is affected:** every user of this demo -- it panics before rendering a single frame.

**What breaks:** the whole demo fails to start; `Camera::new`'s own precondition (`far > near`) is
violated by the near/far values this crate derives from its bundled asset's actual bounding-box
diagonal.

**Entity Scope:** `None` -- confined to this crate's own camera-setup derivation.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184), sharing the same defect class independently found in sibling crates `skeletal_animation`
(BUG-331) and `pbr_lighting` (BUG-332) -- three different `near_far_from_exponent`-style formulas,
each collapsing `far <= near` for a different input band, filed as three separate cross-referenced
bugs (matching the BUG-307/308 precedent for related-but-independently-formulated defects) rather
than one combined bug, since each crate's formula and collapse condition genuinely differ.
Independently verified by the orchestrating session by reading the actual formula and confirming
`Camera::new`'s `far > near` precondition against it.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p animation_blending --test camera_near_far_test
```
(note: this crate's real Cargo.toml package name is `animation_blending`, diverging from its own
`animation_amplitude_change` directory name.)
**Expected** (fixed): `test_far_always_exceeds_near_across_exponent_range`-equivalent coverage
passes for the crate's own bundled scene scale. **Actual** (pre-fix): `far <= near` for the
bundled asset's diagonal, `Camera::new` panics.

## Root Cause

A base-2-derived scale factor (extracted from `diagonal`'s raw floating-point bit layout) was
paired with a base-10 power computation to derive `far` from `near` -- an internally inconsistent
base mismatch that, combined with the resulting formula's shape, fails to guarantee `far > near`
across the input domain the crate's own bundled scene actually falls into.

## Why Not Caught

No test exercised `camera_setup`'s near/far derivation against the crate's actual bundled asset's
bounding-box diagonal -- the panic only surfaces at runtime, on the very first frame, with no
compile-time signal that the formula's output could violate `Camera::new`'s precondition.

## Fix Applied (2026-08-18)

Replaced the base-2 bit-layout-derived scale with `diagonal.max(f32::EPSILON).log10().floor()` --
the scene's true base-10 order of magnitude, computed directly rather than via raw bit-layout
inspection -- combined with a fixed `far`/`near` ratio (`1_000_000`) that guarantees `far > near`
for every finite positive `diagonal`. Added `tests/camera_near_far_test.rs`: sweeps a range of
representative `diagonal` values (including the bundled asset's actual scale) asserting
`far > near` holds and both remain finite and positive throughout.

## Verification

- **Pre-fix (RED):** reverted to the base-2/base-10 mismatched formula; new test failed
  (`far <= near` reproduced for the bundled asset's diagonal).
- **Post-fix (GREEN):** `cargo test -p animation_blending -p skeletal_animation -p pbr_lighting -p character_control --no-fail-fast`
  (run together with sibling BUG-331/332 and this crate's own BUG-339 lil_gui fix) -- 12 tests
  passed, 0 failed; `cargo check --target wasm32-unknown-unknown` and
  `cargo clippy --all-targets --all-features -- -D warnings` (native + wasm32) all clean.

## Generalized Version

A derived camera-projection parameter (near/far, FOV) that depends on a scene's own runtime scale
must be validated against the actual asset(s) the crate bundles, not just against the formula's
algebraic shape in isolation -- a formula relying on a mismatched numeric base (bit-layout
extraction feeding a decimal power function) or a non-monotonic shape can collapse a required
invariant (`far > near`) for exactly the ordinary scale the crate's own bundled data uses, with no
compile-time signal.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-WWW` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-320 after a fresh on-disk collision scan. Related: BUG-331 (`skeletal_animation`), BUG-332 (`pbr_lighting`) -- same defect class, independently derived, different formulas. |
