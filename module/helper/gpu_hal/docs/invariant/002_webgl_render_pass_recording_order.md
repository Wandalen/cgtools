# Invariant: WebGL Render-Pass Recording Order

### Scope

- **Purpose**: Guarantee that a caller who orders `RenderPass` calls per this invariant gets consistent behavior on WebGPU and WebGL, since WebGL resolves bindings eagerly against the active pipeline's introspected maps rather than deferring resolution to validation.
- **Responsibility**: Document the call-order contract on `RenderPass` recording methods and why WebGL depends on it.
- **In Scope**: The `pipeline_set`-before-`bind_group_set`/`vertex_buffer_set` ordering requirement.
- **Out of Scope**: Bind-group *entry* order within a single `bind_group_layout_create`/`bind_group_create` call (see `invariant/003`).

### Invariant Statement

Within one render pass, `pipeline_set()` must be called before `bind_group_set()` and `vertex_buffer_set()`. WebGPU tolerates other orderings "in spirit" per the source's own wording, but WebGL's implementation actively depends on the order: both `bind_group_set` and `vertex_buffer_set` resolve their targets through the currently-active pipeline's introspected GLSL binding maps, which do not exist until `pipeline_set` has run.

### Enforcement Mechanism

Not compiler-enforced — this is a documented call-order contract, not a type-state machine that would reject an out-of-order call at compile time. `pass.rs`'s own doc comment on the `RenderPass` recording methods states it explicitly: "The WebGL backend applies state eagerly, which imposes one ordering requirement WebGPU shares in spirit: `pipeline_set` must precede `bind_group_set` and `vertex_buffer_set`, as both resolve through the active pipeline's introspected binding maps." Every call site in the crate's own test suite follows `pipeline_set → bind_group_set → vertex_buffer_set → index_buffer_set → draw_indexed → end`, in that order.

### Violation Consequences

Calling `bind_group_set` or `vertex_buffer_set` before `pipeline_set` on the WebGL backend resolves against no (or a stale) introspected binding map — an incorrect draw, not a caught error, since none of these methods' signatures can detect the ordering violation at the type level.

### Features

| File | Relationship |
|------|--------------|
| [feature/005_command_recording_and_submission.md](../feature/005_command_recording_and_submission.md) | `RenderPass`'s recording methods are exactly what this invariant constrains |

### Sources

| File | Relationship |
|------|--------------|
| `src/pass.rs` | `RenderPass` recording-method doc comment stating the ordering requirement |

### Tests

`tests/native_backend_test.rs`'s three tests all follow the required order, but none currently runs against the WebGL backend specifically (the crate's only automated tests build under `--features native`), so no test yet demonstrates the WebGL-specific failure mode this invariant protects against.
