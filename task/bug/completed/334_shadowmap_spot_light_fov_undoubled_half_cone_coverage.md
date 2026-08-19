# BUG-334: `shadowmap` manually re-derives its spot-light shadow projection using the raw (undoubled) cone half-angle as a full FOV, leaving half the light's illumination cone without valid shadow-map depth data

- **Severity:** Medium (visible shadow gap on the outer half of every lit surface, not a crash)
- **state:** Completed
- **Affects:** `examples/minwebgl/shadowmap/src/main.rs`
- **Component:** examples/minwebgl/shadowmap
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

This crate previously constructed `shadow::Light` manually --
`mat3x3h::perspective_rh_gl(60.0_f32.to_radians(), 1.0, 0.1, 30.0)` with a hardcoded `light_size`
of `0.5` -- instead of using `renderer::webgl::shadow`'s canonical `impl From<SpotLight> for
Light`, which correctly doubles `outer_cone_angle` into a full FOV, derives `far` from the spot
light's own `range`, and derives `light_size` from the cone angle. Using the raw (undoubled)
`outer_cone_angle` directly as the shadow map's `fovy` made the shadow-map frustum only half as
wide as the spot light's actual illumination cone.

## Impact

**Who is affected:** every user of this demo -- the shadow map never covers the full illuminated
area.

**What breaks:** the outer half of every lit surface, including a visible band of this scene's
floor plane, has no valid shadow-map depth data -- surfaces outside the (incorrectly narrow)
shadow frustum render as if permanently unshadowed regardless of actual occlusion.

**Entity Scope:** `None` -- confined to this crate's own shadow-light construction; the canonical
`From<SpotLight> for Light` conversion in `module/helper/renderer` was already correct and
unaffected.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184), by comparing this crate's manual `shadow::Light` construction against the canonical
`impl From<SpotLight> for Light` the same `renderer::webgl::shadow` module already provides,
rather than assuming a hand-rolled re-derivation is equivalent. Independently verified by the
orchestrating session: `module/helper/renderer/src/webgl/shadow.rs:431-460`'s `From<SpotLight>`
impl does double `outer_cone_angle` (`outer_cone_angle * 2.0`) into its `fovy`, confirming this
crate's raw, undoubled use was the divergence.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p shadowmap test_shadow_light_projection_matches_canonical_doubled_fov
```
**Expected** (fixed): `shadow_light_from_spot`'s output projection matches
`mat3x3h::perspective_rh_gl(spot.outer_cone_angle * 2.0, 1.0, 0.1, spot.range)` exactly.
**Actual** (pre-fix): the manually-constructed projection used the raw, undoubled cone angle and a
hardcoded, disconnected `far = 30.0`.

## Root Cause

Manual re-derivation of a conversion the `shadow` module already provides correctly, using the
spot light's cone half-angle directly as a full FOV instead of doubling it first -- a shadow-casting
light's projection FOV must cover at least the light's own visible cone/angle, and reusing one of
the light's own angle fields as the shadow camera's FOV without checking whether that field is a
half-angle or full-angle silently under-covers the lit area.

## Why Not Caught

No test exercised this crate's shadow-light construction against the canonical `From<SpotLight>`
conversion -- the demo still renders a shadow map that "looks like" a shadow map either way, so an
undersized frustum has no symptom short of visually comparing shadow coverage against the light's
actual illumination cone.

## Fix Applied (2026-08-18)

Replaced the manual `shadow::Light` construction with `spot.into()`, delegating to
`renderer::webgl::shadow`'s canonical `impl From<SpotLight> for Light` -- which correctly doubles
`outer_cone_angle` into a full FOV, derives `far` from the spot light's own `range`, and derives
`light_size` from the cone angle, eliminating the manual re-derivation entirely. Added 2 tests to
the crate's existing test module: `test_shadow_light_projection_matches_canonical_doubled_fov`
asserts the fixed function's output matches the doubled-FOV formula element-wise;
`test_pre_fix_undoubled_fov_formula_would_have_diverged` confirms the fixed projection is not a
no-op relative to the old (undoubled FOV, disconnected `far=30.0`) formula.

## Verification

- **Pre-fix (RED):** reverted `shadow_light_from_spot` to its manual construction; new tests
  failed (projection diverged from the canonical doubled-FOV formula).
- **Post-fix (GREEN):** `cargo test -p shadowmap` -- both new tests pass;
  `cargo check --target wasm32-unknown-unknown -p shadowmap` and
  `cargo clippy --all-targets --all-features -p shadowmap -- -D warnings` both clean.

## Generalized Version

Reusing one of a light's own fields as an unrelated derived parameter (here, a cone half-angle
reused directly as a shadow camera's full FOV) requires checking whether the units/semantics
actually match, not just whether the value "looks like" a plausible angle -- when a canonical
conversion already exists elsewhere in the codebase for exactly this derivation, prefer it over a
manual re-derivation, which can silently diverge in ways a visual "it renders a shadow" check won't
catch.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-XXX-D` placeholder marker (disambiguated from sibling findings in the same fork's other crates) since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-334 after a fresh on-disk collision scan. |
