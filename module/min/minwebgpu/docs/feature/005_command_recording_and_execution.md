# Feature: Command Recording & Execution

### Scope

- **Purpose**: Record render passes and submit encoded work to the GPU queue.
- **Responsibility**: Document the command-encoding and submission API's design.
- **In Scope**: Render pass recording, command buffer finishing, and queue submission.
- **Out of Scope**: Pipeline and bind-group setup consumed during a pass (see `feature/003`, `feature/004`); compute pass recording, which this crate does not currently implement (see below).

### Design

A `GpuCommandEncoder`, acquired from the device, begins a render pass via a descriptor built from color attachments and an optional depth/stencil attachment. Inside the pass, callers set the active pipeline, vertex/index buffers, and bind groups, then issue draw calls before ending the pass. Finishing the encoder yields a `GpuCommandBuffer`; the queue module provides the submission call plus a convenience path for writing data directly into an existing buffer without a full command-buffer round-trip.

**Gap versus the original specification**: the pre-migration specification's Public Contract (FR-6.4/FR-6.5) called for a wrapped `GpuComputePassEncoder` — begin a compute pass, set pipeline/bind groups, `dispatch_workgroups()`. No such wrapper exists anywhere in `src/` (verified: no `ComputePass`/`compute_pass`/`dispatch_workgroups` hits crate-wide). `feature/003`'s `ComputePipeline` can be created, but this crate currently provides no way to actually record and dispatch it — only render passes are wired end-to-end.

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_result_based_error_handling.md](../invariant/001_result_based_error_handling.md) | All fallible functions here return `Result<_, WebGPUError>` |

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_facade_over_descriptor_builders.md](../pattern/001_facade_over_descriptor_builders.md) | Render pass recording follows the crate's descriptor-plus-explicit-device shape |

### Sources

| File | Relationship |
|------|--------------|
| `src/render_pass.rs` | Render pass recording |
| `src/render_pass/color_attachment.rs` | Color attachment construction |
| `src/render_pass/depth_stencil_attachment.rs` | Depth/stencil attachment construction |
| `src/descriptor/render_pass.rs` | Render pass descriptor builder |
| `src/queue.rs` | Command buffer submission, direct buffer writes |

### Tests

No automated tests exist for this crate at the time of this migration.
