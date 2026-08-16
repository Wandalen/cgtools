# BUG-199: `Device::new_native` performs no validation on zero width/height

- **Severity:** High (panics the whole process on entirely ordinary caller input)
- **state:** Completed
- **Affects:** Every caller of `gpu_hal::Device::new_native` on the native backend --
  `renderer::webgpu::GpuContext::new_native`, `tilemap_renderer`'s native adapter
  (`gpu_state_build`), and the crate's own test fixtures.
- **Component:** `module/helper/gpu_hal` (`src/device.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** Third occurrence of the same unguarded-zero-size-`Extent3d`-panic defect
  class in this crate: BUG-165 (`minwgpu::Surface::configure`, sibling crate) and BUG-176
  (`gpu_hal::Device::texture_create`, this crate, this file) both fixed this session.
  `new_native` is a third, independent call site into the same `wgpu::Device::create_texture`
  sink, missed by BUG-176's fix because it lives in a different function.

## Symptom

```rust
// pre-fix -- gpu_hal/src/device.rs, Device::new_native
pub fn new_native( width : u32, height : u32 ) -> Result< ( Device, Queue, Surface ), Error >
{
  let context = minwgpu::context::Context::builder()
  .instance_make()
  .adapter_request()?
  .context_finish()?;
  let device = context.device_get().clone();
  let queue = context.queue_get().clone();
  let format = TextureFormat::Rgba8Unorm;
  let texture = device.create_texture( &wgpu::TextureDescriptor
  {
    label : Some( "gpu_hal offscreen surface" ),
    size : wgpu::Extent3d { width, height, depth_or_array_layers : 1 },
    // ...
  } );
  // `width`/`height` reach `create_texture` with zero validation
}
```

`width`/`height` are plain public `u32` parameters, forwarded straight into
`wgpu::Extent3d` with no validation anywhere between the caller and
`wgpu::Device::create_texture`.

## Impact

**Who is affected:** Every native-backend caller of `new_native` -- most directly
`renderer::webgpu::GpuContext::new_native` (`module/helper/renderer/src/webgpu/context.rs:92-94`)
and `tilemap_renderer`'s native adapter (`module/helper/tilemap_renderer/src/adapters/
native.rs:259`, `gpu_state_build`, shared by both `new` and `resize`). Both already propagate
`Result` via `?`/`.map_err(...)?`, so a caller-side size of `(0, h)` or `(w, 0)` -- e.g. a resize
event firing with a not-yet-laid-out or minimized window -- reaches `new_native` with no guard
between the resize event and the panic.

**What breaks:** `wgpu::Device::create_texture` panics outright on a zero-component `Extent3d`,
taking down the entire process (native backend has no error-recovery boundary around a panic,
unlike a `Result::Err` which every real call site already handles cleanly).

**Magnitude:** Single chokepoint -- every native-backend `new_native` call site was exposed
identically.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Surfaced by task #121's dedicated review pass of `gpu_hal` (this session), as the third of three
zero-size-validation gaps in this file: `texture_create` was already fixed as BUG-176, but
`new_native`'s own separate `create_texture` call (building the offscreen surface's backing
texture, not a caller-supplied `TextureDesc`) was missed by that fix since it lives in a
different function with its own independent construction of `wgpu::Extent3d`.

## Minimum Reproducible Example

```rust
// module/helper/gpu_hal/tests/native_backend_test.rs -- pre-fix, this panics the test process
let _ = Device::new_native( 0, 64 );
// panics inside wgpu::Device::create_texture -- Extent3d width must be > 0
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/gpu_hal && cargo nextest run --all-features --test native_backend_test -E 'test(new_native_rejects)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `new_native` never validates `width`/`height` before constructing the offscreen surface texture, so a zero component panics the same way BUG-176's pre-fix `texture_create` did. | ✅ Root Cause | Confirmed by reading the function directly: `width`/`height` flow unchecked into `wgpu::Extent3d`, the identical construct whose zero-size case panics per BUG-165/BUG-176's own established evidence. | E1, E2 |
| H2 | Every real call site already guarantees a nonzero size before calling `new_native`, making this defect unreachable in practice. | ❌ Falsified | Both real call sites (`renderer::GpuContext::new_native`, `tilemap_renderer::gpu_state_build`) forward a caller-supplied `width`/`height` straight through with no guard of their own -- `gpu_state_build` is shared by `new` *and* `resize`, and a resize event can legitimately fire with a transiently-zero dimension. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/gpu_hal/src/device.rs` (pre-fix, `new_native`) | No validation of `width`/`height` anywhere before `device.create_texture(&wgpu::TextureDescriptor{ size: wgpu::Extent3d{ width, height, .. }, .. })`. | H1 ✅ |
| E2 | BUG-176's own report, `## Root Cause` | Documents the identical `wgpu::Extent3d` zero-size panic contract, already confirmed against upstream `wgpu` behavior for this same crate's `texture_create`. | H1 ✅ |
| E3 | `module/helper/tilemap_renderer/src/adapters/native.rs:257-259` (`gpu_state_build`) | Doc comment states this function is "shared by `new` and `resize`, since nothing in `GpuState` survives a resize" -- confirming `width`/`height` here can originate from a live resize event, not just initial construction. | H2 ❌ |

## Root Cause

```rust
// before -- width/height reach create_texture with zero validation
pub fn new_native( width : u32, height : u32 ) -> Result< ( Device, Queue, Surface ), Error >
{
  // ... no guard ...
  let texture = device.create_texture( &wgpu::TextureDescriptor
  {
    size : wgpu::Extent3d { width, height, depth_or_array_layers : 1 },
    // ...
  } );
}
```

No validation existed between the caller and `wgpu::Device::create_texture`, so the same
zero-size `Extent3d` panic already fixed once in this file (BUG-176, a different function) was
still reachable through this second, independent construction site.

## Why Not Caught

`new_native` had no test exercising a zero `width`/`height` prior to this bug -- every existing
call site in `native_backend_test.rs` passes a hardcoded nonzero size (`64, 64` or similar).

## Fix Location

`module/helper/gpu_hal/src/device.rs`, `Device::new_native`: added a guard rejecting a zero
`width` or `height` with the existing `Error::InvalidInput` variant (added by BUG-176, reused
here rather than introducing a new variant), before constructing the offscreen surface texture.

This is a purely additive fix -- `new_native`'s signature (`Result<(Device, Queue, Surface),
Error>`) is unchanged, since the function was already fallible. A workspace-wide audit of both
real call sites (`renderer/src/webgpu/context.rs:94`, `tilemap_renderer/src/adapters/
native.rs:259`) confirmed both already `?`/`.map_err(...)?`-propagate `Result` -- neither
required modification; the fix strictly improves both sites' behavior (clean `Err` instead of a
process panic) with zero regression risk.

## Prevention

2 new tests added, `module/helper/gpu_hal/tests/native_backend_test.rs`:
`new_native_rejects_zero_width`, `new_native_rejects_zero_height` (each asserts
`Err(Error::InvalidInput(_))`).

## Pitfall

The same backend call (`wgpu::Device::create_texture` with a caller-derived `Extent3d`) can
appear at more than one independent call site within a single file -- fixing the defect at one
call site (BUG-176's `texture_create`) does not fix a structurally identical but textually
separate call site (`new_native`) reachable through a different code path. Each construction of
a validation-sensitive type must be checked individually, not assumed covered because a sibling
function already got the same fix.

## Generalized Version

**Broken assumption:** "this class of bug was already fixed in this file, so it can't recur
here."

**Confirmed general rule:** A defect class fixed at one call site of a shared, panic-prone
downstream API does not protect a second, independently-constructed call site of the same API
elsewhere in the same file -- each construction point needs its own explicit guard, since the
fix lives at the call site, not inside the downstream function itself.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Surfaced by task #121's dedicated `gpu_hal` review pass as the third zero-size-validation gap in this file, alongside the already-fixed BUG-176. |
| 2026-08-16 | fixed | Added a zero-component guard to `new_native`, reusing BUG-176's `Error::InvalidInput` variant; 2 new regression tests added; workspace-wide call-site audit confirmed zero downstream changes needed. |
| 2026-08-16 | verified | Empirical fail-then-pass: guard temporarily reverted, `cargo nextest run --all-features --test native_backend_test -E 'test(new_native_rejects)'` confirmed both new tests FAIL (2/2 failed) against the pre-fix code; guard restored, same command confirmed both PASS (2/2 passed, 7 skipped). `cargo check --target wasm32-unknown-unknown --no-default-features --features "enabled,webgl" -p gpu_hal`: clean (unaffected by this native-only fix, run as part of the adjacent BUG-200 work in the same session). |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote the fail-then-pass empirical check (guard reverted -> both tests FAIL; guard restored -> both PASS). Adversarial pass re-read the restored `new_native` source in full to confirm the reverted comment block left no stray syntax or duplicated guard behind. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-165 and BUG-176 -- correctly identified as the same defect class at a third, independent call site, not a duplicate report (disjoint function, same file). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct source read of `new_native` plus BUG-176's own already-confirmed `wgpu::Extent3d` panic contract for this same crate -- not re-derived from scratch, correctly reused as precedent. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is a single validation guard reusing an existing `Error` variant; no unrelated refactor attempted. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Fix lives in `gpu_hal` (the actual defect site); both real call sites in `renderer`/`tilemap_renderer` audited and confirmed to need no changes, consistent with BUG-176's own precedent for this exact call chain. | — |

**Reproduced:** YES -- pre-fix (guard reverted), both `new_native_rejects_zero_width` and
`new_native_rejects_zero_height` FAIL (test process panics inside `wgpu::Device::create_texture`);
post-fix (guard restored), both PASS. 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/gpu_hal/src/device.rs` | `new_native`: added a zero-component guard on `width`/`height` returning `Error::InvalidInput` before constructing the offscreen surface texture (full `Fix(BUG-199)` comment block); updated the `# Errors` doc section. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/gpu_hal/tests/native_backend_test.rs` | Appended 2 tests: `new_native_rejects_zero_width`, `new_native_rejects_zero_height`. Reuses the file's existing 5-section doc-comment convention -- no new test file created. |
