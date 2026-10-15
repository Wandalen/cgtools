# BUG-257: WebGPU renderer's normal-matrix singular fallback packs the raw un-inverted block
instead of identity, despite a comment claiming parity with `webgl::Node`'s BUG-171 fix

- **Severity:** Medium (silently wrong lighting output for a realistic, previously-fixed-once
  trigger condition -- not a crash, no NaN/Inf, but a visibly incorrect result the sibling
  `webgl` backend already solved correctly)
- **state:** Completed
- **Affects:** `WebGpuRenderer`'s private `model_raw` method, via the newly-extracted
  `normal_matrix_compute`
- **Component:** `module/helper/renderer` (`src/webgpu/renderer.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`WebGpuRenderer`'s private `model_raw` method derives a "normal matrix" for lighting as
`rotation_scale.inverse().map_or( rotation_scale, | m | m.transpose() )`. On a well-conditioned
(invertible) `rotation_scale`, this correctly computes the inverse-transpose. On a **singular**
`rotation_scale` (`inverse()` returns `None`), it fell back to `rotation_scale` itself -- the raw,
un-inverted, un-transposed block -- packed directly into the uniform buffer and used as-is by the
shader's lighting math. The function's own comment claimed "Singular world matrices fall back to
the untransposed block -- same degenerate result the WebGL node path would produce
lighting-wise" -- but `webgl::Node::world_matrix_set` (BUG-171, an earlier session's fix) actually
falls back to **identity**, not the raw block. The two backends' behavior on a singular world
matrix therefore silently diverged, contradicting the comment's own explicit claim.

## Impact

**Who is affected:** Any WebGPU-rendered item whose accumulated world matrix has a singular
linear (rotation-scale) part -- per BUG-171's own root-cause note, a realistic, not merely
theoretical, trigger: "a common glTF 'flatten'/hide trick, or an animation channel interpolating
scale through `0.0`."

**What breaks:** Using the raw `rotation_scale` block directly as a normal transform (instead of
its inverse-transpose, or a safe identity fallback) scales normals by the object's own local
scale in the wrong direction -- e.g. for a `diag( 2, 0, 2 )` world scale (Y collapsed to zero,
the exact "flatten" trick BUG-171 names), every transformed normal's Y-component is forced to
exactly `0`, producing non-unit-length, direction-distorted normals fed straight into the
fragment shader's lighting math. No panic, no NaN -- just visibly wrong (but finite) lit output,
silently, for any WebGPU item whose world transform hits this condition.

**Entity Scope:** `None` -- source-level lighting-correctness gap, not entity directory
instances.

## How Discovered

During this session's `renderer` crate scout (task #174), reviewing the remaining WebGPU-module
files after closing BUG-255 (`webgpu/light.rs`). `webgpu/renderer.rs`'s `model_raw` carried an
inline comment explicitly asserting behavioral parity with "the WebGL node path" for the singular
case -- a specific, checkable claim. Cross-referencing `webgl::Node::world_matrix_set`
(`src/webgl/node.rs:280-294`) directly showed its own BUG-171 fix falls back to
`gl::math::mat3x3::identity`, not the raw block -- the comment's claim of parity was false.

## Minimum Reproducible Example

Pure CPU-side math, no GPU device needed -- the singular-fallback divergence is observable
directly on the extracted `normal_matrix_compute` function. See
`tests/webgpu_normal_matrix_test.rs`'s 2 new tests.

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p renderer --features native --test webgpu_normal_matrix_test
```
**Expected** (fixed): both tests pass (1 non-singular round-trip, 1 singular-fallback). **Actual**
(pre-fix, confirmed via temporary direct-edit revert-and-rerun): the singular-fallback test
failed -- the function returned the raw all-zero block instead of identity.

## Root Cause

`model_raw` (pre-fix, inlined):
```rust
fn model_raw( world : &gl::math::F32x4x4 ) -> ModelRaw
{
  let rotation_scale = world.truncate();
  // Singular world matrices fall back to the untransposed block — same
  // degenerate result the WebGL node path would produce lighting-wise.
  let normal = rotation_scale.inverse().map_or( rotation_scale, | m | m.transpose() );
  // ...
}
```
BUG-171 fixed exactly this same class of problem (a singular linear part reaching an
inverse-transpose computation) in `webgl::Node::world_matrix_set` by falling back to identity --
but that fix was applied only to the `webgl` backend. The `webgpu` backend's parallel
normal-matrix computation was never updated to match, and its own comment incorrectly asserted
the two already agreed.

## Why Not Caught

The normal-matrix computation was inlined directly in the private `model_raw` method with zero
test coverage of any kind -- no test exercised a singular world matrix through the WebGPU
renderer, and the inline comment's "same as WebGL" claim was never checked against
`webgl::Node`'s actual fallback value at the time it was written.

## Fix Applied (2026-08-17)

**`src/webgpu/renderer.rs`:** extracted the normal-matrix computation into its own `pub fn
normal_matrix_compute( rotation_scale : gl::math::F32x3x3 ) -> gl::math::F32x3x3`, whose singular
fallback now matches `webgl::Node`'s BUG-171 fix exactly:
```rust
pub fn normal_matrix_compute( rotation_scale : gl::math::F32x3x3 ) -> gl::math::F32x3x3
{
  rotation_scale.inverse().map_or_else( gl::math::mat3x3::identity, | m | m.transpose() )
}
```
`model_raw` now calls this function instead of inlining the (previously incorrect) logic.
Exported via `mod_interface!` (`orphan use { ..., normal_matrix_compute }`) to make it directly
unit-testable, matching this session's established `displacement_texture_size_compute` (BUG-252)
precedent for extracting pure-math logic out of a rendering method for testability.

**`tests/webgpu_normal_matrix_test.rs`** (new file): 2 native `#[ test ]` functions --
`accepts_a_non_uniform_scale_and_computes_its_inverse_transpose` (a hand-computable
`diag( 2, 4, 1 )` fixture, confirming the non-singular path still computes a real inverse-transpose
and doesn't just always return identity), and
`rejects_a_singular_matrix_by_falling_back_to_identity_not_the_raw_block` (an all-zero singular
fixture via `Mat::_fill( 0.0 )`, confirming the fallback is identity, not the raw block).

## Verification

`longrun`-detached, from repo root:
- `cargo test -p renderer --features native --test webgpu_normal_matrix_test` -- pre-fix
  (temporary direct-source-edit revert of `map_or_else(...)` back to `map_or( rotation_scale,
  ... )`): the singular-fallback test failed, the non-singular test still passed. Post-fix
  (restored): both tests passed.
- `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: exit 0, clean.
- `cargo check -p renderer --features native`: exit 0, confirming the extracted function's
  `gl::math::F32x3x3`/`gl::math::mat3x3::identity` paths resolve correctly through
  `minwebgpu`'s `pub use mingl::math;` re-export.

Note: a broader `cargo test -p renderer --all-features` run during this same session hit an
unrelated, pre-existing failure in `webgl::camera::from_bounding_box_accepts_a_degenerate_zero_radius_box`
-- confirmed via `git status`/mtime to be a different concurrent session actor's own in-flight,
uncommitted work on a new `Camera::from_bounding_box` method (not touched by this fix, in a
different file), so it was excluded from this bug's own verification scope.

## Generalized Version

**Broken assumption:** a comment asserting behavioral parity between two sibling implementations
(here, the `webgl` and `webgpu` backends' normal-matrix computations) is itself proof of that
parity -- it isn't; a fix applied to one backend (BUG-171, `webgl::Node`) does not propagate to a
structurally similar sibling backend automatically, and any comment claiming otherwise must be
checked against the sibling's actual current source, not trusted at face value. This is the same
generalized lesson as BUG-253 (`Camera::projection_matrix_set` bypassing BUG-174's sibling
`Camera::new` validation) and BUG-255 (`Lights::spot_push`'s doc comment permitting an input that
breaks its own shader consumer) applied one level up: whenever this codebase maintains parallel
`webgl`/`webgpu` implementations of the same concept, a fix to one is not evidence the other was
fixed too.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found by direct review of `webgpu/renderer.rs` during task #174's `renderer` crate scout, immediately after closing BUG-255 in the sibling `webgpu/light.rs`. Root cause: `model_raw`'s inline normal-matrix computation fell back to the raw, un-inverted `rotation_scale` block on a singular matrix, while its own comment claimed this matched `webgl::Node::world_matrix_set`'s BUG-171 fix (which actually falls back to identity) -- the comment's claim was checked directly against BUG-171's fix and found false. Fixed by extracting the computation into its own `pub fn normal_matrix_compute`, whose singular fallback now genuinely matches BUG-171's identity fallback. Verified via 2 new native unit tests (1 confirmed fail pre-fix / pass post-fix via temporary revert-and-rerun) plus clean clippy and `cargo check`. Filed as BUG-257 after a fresh on-disk scan (immediately following the Group E review fork's own BUG-256 filing in a sibling file, `webgl/mesh.rs`, confirmed non-overlapping) found 256 claimed and 257 free. Closed same-session (Tier 2 Dual-Role Self-Check). |
