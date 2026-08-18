# gpu_hal Triangle (Vulkan swapchain)

**Keywords:** Vulkan, gpu_hal, swapchain, winit, Rust, native

A color-cycling triangle in a real window, rendered through `gpu_hal`'s `vulkan` backend and presented through a real `VK_KHR_swapchain` — the only example in this workspace whose process links no `wgpu` at all. It is the windowed counterpart of `gpu_hal/tests/vulkan_backend_test.rs`'s `triangle_render_readback`, reusing that test's WGSL shader, vertices and uniform layout unchanged; the one difference is where the frame goes, offscreen readback there and on screen here. Resizing the window rebuilds the swapchain and the triangle stays centered and correctly proportioned, which together with the continuously cycling color is what distinguishes a live acquire/present loop from one stuck on a single stale image.

This exists as an example rather than a test because the presentation half of
the path cannot be `cargo test`-automated: it needs a real window handle, and
`minvulkan` deliberately depends on no windowing library (see
[ADR-006](../../../docs/adr/006_vulkan_windowed_presentation.md)), so no crate
inside `module/` can produce one.

Verify the wgpu-free claim yourself:

```bash
cargo tree -p gpu_hal_triangle_vulkan_window --depth 1   # gpu_hal, winit -- no wgpu
cargo tree -p gpu_hal_triangle_vulkan_window | grep -c wgpu   # 0
```

## Run

```bash
cargo run -p gpu_hal_triangle_vulkan_window --release
```

Requires a Vulkan ICD exposing `VK_KHR_swapchain` for the current display
server. A software rasterizer (lavapipe / `mesa-vulkan-drivers`) is enough —
`vulkaninfo | grep -i swapchain` confirms the extension is present.

## What it exercises

| Path | Where |
| ---- | ----- |
| `Device::new_vulkan_windowed` | instance with the platform's surface extensions, present-capable device selection, swapchain creation |
| `Surface::current_view` | `vkAcquireNextImageKHR`, fence wait, out-of-date reporting as `Error::SurfaceNotReady` |
| `Surface::present` | `TRANSFER_SRC_OPTIMAL` → `PRESENT_SRC_KHR` transition, then `vkQueuePresentKHR` |
| `Surface::resize` | `vkDeviceWaitIdle`, rebuild with `oldSwapchain`, retire the previous chain |

The last row is the one worth driving by hand — drag the window's edge and
watch the triangle stay centered and correctly proportioned rather than
stretching, which is what a rebuilt chain looks like versus a stale one.

## Known limitation

The `vulkan` backend destroys no per-frame resources: `command_encoder_create`
allocates a command pool and `render_pass_begin` a render pass and framebuffer,
and none of the three is destroyed (see `module/helper/gpu_hal/src/vulkan.rs`'s
module doc comment for the v0 tradeoff and its stated rationale — one isolated
process per test). A windowed loop is the first consumer that invalidates that
rationale, since it runs thousands of frames in one process. It is fine for a
demo of this length and a genuine defect for a shipping application.

## Responsibility Table

| File | Responsibility |
| ---- | -------------- |
| src/main.rs | Winit app driving the windowed Vulkan triangle render loop |
| Cargo.toml | Crate manifest: gpu_hal with the vulkan backend, winit for the window |
