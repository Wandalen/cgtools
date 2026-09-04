# Invariant: PBR Metallic-Roughness Baseline

Every mesh renders through one material model — glTF-style physically-based
metallic-roughness. There is no fixed-function path, no unlit default, no
per-mesh shading model switch: extensions refine the baseline, they never
replace it.

### Scope

- **Purpose**: Pin the d3 stack's material baseline — the shading contract all content shares.
- **Responsibility**: State the single-model rule, its enforcement in the shader/material architecture, and the cost of bypassing it.
- **In Scope**: The material model applied to scene meshes and its fixed capacity limits.
- **Out of Scope**: How lighting energy reaches materials from environments (see [../feature/002_image_based_lighting.md](../feature/002_image_based_lighting.md)); the numeric range lighting is computed in (see [003_hdr_internal_tone_mapped_output.md](003_hdr_internal_tone_mapped_output.md)).

### Invariant Statement

Every mesh drawn by `Renderer` is shaded by the PBR metallic-roughness
material model as loaded from glTF (base color, metallic, roughness, normal,
occlusion, emission — extended by the KHR material extensions the crate
supports). Analytic lighting is bounded by fixed capacities: at most 8 point,
8 directional, and 8 spot lights per draw.

### Enforcement Mechanism

- **One main shader pair**: `src/webgl/material/pbr.rs` compiles
  `shaders/main.vert` / `shaders/main.frag` (embedded via `include_str!`) —
  the single program family through which scene geometry renders; its
  uniform surface (camera, node, skeleton, material, lights) is the
  crate-wide shading vocabulary.
- **Capacity as constants**: `MAX_POINT_LIGHTS`, `MAX_DIRECT_LIGHTS`,
  `MAX_SPOT_LIGHTS` (all 8) are compile-time constants in
  `src/webgl/material/pbr.rs`, sized into the shader — not a runtime
  configuration.
- **Loader mapping**: the glTF loader maps every imported material into this
  model — there is no alternative shading path for imported content to land
  on.

### Violation Consequences

- Content needing non-PBR looks (stylized, unlit, CAD-style) must either
  fake it through PBR parameters (e.g. emission-only) or step outside the
  invariant with a custom material — losing the IBL and shadow integration
  the baseline provides for free.
- Scenes exceeding a light capacity do not degrade gracefully — lights past
  the fixed array size simply cannot be uploaded; scene design must respect
  the limits.

### Features

| File | Relationship |
|------|--------------|
| [../feature/001_pbr_rendering_core.md](../feature/001_pbr_rendering_core.md) | The pipeline that drives this material model every frame |
| [../feature/002_image_based_lighting.md](../feature/002_image_based_lighting.md) | Environment lighting feeding the same BRDF |

### Sources

| File | Relationship |
|------|--------------|
| `src/webgl/loaders/gltf.rs` | Maps imported glTF materials onto the baseline model |
| `src/webgl/material/pbr.rs` | The main shader pair, its uniform surface, and the light-capacity constants |
| `src/webgl/shaders/main.frag` | The BRDF implementation itself |

### Tests

| File | Relationship |
|------|--------------|
| `tests/tests.rs` | Aggregates the crate's test modules exercising material and loader behavior |
