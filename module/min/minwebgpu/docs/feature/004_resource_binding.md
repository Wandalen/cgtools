# Feature: Resource Binding

### Scope

- **Purpose**: Construct bind group layouts and bind groups so shaders can access buffers, textures, and samplers.
- **Responsibility**: Document the resource-binding API's design.
- **In Scope**: Bind group layout description and bind group instance construction.
- **Out of Scope**: Pipeline construction that consumes the resulting `GpuPipelineLayout` (see `feature/003`).

### Design

Resource binding is split into layout description and instance binding: the layout builders describe a `GpuBindGroupLayout`'s binding-slot types and shader stages (buffer, sampler, texture, storage-texture, external-texture — see `binding_type/`), while the bind-group builders bind concrete resources (a `GpuBuffer`, `GpuSampler`, or `GpuTextureView`) to those slots to produce a `GpuBindGroup`. A separate layout builder combines one or more bind group layouts into a `GpuPipelineLayout` consumed by pipeline creation (see `feature/003`). The library performs no automatic shader reflection — callers must define every layout explicitly to match their WGSL shader's `@group`/`@binding` declarations.

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_facade_over_descriptor_builders.md](../pattern/001_facade_over_descriptor_builders.md) | Bind group construction follows the crate's descriptor-plus-explicit-device shape |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_result_based_error_handling.md](../invariant/001_result_based_error_handling.md) | All fallible functions here return `Result<_, WebGPUError>` |

### Sources

| File | Relationship |
|------|--------------|
| `src/bind_group.rs` | `GpuBindGroup` construction |
| `src/bind_group_entry.rs`, `src/bind_group_entry/` | Bind group entry construction (buffer binding, binding resource) |
| `src/binding_type.rs`, `src/binding_type/` | Buffer, sampler, texture, storage-texture, external-texture binding types |
| `src/layout/bind_group.rs` | `GpuBindGroupLayout` builder |
| `src/layout/pipeline.rs` | `GpuPipelineLayout` builder |
| `src/descriptor/bind_group.rs`, `src/descriptor/bind_group_layout.rs`, `src/descriptor/bind_group_layout_entry.rs` | Descriptor builders backing the above |

### Tests

No automated tests exist for this crate at the time of this migration.
