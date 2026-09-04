# BUG-176: `Device::texture_create` performs no validation on zero-sized dimensions

- **Severity:** High (native backend panics the whole process; WebGL backend silently returns
  `Ok` for a texture that was never actually allocated, corrupting later draws with no error
  signal at all)
- **state:** Completed
- **Affects:** Every caller of `gpu_hal::Device::texture_create` across all 3 backends --
  `renderer`'s WebGPU frame-target/dummy-texture creation, `tilemap_renderer`'s WebGPU and
  native adapters, and any future caller that can be handed a zero dimension (e.g. from a
  transiently-zero-sized live canvas).
- **Component:** `module/helper/gpu_hal` (`src/device.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** Pre-identified by task #98's review pass under the working title "renderer
  webgpu: zero width/height unguarded texture creation." Same defect class (unguarded zero-size
  descriptor reaching a backend that panics on it) as BUG-165 (`minwgpu::surface_configure`),
  fixed earlier this session -- that fix did not cover `texture_create`, a separate call path in
  a sibling crate.

## Symptom

```rust
// pre-fix -- gpu_hal/src/device.rs, Device::texture_create
pub fn texture_create( &self, desc : &TextureDesc ) -> Result< Texture, Error >
{
  match self
  {
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    Self::WebGpu( device ) => { /* desc.size forwarded to gl::texture::desc().size(...) unchecked */ }
    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    Self::WebGl( context ) => { /* desc.size forwarded to tex_storage_2d(...) unchecked */ }
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    Self::Native( device ) => { /* desc.size forwarded to wgpu::Extent3d unchecked */ }
  }
}
```

`desc.size : [u32; 3]` reaches all 3 backend-specific texture-creation calls with zero
validation. A zero on any component behaves differently -- and wrongly -- on every backend.

## Impact

**Who is affected:** Every caller of `texture_create`, most directly `renderer`'s
`frame_targets_create` (`src/webgpu/renderer.rs:122-148`), which builds the HDR (`Rgba16Float`)
and depth (`Depth24Plus`) render targets from `device.size()` -- a value that, on wasm32, reads
`self.canvas.width()`/`.height()` live off the DOM. A hidden tab or a canvas not yet laid out can
legitimately report `0` for either dimension with no malformed caller input at all.

**What breaks, per backend:**
- **Native:** `wgpu::Device::create_texture` panics outright on a zero-component `Extent3d` --
  the same class of defect already fixed for `Surface::configure` in BUG-165, now confirmed to
  also exist in the separate `texture_create` path.
- **WebGL:** `context.tex_storage_2d(...)` raises `INVALID_VALUE` on the WebGL error flag for a
  zero dimension, but that flag is never checked anywhere in this function -- so `texture_create`
  returns `Ok` claiming success despite allocating nothing. This is the same "WebGL errors aren't
  surfaced as `Result::Err`" pattern already documented for BUG-160.
  Every subsequent draw against the returned `Texture::WebGl` then operates on effectively
  undefined GPU state, with no error at the point of failure.
- **WebGPU:** the descriptor-validation error from a zero-sized `builder.create(device)?` is
  surfaced as `Err`, the least-bad of the three outcomes -- but still uncharacterized/untested
  prior to this fix.

**Magnitude:** Every `texture_create` call site across the workspace was exposed identically,
since the defect is in the single shared chokepoint, not any one caller.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Pre-identified by task #98's review pass (this session) as "renderer webgpu: zero width/height
unguarded texture creation." Traced from the originally-named symptom site
(`renderer::WebGpuRenderer::new` -> `frame_targets_create` -> `device.texture_create`) down
through `GpuContext::size()`'s live-canvas-dimension source, to the actual shared root cause:
`gpu_hal::Device::texture_create` itself, a sibling crate to `renderer` and the single
chokepoint feeding all 3 backends.

## Minimum Reproducible Example

```rust
// module/helper/gpu_hal/tests/native_backend_test.rs -- pre-fix, this panics the test process
let ( device, _queue, _surface ) = Device::new_native( 64, 64 ).unwrap();
let _ = device.texture_create( &TextureDesc
{
  size : [ 0, 64, 1 ], // zero width
  format : TextureFormat::Rgba8Unorm,
  usage : TextureUsage::TEXTURE_BINDING
} );
// panics inside wgpu::Device::create_texture -- Extent3d width must be > 0
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/gpu_hal && cargo nextest run -p gpu_hal --features native texture_create_rejects_zero_width
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `texture_create` never validates `desc.size` before dispatching to any backend, and each backend fails differently -- native panics, WebGL silently no-ops into a false `Ok`, WebGPU raises an uncaught validation error. | ✅ Root Cause | Confirmed by reading all 3 backend match arms directly: none checks `desc.size` for zero components; the native arm's `wgpu::Extent3d` construction is a direct pass-through into a call documented upstream to panic on a zero dimension. | E1, E2, E3 |
| H2 | Zero dimensions are already prevented upstream (e.g. by `GpuContext::size()` or by every real caller passing a hardcoded nonzero size), making this defect unreachable in practice. | ❌ Falsified | `GpuContext::size()` on wasm32 returns live `canvas.width()`/`.height()`, which can legitimately be `0` for a hidden or not-yet-laid-out canvas -- no caller-side guard exists between that read and `frame_targets_create`'s `texture_create` calls. | E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/gpu_hal/src/device.rs` (pre-fix, `texture_create`) | No validation of `desc.size` anywhere before the 3-way backend `match`. | H1 ✅ |
| E2 | `module/helper/gpu_hal/src/device.rs`, WebGl arm (`context.tex_storage_2d(...)`) | Return value/error state never checked after the call -- the function unconditionally returns `Ok`. | H1 ✅ |
| E3 | `module/helper/gpu_hal/src/device.rs`, Native arm (`device.create_texture(&wgpu::TextureDescriptor{ size: wgpu::Extent3d{...}, .. })`) | Direct pass-through of `desc.size` into `wgpu::Extent3d`, the same construct whose zero-size case already panicked `Surface::configure` pre-BUG-165. | H1 ✅ |
| E4 | `module/helper/renderer/src/webgpu/context.rs:108-121` (`GpuContext::size`) | wasm32 arm reads `self.canvas.width()`/`.height()` live off the DOM every call -- no caching, no lower-bound guard. | H2 ❌ |

## Root Cause

```rust
// before -- desc.size reaches every backend with zero validation
pub fn texture_create( &self, desc : &TextureDesc ) -> Result< Texture, Error >
{
  match self { /* 3 backend arms, none checking desc.size */ }
}
```

No validation existed between the caller and any of the three backend-specific texture-creation
calls, so each backend's own, differing failure behavior on a zero-sized descriptor (panic /
silent-`Ok` / uncaught validation error) was the only thing standing between a legitimately
reachable zero-sized live-canvas read and undefined behavior.

## Why Not Caught

`texture_create` had no test exercising a zero-sized dimension on any backend prior to this bug --
`device_creation`, `triangle_render_readback`, and `texture_write_readback` (the crate's
pre-existing native tests) all use fixed, realistic nonzero sizes.

## Fix Location

`module/helper/gpu_hal/src/device.rs`, `Device::texture_create`: added a guard rejecting any
zero component of `desc.size` with a new `Error::InvalidInput` variant, before the backend
`match` dispatch. `module/helper/gpu_hal/src/error.rs`: added the `InvalidInput(String)` variant
itself (matching BUG-162's precedent of adding a new `Error` variant when no existing one fits a
genuinely new failure class) plus its `Display` arm.

This is a purely additive fix -- `texture_create`'s signature (`Result<Texture, Error>`) is
unchanged, since the function was already fallible. A workspace-wide audit of all 7 real call
sites (`renderer/src/webgpu/renderer.rs:125,136,156`; `tilemap_renderer/src/adapters/
webgpu.rs:253`; `tilemap_renderer/src/adapters/native.rs:167`; `gpu_hal/tests/
native_backend_test.rs:212` plus the 4 new test call sites) confirmed every existing site already
`?`/propagates `Result` and passes either a hardcoded nonzero literal or a live decoded-image/
canvas dimension -- none required modification; the fix strictly improves every site's behavior
(clean `Err` instead of panic or silent corruption) with zero regression risk.

## Prevention

4 new tests added, `module/helper/gpu_hal/tests/native_backend_test.rs`:
`texture_create_rejects_zero_width`, `texture_create_rejects_zero_height`,
`texture_create_rejects_zero_depth_or_array_layers` (each asserts `Err(Error::InvalidInput(_))`
for a `TextureDesc` with exactly one zero component), and
`texture_create_accepts_well_formed_size` (confirms the fix doesn't reject valid input).

## Pitfall

A shared backend-dispatch chokepoint (`Device::texture_create`, forwarding one descriptor to 3
independently-implemented backend arms) needs its input validated once, before dispatch --
validating in only one arm, or relying on each backend's own native error behavior, leaves the
other backends exposed to whatever that backend happens to do with invalid input by default
(panic, silent corruption, or a proper error, unpredictably different per backend). The function's
own pre-fix doc comment ("the native backend never fails this call") was itself misleading: true
only in the narrow "never returns `Err`" sense, false in the sense that mattered -- it panics.

## Generalized Version

**Broken assumption:** "if the function signature is already `Result`-returning, every failure
mode is necessarily surfaced as `Err`."

**Confirmed general rule:** A `Result`-returning function can still panic (native backends
often trust their input is pre-validated) or still silently succeed on failure (WebGL's
error-flag model is not exception-based) -- the return type alone says nothing about a
particular backend's actual failure behavior for a given invalid input. Each backend arm behind
a shared dispatch point must be checked individually against its own upstream documentation, not
assumed uniform because they share a `Result` return type.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Pre-identified by task #98's review pass; root cause traced this session from the originally-named `renderer` symptom site down to the shared `gpu_hal::Device::texture_create` chokepoint. |
| 2026-08-16 | fixed | Added a zero-component guard plus new `Error::InvalidInput` variant in `gpu_hal`; 4 new regression tests added; workspace-wide call-site audit confirmed zero downstream changes needed. |
| 2026-08-16 | verified | `cargo check -p gpu_hal --tests --all-features`: clean (27.06s), run first to catch any compile error in the new test file before the full chain. Full workspace `cargo check --workspace --all-targets --all-features --exclude flecs_bouncing_circles`: clean (55.00s). `cargo nextest run --workspace --all-features --exclude flecs_bouncing_circles`: 1889/1889 passed, 0 skipped (includes all 4 new tests individually confirmed PASS, plus the 3 pre-existing `gpu_hal` native tests unaffected). `cargo test --doc --workspace --all-features --exclude flecs_bouncing_circles`: all crates `test result: ok`, 0 failed. `cargo clippy --workspace --all-targets --all-features --exclude flecs_bouncing_circles -- -D warnings`: clean (35.70s), zero warnings. `--exclude flecs_bouncing_circles` scopes out an unrelated example binary mid-edit by another concurrent actor in this shared workspace (confirmed via `git status`/mtime: its `main.rs` was modified at 10:00:25, after this verification run started, with 2 compile errors unrelated to `gpu_hal`/`renderer`/`tilemap_renderer` -- a stale-API-under-refactor call-site mismatch, not a regression from this fix). |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote the 4 differential tests against real backend behavior. Adversarial pass re-checked the verification log for stale-file risk (per BUG-175's own near-miss lesson): a concurrent actor was found launching its own `longrun` jobs in the same repo root concurrently with this one, causing bare `longrun .wait` auto-discovery to land on a *different* job's log file. Caught by cross-checking each log's own launch-prologue PID against the PID this session's own `.launch` call actually reported, then re-polling with explicit `log::`/`pid::` against the correct file (`-0093_longrun.log`, pid 3316269) before trusting its content. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-165 (same zero-size-`Extent3d`-panic defect class, different call path: `Surface::configure` vs. `texture_create`) and BUG-160 (same "WebGL error flag never checked" pattern) -- both correctly cited as related defect classes, not duplicates (disjoint code paths, no overlap with this fix). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct source read of all 3 backend arms plus their own upstream documentation (`wgpu::Extent3d`'s panic contract, WebGL's `INVALID_VALUE` error-flag model), not inferred from the diff's shape alone. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is a single validation guard plus one new `Error` variant; no unrelated refactor attempted. `gpu_hal`'s own dedicated review (task #121) remains separately pending -- this fix is scoped narrowly to the one named defect, not a broader audit. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Fix lives in `gpu_hal` (the actual shared chokepoint), not `renderer` (the task title's crate) -- justified by this session's own established precedent (BUG-053/080/120/174) of crossing crate boundaries when the root cause lives in a shared dependency, and by the Anti-Duplication Principle (guarding 3 call sites in `renderer.rs` separately would duplicate the same check 3 times instead of fixing the shared function once). | — |
| D7 | Crate Locality | 🟢 | 🟢 | Confirmed via grep that `Device::texture_create` has exactly one definition site, already fixed; all 7 real call sites workspace-wide audited and confirmed to need no changes. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | The fix corrects the function's existing responsibility (create a texture matching the given descriptor, failing cleanly on an invalid one); no responsibility added or removed. | — |

**Reproduced:** YES -- pre-fix, the equivalent of `texture_create_rejects_zero_width` panics the
test process instead of returning `Err`; post-fix, all 4 new tests pass. Full workspace suite
(1889/1889, 0 skipped, +4 new), doctests (0 failed across every crate), and clippy all clean,
2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/gpu_hal/src/device.rs` | `texture_create`: added a zero-component guard on `desc.size` returning `Error::InvalidInput` before the backend `match` dispatch (full `Fix(BUG-176)` comment block); updated the `# Errors` doc section. |
| `module/helper/gpu_hal/src/error.rs` | Added new `InvalidInput(String)` `Error` variant plus its `Display` arm. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/gpu_hal/tests/native_backend_test.rs` | Appended 4 tests: `texture_create_rejects_zero_width`, `texture_create_rejects_zero_height`, `texture_create_rejects_zero_depth_or_array_layers`, `texture_create_accepts_well_formed_size`. Reuses the file's existing `Device::new_native(width, height)` fixture pattern -- no new test file created. |
