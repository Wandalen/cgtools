# Feature: Window Surface and Swapchain

`minvulkan` creates a `VkSurfaceKHR` over a caller-supplied window and a real `VK_KHR_swapchain` over that surface, and hands out one image per frame through an acquire/present pair — the `wgpu`-free counterpart to `minwgpu`'s [Surface Configuration](../../../minwgpu/docs/feature/003_surface_configuration.md). Together with [Native Context and Device](001_native_context_and_device.md) this is what lets a process present to a window with no `wgpu` linked anywhere in its dependency graph.

### Scope

- **Purpose**: Make windowed presentation reachable from `minvulkan` without introducing a dependency on any windowing library, so that L1 (`gpu_hal`) can offer a genuinely `wgpu`-free windowed backend.
- **Responsibility**: Cross-reference the source and tests behind surface creation, swapchain construction/rebuild, and the per-frame acquire/present pair.
- **In Scope**: `VkSurfaceKHR` creation from raw handle traits, the surface capability/format queries a swapchain is built from, swapchain creation and resize-driven rebuild, per-frame image acquisition and presentation, and the `Windowed` value binding context, surface and swapchain together.
- **Out of Scope**: Render passes, framebuffers, pipelines, and command recording against an acquired image — a consumer reaches those through `gpu_hal`'s Vulkan backend or raw `ash`. Also out of scope: semaphore-based frame pipelining (see **Synchronization** below), multi-frame-in-flight, and any present mode other than FIFO.

### Design

**A window enters as handle traits, never as a windowing type.** `Surface::from_window` takes `&( impl HasDisplayHandle + HasWindowHandle )` and nothing else, so `winit`, `sdl2` and `glfw` are all equally usable and none is depended upon — the decision recorded in [ADR-005](../../../../docs/adr/005_windowed_native_presentation.md) and extended to this crate by [ADR-006](../../../../docs/adr/006_vulkan_windowed_presentation.md). `raw_window_handle` is re-exported as `minvulkan::raw_window_handle` so a consumer reaches the traits through the driver rather than declaring a second, independently versioned copy. The one dependency this adds, `ash-window`, is part of the `ash` project itself and is the Vulkan counterpart of `wgpu::SurfaceTarget`: it turns raw handles into a `VkSurfaceKHR` and reports which platform instance extensions that needs, and knows nothing about any windowing library.

**Construction is one function, not four builder steps.** `context::windowed` performs instance → surface → present-capable device → swapchain in that order, because Vulkan requires it: the instance must already carry the platform's surface extensions, and present support is a per-(device, queue family) property that cannot be queried before the surface exists. That is why the windowed path has its own physical-device selection (`present_device_select`, filtering on graphics *and* present capability) rather than reusing `context_finish`'s graphics-only one, and enables `VK_KHR_swapchain` on the logical device. The device-creation step itself is shared (`device_create`) so the two paths cannot drift.

**Cleanup ordering is enforced structurally.** `Windowed` declares its fields `swapchain`, `surface`, `context` in that order; Rust drops fields in declaration order, and Vulkan requires exactly that sequence. `into_parts` returns the three in the same order for a caller who takes them apart. Between instance creation and the `Context` that finally owns it, `windowed` guards the instance with an RAII `InstanceGuard` rather than a per-error-path cleanup call, because the surface must be destroyed before the instance and only drop order can express that.

**Frame lifecycle.** `frame_acquire` returns `Frame::Ready { index, image, view, extent }` or `Frame::Reconfigure`. Two arms rather than `minwgpu::surface::Frame`'s three: acquisition uses an infinite timeout, so "no image available yet" blocks instead of returning, and there is no `Skip` outcome to report. `frame_present` returns `true` when the chain should be rebuilt, so an out-of-date swapchain surfaces as an ordinary value on both halves of the pair rather than as an error. `resize` waits for the device to go idle, then rebuilds passing the old chain as `oldSwapchain` and destroying it only after the new one exists.

**Synchronization is deliberately minimal.** Acquisition waits on a fence rather than signalling a semaphore, so the acquired image is provably owned by the host before any rendering is recorded, and presentation passes no wait semaphore. This is correct only because every consumer of an acquired image submits synchronously — `gpu_hal`'s Vulkan `Queue::submit` ends with `vkQueueWaitIdle` — and it is what keeps the swapchain free of any cross-backend signature change for semaphore plumbing. It costs frame pipelining: the GPU is idle at each frame boundary. Revisit together with `submit`, not separately.

**A zero-sized drawable is an error, not a validation failure.** A minimized window reports a zero `currentExtent`, which `vkCreateSwapchainKHR` rejects as an undiagnosable validation error. `rebuild` bails with `Error::ZeroExtent` *before* creating or destroying anything, so the existing chain stays intact and rendering resumes on its own once the window returns.

**Format selection** prefers the first sRGB-encoded format the surface reports, falling back to the first reported format — the same choice `minwgpu::surface::preferred_format` makes, so a shader writing linear-space color is gamma-corrected on present. Unlike its `minwgpu` counterpart, an empty list is an error (`Error::NoSurfaceFormat`) rather than a panic: the list comes straight from a driver query here, not from an infallible `wgpu` capability struct.

### Sources

| File | Relationship |
|------|--------------|
| `src/surface.rs` | `Surface`, `Windowed`, `required_instance_extensions`, `preferred_format` |
| `src/swapchain.rs` | `Swapchain`, `Frame`, extent clamping and image-count selection |
| `src/context.rs` | `context::windowed` — the instance → surface → device → swapchain sequence, and `present_device_select` |
| `src/error.rs` | The ten surface/swapchain error variants |

### Tests

| File | Relationship |
|------|--------------|
| `tests/surface_test.rs` | `preferred_format`'s selection, fallback, empty-list and color-space behavior (T01–T05), and the `Windowed` field-order invariant that encodes Vulkan's destruction ordering (T06) |
| `examples/gpu_hal/triangle_vulkan_window` | End-to-end coverage of everything needing a live window — surface creation, swapchain construction, acquire/present, and resize-driven rebuild — since no crate in this workspace can produce a window handle for a `cargo test` to use |
