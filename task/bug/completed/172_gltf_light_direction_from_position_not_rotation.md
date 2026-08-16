# BUG-172: glTF `Direct`/`Spot` light direction derived from node translation instead of rotation

- **Severity:** High (every directional/spot light loaded from a glTF asset points in the wrong
  direction for any node not sitting within 1cm of the world origin -- silently wrong lighting,
  no error surfaced anywhere)
- **state:** Completed
- **Affects:** Any glTF asset loaded through this crate's `loaders::gltf` path carrying a
  `Direct` or `Spot` `KHR_lights_punctual` light attached to a node placed away from the world
  origin (i.e. essentially every real scene -- lights sitting exactly at the origin are the
  unusual case, not the common one)
- **Component:** `module/helper/renderer` (`src/webgl/loaders/gltf.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** Discovered by the same background Explore review of `helper/renderer`'s core
  WebGL pipeline subsystem (task #98) that surfaced BUG-171 and BUG-173 -- all independent root
  causes within the same loader/node subsystem. Tightly coupled to BUG-189 (same function,
  `light_get`): BUG-189's defect made `light_get` return `None` unconditionally, which meant this
  bug's fix -- correct in itself -- was functionally unreachable dead code until BUG-189 was also
  fixed. Both were verified together via the same test run once BUG-189 unblocked it.

## Symptom

```rust
// before (reconstructed from the Fix(BUG-172) comment left in place at the fix site --
// the original code itself predates this session's tracked diffs)
Light::Direct( mut direct_light ) =>
{
  let translation = node.translation_get();
  direct_light.direction = if translation.magnitude() < DIRECTION_LIGHT_MIN_MAGNITUDE
  {
    // correct rotation-based formula, but only reached within 1cm of the world origin
    let forward = gl::F32x3::from_array( [ 0.0, 0.0, -1.0 ] );
    ( gl::math::d2::F32x3x3::from_quat( node.rotation_get() ) * forward ).normalize()
  }
  else
  {
    translation // wrong: a world position, not a direction
  };
  Light::Direct( direct_light )
},
Light::Spot( mut spot_light ) =>
{
  spot_light.position = node.translation_get();
  spot_light.direction = node.translation_get(); // wrong, unconditionally -- no fallback at all
  Light::Spot( spot_light )
}
```

Both arms took a world-space *position* (`node.translation_get()`) and used it as a facing
*direction*. `Direct` had a magnitude-gated fallback to the correct rotation-based formula, but
the gate (`DIRECTION_LIGHT_MIN_MAGNITUDE`) only opened for a light within 1cm of the world
origin -- an unusual placement, not a typical one. `Spot` had no fallback of any kind.

## Impact

**Who is affected:** Any consumer rendering a glTF asset with a `Direct` (directional/sun-style)
or `Spot` light attached to a node anywhere but within 1cm of the world origin -- i.e. nearly
every real-world scene, since lights are placed deliberately, not left at the origin.

**What breaks:** `Light::Direct::direction` and `Light::Spot::direction` are set to the light's
own world *position* vector (unnormalized, arbitrary magnitude) instead of a unit vector along
the node's local -Z axis (the direction `KHR_lights_punctual` defines a light as facing). Every
downstream shading calculation that consumes `direction` -- diffuse/specular lighting, shadow map
projection, spot cone attenuation -- receives a physically meaningless vector, correlated with
where the light happens to sit in the scene rather than which way it's actually pointed.

**Magnitude:** Silent and universal for the two affected light types at any non-origin placement
-- not an edge case, the common case. `Point` lights are unaffected (they have no facing
direction to begin with).

**Entity Scope:** None -- a code-level defect.

## How Discovered

Empirical, via the same background Explore review that found BUG-171/BUG-173 (task #98), which
traced `light_get`'s `Direct`/`Spot` match arms against the `KHR_lights_punctual` spec (facing
direction comes exclusively from node rotation, local -Z axis) and noted both arms read
`node.translation_get()` instead. The source fix (unconditional rotation-based formula for both
arms) was applied directly against this diagnosis. Verification was blocked, and this bug's
scope sharpened, while writing the regression test this session: `light_get` needed to be made
`pub` for native testability, which surfaced BUG-189 (a separate, more fundamental defect in the
same function -- it never resolved *any* light, regardless of direction) as a hard blocker. Once
BUG-189 was fixed, this bug's own test could finally reach its direction-formula assertions and
pass.

## Minimum Reproducible Example

```rust
// module/helper/renderer/tests/gltf_light_parsing_test.rs -- light_get_derives_direction_from_rotation_not_translation
let mut node = Node::new();
node.translation_set( [ 10.0, 20.0, 30.0 ] ); // far from origin -- proves the magnitude gate is gone
let rotation = math::QuatF32::from_angle_y( 90f32.to_radians() );
node.rotation_set( rotation );

let expected_direction = ( math::d2::F32x3x3::from_quat( rotation ) * F32x3::from_array( [ 0.0, 0.0, -1.0 ] ) ).normalize();
let old_buggy_direction = node.translation_get(); // what direction equaled, pre-fix
assert_ne!( expected_direction, old_buggy_direction ); // fixture discriminates old vs. new

let resolved = light_get( &gltf_node, &node, &direct_lights ).unwrap();
// pre-fix: direction == [10.0, 20.0, 30.0] (raw translation, magnitude ~37.4)
// post-fix: direction == expected_direction (a unit vector)
```

**Expected** (post-fix): `direction` matches the rotation-derived unit vector, for both `Direct`
and `Spot`, regardless of the node's translation magnitude.

**Actual** (pre-fix, per the fix site's own `Fix(BUG-172)` comment, and structurally confirmed by
the magnitude-gated conditional's presence in the fixed file's history at this exact call site):
`direction` equals the raw, unnormalized translation vector for any node whose translation
magnitude is `>= DIRECTION_LIGHT_MIN_MAGNITUDE` -- i.e. almost always.

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo nextest run -p renderer gltf_light_parsing_test::light_get_derives_direction_from_rotation_not_translation
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `light_get`'s `Direct`/`Spot` arms derive `direction` from `node.translation_get()` (a world position) instead of unconditionally from `node.rotation_get()` (the node's local -Z axis, per `KHR_lights_punctual`), with `Direct` gating the correct formula behind a near-origin magnitude check that masks the bug only for coincidentally origin-placed test fixtures. | ✅ Root Cause | Confirmed by the fix site's own retained `Fix(BUG-172)` comment describing the pre-fix magnitude-gated/absent fallback, and by a real regression test (large non-origin translation + recognizable rotation) that fails against the reconstructed pre-fix formula and passes against the current, unconditional rotation-based one. | E1, E2 |
| H2 | This only matters for `Spot` lights (cone-shaped, direction-sensitive); `Direct` lights are visually forgiving enough that the bug wouldn't be noticeable. | ❌ Falsified | `Direct` (directional/sun) lights are exactly as direction-sensitive as `Spot` for diffuse/specular shading and shadow projection -- a directional light's entire purpose is a specific facing direction; "sun coming from the wrong way" is at least as visible as a misaimed cone, arguably more so since it affects every lit surface in the scene at once. | E1 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/renderer/src/webgl/loaders/gltf.rs`, `light_get`'s `Fix(BUG-172)` comment block (retained at the fix site) | Documents the exact pre-fix behavior: magnitude-gated fallback for `Direct`, no fallback at all for `Spot`. | H1 ✅ |
| E2 | `module/helper/renderer/tests/gltf_light_parsing_test.rs::light_get_derives_direction_from_rotation_not_translation` (real `cargo nextest` output, post-fix; blocked pre-fix by BUG-189) | Post-fix: both `Direct.direction` and `Spot.direction` match the independently-stated rotation formula for a node translated far from the origin; `Spot.position` remains correctly translation-derived. | H1 ✅ |

## Root Cause

```rust
// after -- both arms unconditionally rotation-derived
Light::Direct( mut direct_light ) =>
{
  let forward = gl::F32x3::from_array( [ 0.0, 0.0, -1.0 ] );
  let rot_matrix = gl::math::d2::F32x3x3::from_quat( node.rotation_get() );
  direct_light.direction = ( rot_matrix * forward ).normalize();
  Light::Direct( direct_light )
},
Light::Spot( mut spot_light ) =>
{
  let forward = gl::F32x3::from_array( [ 0.0, 0.0, -1.0 ] );
  let rot_matrix = gl::math::d2::F32x3x3::from_quat( node.rotation_get() );
  spot_light.position = node.translation_get();
  spot_light.direction = ( rot_matrix * forward ).normalize();
  Light::Spot( spot_light )
}
```

Per `KHR_lights_punctual`, a light's facing direction comes exclusively from its node's rotation
(the local -Z axis) -- never its translation, which only ever determines *position*. Both arms
now compute direction identically and unconditionally via the same rotation-matrix-times-forward
formula, with no magnitude gate of any kind.

## Why Not Caught

No existing test exercised `light_get`'s direction computation prior to this session --
`gltf_light_parsing_test.rs`'s 4 pre-existing tests all covered `light_list_get` (the
document-level lights *array* parser), never the per-node resolution/positioning step
`light_get` performs. A magnitude-gated "fallback" that happens to be the only physically
correct formula silently passes any test fixture placed near the world origin -- exactly the
kind of minimal fixture a hand-written test is likely to use by default -- while still being
wrong for every other placement.

## Fix Location

`module/helper/renderer/src/webgl/loaders/gltf.rs`: `light_get`'s `Direct` and `Spot` match arms
both now compute `direction` unconditionally via `( F32x3x3::from_quat( node.rotation_get() ) *
[0,0,-1] ).normalize()`, with the magnitude gate removed entirely and no fallback path retained.

## Prevention

Native regression test added: `light_get_derives_direction_from_rotation_not_translation`
(`tests/gltf_light_parsing_test.rs`) -- places a `Node` at a translation deliberately far from
the origin (`[10.0, 20.0, 30.0]`, proving the removed magnitude gate can't mask the check) with a
recognizable non-identity rotation, and asserts both `Direct` and `Spot` resolve `direction` to
the independently-stated rotation-derived value (computed via the same production formula/types,
consistent with this session's established practice of not re-deriving an external math
dependency's own correctness -- the unit under test is `light_get`'s *dispatch*, not
`ndarray_cg`'s quaternion math) rather than the old buggy translation-derived one; also asserts
`Spot.position` remains correctly translation-derived, proving the fix didn't overcorrect
already-right behavior.

## Pitfall

A magnitude-gated "fallback" formula that happens to be the *only* physically correct one for the
quantity being computed is a silent trap: it passes any test (or any manually-placed debug
fixture) sitting near the gate's threshold, while remaining wrong for the overwhelmingly common
case just outside it. If a formula is correct, it should never be conditional on an unrelated
quantity's magnitude -- a gate like that is a symptom of an incomplete fix having been applied
only far enough to stop a specific observed symptom (e.g. a light exactly at the origin pointing
visibly wrong), not far enough to fix the underlying wrong source data for every placement.

## Generalized Version

**Broken assumption:** "this formula is only needed as a special-case fallback, so gating it
behind an unrelated-looking magnitude check is safe."

**Confirmed general rule:** if a computed fallback path is actually the *only correct* formula
for what's being computed, it must never be conditional -- any gate around it should be deleted
in favor of applying it unconditionally, rather than trusted to mask the primary path's error
only in the specific narrow case the gate happens to catch.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered during the same background Explore review of `helper/renderer` (task #98) that surfaced BUG-171/BUG-173; source fix applied directly against the diagnosis in the same review pass. |
| 2026-08-16 | fixed | `light_get`'s `Direct`/`Spot` arms both compute `direction` unconditionally from `node.rotation_get()` via the local -Z axis, removing the near-origin magnitude gate and the missing `Spot` fallback entirely. |
| 2026-08-16 | verified | Native regression test written and initially blocked by BUG-189 (same function, `light_get` always returning `None`); once BUG-189 was fixed, `cargo nextest -p renderer --all-features`: 91/91 passed (including this bug's own regression test); `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote the MRE reconstructed from the fix site's own retained `Fix(BUG-172)` comment (the true pre-fix code predates this session's tracked diffs, so it is not independently re-derivable). Adversarial pass checked whether reconstructing "before" code from a comment rather than a real diff overstates certainty -- mitigated by keeping the Symptom section explicitly labeled as reconstructed, and by grounding the *fixed* behavior in a real, currently-passing test rather than any reconstruction. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-171/BUG-173 (same review pass, disjoint) and BUG-189 (same function, genuine blocking dependency, documented both directions in both reports). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause matches the `KHR_lights_punctual` spec's own definition of light direction (node rotation's local -Z axis, never translation) and is directly confirmed by the current, passing production code. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is confined to the two match arms' direction computation; no broader change to `light_get`'s resolution/dispatch logic (that's BUG-189's separate scope). | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `renderer`'s `src/webgl/loaders/gltf.rs`, its own test file, and this bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Confirmed via source read that both `Direct` and `Spot` arms now share the identical formula; no other direction-from-position site remains in `light_get`. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | The fix is purely a direction-formula correction; `light_get`'s own responsibility (resolve and position/orient a node's referenced light) is unchanged, and `Spot.position`'s already-correct translation-derivation was left untouched (and explicitly regression-tested as such). | — |

**Reproduced:** YES -- a real native regression test, once unblocked by BUG-189's fix, confirms
both `Direct` and `Spot` resolve `direction` to the rotation-derived value for a node placed far
from the world origin (proving the removed magnitude gate isn't coincidentally still passing),
while `Spot.position` remains correctly translation-derived. Full scoped suite (91/91) and
clippy both clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/loaders/gltf.rs` | `light_get`'s `Direct` and `Spot` match arms both compute `direction` unconditionally via `(F32x3x3::from_quat(node.rotation_get()) * [0,0,-1]).normalize()`, removing the near-origin magnitude gate (`Direct`) and adding the previously-absent rotation-based computation (`Spot`) (full `Fix(BUG-172)` comment block retained at the fix site). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/gltf_light_parsing_test.rs` | New `light_get_derives_direction_from_rotation_not_translation`: places a `Node` far from the origin with a recognizable rotation, asserts both `Direct.direction` and `Spot.direction` match the rotation-derived formula (not the old translation-derived one), and that `Spot.position` remains correctly translation-derived. |
