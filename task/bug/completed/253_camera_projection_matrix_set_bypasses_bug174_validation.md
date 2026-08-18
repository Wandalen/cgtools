# BUG-253: `Camera::projection_matrix_set` accepts any matrix with zero validation, completely
bypassing BUG-174's construction-time guards

- **Severity:** High (identical panic mechanism and public-API exposure as BUG-174 -- any external
  consumer of this crate's public `Camera` type can reach the same downstream crash through this
  setter alone, with no validation of any kind standing in the way)
- **state:** Completed
- **Affects:** `Camera::projection_matrix_set`, and (pre-fix) its one real call site in
  `examples/minwebgl/gltf_viewer/src/main.rs`'s canvas-resize handler
- **Component:** `module/helper/renderer` (`src/webgl/camera.rs`) + `examples/minwebgl/gltf_viewer`
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`Camera::new` (BUG-174) validates `aspect_ratio`/`fov`/`near`/`far` before ever calling
`perspective_rh_gl`, specifically to stop a degenerate ( non-finite or zero-determinant )
projection matrix from being constructed in the first place. `Camera::projection_matrix_set` --
the only other way to change a `Camera`'s projection matrix after construction -- assigned its
argument straight to `self.projection_matrix` with no check at all, silently discarding BUG-174's
entire protection the moment a caller recomputed the matrix itself instead of going back through
`Camera::new`.

## Impact

**Who is affected:** Any consumer of `renderer::webgl::Camera::projection_matrix_set` -- a fully
public method (`pub fn`, exported via `mod_interface!`) on a fully public type. This crate's own
bundled `gltf_viewer` example was itself an affected caller: its canvas-resize handler recomputes
`perspective_rh_gl( fov, w as f32 / h as f32, near, far )` on every resize and passed the result
straight into this setter.

**What breaks:** A non-finite or singular projection matrix assigned via this setter is stored with
no error signal, deferring the failure to whatever downstream code calls `.inverse()` on it next --
in this codebase, `Renderer::skybox_draw` (`src/webgl/renderer.rs`) does exactly that and
`.unwrap()`s the result, so the actual panic surfaces several frames away from the setter call that
caused it, with a message ( `Option::unwrap()` on `None` ) that names neither `Camera` nor the
resize handler.

**Entity Scope:** `None` -- source-level API validation gap, not entity directory instances.

## How Discovered

During this session's `renderer` crate scout (task #174), direct review of `camera.rs` --
immediately after tracing BUG-174's own fix in the same file -- noticed `projection_matrix_set`
sits right below `Camera::new` but was never touched by that fix. A workspace-wide grep for
`projection_matrix_set` confirmed exactly 1 real call site, in `gltf_viewer`'s resize handler,
recomputing an unvalidated matrix on every call -- the identical trigger BUG-174's own fix
documents for `Camera::new` (a transiently zero-height canvas), just reachable through a second,
unguarded entry point.

## Minimum Reproducible Example

`perspective_rh_gl`'s own domain restrictions are exactly what BUG-174 already validates for
`Camera::new` -- reused here directly by constructing a degenerate matrix independent of that
function ( an all-zero matrix is already singular; an all-NaN matrix is already non-finite ), so no
live `WebGl2RenderingContext` is needed. See `tests/webgl/camera.rs`'s 3 new
`projection_matrix_set_*` tests.

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p renderer --test tests webgl::camera::projection_matrix_set
```
**Expected** (fixed): all 3 new tests pass (2 reject, 1 accepts a valid matrix). **Actual**
(pre-fix, confirmed via temporary direct-edit revert-and-rerun of the validation block): the 2
`rejects_*` tests fail -- `result.is_err()` sees `Ok(())` for both the singular and the NaN-poisoned
matrix, since the pre-fix setter had no check to reject either.

## Root Cause

`projection_matrix_set` (pre-fix):
```rust
pub fn projection_matrix_set( &mut self, projection_matrix : gl::F32x4x4 )
{
  self.projection_matrix = projection_matrix;
}
```
`Camera::new` validates its *scalar* inputs (`aspect_ratio`/`fov`/`near`/`far`) before building a
matrix from them -- but `projection_matrix_set` accepts an already-built matrix directly, so those
scalar checks structurally can never run for a caller going through the setter. Validating a
constructor's inputs protects only that constructor's own code path; it does nothing for a sibling
entry point accepting the constructor's *output type* directly as input.

## Why Not Caught

`projection_matrix_set` had zero test coverage of any kind prior to this bug -- BUG-174's own new
tests exercised only `Camera::new`, never this setter, and no existing test called
`projection_matrix_set` at all, degenerate or otherwise.

## Fix Applied (2026-08-17)

**`src/webgl/camera.rs`:** `projection_matrix_set` now returns `Result< (), gl::WebglError >` and
rejects a non-finite-component or non-invertible ( singular ) matrix before assigning it to
`self.projection_matrix`:
```rust
pub fn projection_matrix_set( &mut self, projection_matrix : gl::F32x4x4 ) -> Result< (), gl::WebglError >
{
  if !projection_matrix.to_array().iter().all( | c | c.is_finite() )
  {
    return Err( gl::WebglError::Other( "Camera::projection_matrix_set: projection_matrix must have all-finite components" ) );
  }
  if projection_matrix.inverse().is_none()
  {
    return Err( gl::WebglError::Other( "Camera::projection_matrix_set: projection_matrix must be invertible" ) );
  }

  self.projection_matrix = projection_matrix;
  Ok( () )
}
```

**`examples/minwebgl/gltf_viewer/src/main.rs`:** updated the one real call site to handle the new
`Result`, via `.expect(...)` matching the resize handler's own already-established convention two
lines below it ( `renderer.borrow_mut().resize( &gl, w, h, samples ).expect( "Failed to resize
renderer" )` ). No separate zero-size guard was added at this call site: `canvas_size()` already
floors both dimensions to a minimum of 1 via `.max( 1 )`, so `w`/`h` can never reach 0 through this
specific caller -- adding a redundant guard for a state this call site cannot produce would validate
a scenario that can't happen. The setter's own validation remains the real fix, protecting this call
site as defense-in-depth and any future caller unconditionally.

**`tests/webgl/camera.rs`** (extended, not new): 3 new native `#[ test ]` functions --
`projection_matrix_set_rejects_a_singular_matrix` (all-zero matrix, via the existing `Mat::_fill`
constructor), `projection_matrix_set_rejects_non_finite_components` (all-NaN matrix, same
constructor), and `projection_matrix_set_accepts_a_valid_matrix` (round-trips a `Camera::new`
-produced matrix back through the setter, confirming the happy path still works and the stored
value is actually updated).

## Verification

`longrun`-detached, from repo root:
- `cargo test -p renderer --test tests webgl::camera::` -- pre-fix (temporary direct-source-edit
  revert of the two validation blocks): the 2 `rejects_*` tests failed, the pre-existing
  `Camera::new`-family tests and the new `accepts_*` test still passed. Post-fix (blocks restored):
  all `camera::*` tests passed, including the 3 new ones.
- `verb/test_only pkg::renderer` (full scoped suite, post-fix): all tests passing, up from 147
  (BUG-245's count) by 3 (this bug's new tests).
- `cargo clippy -p renderer -p gltf_viewer --all-features --all-targets -- -D warnings`: exit 0,
  clean.

## Generalized Version

**Broken assumption:** validating a constructor's scalar inputs fully protects the invariant those
inputs establish on a struct field -- it doesn't, the moment a *second* entry point (a setter,
a deserializer, a `From` impl) can assign that same field from a value of the field's own output
type, bypassing the constructor's input-side checks entirely. Whenever a bug fix adds validation to
a constructor, grep the same type for every other `pub fn` that assigns the same field directly and
check each one independently -- BUG-174 fixed `Camera::new` but left `projection_matrix_set`, its
own immediate sibling in the same file, completely unguarded for an entire session until this
follow-up review caught it.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found by direct review of `camera.rs` during task #174's `renderer` crate scout, immediately after re-reading BUG-174's own fix in the same file and noticing its sibling setter was never touched. Root cause: `projection_matrix_set` assigned its argument directly to `self.projection_matrix` with no validation, structurally bypassing every one of BUG-174's scalar-input guards for any caller not going through `Camera::new`. Confirmed reachable via this crate's own bundled `gltf_viewer` example (canvas-resize handler, the only real call site workspace-wide). Fixed by adding finite-component and invertibility checks to the setter itself, converting it to `Result< (), WebglError >`; call site updated via `.expect(...)` matching its neighboring convention, no redundant zero-size guard added since `canvas_size()` already floors to a minimum of 1. Verified via 3 new native unit tests (2 confirmed fail pre-fix / pass post-fix via temporary revert-and-rerun) plus the full scoped suite and clean clippy. Filed as BUG-253, not BUG-246, after discovering the concurrent session actor had already claimed TASK-246/247/248 and BUG-249/250/252 by the time of filing -- verified via direct directory scan (`task/`, `task/bug/`), not the `highest_id` marker alone, which had gone stale. Closed same-session (Tier 2 Dual-Role Self-Check). |
