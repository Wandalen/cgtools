# Feature: Native Context and Device

`minvulkan` provides a type-state fluent builder (`Context::builder()`) that walks a caller through raw Vulkan instance, physical-device selection, and logical-device/queue creation in the required order, producing a single `Context` holding all the core `ash` handles — the `wgpu`-free counterpart to `minwgpu`'s [Context Builder](../../../minwgpu/docs/feature/001_context_builder.md).

### Scope

- **Purpose**: Reduce raw `ash`/Vulkan setup boilerplate while making it impossible to request a physical device or logical device out of order, without hiding the underlying Vulkan objects or introducing a `wgpu` dependency.
- **Responsibility**: Cross-reference the source and tests that make up `minvulkan`'s instance/device construction.
- **In Scope**: Instance configuration and construction, physical-device selection, logical-device/queue construction, the `Context` accessors, and the crate's error type.
- **Out of Scope**: Surface/swapchain creation and presentation — that is [Window Surface and Swapchain](002_window_surface_and_swapchain.md), which builds its own instance/device chain because the two cannot be interleaved after the fact; buffer/image/pipeline/command-pool resource construction (future features); validation-layer/debug-messenger setup — see `task/201_minvulkan_native_context_and_device.md § Out of Scope`.

### Design

`Context::builder()` returns a `ContextBuilder` parameterized by a phantom `_state` type (`InstanceBuilder` or `DeviceBuilder`); each state exposes only the configuration and transition methods valid at that stage, so the compiler rejects calling `context_finish` before `instance_make`.

**Instance stage** (`InstanceBuilder`): `flags` configures the `ash::vk::InstanceCreateFlags` used for `vk::InstanceCreateInfo`. `instance_make()` dynamically loads the Vulkan loader library at runtime via `ash::Entry::load()` (the crate's `"loaded"` `ash` feature — not `ash::Entry::linked()`, which requires a link-time `-lvulkan` dev symlink this environment does not provide), creates the `ash::Instance`, and transitions to `DeviceBuilder`.

**Device stage** (`DeviceBuilder`): `context_finish()` enumerates physical devices (`enumerate_physical_devices`) and selects the first one exposing a graphics-capable queue family (`get_physical_device_queue_family_properties` + `find_map` — no scoring/preference heuristic; see `physical_device_selector` in `minwgpu::ContextBuilder`'s `adapter_selector` for the analogous future extension point this crate does not yet have), creates the logical `ash::Device` with one graphics queue requested on that family (`create_device`), retrieves the resulting `ash::vk::Queue` (`get_device_queue`), and consumes the builder into a `Context`.

**Accessors**: `Context` exposes `entry_get`, `instance_get`, `physical_device_get`, `device_get`, `queue_get`, `queue_family_index_get` — the underlying `ash` types are returned directly, not wrapped further. `Context` does not hold a swapchain or surface: a builder-produced context is offscreen-only by construction, since its instance carries no platform surface extensions and its physical device was selected without regard to present support. A context that can present is produced by `context::windowed` instead — see [Window Surface and Swapchain](002_window_surface_and_swapchain.md).

**Lifetime**: `Context` implements `Drop`, destroying the logical device then the instance, in that order — Vulkan performs no automatic cleanup. `Context` deliberately does not derive `Clone`: it owns single-destruction semantics over its handles, and a second `Context` sharing the same handles would double-free on drop.

**Errors**: fallible steps return `crate::Error` (`src/error.rs`), a `#[non_exhaustive]` enum built with `error_tools`/`thiserror` covering Vulkan-loader-load failure (`ash::LoadingError`), instance/device creation failure, physical-device enumeration failure, and the no-suitable-device condition (`Error::NoSuitableDevice`) — never a panic.

The full chain — instance flags, physical-device/queue-family selection, and device/queue creation — is exercised end to end against a real local Vulkan implementation (lavapipe software rasterizer) in `tests/context_test.rs`.

### Sources

| File | Relationship |
|------|--------------|
| `src/context.rs` | `Context` / `ContextBuilder` type-state builder implementation |
| `src/error.rs` | `crate::Error` — error type returned by instance/device construction |

### Tests

| File | Relationship |
|------|--------------|
| `tests/context_test.rs` | Real (not mocked) instance/device/queue construction against a live Vulkan ICD — valid non-null handles (T01), a genuinely live device via `device_wait_idle` (T02), and an independently re-derived graphics queue-family index (T03) |
