# gpu_hal Triangle (native wgpu swapchain)

**Keywords:** wgpu, gpu_hal, swapchain, winit, Rust, native

A color-cycling triangle in a real window, rendered through `gpu_hal`'s `native` backend and presented through a real `wgpu::Surface` swapchain — the windowed counterpart of `gpu_hal/tests/native_backend_test.rs`'s `triangle_render_readback`, reusing that test's WGSL shader, vertices and uniform layout unchanged; the one difference is where the frame goes, offscreen readback there and on screen here. Resizing the window rebuilds the swapchain and the triangle stays centered and correctly proportioned, which together with the continuously cycling color is what distinguishes a live acquire/present loop from one stuck on a single stale image.

This exists as an example rather than a test because the presentation half of
the path cannot be `cargo test`-automated: it needs a real window handle, and
no crate inside `module/` may depend on a windowing library (see
[ADR-005](../../../docs/adr/005_windowed_native_presentation.md)), so no crate
under `module/` can produce one.

## Run

```bash
cargo run -p gpu_hal_triangle_native_window --release
```

Requires a `wgpu`-compatible adapter (Vulkan, Metal, DX12, or GL) for the
current display server. A software rasterizer (lavapipe / `mesa-vulkan-drivers`)
is enough on Linux.

## What it exercises

| Path | Where |
| ---- | ----- |
| `Device::new_native_windowed` | `minwgpu::surface::Windowed` construction : instance, surface, compatible adapter, device, initial swapchain configuration |
| `Surface::current_view` | `wgpu::Surface::get_current_texture`, out-of-date reporting as `Error::SurfaceNotReady` |
| `Surface::present` | the acquired `wgpu::SurfaceTexture`'s `present()` |
| `Surface::resize` | reconfigures the `wgpu::Surface` at the new drawable size |

The last row is the one worth driving by hand — drag the window's edge and
watch the triangle stay centered and correctly proportioned rather than
stretching, which is what a rebuilt chain looks like versus a stale one.

## Window handle: `Arc<Window>`, not `&Window`

Unlike the Vulkan sibling example (`examples/gpu_hal/triangle_vulkan_window`),
which passes `&Window` because `minvulkan` takes the raw `HasWindowHandle` /
`HasDisplayHandle` traits by reference, this example passes `Arc<Window>`.
`Device::new_native_windowed` takes `impl Into<wgpu::SurfaceTarget<'static>>`,
and `wgpu` only has a blanket conversion into that `'static`-bound type from an
*owned* handle — notably `Arc<winit::window::Window>` — never from a borrow.
See `examples/minwgpu/flecs_bouncing_circles` for the same pattern one layer
below this HAL.

## Responsibility Table

| File | Responsibility |
| ---- | -------------- |
| src/main.rs | Winit app driving the windowed native wgpu triangle render loop |
| Cargo.toml | Crate manifest: gpu_hal with the native backend, winit for the window |
