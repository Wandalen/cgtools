# Layer: L0 Drivers

The bottom of every stack: one thin, backend-faithful wrapper crate per
GPU API. A driver's job is to make its backend *usable from Rust/wasm*, not
to hide it — cross-backend abstraction is exactly what L0 must not do
(that is [L1](002_l1_gpu_hal.md)'s single responsibility).

### Scope

- **Purpose**: Define the driver layer's role, contract, and current occupants.
- **Responsibility**: Record what a driver may and may not abstract, and who currently depends on L0 directly.
- **In Scope**: `minwebgl`, `minwebgpu`, `minwgpu`, and the `mingl` substrate's relationship to them.
- **Out of Scope**: The abstraction over drivers (see [002_l1_gpu_hal.md](002_l1_gpu_hal.md)); layering rules (see [../pattern/002_strict_layering_one_step_drilldown.md](../pattern/002_strict_layering_one_step_drilldown.md)).

### Role and Contract

- **Backend-faithful**: a driver exposes its backend's own concepts and
  shader language (GLSL ES for `minwebgl`, WGSL for `minwebgpu`/`minwgpu`)
  truthfully — no cross-backend vocabulary, no lowest-common-denominator API.
- **Thin**: helpers for ergonomics (context setup, buffer upload, error
  surfacing), never policy (pass scheduling, materials, scenes).
- **Terminal drill-down target**: every drill-down chain from higher layers
  bottoms out at a driver handle; there is nothing below to expose except
  the raw web/native API itself.

### Occupants

| Crate | Backend | State |
|-------|---------|-------|
| `minwebgl` | WebGL2 (web) | Mature — the workspace's primary driver |
| `minwebgpu` | WebGPU (web) | Functional |
| `minwgpu` | `wgpu` (native) | Embryonic — helper/buffer/context/texture layers exist |

**`mingl` is not a layer.** All three drivers depend on it as a shared
substrate of backend-independent helpers — it sits *below* L0, which is why
it cannot become the HAL (dependency arrow points the wrong way; ADR-001,
alternatives).

### Current Direct Consumers (pre-HAL)

[L1](002_l1_gpu_hal.md) exists as v0 and `renderer`'s canonical opaque path
routes through it; the remaining code still reaching L0 directly is:
`renderer`'s legacy `webgl` tree, `tilemap_renderer`'s WebGL2 adapter
(optional `dep:minwebgl`) — both L3 stack engines — and `line_tools`
(optional `dep:minwebgl`; straddles d2/d3, stack classification pending,
see [rulebook.md](../../rulebook.md#rendering-layer-placement)). These are
the accepted violations named in
[../pattern/002](../pattern/002_strict_layering_one_step_drilldown.md),
scheduled to strangle onto L1.

### Non-Stack Tooling Consumers

Not every L0 consumer is stack code awaiting HAL migration.
`shader_chunks_render_core` (`dep:minwgpu`) and `shader_chunks_preview_web`
(`dep:minwebgpu`) render individual WGSL shader chunks in isolation —
headless and browser-side respectively — as authoring/preview tooling, not
as part of any d2/tile/d3 stack. Single-backend access is intentional here:
the tooling's job is to exercise one exact chunk against one exact backend,
not to portray a stack-vocabulary scene across backends. These are **not**
scheduled to migrate onto L1 — see
[rulebook.md](../../rulebook.md#rendering-layer-placement)'s "beside the
ladder" list.

### Layers

| File | Relationship |
|------|--------------|
| [002_l1_gpu_hal.md](002_l1_gpu_hal.md) | The only layer that should depend on L0 once it exists |

### Patterns

| File | Relationship |
|------|--------------|
| [../pattern/003_cross_stack_bridge_via_foundation_resources.md](../pattern/003_cross_stack_bridge_via_foundation_resources.md) | Foundation resources crossing a stack boundary are driver-level handles (textures, buffers) at this layer |

### Sources

| File | Relationship |
|------|--------------|
| `module/min/mingl/` | Shared substrate below the drivers |
| `module/min/minwebgl/` | WebGL2 driver |
| `module/min/minwebgpu/` | WebGPU driver |
| `module/min/minwgpu/` | Native `wgpu` driver (embryonic) |
