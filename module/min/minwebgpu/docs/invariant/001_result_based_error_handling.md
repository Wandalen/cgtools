# Invariant: Result-Based Error Handling, No Unsafe Code

### Scope

- **Purpose**: Guarantee every fallible WebGPU operation surfaces failure through a typed `Result`, never a panic or silent `JsValue`.
- **Responsibility**: Document the crate-wide error contract and how it is enforced.
- **In Scope**: The `WebGPUError` type hierarchy, the no-`unsafe`-code guarantee, and the no-panics-except-internal-bug policy.
- **Out of Scope**: Per-module error variant wording (read `src/error.rs` directly for the current, authoritative list).

### Invariant Statement

Every public, fallible `minwebgpu` function returns `Result<T, WebGPUError>`; the crate contains zero `unsafe` blocks; and the crate must not panic on invalid caller input or WebGPU API misuse — panics are reserved for unreachable internal logic bugs only.

### Enforcement Mechanism

`WebGPUError` (`src/error.rs`) is a thin top-level enum whose variants (`DomError`, `CanvasError`, `DeviceError`, `ContexError`, `TextureError`, `BufferError`) each wrap a dedicated sub-error enum via `#[from]`, all derived with the `error_tools`-based `error::typed::Error` macro — a more modular hierarchy than the single flat enum originally sketched in the pre-migration specification, letting each module (buffer, context, texture, device) own its own error variants while still converging on one crate-wide type at the public boundary. `web-sys` calls that return `Result<_, JsValue>` are mapped into the matching sub-error variant with `.map_err(...)` at the call site — see `buffer::create`/`buffer::init` for the pattern. Verified for this migration: `grep -rn "unsafe" src/` returns zero matches crate-wide.

### Violation Consequences

A function that panics or lets a raw `JsValue`/`wasm_bindgen` error escape past the public API breaks the safety contract WASM consumers rely on, and would be a regression from the crate's current, verified state.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_context_device_and_shader_setup.md](../feature/001_context_device_and_shader_setup.md) | Public functions return `Result<_, WebGPUError>` |
| [feature/002_buffer_management.md](../feature/002_buffer_management.md) | Public functions return `Result<_, WebGPUError>` |
| [feature/003_pipeline_management.md](../feature/003_pipeline_management.md) | Public functions return `Result<_, WebGPUError>` |
| [feature/004_resource_binding.md](../feature/004_resource_binding.md) | Public functions return `Result<_, WebGPUError>` |
| [feature/005_command_recording_and_execution.md](../feature/005_command_recording_and_execution.md) | Public functions return `Result<_, WebGPUError>` |

### Sources

| File | Relationship |
|------|--------------|
| `src/error.rs` | `WebGPUError` hierarchy and sub-error enums |
| `src/buffer.rs` | `JsValue` → `WebGPUError` mapping pattern |

### Tests

No automated tests exist for this crate at the time of this migration.
