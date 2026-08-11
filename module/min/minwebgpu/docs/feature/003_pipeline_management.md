# Feature: Pipeline Management

### Scope

- **Purpose**: Configure and create render and compute pipelines through descriptor builders that mirror the WebGPU spec's own dictionary shape.
- **Responsibility**: Document the render/compute pipeline builder API's design.
- **In Scope**: Render pipeline and compute pipeline descriptor builders and their nested state objects.
- **Out of Scope**: Bind group / pipeline layout construction (see `feature/004`) and command/render-pass recording (see `feature/005`).

### Design

Render and compute pipelines are each built through a nested descriptor-builder chain: a render pipeline builder composes a vertex state, an optional fragment state (color targets, blending), a primitive state (topology, culling), an optional depth/stencil state, and a multisample state, each with its own builder under `state/`; a `GpuPipelineLayout` binds the pipeline to its resource layouts (see `feature/004`). Compute pipelines use the same programmable-stage/pipeline-layout shape but skip the render-only state. Both builders terminate in a `GpuDevice`-scoped creation call that converts failures into `WebGPUError::DeviceError::FailedToCreateRenderPipeline`/`FailedToCreateComputePipeline` (see `invariant/001`). This is the heaviest user of the crate's nested descriptor-builder pattern (see `pattern/001`), and the area the crate's abstraction-overhead target (see `non_functional_requirement/001`) most directly bounds, since pipeline (re)configuration is the most likely per-frame builder use.

**Note**: this feature covers pipeline *creation* only. A `ComputePipeline` built here currently has no wrapped execution path — `feature/005` does not yet implement compute pass recording/dispatch, so a created compute pipeline cannot presently be run through this crate's own API.

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_facade_over_descriptor_builders.md](../pattern/001_facade_over_descriptor_builders.md) | Heaviest user of the nested descriptor-builder pattern |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_result_based_error_handling.md](../invariant/001_result_based_error_handling.md) | All fallible functions here return `Result<_, WebGPUError>` |

### Non Functional Requirements

| File | Relationship |
|------|--------------|
| [non_functional_requirement/001_minimal_abstraction_overhead.md](../non_functional_requirement/001_minimal_abstraction_overhead.md) | This feature's nested builders are the primary source of the overhead this target bounds |

### Sources

| File | Relationship |
|------|--------------|
| `src/render_pipeline.rs` | Render pipeline creation |
| `src/compute_pipeline.rs` | Compute pipeline creation |
| `src/descriptor/render_pipeline.rs` | Render pipeline descriptor builder |
| `src/descriptor/compute_pipeline.rs` | Compute pipeline descriptor builder |
| `src/state/` | Vertex, fragment, primitive, blend, color-target, depth-stencil, multisample, programmable-stage, stencil-face state builders |

### Tests

No automated tests exist for this crate at the time of this migration.
