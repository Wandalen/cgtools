# Feature: Backend Construction & Device Acquisition

### Scope

- **Purpose**: Acquire a `Device`/`Queue`/`Surface` triple for exactly one backend — the setup phase every `gpu_hal` consumer starts from.
- **Responsibility**: Document the backend-construction API's design and the clip-space contract it hands callers.
- **In Scope**: `Device::new_webgpu`/`new_webgl`/`new_native`, `Surface`, `Device::depth_range()`.
- **Out of Scope**: Resource creation (see `feature/002`), shader/pipeline setup (see `feature/003`).

### Design

Backend selection happens once, at construction, through three separate constructors rather than one constructor plus a backend enum argument: `Device::new_webgpu(canvas)` and `Device::new_webgl(canvas)` take an `HtmlCanvasElement` and return a `(Device, Queue, Surface)` triple wrapping the browser backend; `Device::new_native(width, height)` builds an offscreen `wgpu` context of the given size and needs a Vulkan ICD on the host (a software one such as `lavapipe`/`mesa-vulkan-drivers` suffices — there is no window). Each constructor exists only when its own `#[cfg(all(feature = "...", target_arch = "..."))]` holds (see `pattern/001`); `new_webgl` additionally requires the `EXT_color_buffer_float` WebGL extension, keeping float color targets renderable on both browser backends.

`Device::depth_range()` returns a `DepthRange` (`ZeroToOne` for WebGPU and native, `NegOneToOne` for WebGL) — the clip-space depth convention the active backend's projection matrices must target. This is owned by the HAL and read at runtime rather than guessed or hardcoded by the caller, since the same caller code may build against any of the three backends.

Every subsequent HAL call is backend-agnostic; `Device`, `Queue`, and `Surface` are the only types a caller constructs differently per backend.

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_result_based_error_handling_scoped_panics.md](../invariant/001_result_based_error_handling_scoped_panics.md) | All three constructors return `Result<(Device, Queue, Surface), Error>` — `new_native` surfaces adapter/device-context failure as `Error::Native`, `new_webgl` surfaces context/extension failure as `Error::WebGl`/`Error::Unsupported` |

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_enum_per_backend_dispatch_one_step_drilldown.md](../pattern/001_enum_per_backend_dispatch_one_step_drilldown.md) | `Device`/`Queue`/`Surface` are the first enums a caller constructs |

### Pitfalls

| File | Relationship |
|------|--------------|
| [pitfall/001_backend_availability_compile_time_not_runtime.md](../pitfall/001_backend_availability_compile_time_not_runtime.md) | A constructor for a backend whose feature/target `cfg` doesn't hold does not exist to call, rather than existing and failing at runtime |

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | `mod_interface!` layer declarations gating which backend module compiles at all |
| `src/device.rs` | `Device::new_webgpu`/`new_webgl`/`new_native`, `depth_range`, `Surface` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/native_backend_test.rs` | `device_creation` asserts `depth_range() == DepthRange::ZeroToOne` and `surface.format() == TextureFormat::Rgba8Unorm` on the native backend |
