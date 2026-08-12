# Pitfall: Pixel Readback Is Native-Only

### Scope

- **Purpose**: Record that `Surface::pixels_read` compiles and type-checks identically across all three backends but only ever succeeds on native.
- **Responsibility**: Document the trap, its observable failure, and the mitigation available to callers.
- **In Scope**: `Surface::pixels_read`'s per-backend behavior.
- **Out of Scope**: The native readback implementation itself (see `feature/006`).

### Trap

Writing backend-agnostic rendering code that calls `Surface::pixels_read` expecting it to behave the same way across all three backends, since every other resource-creation and draw call in the public surface genuinely is backend-agnostic — nothing about `pixels_read`'s signature signals that it isn't.

### Failure

`Surface::pixels_read(&device, &queue) -> Result<Vec<u8>, Error>` is one method on one enum, so it compiles and type-checks identically regardless of which backend is active. But the WebGPU and WebGL arms both unconditionally return `Err(Error::Unsupported("pixels_read is a native-backend operation; browser surfaces present to their canvas"))` — only the native arm (`native::texture_rgba8_read`) performs a real GPU→CPU readback. Code written and tested only against the native backend (the crate's own `tests/native_backend_test.rs` does exactly this) will compile and pass there, then return a guaranteed `Unsupported` the moment the same call path runs against a WebGPU or WebGL build.

### Mitigation

Treat `pixels_read` as a native-only debugging/testing facility, not a portable rendering-path primitive. Browser backends present to their canvas directly — the browser's own compositor is the counterpart of a native readback for a browser target, not a HAL call.

### Features

| File | Relationship |
|------|--------------|
| [feature/006_native_pixel_readback.md](../feature/006_native_pixel_readback.md) | This pitfall's subject |

### Sources

| File | Relationship |
|------|--------------|
| `src/device.rs` | `Surface::pixels_read`'s WebGPU/WebGL arms, both returning `Error::Unsupported` unconditionally |
| `src/native.rs` | `texture_rgba8_read` — the only arm that actually reads pixels |

### Tests

`tests/native_backend_test.rs::triangle_render_readback` and `::texture_write_readback` both call `pixels_read` successfully, but only under `--features native` — no test in the crate currently demonstrates the WebGPU/WebGL `Unsupported` path this pitfall describes.
