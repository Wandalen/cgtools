# BUG-276: `texture::render_target_2d` panics on a zero-sized render target instead of failing cleanly

- **Severity:** Medium (an undocumented panic on an ordinary, reachable event -- sizing a render
  target to a live window/canvas that is transiently minimized or not yet laid out -- not data
  corruption, matching BUG-165's own severity for the identical defect class)
- **state:** Completed
- **Affects:** `texture::render_target_2d` -- any caller that creates a render target sized to a
  live window or canvas, including a transient `(0, height)`/`(width, 0)` size
- **Component:** `module/min/minwgpu` (`src/texture.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`texture::render_target_2d( device, size, format )` forwards `size` straight to
`wgpu::Device::create_texture` with zero validation. When either component of `size` is `0`,
`wgpu-core`'s texture-creation validation rejects it with
`CreateTextureError::InvalidDimension(TextureDimensionError::Zero(..))`, which surfaces through
`wgpu`'s default "uncaptured error" handler -- and by default that handler panics
(`panic!("wgpu error: {err}")`, confirmed by reading `wgpu-core-30.0.0`'s
`backend/wgpu_core.rs:692` `default_error_handler` directly), since this crate never installs a
custom `on_uncaptured_error` handler anywhere. The caller gets an unrecoverable process panic
instead of a clean, recoverable failure.

## Impact

**Who is affected:** any caller of `minwgpu::texture::render_target_2d` that sizes a render
target to match a live window or canvas -- the exact scenario BUG-165 already identified for
`surface::surface_configure`'s `size` parameter. A minimized or not-yet-laid-out window/canvas
can legitimately report a `0` dimension with no malformed caller input at all.

**What breaks:** the whole process panics the moment `render_target_2d` is called with a
zero-sized dimension. The caller has no way to catch and recover via the crate's own
`Result`-based error type, since `render_target_2d`'s pre-fix signature (`-> Texture`, infallible)
gave no signal this could fail at all.

**Entity Scope:** None -- a code-level defect.

## How Discovered

During this session's file-by-file review of `minwgpu`'s helper/pipeline/pass/readback/surface/
texture layer (this fork's assigned file list), the fork cross-checked `task/bug/completed/` for
prior findings touching the same files and found BUG-165 (`surface_configure`, same crate) and
BUG-176 (`gpu_hal::Device::texture_create`, a sibling crate) both already fixed the identical
defect class -- an unguarded `wgpu`/`gpu_hal` texture/surface creation call panicking on
zero-sized input. That raised the question of whether `texture.rs`'s own `render_target_2d`, a
third call path taking the same shape of caller-supplied `( u32, u32 )` size, had received the
same guard. Direct inspection confirmed it forwarded `size` to `device.create_texture` with no
validation at all. Independently confirmed via direct reads of `wgpu-core-30.0.0`'s `resource.rs`
(`TextureDimensionError::Zero` variant) and `wgpu-30.0.0`'s `backend/wgpu_core.rs`
(`default_error_handler`'s `panic!("wgpu error: {err}")`) that this call path panics by the exact
mechanism BUG-165 diagnosed, not merely a hypothetical extrapolation.

## Minimum Reproducible Example

```rust
// module/min/minwgpu/src/texture.rs -- pre-fix, render_target_2d, derived from source analysis
// ( no live GPU device available in this crate's own test harness -- see Verification below )
let texture = minwgpu::texture::render_target_2d( &device, ( 0, 600 ), wgpu::TextureFormat::Rgba8Unorm );
// panics via wgpu-core's default uncaptured-error handler:
// panic!( "wgpu error: {err}" ) -- wgpu-30.0.0/src/backend/wgpu_core.rs:692, default_error_handler
// err == CreateTextureError::InvalidDimension( TextureDimensionError::Zero( .. ) )
```

**Verify Command** (<=3 lines, standalone):
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p minwgpu --all-features is_nonzero_size
```

## Root Cause

```rust
// before -- src/texture.rs, render_target_2d
pub fn render_target_2d( device : &wgpu::Device, size : ( u32, u32 ), format : wgpu::TextureFormat ) -> Texture
{
  let ( width, height ) = size;
  let extend = wgpu::Extent3d { width, height, depth_or_array_layers : 1 };
  let texture = device.create_texture( &wgpu::TextureDescriptor { size : extend, /* .. */ } );
  // ..
}
```
No validation existed between the caller-supplied `size` and `device.create_texture`.
`wgpu-core`'s own texture-creation validation (`wgpu-core-30.0.0/src/resource.rs`,
`CreateTextureError::InvalidDimension(TextureDimensionError::Zero(..))`) rejects any zero-sized
`Extent3d` component, and that rejection is surfaced through `wgpu`'s "uncaptured error"
mechanism -- which, absent a custom handler (`Device::on_uncaptured_error`) or an active error
scope (`Device::push_error_scope`), defaults to `panic!("wgpu error: {err}")`
(`wgpu-30.0.0/src/backend/wgpu_core.rs:692`, `default_error_handler`). `minwgpu` never installs a
custom handler anywhere in the crate, so this default panic path is exactly what a caller hits.

## Why Not Caught

BUG-165's fix (2026-08-16) added the identical guard to `surface::surface_configure`'s `size`
parameter, and BUG-176's fix (same prior session) added it to `gpu_hal::Device::texture_create`'s
`desc.size` -- but neither swept sibling call paths taking the same shape of input elsewhere in
the workspace. `texture::render_target_2d` takes the same `( u32, u32 )` size directly from the
caller with no shared validation chokepoint connecting it to either of those two fixes, so it was
never touched by either pass. No test called `render_target_2d` (or any precondition check
backing it) with a zero size prior to this fix.

## Fix Applied (2026-08-17)

**`src/texture.rs`:** added a new pure `is_nonzero_size( size : ( u32, u32 ) ) -> bool`
precondition check (`#[doc(hidden)]`, exposed via `mod_interface!` for testability, mirroring
`surface::validate_size`'s "split out for unit-testability" pattern), and `render_target_2d` now
`assert!`s on it before calling `device.create_texture`, panicking immediately with a clear,
crate-authored message ("render_target_2d: width and height must both be non-zero, got WxH --
wgpu::Device::create_texture panics on a zero-sized Extent3d") instead of letting the caller hit
`wgpu-core`'s opaque validation panic several layers down.

**Scope note:** this fix is a fail-fast improvement (clear, immediate, crate-attributed panic),
not full recoverable-`Result` treatment matching BUG-165/176's own precedent
(`Result<Texture, crate::Error>` returning a new error variant shaped like `ZeroSizeSurface`).
Achieving that would require adding a new variant to `crate::Error` in `src/error.rs`, a file
outside this fork's assigned scope in this session's parallel multi-fork review (owned by a
concurrently-running sibling fork covering `bind.rs`/`buffer.rs`/`context.rs`/`error.rs`/
`lib.rs`) -- see Generalized Version below for the recommended follow-up.

**`tests/texture_test.rs`** (new file): `is_nonzero_size_rejects_zero_width_or_height` and
`is_nonzero_size_accepts_nonzero_width_and_height` pin the pure precondition logic.
`tests/readme.md`'s Responsibility Table updated with the new file's row.

## Verification

`longrun`-detached, from repo root:
- `cargo test -p minwgpu --all-features`: pre-fix (source reverted to HEAD's pre-fix content via
  a manual scratchpad file-swap, **not** `git stash` -- see process note below -- while the new
  test file stayed in place): fails to compile, `error[E0432]: unresolved import
  minwgpu::texture::is_nonzero_size` (the symbol doesn't exist pre-fix), exit 101 -- a real,
  expected failure. Post-fix (restored from the scratchpad backup, genuine recompilation
  confirmed via a fresh `Compiling minwgpu` log line): 40 passed / 0 failed across all 6 test
  binaries + lib unittests + doctests (`buffer_test` 12, `context_test` 12, `helper_test` 2,
  `readback_test` 7, `surface_test` 5, `texture_test` 2 -- including both new tests -- lib
  unittests 0, doctests 0), exit 0.
- `cargo clippy -p minwgpu --all-targets --all-features -- -D warnings`: clean, exit 0.
- **Not directly tested:** `render_target_2d`'s own `assert!` call site (the actual fail-fast
  behavior replacing `wgpu`'s panic) needs a live `wgpu::Device`, which this crate's test suite
  deliberately never constructs -- every file in `tests/` uses `wgpu::Backends::empty()`
  specifically for host-independent, deterministic execution (see each file's own module doc
  comment). Only the pure `is_nonzero_size` predicate backing it is unit-tested, matching this
  crate's established convention that GPU-dependent behavior has no native test story here (the
  same is already true of `surface_configure` itself -- see `surface_test.rs`'s own module doc
  comment).

**Process note (a hazard in the verification *technique*, not a defect in the fix):** the first
revert-and-rerun attempt used `git stash push -- src/texture.rs` per this session's usual
technique, but with 13 other forks concurrently running their own `git stash` operations in this
same shared working tree, the process-global stash stack is not fork-safe -- a concurrent fork's
own bare `git stash` operation appears to have popped this fork's entry before its "reverted"
test run actually executed (confirmed: the "reverted" run's test binary hash was byte-for-byte
identical to the pre-revert run's, meaning no recompilation occurred and the fix was still live
throughout that run). Re-done safely via a manual file-content swap through the session
scratchpad directory instead (`cp` to/from a `-texture_rs_fixed_backup.rs` /
`-texture_rs_prefix_head.rs` pair, both outside git entirely), which produced the genuine
failing-then-passing proof recorded above.

## Generalized Version

**Broken assumption:** "fixing one unguarded call path into a `wgpu`/`gpu_hal` API that panics on
zero-sized input protects every call path with the same shape of vulnerability." BUG-165
(`surface_configure`) and BUG-176 (`gpu_hal::texture_create`) each fixed one call path;
`render_target_2d` -- a third, independent call path accepting the identical `( u32, u32 )` size
shape -- was covered by neither and remained exposed until this review. A defect class discovered
once in a shared-shape API surface (any function taking a raw `( width, height )` and forwarding
it to a `wgpu`/GPU-backend allocation call) should prompt a sweep of every sibling call path with
the same input shape, not just a fix at the one site where it was first noticed. **Follow-up
recommended, out of this fix's scope:** add a `ZeroSizeTexture( u32, u32 )`-shaped variant to
`crate::Error` in `src/error.rs` and upgrade `render_target_2d` to `Result<Texture, crate::Error>`
for full recoverable-error parity with `surface_configure`.

**Secondary lesson (verification technique):** `git stash` is a single, process-global stack
shared by the entire working tree -- when multiple autonomous agents operate concurrently in the
same non-worktree-isolated repo directory, a bare (unindexed) `git stash push`/`pop` from any one
of them can silently interact with another's in-flight stash entry. A revert-and-rerun proof run
under these conditions must use a manual, git-independent backup (a file copy outside version
control, as done here) rather than a bare `push`/`pop` assumed to be operating on a stack no one
else touches.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during this fork's file-by-file review of `minwgpu`'s helper/pipeline/pass/readback/surface/texture layer, after cross-checking `task/bug/completed/` and noticing BUG-165 (`surface_configure`) and BUG-176 (`gpu_hal::texture_create`) both already fixed the identical zero-size-panics-via-wgpu's-default-uncaptured-error-handler defect class via separate call paths that never covered `render_target_2d`. Root cause confirmed via direct reads of `wgpu-core-30.0.0/src/resource.rs` and `wgpu-30.0.0/src/backend/wgpu_core.rs`. Fixed with a fail-fast `assert!` plus a new testable `is_nonzero_size` precondition helper, scoped to stay within `src/texture.rs` + `tests/` since full `Result`-based treatment would require a `crate::Error` addition in `src/error.rs`, outside this fork's assigned file list in this session's 14-way parallel review (flagged as a recommended follow-up, not performed). **ID collision note:** originally drafted as BUG-273 (the on-disk-scan-computed next-free ID at fix time), but before filing, inspection of concurrently-running sibling forks' in-progress `git stash` entries revealed BUG-273 was simultaneously claimed by another fork's fix in `module/min/mingl/src/model/obj.rs` (`num_faces_compute`) -- confirmed post-hoc when two other forks independently filed `273_report_obj_model_num_faces_zero_for_triangulated_meshes.md` and `273_storage_texture_binding_layout_default_format_not_storage_capable.md`. Renumbered to 276 pre-emptively (skipping 274/275, also observed claimed by other in-flight forks) before ever filing; all source comments, test annotations, and readme rows updated accordingly. Recorded here per this session's established renumbering-and-note precedent (see `task/readme.md`'s own ID-namespace-collision history). |
