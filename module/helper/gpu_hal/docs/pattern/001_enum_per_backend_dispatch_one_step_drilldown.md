# Pattern: Enum-Per-Backend Dispatch with One-Step Drill-Down

### Scope

- **Purpose**: Explain why every `gpu_hal` handle type is a backend-tagged enum with matching accessor methods, rather than a trait object or a type generic over backend.
- **Responsibility**: Document the crate's core architectural approach, applicable to every module.
- **In Scope**: The enum-per-backend shape, the public non-panicking drill-down accessors, and the internal panicking dispatch accessors.
- **Out of Scope**: Per-feature API details (see the `feature/` instances); the crate's error type hierarchy (see `invariant/001`).

### Problem

WebGPU, WebGL, and native `wgpu` expose genuinely different underlying objects for the same HAL concept (`web_sys::GpuBuffer` vs. a crate-local `BufferWebGl` vs. `wgpu::Buffer`), with different capabilities across backends (WebGL requires GLSL shader source; only native can read pixels back). A trait-object design would force every method down to the lowest common denominator all three backends support, or grow an unwieldy set of associated types; a design generic over a backend type parameter would monomorphize the whole call graph per backend and leak a backend type parameter into every caller's function signatures — directly against the "shader-access, not stack-vocabulary" contract `docs/layer/002_l1_gpu_hal.md` sets for this layer.

### Solution

Every resource and handle type — `Device`, `Queue`, `Surface`, `Buffer`, `Texture`, `TextureView`, `Sampler`, `ShaderModule`, `BindGroupLayout`, `BindGroup`, `RenderPipeline`, `CommandEncoder`, `RenderPass` — is a plain enum with one variant per backend, each variant `#[cfg(all(feature = "...", target_arch = "..."))]`-gated so only the compiled-in backends' variants exist at all (see `pitfall/001`). Every method body is a `match self { ... }` over these variants — verbose by construction, but exhaustively checked by the compiler on every backend combination that's actually compiled.

Two accessor families exist side by side, with deliberately different failure behavior:

- **Public, non-panicking**: `pub fn as_webgpu(&self) -> Option<&Raw>` / `as_webgl` / `as_native`, present on every handle type, returning `None` on a backend mismatch rather than panicking — the one-step escape hatch to the raw driver object for anything the portable surface doesn't cover.
- **Internal, panicking**: `pub(crate) fn expect_webgpu(&self) -> &Raw` / `expect_webgl` / `expect_native`, used only inside `gpu_hal`'s own per-backend match arms so internal dispatch code can assume "this handle matches the device's active backend" without re-threading a `Result`. A caller can never observe a mismatched-backend handle through the public API, so `expect_*`'s panic (`"backend mismatch : expected a WebGPU <thing>"`) is unreachable except by a bug in the crate's own dispatch (see `invariant/001`).

This directly instantiates the workspace's one-step-drill-down pattern (`docs/pattern/002_strict_layering_one_step_drilldown.md`) at the resource-handle level: each layer hands the caller the layer below rather than sealing it away, reachable in exactly one step from any handle.

### Applicability

Applies to every public handle type `gpu_hal` exposes today, and to any future one — a new resource wrapper should follow the same enum-per-backend-variant-plus-`as_*`/`expect_*`-pair shape. The `expect_*` family must stay `pub(crate)`; it is not part of the public contract and exists solely to keep internal dispatch code terse.

### Consequences

Callers get one flat, backend-agnostic API surface plus a one-step escape hatch to the raw driver object; adding a fourth backend means adding one `#[cfg]`-gated variant to every enum and one arm to every match — mechanical, and the compiler's exhaustiveness check catches anything missed — rather than restructuring a trait hierarchy. The tradeoff, shared with the workspace pattern this instantiates, is verbosity: every method on every handle type is a per-backend match arm (see `device.rs`, `resource.rs`, `pass.rs`), so the crate's line count scales with `backends × operations` rather than `operations` alone.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_backend_construction_and_device_acquisition.md](../feature/001_backend_construction_and_device_acquisition.md) | `Device`/`Queue`/`Surface` are the first enums a caller constructs |
| [feature/002_resource_creation.md](../feature/002_resource_creation.md) | `Buffer`/`Texture`/`TextureView`/`Sampler` follow this pattern |
| [feature/003_shader_modules_and_render_pipelines.md](../feature/003_shader_modules_and_render_pipelines.md) | `ShaderModule`/`RenderPipeline` follow this pattern |
| [feature/004_bind_groups_and_layouts.md](../feature/004_bind_groups_and_layouts.md) | `BindGroupLayout`/`BindGroup` follow this pattern |
| [feature/005_command_recording_and_submission.md](../feature/005_command_recording_and_submission.md) | `CommandEncoder`/`RenderPass` follow this pattern |
| [feature/006_native_pixel_readback.md](../feature/006_native_pixel_readback.md) | `texture_rgba8_read` is the raw-`wgpu`-typed function this pattern's drill-down reaches |

### Pitfalls

| File | Relationship |
|------|--------------|
| [pitfall/001_backend_availability_compile_time_not_runtime.md](../pitfall/001_backend_availability_compile_time_not_runtime.md) | The `#[cfg]`-gated variant mechanism this pattern relies on is exactly what that pitfall is a consequence of |

### Cross-References

| File | Relationship |
|------|--------------|
| [../../../../../docs/pattern/002_strict_layering_one_step_drilldown.md](../../../../../docs/pattern/002_strict_layering_one_step_drilldown.md) | The workspace-level ancestor pattern this one instantiates at the resource-handle level |
| [../../../../../docs/adr/002_gpu_hal_in_house.md](../../../../../docs/adr/002_gpu_hal_in_house.md) | The decision (build in-house over adopting `wgpu` wholesale as the HAL) this pattern implements |

### Sources

| File | Relationship |
|------|--------------|
| `src/device.rs` | `Device`/`Queue`/`Surface` enums and their `as_*`/`expect_*` pairs |
| `src/resource.rs` | Every resource handle enum and its `as_*`/`expect_*` pairs |
| `src/pass.rs` | `CommandEncoder`/`RenderPass` enums and their `as_*`/`expect_*` pairs |

### Tests

`tests/native_backend_test.rs` exercises the enum surface throughout but never calls `as_native()`/`expect_native()` directly, since native is the only backend compiled into that test's feature set — the drill-down accessors themselves have no dedicated test.
