# BUG-171: `Node` panics on `.inverse().unwrap()` for zero-scale (singular) transforms

- **Severity:** High (crashes the entire per-frame render loop, not just the offending node --
  `world_matrix_set` is reached from `Renderer::render()` every frame for every node whose
  transform changed)
- **state:** Completed
- **Affects:** Any `Scene`/`Node` graph containing a node whose accumulated world scale has a
  zero on any axis -- a common glTF authoring pattern ("flatten"/hide a mesh via `scale: [1,0,1]`)
  or the natural result of an animation channel interpolating scale through `0.0` (a common
  shrink-to-nothing pop/disappear effect)
- **Component:** `module/helper/renderer` (`src/webgl/node.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** Discovered by a background Explore review of `helper/renderer`'s core WebGL
  pipeline subsystem (task #98), part of the same pass that surfaced BUG-172 through BUG-177.
  Independent root cause from all of them -- this is `Node`'s own matrix-inversion handling, not
  a glTF-loading or GPU-resource-sizing defect. Same defect *class* as BUG-161 (unguarded
  caller-supplied value reaching a panicking operation), a different concrete site.

## Symptom

```rust
// pre-fix -- webgl/node.rs
fn world_matrix_set( &mut self, matrix : F32x4x4 )
{
  self.world_matrix = matrix;
  self.normal_matrix = matrix.truncate().inverse().unwrap().transpose();  // panics on singular input
  self.bounding_box_compute();
  self.needs_world_matrix_update = false;
}
```

Two more call sites in the same file shared the identical pattern: `upload()`'s
`inverseWorldMatrix` uniform branch (`self.world_matrix.inverse().unwrap()`) and
`local_bounding_box_hierarchical()` (`self.world_matrix_get().inverse().unwrap()`, at least
disclosed via a `# Panics` doc comment, unlike the other two).

## Impact

**Who is affected:** Any consumer of `renderer::webgl::{Node, Scene}` whose scene graph contains
a node with a degenerate (zero-on-one-axis) world scale -- reachable via `Node::scale_set`
directly, via `local_matrix_set`/`matrix_apply` composing to a singular result, or via any glTF
asset loaded through this crate's own `loaders::gltf` path that contains a zero-scale node or an
animated scale channel that passes through `0.0`.

**What breaks:** `world_matrix_set` is called from `Node::world_matrix_update`, itself called
from `Scene::world_matrix_update()` -- the crate's own per-frame update entry point
(`renderer.rs`'s `Renderer::render()`). A single degenerate node anywhere in the scene panics the
entire render loop on the very next frame, not just at load time, taking down every other node's
rendering along with it.

**Magnitude:** 100% reproducible for any node whose world-space linear part has
`determinant() == 0` -- not a rare corner case; zero-scale is a standard technique both for
"hide this mesh without removing it from the hierarchy" and for scale-to-zero disappear
animations.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Empirical, via a background Explore review of `helper/renderer/src/webgl/{camera,geometry,
helpers,ibl,light,mesh,node,primitive,program,renderer,sampler,scene,shadow,texture}.rs` and
`loaders/*.rs` (task #98). The reviewer cross-checked `Mat3::inverse()`/`Mat4::inverse()`
(`ndarray_cg/src/d2/{mat3x3,mat4x4}/general.rs`) and confirmed they return `None` exactly when
`determinant() == 0`, then traced `world_matrix_set`'s caller chain to `Renderer::render()` to
confirm per-frame reachability. Independently reproduced here via a native unit test
(`scale_set([1.0, 0.0, 1.0])` + `world_matrix_update`) before the fix, which panicked as
predicted.

## Minimum Reproducible Example

```rust
// module/helper/renderer/tests/webgl/node.rs -- pre-fix, panics
let mut scene = Scene::new();
let node_root = Rc::new( RefCell::new( Node::new() ) );
scene.add( node_root.clone() );

node_root.borrow_mut().scale_set( [ 1.0, 0.0, 1.0 ] );  // zero on the y axis
node_root.borrow_mut().local_matrix_update();

scene.world_matrix_update();  // panics: `.inverse().unwrap()` on a singular 3x3 linear part
```

**Expected** (post-fix): completes without panicking; `normal_matrix` falls back to identity.

**Actual** (pre-fix):
```
thread '...' panicked at module/helper/renderer/src/webgl/node.rs:280:57:
called `Option::unwrap()` on a `None` value
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo nextest run -p renderer webgl::node::test_zero_scale_node_does_not_panic_on_singular_matrix_paths
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `Mat3::inverse()`/`Mat4::inverse()` return `None` for a singular matrix (`determinant() == 0`), and `world_matrix_set`/`upload`/`local_bounding_box_hierarchical` all `.unwrap()` the result unconditionally, with no caller-side or callee-side validation that the input transform is well-conditioned. | ✅ Root Cause | Confirmed by reading `ndarray_cg/src/d2/mat3x3/general.rs:55` and `mat4x4/general.rs:113` (`None` iff `determinant() == 0`), and reproduced directly: a native test constructing a zero-y-scale node and calling `world_matrix_update` panicked at the exact `.unwrap()` call site pre-fix. | E1, E2 |
| H2 | The panic is only reachable via a contrived, out-of-spec input (e.g. malformed test data), not a realistic scene-graph or glTF-loading scenario. | ❌ Falsified | Zero-scale is a documented, legal glTF authoring pattern (a `scale: [1,0,1]` node "flattens" a mesh to invisible without removing it), and this crate's own `tests/webgl/node.rs::test_set_scale` already exercises a zero-component scale (`[1.0, 5.0, 0.0]``) as an ordinary fixture, without ever calling `world_matrix_update` -- the coverage simply never reached the buggy path, not because the input is unrealistic. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/math/ndarray_cg/src/d2/mat3x3/general.rs:55`, `mat4x4/general.rs:113` | `inverse()` returns `None` exactly when `determinant() == 0`. | H1 ✅ |
| E2 | `module/helper/renderer/tests/webgl/node.rs` (pre-fix run, real `cargo nextest` output) | `scale_set([1.0, 0.0, 1.0])` + `local_matrix_update()` + `world_matrix_update()` panics at `node.rs:280:57` with `Option::unwrap() on a None value`, matching the predicted mechanism exactly. | H1 ✅ |
| E3 | `module/helper/renderer/tests/webgl/node.rs::test_set_scale` (pre-existing) | Already uses `scale = [1.0, 5.0, 0.0]` as an ordinary fixture, calling only `local_matrix_update()` (which doesn't touch `normal_matrix`/`world_matrix_set`) -- confirms zero-scale is a normal test input in this codebase, just never previously routed through the buggy per-frame path. | H2 ❌ |

## Root Cause

```rust
// before -- all 3 sites unconditionally unwrap Option<Mat>::None
self.normal_matrix = matrix.truncate().inverse().unwrap().transpose();                 // world_matrix_set
self.world_matrix.inverse().unwrap().to_array().as_slice()                             // upload (inverseWorldMatrix uniform)
bbox.transform_apply_mut( self.world_matrix_get().inverse().unwrap() );                // local_bounding_box_hierarchical
```

`world_matrix_update` (the crate's per-frame scene-graph update entry point) multiplies a
node's local matrix into its parent's world matrix with no constraint that the result stays
invertible. Any node whose accumulated scale has a zero on one axis produces a singular 3x3 (or
4x4) linear part, so `inverse()` returns `None` at all three sites, and `.unwrap()` panics.

## Why Not Caught

No existing test in `tests/webgl/node.rs` routed a zero-scale node through `world_matrix_update`
-- `test_set_scale` uses a zero-component scale but only calls `local_matrix_update()`, which
builds the local `matrix` field directly and never touches `normal_matrix` or `world_matrix_set`.
The world-matrix-update path (the crate's actual per-frame hot path) was never exercised with a
degenerate scale.

## Fix Location

`module/helper/renderer/src/webgl/node.rs`, all 3 sites now fall back to identity instead of
panicking when the linear part is singular:

```rust
// after -- world_matrix_set
self.normal_matrix = matrix.truncate().inverse().map_or_else( gl::math::mat3x3::identity, | m | m.transpose() );

// after -- upload (inverseWorldMatrix uniform)
let inverse_world_matrix = self.world_matrix.inverse().unwrap_or_else( gl::math::mat4x4::identity );

// after -- local_bounding_box_hierarchical
let inverse_world_matrix = self.world_matrix_get().inverse().unwrap_or_else( gl::math::mat4x4::identity );
bbox.transform_apply_mut( inverse_world_matrix );
```

Identity was chosen (rather than propagating an `Err`/`Option` through these call sites, which
would ripple `world_matrix_update`'s signature through its recursive, void-returning public API)
to match this crate's own precedent of graceful degradation over hard failure for degenerate
geometric input, and matches standard graphics-engine practice (e.g. three.js's `Matrix3.invert`
degrades to a zero/best-effort matrix rather than throwing on a singular input) -- a node with a
zero-scale axis already has zero visible cross-section in that dimension, so a best-effort
identity normal/inverse matrix has no meaningfully "more correct" alternative and avoids
propagating `NaN`/`Infinity` into shading or bounding-box math. `local_bounding_box_hierarchical`'s
now-stale `# Panics` doc comment was removed and replaced with a note describing the fallback.

## Prevention

Native regression test added: `test_zero_scale_node_does_not_panic_on_singular_matrix_paths`
(`tests/webgl/node.rs`) drives a zero-y-scale node through `Scene::world_matrix_update()` (hot
path, covers `world_matrix_set`) and `Node::local_bounding_box_hierarchical()` (covers the third
site) without a live GL context. `upload()`'s `inverseWorldMatrix` branch takes `&GL` and cannot
be exercised natively -- consistent with this crate's own established GL-boundary testing
precedent (task #75: GL-bound code paths documented as untestable natively, not filled with a
mock per this workspace's no-mocking policy) -- verified instead by direct code-parity with the
other two fixed sites (identical `.unwrap_or_else(identity)` pattern, confirmed by reading the
diff).

## Pitfall

A per-frame hot path computing a normal matrix (or any inverse-dependent value) via
inverse-transpose must treat non-invertible input as an expected, not exceptional, case --
zero-scale nodes are a normal authoring/animation pattern, not malformed data. `Option::unwrap()`
on a geometric `inverse()` call should be treated as a standing red flag in any code that accepts
caller- or asset-supplied transforms.

## Generalized Version

**Broken assumption:** "a node's accumulated world transform is always well-conditioned enough
to invert, so `.inverse().unwrap()` is safe in scene-graph update code."

**Confirmed general rule:** any transform reachable from caller-supplied scale/rotation/
translation values (directly, or via asset loading, or via animation) must treat `inverse()`
returning `None` as a normal, expected outcome -- fall back gracefully (identity, or skip the
dependent step) rather than unwrapping, in any code path reachable every frame.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered during a background Explore review of `helper/renderer`'s core WebGL pipeline subsystem (task #98); confirmed via a native test panicking at the predicted `.unwrap()` site. |
| 2026-08-16 | fixed | All 3 call sites (`world_matrix_set`, `upload`, `local_bounding_box_hierarchical`) fall back to identity instead of unwrapping `None`. |
| 2026-08-16 | verified | Native `cargo nextest -p renderer --all-features`: 88/88 passed (including the new regression test); `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote the reproducer from the agent's traced call chain; adversarial pass demanded the real panic first (initial reproducer via `local_matrix_set` silently no-op'd due to `decompose()` returning `None` on singular input -- caught by re-running and inspecting the actual assertion failure, not assumed), then rewrote via `scale_set`+`local_matrix_update` and confirmed the real pre-fix panic message before accepting the fix. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against sibling findings from the same review pass (BUG-172 through BUG-177) and against BUG-161 (same defect class, different site) -- no coupling, recorded rather than left unstated. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by direct source reading of `ndarray_cg`'s `inverse()` implementations and a real, reproduced panic with matching location/message. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Minimal fix at all 3 affected sites (identity fallback), no broader refactor of `Node`'s matrix-update flow attempted. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `renderer`'s `src/webgl/node.rs` + its own test file + this bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | All 3 unconditional `.unwrap()` sites in this exact failure mode were found via full-file read (confirmed against the reviewing agent's own grep-for-`.unwrap()` sweep) and fixed at their own definition sites; no other `.inverse().unwrap()` remains in this file. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | The fix is purely a graceful-degradation correction; `Node`'s own responsibilities (matrix bookkeeping, bounding-box computation) are unchanged. | — |

**Reproduced:** YES -- pre-fix, the native regression test panicked at `node.rs:280:57` with
`Option::unwrap() on a None value`, exactly as predicted from reading `ndarray_cg`'s `inverse()`
implementation. Post-fix, the identical test passes; full scoped suite (88/88) and clippy both
clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/node.rs` | `world_matrix_set`, `upload` (`inverseWorldMatrix` branch), and `local_bounding_box_hierarchical` all fall back to identity via `.map_or_else`/`.unwrap_or_else` instead of `.unwrap()`-panicking on a singular matrix (full `Fix(BUG-171)` comment blocks at the first two sites); the third site's `# Panics` doc comment was replaced with a note describing the new fallback behavior. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/webgl/node.rs` | New `test_zero_scale_node_does_not_panic_on_singular_matrix_paths`: drives a zero-y-scale node through `Scene::world_matrix_update()` and `Node::local_bounding_box_hierarchical()`, asserting no panic and that `world_matrix_get()` reflects the degenerate scale correctly. |
