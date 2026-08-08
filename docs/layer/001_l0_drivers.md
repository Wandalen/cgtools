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

Because L1–L2 do not exist yet, L3 code reaches L0 directly: `renderer`,
`tilemap_renderer`'s WebGL2 adapter (optional `dep:minwebgl`), and
`line_tools`. These are the accepted violations named in
[../pattern/002](../pattern/002_strict_layering_one_step_drilldown.md),
scheduled to route through [L1](002_l1_gpu_hal.md) once it exists.

### Layers

| File | Relationship |
|------|--------------|
| [002_l1_gpu_hal.md](002_l1_gpu_hal.md) | The only layer that should depend on L0 once it exists |

### Sources

| File | Relationship |
|------|--------------|
| `module/min/mingl/` | Shared substrate below the drivers |
| `module/min/minwebgl/` | WebGL2 driver |
| `module/min/minwebgpu/` | WebGPU driver |
| `module/min/minwgpu/` | Native `wgpu` driver (embryonic) |
