# Feature: WebGPU Backend Adapter

`adapters::WebGpuBackend` implements the core `Backend` trait over `gpu_hal`'s browser WebGPU surface, behind the `adapter-webgpu` feature — the first `tilemap_renderer` adapter to route through the L1 HAL rather than a driver crate directly ([ADR-003](../../../../../docs/adr/003_d2_stack_hal_adoption.md)).

### Scope

- **Purpose**: Let a command stream drive real-time WebGPU rendering in a browser via the shared `gpu_hal` abstraction, rather than a WebGPU-specific driver dependency.
- **Responsibility**: Cross-reference the WebGPU adapter's source, its honestly-narrow `Capabilities`, and the one functional gap that keeps it partial.
- **In Scope**: Sprite rendering through one shared textured-quad pipeline; the async `gpu_hal`-backed construction path.
- **Out of Scope**: The native offscreen counterpart (see [006_native_backend_adapter.md](006_native_backend_adapter.md), which shares the same minimal command family but a different surface and output mechanism); SVG/WebGL2/terminal/none adapters (see [001](001_svg_backend_adapter.md), [002](002_webgl2_backend_adapter.md), [003](003_terminal_backend_adapter.md), [004](004_none_backend_adapter.md)).

### Design

`WebGpuBackend::new` is the crate's first async adapter constructor, because `gpu_hal::Device::new_webgpu` is itself async. It builds one shared sprite pipeline (`WGSL_SOURCE`) up front: the 2D affine transform (`Transform::to_mat3`) is embedded into a `mat4x4` (`mat3_to_mat4`) purely so every `Uniforms` member lands on a natural 16-byte WGSL alignment boundary, not because the adapter does any 3D work.

Like [`NativeBackend`](006_native_backend_adapter.md), this adapter draws `RenderCommand::Sprite` only — the minimal command family `pingpong_animation`'s compiler (task 085) produces. Every other command family returns `RenderError::Unsupported` rather than being silently skipped, so `capabilities()` never over-claims what `submit` actually translates. Three inherent methods exist specifically to make this checkable without a live device/canvas: `declared_capabilities()` (a `const fn`, distinctly named to avoid colliding with the trait method), `sprite_draw_params()` (the position/resource-id `submit` draws a sprite with), and `command_classify()` — the exact function `submit`'s loop calls, so it is the real anti-faking gate rather than a decoy that could drift from the code path it claims to cover.

**Pixel upload (task 218)**: `assets_load` now uploads real pixel data for every `ImageSource::Bitmap` image — sized from the bitmap's own `width`/`height` (no longer a hardcoded `1x1` placeholder), converted to tightly-packed RGBA8 via `crate::assets::to_rgba8`, and written through `gpu_hal::Queue::texture_write`, which has covered the WebGPU surface since task 089. This is the same shared helper and the same `Queue::texture_write` call [`NativeBackend`](006_native_backend_adapter.md) already used — no duplicated conversion logic between the two adapters. The placeholder quad (`QUAD_VERTICES`) still doubles as its own UV: the whole sheet image is drawn as one unmapped quad regardless of its real size, since sub-region/atlas addressing remains unwired (unchanged, out of this task's scope too). Live browser pixel-correctness re-verification of the new upload path — confirming what color the sprite now actually paints — is a distinct, not-yet-done follow-up; see task 251's own manual browser-verification procedure, whose recorded opaque-black reading predates this fix and needs a fresh live run.

`output()` always returns `Output::Presented` (the browser canvas itself is the destination — contrast [`NativeBackend::output`](006_native_backend_adapter.md), which reads back a `Bitmap`). `resize()` only updates the canvas dimensions and `RenderConfig`; unlike the native adapter it does not rebuild any GPU state, since the WebGPU surface tracks the canvas directly.

Given the pixel-upload gap is now closed at the source level (task 218) — verified via the shared `to_rgba8` helper's own unit tests and a source-level confirmation the `1x1` placeholder literal is gone, not yet a live browser pixel readback like [`NativeBackend`](006_native_backend_adapter.md)'s — status is tracked as complete (✅) for this adapter's declared (Sprite-only) scope; live pixel-color re-verification remains a separate, tracked follow-up (see above).

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_y_up_coordinate_system.md](../invariant/001_y_up_coordinate_system.md) | Satisfied natively for `Transform` placement — world-space Y maps directly into WGSL clip space with no adapter-side flip. Raster UV row-order is still not provably exercised: real pixel data is now uploaded (task 218 — see the Design section above), but the only asset exercised so far (`adapter_browser`'s uniform solid-red sprite, task 251) is vertically symmetric and cannot distinguish a row-order flip from no flip either way; unlike [`NativeBackend`](006_native_backend_adapter.md), this adapter still does not flip UV explicitly |

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_ports_and_adapters_backend_architecture.md](../pattern/001_ports_and_adapters_backend_architecture.md) | This adapter is one `Backend` implementation within the crate's hexagonal architecture |

### Sources

| File | Relationship |
|------|--------------|
| `src/adapters/webgpu.rs` | `WebGpuBackend`, its shared sprite pipeline, and the `declared_capabilities`/`sprite_draw_params`/`command_classify` testable-without-a-device seams |

### Tests

| File | Relationship |
|------|--------------|
| `tests/webgpu_backend_test.rs` | `wasm_bindgen_test`-gated (`adapter-webgpu` + `wasm32` only): `declared_capabilities()` matches the honest sprites-only subset, `sprite_draw_params` tracks distinct inputs rather than returning a constant, `command_classify` rejects every non-`Sprite` family including a mid-batch `Clear`. No live-device/canvas test exists yet — this workspace has no proven browser-runtime WebGPU test infrastructure, per the module's own doc comment |
