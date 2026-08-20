# BUG-396: `primitive_generation::solid`'s box/cylinder/cone faces wound backwards, breaking shadow-map front-face culling

- **Severity:** Medium (no crash, no shape corruption -- positions/indices were always correct -- but
  produces near-total self-shadow acne on every mesh this module generates, for any consumer using
  a front-face-culling shadow-mapping technique)
- **state:** Completed
- **Affects:** Every consumer of `primitive_generation::box_mesh`/`cylinder_mesh` (the cone case is
  `cylinder_mesh` with a zero radius) whose renderer derives lighting from mesh winding rather than
  screen-space derivatives -- confirmed concretely for `renderer::webgl::shadow::ShadowMap`'s
  front-face-culling occluder technique (`self.gl.cull_face( gl::FRONT )` in `ShadowMap::bind()`),
  first exercised by `examples/minwebgl/falling_frontier`'s new shadow-mapped hull rendering (PR
  #212). `hull.frag`'s own flat-shading normal (derived via `dFdx`/`dFdy`, self-correcting via
  `gl_FrontFacing` regardless of winding) is unaffected either way.
- **Component:** `module/helper/primitive_generation` (`src/solid.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-19
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-19
- **Related Bugs:** Same general defect *class* as BUG-217 (a geometry generator in this same crate
  producing an orientation-dependent value -- there a missing normal, here backwards winding -- that
  a downstream shader silently mishandled with no error), but a different mechanism (winding vs. a
  missing attribute) and a different generator family (`solid.rs`'s procedural primitives vs.
  `primitive.rs`'s stroke/fill/plane geometry) -- filed separately, no shared root cause.

## Symptom

```rust
// pre-fix -- src/solid.rs, box_mesh
let indices = vec!
[
  0, 1, 2,  0, 2, 3, // back  (z = -hz)
  4, 6, 5,  4, 7, 6, // front (z = +hz)
  // ... all 6 faces wound the same (backwards) way
];
```

Every face's `cross(edge1, edge2)` pointed *into* the box, not outward -- confirmed algebraically for
the back face: with `edge1 = positions[1] - positions[0] = (2hx, 0, 0)` and
`edge2 = positions[2] - positions[0] = (2hx, 2hy, 0)`, `cross(edge1, edge2) = (0, 0, 4·hx·hy)`,
positive Z -- but the back face sits at `z = -hz` and should face *away* from the box center, i.e.
`-Z`. Same backwards pattern in `cylinder_mesh`'s two cap fans (side-wall quads were already correct).

## Impact

**Who is affected:** Any consumer whose renderer treats mesh winding as meaningful -- GL face
culling, or vertex normals derived from winding rather than computed per-pixel. Confirmed concretely
for `ShadowMap::bind()` (`module/helper/renderer/src/webgl/shadow.rs:83-84`):

```rust
self.gl.enable( gl::CULL_FACE );
self.gl.cull_face( gl::FRONT );
```

This shadow-mapping technique deliberately culls the *front*-facing (light-facing) triangles and
keeps the back-facing ones as the occluder written into the depth map -- a standard peter-panning /
acne mitigation that only works if winding correctly identifies which side is which. With backwards
winding, the near (light-facing) surface was recorded as the occluder instead of the far surface,
which reads as near-total self-shadow acne on anything facing the light.

**What breaks:** Visual only -- mesh shape/positions/indices were always correct, and any consumer
using screen-space-derivative normals (`hull.frag`'s `dFdx`/`dFdy` flat shading, which self-corrects
via `gl_FrontFacing` regardless of a mesh's winding) is unaffected. Only winding-dependent techniques
(GL culling, precomputed vertex normals derived from winding) are impacted.

**Consumer audit:** `box_mesh`/`cylinder_mesh` (the two functions this fix touches) have exactly 3
call sites workspace-wide, all within `falling_frontier` itself: `gizmo.rs`, `station.rs`, `ships.rs`
(`grep -rln` from the repo root, excluding `primitive_generation` itself). `station.rs`/`ships.rs`
feed `HullPart`s into the exact same `hull_program`/`shadow_map` pipeline as `asteroids.rs` (per
`main.rs`'s render loop, which chains `asteroids.parts()`/`ships.parts()`/`station.parts()`
identically into both the shadow pass and the lit draw pass) -- so the fix uniformly improves all
three, not just asteroids, with no divergent behavior to reconcile. `gizmo.rs`'s usage (the M6
transform-gizmo handle) is drawn in a separate call outside the shadow-sensitive pipeline, and
`GL::CULL_FACE` is disabled for the entire main render pass (`main.rs`, right after the shadow pass) --
only enabled during the shadow pass itself -- so winding has no effect on the gizmo's own rendering
either. No other crate in the workspace calls either function, so the "wide consumer footprint"
concern raised when this defect was first found (this crate has 6+ dependents in aggregate, for its
*other* generators) does not apply to this specific fix -- confirmed by direct audit, not assumed.

**Magnitude:** 12 triangles in `box_mesh` (last two indices of each swapped) + 2 cap fans in
`cylinder_mesh` (index order swapped) -- both already fixed, see Fix Location.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during code review of PR #212 ("Space game demo" shadow-mapping extension to
`falling_frontier`, merged as `f9a5ca4f`), not a dedicated bug hunt. The PR's own diff included a
fix to `solid.rs` (originally commented `Fix(winding): ...`, no bug reference) alongside new shadow
mapping code; review cross-checked the fix's stated rationale against `ShadowMap::bind()`'s actual
`cull_face(FRONT)` call (confirmed correct) and against this crate's existing `solid_test.rs`
(confirmed to assert only vertex/index counts and coordinates -- zero winding/orientation coverage,
so this exact defect had no regression guard before or after the PR's fix). Filed retroactively to
close that documentation/test gap, consistent with this crate's own BUG-217 precedent for exactly
this situation (an incidentally-discovered geometry defect in this same crate).

## Minimum Reproducible Example

```rust
// module/helper/primitive_generation/tests/solid_test.rs
let ( positions, indices ) = box_mesh( 1.0, 1.0, 1.0 );
// for each triangle, the face normal ( cross of its own edges ) must point
// away from the origin ( the box's own center ) -- pre-fix: pointed inward
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/primitive_generation && cargo nextest run -E 'binary(solid_test) and test(wind_outward)'
```

## Root Cause

`box_mesh`'s 6 faces and `cylinder_mesh`'s 2 cap fans were authored with the vertex *positions*
verified correct (matching three.js's `BoxGeometry`/`CylinderGeometry` layout) but the triangle
*index order* within each face never checked against outward-facing winding -- nothing in this
module's original design ever consumed winding (its own module doc explicitly disclaimed it: "None
of these bother getting triangle winding \"correct\" for outward normals"), so the mismatch was
invisible until a winding-dependent consumer (shadow-map face culling) was added downstream.

## Why Not Caught

`solid_test.rs` (pre-existing, covers `box_mesh`/`cylinder_mesh`/`torus_mesh`/`icosphere`) asserts
vertex counts, ring radii/heights, and index-bounds -- geometric invariants that hold regardless of
winding direction, so a backwards-wound mesh passes every existing assertion. The module's own doc
comment explicitly documented winding as unspecified/don't-care at the time, which was true until
`falling_frontier`'s shadow-mapping consumer (PR #212) made it load-bearing.

## Fix Location

Already applied via PR #212 (merged `f9a5ca4f`), before this bug report was filed --
`module/helper/primitive_generation/src/solid.rs`: `box_mesh`'s 12 triangles (all 6 faces) and
`cylinder_mesh`'s 2 cap fans (top and bottom) each have their last-two-indices swapped, reversing
winding to point outward. Side-wall quads in `cylinder_mesh` were already correctly wound and left
unchanged.

## Prevention

Two new tests in `solid_test.rs` -- `box_mesh_triangles_wind_outward` and
`cylinder_mesh_triangles_wind_outward` -- sharing a private helper
`assert_triangle_faces_outward`, asserting every triangle in `box_mesh` and every triangle in
`cylinder_mesh` (side quads and both cap fans) has a face normal (`cross` of its own two edges)
pointing away from the shape's own center -- the general, shape-agnostic invariant the fix
restores, not a pinned per-triangle expectation. Covering `cylinder_mesh`'s side quads too (not
just the two fans the fix touched) also independently re-verifies the PR's own claim that those
quads were already correctly wound.

## Pitfall

A geometry generator with no stated winding contract can silently become winding-*dependent* the
moment any consumer adds a technique that reads it (GL face culling, winding-derived normals) --
existing count/shape/coordinate tests give zero signal either way, since they hold regardless of
winding. A module doc comment disclaiming winding ("callers relying on `CULL_FACE` ... will need to
fix winding order ... themselves") is not a substitute for the module actually being internally
consistent about it -- three.js's own reference shapes (which this module matches for positions) are
correctly wound, so the safer default is outward-facing winding even when today's only consumer
doesn't care, rather than requiring every future winding-sensitive consumer to independently
rediscover and patch this module.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-19 | filed | Found during review of PR #212 (shadow-mapping extension to `falling_frontier`); fix had already landed via that PR's merge (`f9a5ca4f`) before this report existed. Filed to close the gap: no BUG-NNN reference, no 3-field source comment, no regression test. |
| 2026-08-19 | fixed | Fix itself unchanged (already correct, verified against `ShadowMap::bind()`'s real `cull_face(FRONT)` behavior) -- this pass rewrote `solid.rs`'s source comment to the mandated `Fix(BUG-396)`/`Root cause`/`Pitfall` format and added the missing regression test. |
| 2026-08-19 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: temporarily re-reversed `box_mesh`'s winding and confirmed the new test fails against it, then restored the fix and confirmed it passes -- test genuinely catches the defect, not vacuous. Final full-crate pass: `cargo nextest run -p primitive_generation` -- 28/28 pass (includes both new tests); `cargo clippy -p primitive_generation -p falling_frontier --all-targets --all-features -- -D warnings` clean; `cargo check --target wasm32-unknown-unknown` clean for `falling_frontier` (the fix's real-world consumer). | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-396)`/`Root cause`/`Pitfall` 3-field format applied to both comment sites (`box_mesh`, `cylinder_mesh`), matching this workspace's established source-comment convention (e.g. `camera_orbit_controls.rs:692-695`). | — |
| D3 | Scope containment | — | 🟢 | Fix itself untouched (already correct and already merged) -- this pass only added documentation/tests, confirmed via `git diff` touching only `solid.rs`'s comment and `solid_test.rs`. | — |

**Reproduced:** YES -- temporary revert of `box_mesh`'s indices to their pre-fix (backwards) order
caused the new `box_mesh_triangles_wind_outward` test to fail with an outward-normal-mismatch
assertion message; restoring the merged fix passes (`cylinder_mesh_triangles_wind_outward` was not
separately reverted -- its fix is structurally identical, see Fix Location). 2026-08-19.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/primitive_generation/src/solid.rs` | Fix already landed via PR #212; this pass rewrote the source comment to `Fix(BUG-396)`/`Root cause`/`Pitfall` format, and corrected the module doc comment's blanket "none of these bother getting winding correct" claim (stale for `box_mesh`/`cylinder_mesh` since the fix -- still accurate for `torus_mesh`/`icosphere`, which this bug never touched). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/primitive_generation/tests/solid_test.rs` | Added `box_mesh_triangles_wind_outward` and `cylinder_mesh_triangles_wind_outward` (plus shared helper `assert_triangle_faces_outward`), asserting every triangle's edge-cross-product points outward. |
