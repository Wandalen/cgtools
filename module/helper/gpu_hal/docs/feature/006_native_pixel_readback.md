# Feature: Off-Browser Pixel Readback

### Scope

- **Purpose**: Read rendered pixels back to the CPU on the native and Vulkan backends — the mechanism a pixel-asserting test relies on as ground truth.
- **Responsibility**: Document the pixel-readback API's design and its off-browser-only availability.
- **In Scope**: `Surface::pixels_read`, `native::texture_rgba8_read`, `vulkan::pixels_read`.
- **Out of Scope**: Presenting to a canvas, which is the browser backends' implicit path and not a HAL call.

### Design

`Surface::pixels_read(&device, &queue) -> Result<Vec<u8>, Error>` is the off-browser counterpart of presenting to a canvas: it returns tightly-packed `rgba8` bytes, top row first. The native and Vulkan arms each perform a real GPU→CPU readback, independently, through unrelated mechanisms; the WebGPU and WebGL arms both return `Err(Error::Unsupported("pixels_read is a native-backend operation; browser surfaces present to their canvas"))` unconditionally (see `pitfall/002`).

The native implementation, `native::texture_rgba8_read(device, queue, texture)`, first checks `texture.format() == Rgba8Unorm`, returning `Error::Unsupported` for any other format. It computes `bytes_per_row = width * 4`, then pads that up to `wgpu::COPY_BYTES_PER_ROW_ALIGNMENT` (256 bytes) via `div_ceil` — `wgpu` buffer-copy destinations require row alignment, so a copy into a plain tightly-packed staging buffer would be rejected. The staging buffer is sized to the *padded* row width, then de-padded back to tightly-packed bytes before the function returns — the alignment is an internal implementation detail, invisible to the caller's `Result<Vec<u8>, Error>`. `Error::Native` covers a failure of the device poll, the readback map callback, or the GPU-side buffer mapping.

The Vulkan implementation, `vulkan::pixels_read(device_vulkan, queue, surface)`, records a `vkCmdCopyImageToBuffer` from the surface image (already left in `TRANSFER_SRC_OPTIMAL` by every render pass's fixed `finalLayout`, so no extra layout transition is needed) into a `HOST_VISIBLE`/`HOST_COHERENT` staging buffer, submits and waits on it via a one-shot command buffer, then `vkMapMemory`s the result. Unlike native's path, Vulkan needs no row-padding dance — `BufferImageCopy::buffer_row_length(0)` means "tightly packed," which Vulkan honors directly, so the staging buffer is exactly `width * height * bytes_per_texel`. `Error::Vulkan` covers a failure of staging-buffer allocation or the `vkMapMemory` call.

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_result_based_error_handling_scoped_panics.md](../invariant/001_result_based_error_handling_scoped_panics.md) | Both the format check and the browser-backend `Unsupported` case return `Result`, never panic |

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_enum_per_backend_dispatch_one_step_drilldown.md](../pattern/001_enum_per_backend_dispatch_one_step_drilldown.md) | `texture_rgba8_read` is `native.rs`'s raw-`wgpu`-typed free function, reachable through `Surface::pixels_read` or directly once a caller already holds raw `wgpu` handles via `as_native()` |

### Pitfalls

| File | Relationship |
|------|--------------|
| [pitfall/002_pixel_readback_native_only.md](../pitfall/002_pixel_readback_native_only.md) | This feature's entire browser-backend behavior is that pitfall's subject |

### Sources

| File | Relationship |
|------|--------------|
| `src/device.rs` | `Surface::pixels_read` per-backend dispatch |
| `src/native.rs` | `texture_rgba8_read` — format check, row-alignment padding/unpadding, staging buffer readback |
| `src/vulkan.rs` | `pixels_read` — `vkCmdCopyImageToBuffer` into a tightly-packed `HOST_VISIBLE` staging buffer, no row-padding needed |

### Tests

| File | Relationship |
|------|--------------|
| `tests/native_backend_test.rs` | `triangle_render_readback` uses a 100×100 surface specifically to exercise the padded-row path (400 bytes/row pads to 512); `texture_write_readback` re-samples after two successive `texture_write` calls to prove overwrite semantics, not just first-write correctness |
| `tests/vulkan_backend_test.rs` | `triangle_render_readback` mirrors the native test's structure and surface size for direct comparability, but exercises no row-padding path — Vulkan's `pixels_read` doesn't have one |
