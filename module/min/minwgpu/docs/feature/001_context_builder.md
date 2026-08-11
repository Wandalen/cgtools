# Feature: Context Builder

`minwgpu` provides a type-state fluent builder (`Context::builder()`) that walks a caller through `wgpu` instance, adapter, and device/queue creation in the required order, producing a single `Context` holding all four core `wgpu` objects.

### Scope

- **Purpose**: Reduce `wgpu` setup boilerplate while making it impossible to request an adapter or device out of order.
- **Responsibility**: Cross-reference the source and tests that make up `wgpu` context construction.
- **In Scope**: Instance/adapter/device/queue configuration and construction, the `Context` accessors, and the crate's error type.
- **Out of Scope**: Buffer and texture construction (see [feature/002_buffer_and_texture_builders.md](002_buffer_and_texture_builders.md)).

### Design

`Context::builder()` returns a `ContextBuilder` parameterized by a phantom `_state` type (`InstanceBuilder`, `AdapterBuilder`, or `DeviceBuilder`); each state exposes only the configuration and transition methods valid at that stage, so the compiler rejects calling `request_adapter` before `make_instance`, for example. `Context::from_instance( instance )` is the alternate entry point for a caller supplying an already-constructed `wgpu::Instance`, skipping straight to the `AdapterBuilder` state.

**Instance stage** (`InstanceBuilder`): `backends`, `flags`, `memory_budget_thresholds`, `backend_options` configure a `wgpu::InstanceDescriptor`; `make_instance()` builds the `wgpu::Instance` and transitions to `AdapterBuilder`.

**Adapter stage** (`AdapterBuilder`): `power_preference`, `force_fallback_adapter`, `compatible_surface` configure a `wgpu::RequestAdapterOptions`; `adapter_selector` accepts a closure that receives the instance and itself returns a selected adapter or an error — when set, it is used instead of the configured options. `request_adapter()` / `request_adapter_async()` perform the request and transition to `DeviceBuilder` — the sync form blocks on the async one via `pollster::block_on`.

**Device stage** (`DeviceBuilder`): `label`, `required_features`, `required_limits`, `memory_hints`, `trace` configure a `wgpu::DeviceDescriptor`; `finish_context()` / `finish_context_async()` request the device and queue and consume the builder into a `Context`, again with the sync form wrapping the async one via `pollster::block_on`. This paired sync/async shape — an `_async` method plus a `pollster`-blocking sync counterpart — is used consistently at every step that calls into `wgpu`'s own async API.

**Accessors**: `Context` exposes `get_instance`, `get_adapter`, `get_device`, `get_queue`, plus `AsRef` implementations for all four — the underlying `wgpu` types are returned directly, not wrapped further. `Context` does not hold a `wgpu::Surface`; surface creation and management are not part of this crate.

**Errors**: fallible steps return `crate::Error` (`src/error.rs`), a `#[non_exhaustive]` enum built with `error_tools`/`thiserror` covering `wgpu::Error`, `wgpu::RequestAdapterError`, and `wgpu::RequestDeviceError`.

The full chain — instance backend selection, adapter power preference, and device label/features/limits — is exercised end to end in the workspace's `grid_render` example, alongside the buffer builders from [feature/002](002_buffer_and_texture_builders.md).

### Sources

| File | Relationship |
|------|--------------|
| `src/context.rs` | `Context` / `ContextBuilder` type-state builder implementation |
| `src/error.rs` | `crate::Error` — error type returned by adapter/device requests |

### Tests

| File | Relationship |
|------|--------------|
| `src/context.rs` | Inline `#[cfg(test)]` coverage of builder configuration methods (backends, flags, power preference, fallback adapter, adapter selector, device label/features/limits/memory hints) |
