# Pitfall: Native-Target Builds Compile to a Non-Functional Stub

### Scope

- **Purpose**: Record that compiling `minwebgpu` for a non-`wasm32` target silently succeeds but every function becomes non-functional.
- **Responsibility**: Document the trap, its observable failure, and the mitigation available to callers.
- **In Scope**: `target_arch`-conditional compilation of the `enabled` feature in `lib.rs`.
- **Out of Scope**: WebGPU behavior on `wasm32` targets themselves (see `feature/001`–`feature/005`).

### Trap

Assuming that because `minwebgpu` is a WebGPU/WASM-only library, building it for a native (non-`wasm32-unknown-unknown`) target will fail to compile — and therefore that a successful native build implies WebGPU functionality is available.

### Failure

`lib.rs` conditionally compiles a `stub` module on `not(target_arch = "wasm32")` in place of the real API; the crate compiles successfully on native targets, but the stub's `WebGPUNotAvailableError` is the only thing calls will actually produce at runtime, silently diverging from the `wasm32` build's real behavior. A caller running `cargo check`/`cargo test` on a native host without targeting `wasm32` gets a green compile with no signal that the resulting code path cannot perform any WebGPU operation.

### Mitigation

Always build and exercise `minwebgpu`-dependent code against `wasm32-unknown-unknown` (e.g. `wasm-pack build --target web`, per the crate readme's Build Commands) rather than the host target; a native `cargo check` is a fine quick syntax pass, but its success says nothing about WebGPU functionality.

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_facade_over_descriptor_builders.md](../pattern/001_facade_over_descriptor_builders.md) | The stub fallback sits behind the same module layering this pattern describes |

### Features

| File | Relationship |
|------|--------------|
| [feature/001_context_device_and_shader_setup.md](../feature/001_context_device_and_shader_setup.md) | Device/adapter acquisition is the first call that would surface `WebGPUNotAvailableError` on a native build |

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | `#[cfg(all(feature = "enabled", not(target_arch = "wasm32")))] pub mod stub` |

### Tests

No automated tests exist for this crate at the time of this migration.
