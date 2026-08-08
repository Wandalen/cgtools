# Layer: L1 GPU Hardware Abstraction

The keystone layer — one API over all three drivers, so everything above is
written once per stack instead of once per backend. The *contract* is
decided; the *implementation strategy* is not: the slot is reserved by the
blank crate `gpu_hal`, gated on
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

Reserved: `module/blank/gpu_hal` holds the name and the slot. Building it
(or making it a thin facade over `wgpu`) waits on the exploration's spike
results. Until then, L3 engines keep their accepted direct-to-L0
dependencies.

### Layers

| File | Relationship |
|------|--------------|
| [001_l0_drivers.md](001_l0_drivers.md) | The only layer L1 may depend on |
| [003_l2_frame_orchestration.md](003_l2_frame_orchestration.md) | The first layer that should consume L1 |

### Explorations

| File | Relationship |
|------|--------------|
| [../explorations/001_gpu_hal_buy_vs_build.md](../explorations/001_gpu_hal_buy_vs_build.md) | The open decision gating this layer's implementation |

### Sources

| File | Relationship |
|------|--------------|
| `module/blank/gpu_hal/` | The reserved crate slot |
