# Layer: L1 GPU Hardware Abstraction

The keystone layer — one API over all drivers, so everything above is
written once per stack instead of once per backend. The *contract* is
decided and an in-house *v0 implementation* exists in `gpu_hal` ( WebGPU +
WebGL2 + native wgpu backends, plus a fourth `wgpu`-free native Vulkan
backend — see Status ); build-vs-buy is closed in-house by
[../adr/002_gpu_hal_in_house.md](../adr/002_gpu_hal_in_house.md).

### Scope

- **Purpose**: Define the HAL layer's contract so that any implementation (in-house or `wgpu`-backed facade) can be checked against it.
- **Responsibility**: Record what L1 must provide, what it must not contain, and its current reservation state.
- **In Scope**: The layer's contract and status.
- **Out of Scope**: The build-vs-buy decision itself (see [ADR-002](../adr/002_gpu_hal_in_house.md)); driver behavior (see [001_l0_drivers.md](001_l0_drivers.md)).

### Contract

- **WebGPU-shaped**: API mirrors WebGPU concepts (device, queue, pipeline,
  bind group) so the WebGPU path is near-zero-cost and the WebGL2 path
  emulates only what it must.
- **Shader access, not shader hiding**: canonical shader source in WGSL,
  transpiled per backend at build time, with a per-backend override slot for
  hand-tuned sources — never a fixed pipeline that owns the shaders.
- **One-step drill-down**: every HAL object exposes a handle to its raw
  driver counterpart ([../pattern/002](../pattern/002_strict_layering_one_step_drilldown.md)).
- **Stack-vocabulary-free**: no sprite, tile, camera, scene, or material
  concepts — those belong to stacks above (variance rule,
  [../pattern/001](../pattern/001_invariant_defined_stack.md)); L1 serves
  every stack equally.

### Status

v0 implemented in `module/helper/gpu_hal` — the exploration's spike extracted
it from the webgl-vs-webgpu diff of `renderer`'s canonical opaque path, which
now builds against it on both browser backends ( `webgpu` / `webgl`
features ). `gpu_hal`'s own webgpu/webgl backends are now
browser-pixel-verified too ( proven by the `triangle_browser` example via
`browsee` — task 191 ); `renderer`'s own opaque-path browser-side pixel tests
are now covered the same way ( proven by the `opaque_path_browser` example via
`browsee` — task 197 ). A third, native backend ( `native`
feature, `minwgpu` + raw `wgpu` ) renders into an
offscreen texture with pixel readback, and is proven by an in-repo render
test ( `triangle_render_readback` ) that draws through the full public
surface and asserts on the bytes read back — no browser involved. The
renderer's canonical opaque path itself also runs on this backend
( renderer feature `native`, `GpuContext::new_native` ) and is
pixel-verified end-to-end in the terminal by `opaque_path_renders_lit_quad`. The v0 surface
covers the opaque path only: buffers, 2d textures, samplers, shader modules,
bind groups, one-color-attachment render passes, and a depth attachment
( `DepthState`, honored by all backends ). Texture upload is now
covered too ( `texture_write()`, proven by the `texture_write_readback`
render test — task 089 ). Not yet covered: mipmaps, MSAA, compute.
`renderer`'s legacy `webgl` tree keeps its accepted direct-to-L0 dependency
until strangled onto the HAL. `tilemap_renderer` (d2) is the second targeted
consumer — its `adapter-webgpu` / `adapter-native` adopt the HAL per
[../adr/003_d2_stack_hal_adoption.md](../adr/003_d2_stack_hal_adoption.md).
`adapter-webgpu` now builds and passes its own compile-and-construct-level
test suite, and is also browser-pixel-verified via the `adapter_browser`
example and `browsee` ( task 198 ) — proving a real, correctly-bounded opaque
**black** quad, its own documented unpopulated-texture behavior, not the
clear color, at the sprite's exact configured location.
`adapter-native` now also builds and passes an in-repo pixel-readback test
suite mirroring `gpu_hal`'s own `triangle_render_readback` precedent, proving
the offscreen-render-plus-readback path with no browser involved. Its existing
`adapter-webgl` keeps its direct `minwebgl` dependency for now, on the same
accepted-until-strangled posture — it now also has its own
compile-and-construct-level test suite (`webgl_backend_test.rs` +
`command_consistency_test.rs`, task 114) and is browser-pixel-verified too, via
the same `adapter_browser` example ( task 198 ) — proving a real solid-red
sprite paint, unlike `adapter-webgpu`'s unpopulated-texture black, since
`adapter-webgl` uploads real pixel bytes — the same shape of coverage as
`adapter-webgpu`'s, without adopting the HAL itself.

A fourth backend, `vulkan` ( `minvulkan` via `ash`, no `wgpu` dependency ),
is now implemented — [ADR-004](../adr/004_native_vulkan_hal_backend.md) adds
it so `examples/orrery/flexible` can offer a Vulkan option that does not link
`wgpu`, distinct from `native`'s existing `wgpu`-picks-its-own-backend
behavior. Same v0 opaque-path surface as `native` ( buffers, 2d textures,
samplers, shader modules — WGSL compiled to SPIR-V via `naga` — bind groups,
one-color-attachment render passes ), proven the same way : an in-repo
offscreen-render-plus-readback test ( `triangle_render_readback` in
`tests/vulkan_backend_test.rs`, task 202 ) draws through the full public
surface and asserts on the bytes read back, no browser involved, mirroring
`native`'s own `triangle_render_readback` precedent. Resources use dedicated
( non-suballocated ) memory — one `vkAllocateMemory` call per buffer/image,
matching the crate's v0 "minimum resource support" tradeoff elsewhere.
Tracked by tasks 201 ( `minvulkan` driver ), 202 ( this crate's `vulkan`
backend variant ), 203 ( the consuming example ).

### ADRs

| File | Relationship |
|------|--------------|
| [../adr/002_gpu_hal_in_house.md](../adr/002_gpu_hal_in_house.md) | Build-vs-buy decision — closed in-house; `gpu_hal` is the L1 HAL |
| [../adr/003_d2_stack_hal_adoption.md](../adr/003_d2_stack_hal_adoption.md) | Extends L1 adoption to the d2 stack ( `tilemap_renderer` ) |
| [../adr/004_native_vulkan_hal_backend.md](../adr/004_native_vulkan_hal_backend.md) | Adds a fourth, `wgpu`-free `vulkan` backend via `minvulkan` — implemented ( task 202 ) |

### Explorations

| File | Relationship |
|------|--------------|
| [../explorations/001_gpu_hal_buy_vs_build.md](../explorations/001_gpu_hal_buy_vs_build.md) | The comparison and spike evidence behind ADR-002 ( closed ) |

### Layers

| File | Relationship |
|------|--------------|
| [001_l0_drivers.md](001_l0_drivers.md) | The only layer L1 may depend on |
| [003_l2_frame_orchestration.md](003_l2_frame_orchestration.md) | The first layer that should consume L1 |

### Patterns

| File | Relationship |
|------|--------------|
| [../pattern/003_cross_stack_bridge_via_foundation_resources.md](../pattern/003_cross_stack_bridge_via_foundation_resources.md) | The HAL is the backend-agnostic layer a cross-stack bridge crate composes foundation resources through |

### Sources

| File | Relationship |
|------|--------------|
| `module/helper/gpu_hal/` | The v0 implementation |
| `module/helper/renderer/src/webgpu/` | First consumer — the canonical opaque path on both backends |
| `module/helper/tilemap_renderer/src/adapters/webgpu.rs`, `src/adapters/native.rs` | Second targeted consumer — `adapter-webgpu` / `adapter-native` ( [ADR-003](../adr/003_d2_stack_hal_adoption.md) ) |
