# Layer: L1 GPU Hardware Abstraction

The keystone layer — one API over all three drivers, so everything above is
written once per stack instead of once per backend. The *contract* is
decided and an in-house *v0 implementation* exists in `gpu_hal` ( WebGPU +
WebGL2 backends ); formally closing the build-vs-buy decision as an ADR
still waits on
[../explorations/001_gpu_hal_buy_vs_build.md](../explorations/001_gpu_hal_buy_vs_build.md).

### Scope

- **Purpose**: Define the HAL layer's contract so that any implementation (in-house or `wgpu`-backed facade) can be checked against it.
- **Responsibility**: Record what L1 must provide, what it must not contain, and its current reservation state.
- **In Scope**: The layer's contract and status.
- **Out of Scope**: The build-vs-buy decision itself (see the exploration); driver behavior (see [001_l0_drivers.md](001_l0_drivers.md)).

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

v0 implemented in `module/blank/gpu_hal` — the exploration's spike extracted
it from the webgl-vs-webgpu diff of `renderer`'s canonical opaque path, which
now builds against it on both backends ( `webgpu` / `webgl` features;
runtime smoke tests still to run ). The v0
surface covers that path only: buffers, 2d textures, samplers, shader
modules, bind groups, one-color-attachment render passes. Not yet covered:
texture upload, mipmaps, MSAA, compute, the `wgpu`-native backend.
`renderer`'s legacy `webgl` tree and the other L3 engines keep their accepted
direct-to-L0 dependencies until strangled onto the HAL.

### Layers

| File | Relationship |
|------|--------------|
| [001_l0_drivers.md](001_l0_drivers.md) | The only layer L1 may depend on |
| [003_l2_frame_orchestration.md](003_l2_frame_orchestration.md) | The first layer that should consume L1 |

### Explorations

| File | Relationship |
|------|--------------|
| [../explorations/001_gpu_hal_buy_vs_build.md](../explorations/001_gpu_hal_buy_vs_build.md) | Build-vs-buy decision — spike delivered the in-house v0; formal closure ( ADR ) pending |

### Sources

| File | Relationship |
|------|--------------|
| `module/blank/gpu_hal/` | The v0 implementation |
| `module/helper/renderer/src/webgpu/` | First consumer — the canonical opaque path on both backends |
