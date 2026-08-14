# Invariant: Result-Based Error Handling with a Scoped Panic Policy

### Scope

- **Purpose**: Guarantee every fallible, caller-input-driven HAL operation surfaces failure through a typed `Result`, while confining the crate's only panics to a narrow, internal-only contract unreachable from correct public use.
- **Responsibility**: Document the crate-wide error contract, the unsafe-code prohibition, and exactly where the crate's panics live and why.
- **In Scope**: The `Error` type, the no-`unsafe`-code guarantee, and the `pub(crate)`-only panic policy.
- **Out of Scope**: Per-backend error message wording (read `src/error.rs` directly for the current, authoritative list).

### Invariant Statement

Every public, fallible `gpu_hal` function returns `Result<T, Error>`; the crate contains zero `unsafe` blocks; and the crate's only `panic!` call sites are the `pub(crate)` `expect_webgpu()`/`expect_webgl()`/`expect_native()` accessors used exclusively by `gpu_hal`'s own backend-dispatch match arms — never reachable from a public call unless the crate's own internal dispatch logic is broken, and never invoked with caller-controlled backend selection.

### Enforcement Mechanism

`Error` (`src/error.rs`) is a 4-variant enum — `WebGpu(String)`, `WebGl(String)`, `Native(String)`, `Unsupported(String)` — with `Display` and `std::error::Error` impls, plus `#[cfg]`-gated `From` impls converting each backend driver's own error type (`minwebgpu::WebGPUError`, `minwebgl::WebglError`, `minwgpu::Error`) into the matching variant via `error.to_string()`. Every public constructor, resource-creation, and submission function returns `Result<_, Error>`. Verified for this crate: `grep -rn "unsafe" src/` returns zero matches.

The panic side of the contract is deliberately narrow and internal: `resource.rs` and `pass.rs` together contain 17 `panic!("backend mismatch : expected a WebGPU/WebGL <thing>")` call sites, every one of them inside a `pub(crate) fn expect_webgpu()`/`expect_webgl()`/`expect_native()` accessor — distinct from the public, non-panicking `as_webgpu()`/`as_webgl()`/`as_native() -> Option<&Raw>` accessors on the same types (see `pattern/001`). The `expect_*` family exists only so `gpu_hal`'s own per-backend match arms can assume "this handle matches the device's own active backend" without re-threading a `Result` through every internal call — a caller can never construct a mismatched-backend handle through the public API, so the panic is unreachable except by a bug in the crate's own dispatch.

The contract stays maintained under change, not just at a point in time: `device.rs`'s `bind_group_layout_create` carries an inline `Fix(BUG-051)` comment recording that `BindGroupLayoutDescriptor::entry` became fallible upstream (its `TryFrom` conversion stopped panicking on an unset binding type) — the call site was updated to propagate with `?` rather than reaching for `.unwrap()`/`.expect()`, preserving the `Result`-returning contract. Separately, `shader_module_create`'s WebGL arm returns `Err(Error::Unsupported(...))` on a missing GLSL override rather than panicking, even though its WebGPU and native arms "never fail this call" per their own doc comment — the function stays uniformly fallible across all three backends despite two of them being infallible in practice.

### Violation Consequences

A function that panics on caller-supplied input (as opposed to the internal `expect_*` contract above), or lets a raw backend-native error type escape past `Error`, breaks the uniform error contract every `#[cfg]`-agnostic call site relies on to stay portable across backends.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_backend_construction_and_device_acquisition.md](../feature/001_backend_construction_and_device_acquisition.md) | All three constructors return `Result<_, Error>` |
| [feature/002_resource_creation.md](../feature/002_resource_creation.md) | All four constructors return `Result<_, Error>` |
| [feature/003_shader_modules_and_render_pipelines.md](../feature/003_shader_modules_and_render_pipelines.md) | Both constructors return `Result<_, Error>` |
| [feature/004_bind_groups_and_layouts.md](../feature/004_bind_groups_and_layouts.md) | Both constructors return `Result<_, Error>` |
| [feature/005_command_recording_and_submission.md](../feature/005_command_recording_and_submission.md) | Every fallible call returns `Result<_, Error>` |
| [feature/006_native_pixel_readback.md](../feature/006_native_pixel_readback.md) | Format check and browser-backend case both return `Result`, never panic |

### Sources

| File | Relationship |
|------|--------------|
| `src/error.rs` | `Error` enum and its `From` impls |
| `src/resource.rs`, `src/pass.rs` | The 17 `expect_*` internal panic sites |
| `src/device.rs` | `Fix(BUG-051)` comment; `shader_module_create`'s uniformly-fallible signature |

### Tests

No dedicated test exercises the `expect_*` panic path directly — it is `pub(crate)`-only and unreachable from `tests/`, which only calls the public, `Result`-returning surface. `grep -rn "unsafe" src/` is the mechanical check for the no-`unsafe` half of this invariant.
