# Pitfall: Pixel Readback Is Unsupported on Browser Backends

### Scope

- **Purpose**: Record that `Surface::pixels_read` compiles and type-checks identically across all four backends but only ever succeeds on the two non-browser ones.
- **Responsibility**: Document the trap, its observable failure, and the mitigation available to callers.
- **In Scope**: `Surface::pixels_read`'s per-backend behavior.
- **Out of Scope**: The native/vulkan readback implementations themselves (see `feature/006`).

### Trap

Writing backend-agnostic rendering code that calls `Surface::pixels_read` expecting it to behave the same way across all four backends, since every other resource-creation and draw call in the public surface genuinely is backend-agnostic — nothing about `pixels_read`'s signature signals that it isn't.

### Failure

`Surface::pixels_read(&device, &queue) -> Result<Vec<u8>, Error>` is one method on one enum, so it compiles and type-checks identically regardless of which backend is active. But the WebGPU and WebGL arms both unconditionally return `Err(Error::Unsupported("pixels_read is a native-backend operation; browser surfaces present to their canvas"))` — only the native (`native::texture_rgba8_read`) and vulkan (`vulkan::vulkan_pixels_read`) arms perform a real GPU→CPU readback. Code written and tested only against one non-browser backend (the crate's own `tests/native_backend_test.rs` and `tests/vulkan_backend_test.rs` each do exactly this, for their own backend) will compile and pass there, then return a guaranteed `Unsupported` the moment the same call path runs against a WebGPU or WebGL build.

### Mitigation

Treat `pixels_read` as a non-browser (native/vulkan) debugging/testing facility, not a portable rendering-path primitive. Browser backends present to their canvas directly — the browser's own compositor is the counterpart of a non-browser readback for a browser target, not a HAL call.

### Features

| File | Relationship |
|------|--------------|
| [feature/006_native_pixel_readback.md](../feature/006_native_pixel_readback.md) | This pitfall's subject |

### Sources

| File | Relationship |
|------|--------------|
| `src/device.rs` | `Surface::pixels_read`'s WebGPU/WebGL arms, both returning `Error::Unsupported` unconditionally |
| `src/native.rs` | `texture_rgba8_read` — one of the two arms that actually reads pixels |
| `src/vulkan.rs` | `vulkan_pixels_read` — the other arm that actually reads pixels |

### Tests

`tests/native_backend_test.rs::triangle_render_readback` and `::texture_write_readback` call `pixels_read` successfully under `--features native`; `tests/vulkan_backend_test.rs::triangle_render_readback` calls it successfully under `--features vulkan`. No test in the crate currently demonstrates the WebGPU/WebGL `Unsupported` path this pitfall describes.
