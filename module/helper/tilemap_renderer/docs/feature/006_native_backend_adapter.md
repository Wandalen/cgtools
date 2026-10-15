# Feature: Native Backend Adapter

`adapters::NativeBackend` implements the core `Backend` trait over `gpu_hal`'s offscreen native `wgpu` surface, behind the `adapter-native` feature — routing through the L1 HAL the same way [`WebGpuBackend`](005_webgpu_backend_adapter.md) does ([ADR-003](../../../../../docs/adr/003_d2_stack_hal_adoption.md)), but off-screen rather than into a browser canvas.

### Scope

- **Purpose**: Let a command stream drive real GPU rendering outside a browser (native `wgpu`), returning raw pixel bytes rather than presenting to any surface.
- **Responsibility**: Cross-reference the native adapter's source, its readback-based output contract, and its pixel-verified test coverage.
- **In Scope**: Sprite rendering through one shared textured-quad pipeline; offscreen construction, resize, and pixel readback.
- **Out of Scope**: The browser WebGPU counterpart (see [005_webgpu_backend_adapter.md](005_webgpu_backend_adapter.md), which shares the same minimal command family but presents to a canvas instead of reading back pixels); SVG/WebGL2/terminal/none adapters (see [001](001_svg_backend_adapter.md), [002](002_webgl2_backend_adapter.md), [003](003_terminal_backend_adapter.md), [004](004_none_backend_adapter.md)).

### Design

`NativeBackend::new` builds an offscreen surface sized to `RenderConfig` via the unmodified `gpu_hal::Device::new_native`, then a shared sprite pipeline (`SPRITE_WGSL`) — the same shape of pipeline [`WebGpuBackend`](005_webgpu_backend_adapter.md) builds, but with vertex positions and UVs arriving already transformed from the CPU (`Transform::to_mat3` plus a pixel-to-NDC projection), so the shader itself carries no matrix math at all.

Like [`WebGpuBackend`](005_webgpu_backend_adapter.md), this adapter draws a leading `RenderCommand::Clear` plus `RenderCommand::Sprite` only; every other command family returns `RenderError::Unsupported`, keeping `capabilities()` (sprites-only) honest.

Unlike the WebGPU adapter, `gpu_hal`'s native surface *does* expose a pixel-upload call (`Queue::texture_write`), so `assets_load` here uploads real image content — every `PixelFormat` (`Rgba8`/`Rgb8`/`Gray8`/`GrayAlpha8`) is expanded into tightly-packed RGBA8 bytes (`to_rgba8`) before upload, since RGBA8 is the only texture format this backend's v0 `gpu_hal` surface accepts. This closes the pixel-upload gap that keeps the WebGPU adapter partial (see [005](005_webgpu_backend_adapter.md)'s Design section), and is why this adapter's tests can assert exact rendered pixel colors rather than only pipeline/logic correctness.

`output()` reads back the offscreen surface into an `Output::Bitmap` (`gpu_hal::Surface::pixels_read`) — there is no on-screen presentation, in contrast to the WebGPU adapter's `Output::Presented`. `resize()` rebuilds every GPU handle from scratch (`gpu_state_build`) and clears loaded images/sprites, because the offscreen surface has no in-place resize — nothing in `GpuState` survives one. A `pipeline_build` doc comment records one cross-adapter constraint worth preserving if the bind-group layout is ever touched: binding order (uniform, then texture, then sampler) keeps the texture entry immediately before the sampler entry, which is load-bearing for the WebGL backend's own sampler-pairing convention (paired with the nearest preceding texture entry — see `gpu_hal`'s own native-backend test).

Given real pixel upload, a fully-honest narrow `Capabilities` surface, and exact-byte pixel-verified tests, status is tracked as complete (✅) for its declared (Sprite-only) scope.

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_y_up_coordinate_system.md](../invariant/001_y_up_coordinate_system.md) | Satisfied natively for `Transform` placement (world-space Y maps directly into clip space, same as WebGL2/WebGPU) — and additionally flips the sampled row for UV only, since row 0 of the uploaded image bytes is the image's top row while `ly = 0.5` is the top of a Y-up world-space quad (`quad_vertices`) |

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_ports_and_adapters_backend_architecture.md](../pattern/001_ports_and_adapters_backend_architecture.md) | This adapter is one `Backend` implementation within the crate's hexagonal architecture |

### Sources

| File | Relationship |
|------|--------------|
| `src/adapters/native.rs` | `NativeBackend`, its shared sprite pipeline, offscreen readback, and the `to_rgba8`/`f32_bytes` upload helpers |

### Tests

| File | Relationship |
|------|--------------|
| `tests/native_backend_test.rs` | Real-GPU pixel-readback contract tests (needs a live `gpu_hal` native device — a software Vulkan ICD such as lavapipe suffices), mirroring `gpu_hal`'s own `triangle_render_readback` style: constructed `Bitmap` dimensions match the configured viewport, sprite-center and clear-color-corner pixels match their configured colors exactly (ruling out an all-clear false pass), and `resize` followed by a fresh render reflects the new dimensions |
