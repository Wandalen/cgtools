# Pattern: Facade Over Descriptor Builders

### Scope

- **Purpose**: Explain why `minwebgpu` wraps `web-sys`'s raw WebGPU bindings behind a safe, stateless, builder-based facade.
- **Responsibility**: Document the crate's core architectural approach, applicable to every module.
- **In Scope**: Facade layering, descriptor-builder configuration style, and stateless explicit-argument design.
- **Out of Scope**: Per-feature API details (see the `feature/` instances) and the crate's error type hierarchy (see `invariant/001`).

### Problem

`web-sys`'s raw WebGPU bindings are verbose, `JsValue`-typed, and offer no compile-time safety; wrapping each one by hand at every call site would scatter ad hoc `JsValue` handling and duplicate boilerplate across the crate, and any retained global `Device`/`Queue` state would hide control flow from callers.

### Solution

`minwebgpu` layers a safe Rust facade over `web-sys`: every fallible operation returns `Result<T, WebGPUError>` instead of a raw `JsValue`; complex WebGPU objects (pipelines, bind groups, buffers) are configured through descriptor-builder structs mirroring the WebGPU spec's own dictionary-based descriptors, then created via an explicit `device`/`queue` argument; the library holds no global state — `Device` and `Queue` are acquired once (see `feature/001`) and threaded explicitly into every function that needs them; and modules are organized by WebGPU concept (`buffer`, `texture`, `pipeline`, `layout`, `state`, `binding_type`, `bind_group`, `bind_group_entry`, `render_pass`, `queue`), each exposed through a `mod_interface!` layer declared in `lib.rs`. The facade does not extend to providing scene-graph, material-system, or geometry-generation helpers — those remain the caller's responsibility.

### Applicability

Applies to every public constructor and configuration path in the crate — a new WebGPU resource wrapper should follow the same descriptor-builder-plus-explicit-device shape rather than introducing ad hoc global state or `JsValue`-returning functions.

### Consequences

Callers get compile-time-checked, fluent configuration and a single consistent error type, and the absence of hidden global state keeps control flow explicit and testable. The tradeoff is verbosity: WebGPU's deeply nested descriptors (e.g. a render pipeline's vertex/fragment/primitive/depth-stencil/multisample state) require a correspondingly nested set of builder types (see `state/` and `descriptor/`), which is more types to learn than a flat or stringly-typed API would need.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_context_device_and_shader_setup.md](../feature/001_context_device_and_shader_setup.md) | Acquires the Device/Queue this pattern threads explicitly |
| [feature/002_buffer_management.md](../feature/002_buffer_management.md) | Built on this pattern |
| [feature/003_pipeline_management.md](../feature/003_pipeline_management.md) | Heaviest user of nested descriptor builders |
| [feature/004_resource_binding.md](../feature/004_resource_binding.md) | Built on this pattern |
| [feature/005_command_recording_and_execution.md](../feature/005_command_recording_and_execution.md) | Built on this pattern |

### Pitfalls

| File | Relationship |
|------|--------------|
| [pitfall/001_native_target_compiles_to_nonfunctional_stub.md](../pitfall/001_native_target_compiles_to_nonfunctional_stub.md) | The facade's native-target stub fallback sits behind this same module layering |

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | `mod_interface!` layer declarations forming the module structure |
| `src/context.rs` | Device/Queue acquisition; no retained global state |
| `src/descriptor/` | Descriptor-builder structs |
| `src/state/` | Nested pipeline state builders |

### Tests

No automated tests exist for this crate at the time of this migration.
