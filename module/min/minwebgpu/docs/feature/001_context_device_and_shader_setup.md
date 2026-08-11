# Feature: Context, Device & Shader Setup

### Scope

- **Purpose**: Establish a WebGPU-enabled canvas, acquire a device/queue, and compile WGSL shader modules — the setup phase every `minwebgpu` application starts from.
- **Responsibility**: Describe the initialization API surface (context acquisition plus shader compilation) and its design.
- **In Scope**: Canvas/context/adapter/device/queue acquisition and WGSL shader module creation.
- **Out of Scope**: Buffer, texture, pipeline, and bind-group creation (see `feature/002`–`feature/004`) and command submission (see `feature/005`).

### Design

`minwebgpu` retrieves or creates an `HtmlCanvasElement`, obtains a `GpuCanvasContext` from it, requests a `GpuAdapter` from the browser navigator, and requests a `GpuDevice`/`GpuQueue` pair from the adapter — then configures the canvas context with the device and the browser's preferred `GpuTextureFormat` (queried via a dedicated helper rather than hardcoded). None of these objects are retained as global state by the library (see `pattern/001`); the caller owns and threads `Device`/`Queue` through subsequent calls. Shader modules are compiled from raw WGSL source strings via a single `GpuDevice`-scoped function — the library performs no WGSL parsing, reflection, or validation of its own; compilation/validation errors surface from the browser's WebGPU implementation through `WebGPUError::DeviceError`/`ContexError` variants (see `invariant/001`).

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_facade_over_descriptor_builders.md](../pattern/001_facade_over_descriptor_builders.md) | This feature acquires the Device/Queue the pattern threads explicitly |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_result_based_error_handling.md](../invariant/001_result_based_error_handling.md) | All fallible functions here return `Result<_, WebGPUError>` |

### Pitfalls

| File | Relationship |
|------|--------------|
| [pitfall/001_native_target_compiles_to_nonfunctional_stub.md](../pitfall/001_native_target_compiles_to_nonfunctional_stub.md) | Device/adapter acquisition is the first call that surfaces the native-target stub error |

### Sources

| File | Relationship |
|------|--------------|
| `src/context.rs` | Adapter/device/queue acquisition, canvas context configuration |
| `src/canvas.rs` | Canvas element retrieval/creation |
| `src/dom.rs` | DOM helpers used during canvas/context setup |
| `src/browser.rs` | Browser-level helpers |
| `src/shader.rs` | WGSL shader module compilation |
| `src/webgpu.rs` | Re-exported `web-sys` WebGPU types and constants |

### Tests

No automated tests exist for this crate at the time of this migration.
