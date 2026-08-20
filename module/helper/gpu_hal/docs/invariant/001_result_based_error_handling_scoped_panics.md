# Invariant: Result-Based Error Handling with a Scoped Panic Policy

### Scope

- **Purpose**: Guarantee every fallible, caller-input-driven HAL operation surfaces failure through a typed `Result`, while confining the crate's only panics to a narrow, internal-only contract unreachable from correct public use.
- **Responsibility**: Document the crate-wide error contract, the unsafe-code prohibition, and exactly where the crate's panics live and why.
- **In Scope**: The `Error` type, the no-`unsafe`-code guarantee, and the `pub(crate)`-only panic policy.
- **Out of Scope**: Per-backend error message wording (read `src/error.rs` directly for the current, authoritative list).

### Invariant Statement

Every public, fallible `gpu_hal` function returns `Result<T, Error>`, with two deliberate Vulkan-only exceptions (Category B below); the crate contains zero `unsafe` code outside `vulkan.rs`, which carries its own scoped, documented exception; and the crate's panics fall into two contracted categories — internal dispatch-integrity panics (`pub(crate)` `expect_webgpu()`/`expect_webgl()`/`expect_native()`/`expect_vulkan()` accessors and their bespoke equivalents), never reachable from a public call unless the crate's own internal dispatch logic is broken, and Vulkan's two intentionally-infallible-signature functions, which panic only on a genuine, caller-independent driver failure.

### Enforcement Mechanism

`Error` (`src/error.rs`) is a 6-variant enum — `WebGpu(String)`, `WebGl(String)`, `Native(String)`, `Vulkan(String)`, `Unsupported(String)`, `InvalidInput(String)` — with `Display` and `std::error::Error` impls, plus `#[cfg]`-gated `From` impls converting each backend driver's own error type (`minwebgpu::WebGPUError`, `minwebgl::WebglError`, `minwgpu::Error`, `minvulkan::Error`) into the matching variant via `error.to_string()`; `InvalidInput` is backend-agnostic, raised before any backend is touched (e.g. `buffer_write`'s data-length mismatch check). Every public constructor, resource-creation, and submission function returns `Result<_, Error>`, except the two Category B functions below. Verified for this crate: `grep -rn "unsafe" src/` returns matches only inside `vulkan.rs` (58 sites, one per direct `ash` FFI call, each with its own `// SAFETY:` comment) plus the single scoped `#[allow(unsafe_code)]` declaration in `lib.rs` that permits it — every other module stays at zero.

The panic side of the contract is deliberately narrow and falls into two categories, 43 sites total:

**Category A — internal dispatch-integrity panics (39 sites: 32 `resource.rs`, 1 `pass.rs`, 6 `device.rs`).** Every HAL resource type (`Buffer`, `Texture`, `TextureView`, `Sampler`, `ShaderModule`, `BindGroupLayout`, `BindGroup`, `RenderPipeline` in `resource.rs`; `Device`, `Queue` in `device.rs`; `CommandEncoder` in `pass.rs`) carries `pub(crate) fn expect_webgpu()`/`expect_webgl()`/`expect_native()`/`expect_vulkan()` accessors, distinct from the public, non-panicking `as_webgpu()`/`as_webgl()`/`as_native()`/`as_vulkan() -> Option<&Raw>` accessors on the same types (see `pattern/001`). Because the `webgpu`/`webgl` features are `wasm32`-gated and `native`/`vulkan` are `not(wasm32)`-gated, only two of the four variants can ever coexist in one compiled binary — so each `expect_X()` is a 2-arm match, never a flat 4-way one: its own success arm, plus one `#[cfg]`-gated panic arm for its single same-target-architecture sibling (e.g. `expect_native()`'s only panic arm rejects `Self::Vulkan`; `WebGpu`/`WebGl` don't exist in that build at all, so there is nothing to match against them). `device.rs`'s `native_submit`/`vulkan_queue_submit` add 2 more sites of the same class, implemented as a direct 2-arm match on `CommandEncoder` rather than through an `expect_*` helper. The whole family exists only so `gpu_hal`'s own per-backend match arms can assume "this handle matches the device's own active backend" without re-threading a `Result` through every internal call — a caller can never construct a mismatched-backend handle through the public API, so the panic is unreachable except by a bug in the crate's own dispatch.

**Category B — deliberately-infallible-signature panics (4 sites, Vulkan-only: `device.rs`'s `command_encoder_create`, `vulkan.rs`'s `submit` ×3 for `vkEndCommandBuffer`/`vkQueueSubmit`/`vkQueueWaitIdle`).** Both functions keep the same infallible signature every other backend's equivalent already has (`Queue::submit` is infallible on WebGPU/WebGL/native too), and panic only if the underlying Vulkan driver call itself fails — a genuine, caller-independent failure surfaced loudly rather than silently lost, not a caller-triggerable bug. Both carry their own `# Panics` doc-comment section naming the exact failure condition.

The contract stays maintained under change, not just at a point in time: `device.rs`'s `bind_group_layout_create` carries an inline `Fix(BUG-051)` comment recording that `BindGroupLayoutDescriptor::entry` became fallible upstream (its `TryFrom` conversion stopped panicking on an unset binding type) — the call site was updated to propagate with `?` rather than reaching for `.unwrap()`/`.expect()`, preserving the `Result`-returning contract. Separately, `shader_module_create` stays uniformly `Result`-returning across all four backends despite an uneven fallibility split: WebGPU's and native's arms "never fail this call" per their own doc comment; WebGL's arm returns `Err(Error::Unsupported(...))` on a missing GLSL override; and Vulkan's arm is genuinely fallible for a third, unrelated reason — it returns `Err(Error::Vulkan(...))` if WGSL-to-SPIR-V compilation via `naga` or `vkCreateShaderModule` itself fails.

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
| `src/resource.rs`, `src/pass.rs` | 33 of Category A's 39 `expect_*` internal panic sites |
| `src/device.rs` | `Fix(BUG-051)` comment; `shader_module_create`'s uniformly-fallible signature; remaining 6 Category A sites (`Device`/`Queue` `expect_*`, `native_submit`/`vulkan_queue_submit`); Category B's `command_encoder_create` |
| `src/vulkan.rs` | Category B's `submit` (3 panic sites); every `unsafe` block in the crate |
| `src/lib.rs` | The crate's sole `#[allow(unsafe_code)]`, scoped to `vulkan.rs` |

### Tests

No dedicated test exercises either panic category directly: Category A is `pub(crate)`-only and unreachable from `tests/`, which only calls the public, `Result`-returning surface; Category B requires an actual Vulkan driver failure, which `tests/vulkan_backend_test.rs` has no way to induce. `grep -rn "unsafe" src/` is the mechanical check confirming `unsafe` stays confined to `vulkan.rs`.
