# ADR-006: Vulkan Windowed Presentation via a Real `VK_KHR_swapchain`

- **Date**: 2026-08-18
- **Status**: Accepted
- **Deciders**: wandalen

## Context

[ADR-005](005_windowed_native_presentation.md) gave the `native` (`wgpu`) leg a
windowed path and explicitly deferred the other one: *"The `vulkan` leg keeps
its offscreen-only surface: a real `VK_KHR_swapchain` in `minvulkan` is a
substantially larger piece of work, and is deliberately left for its own
decision rather than folded in here."* It recorded the resulting asymmetry as a
consequence in its own terms — *"the four backends are no longer uniform in
presentation capability until that gap is closed separately."*

This is that separate decision.

The gap mattered because ADR-005 only half-delivered [ADR-004](004_native_vulkan_hal_backend.md)'s
stated objective. ADR-004 exists so a consumer *"can offer Vulkan without
linking `wgpu`"*. After ADR-005, a consumer that wanted a **window** could
avoid naming `wgpu` in its manifest — but `minwgpu` was still the only windowed
route, so `wgpu` was still linked into the process. A genuinely wgpu-free
windowed process was unreachable by construction.

`minvulkan` at that point held `context.rs`, `error.rs` and `lib.rs` and
nothing else: no surface, no swapchain, no pipeline, no pass. Its own feature
doc listed *"Surface/swapchain creation and presentation"* as out of scope.

## Decision

**1. `minvulkan` (L0) gains `VK_KHR_surface` and `VK_KHR_swapchain`.**
`Surface::from_window`, `Swapchain` with `frame_acquire`/`frame_present`/
`resize`, and a `Windowed` type owning context + surface + swapchain together —
structurally parallel to `minwgpu::surface::Windowed`, so the two L0 drivers
present the same shape through different backends.

**2. The window enters as handle traits here too**, exactly as ADR-005 decided
for `minwgpu`: `&( impl HasDisplayHandle + HasWindowHandle )`, no windowing
library depended upon, `raw_window_handle` re-exported so a consumer reaches
the traits through the driver. **`ash-window` is added as a dependency and is
not a windowing library** — it is part of the `ash` project, it takes raw
handles, and it is the exact Vulkan counterpart of the
`impl Into< wgpu::SurfaceTarget >` bound ADR-005 already accepted. The
alternative is hand-rolling per-platform `unsafe` surface creation for
xlib/xcb/wayland/win32/metal, which duplicates solved work for no principle.

**3. `gpu_hal` (L1) gains a `Surface::VulkanWindow` variant** and
`Device::new_vulkan_windowed`, following the same enum-per-backend dispatch as
ADR-005's `NativeWindow` and carrying the acquired image index in a `RefCell`
for the same reason — `current_view` keeps the `&self` signature all backends
share. The variant is boxed: `minvulkan::surface::Windowed` carries whole `ash`
dispatch tables inline, and unboxed it would set the size of every `Surface`
value including the browser ones.

**4. Presentation bridges layouts rather than making the render pass
conditional.** `color_attachment_description`'s `final_layout` is
`TRANSFER_SRC_OPTIMAL`, chosen so `pixels_read` can copy straight out of an
offscreen surface; presentation needs `PRESENT_SRC_KHR`. `Surface::present`
records a one-shot `TRANSFER_SRC_OPTIMAL → PRESENT_SRC_KHR` barrier before
`vkQueuePresentKHR`. Vulkan's render-pass compatibility rules ignore attachment
layouts, so every existing pipeline and render pass stays valid unchanged.

**5. Synchronization stays fence-only, deliberately.** Acquisition waits on a
fence and presentation passes no wait semaphore. This is correct only because
`gpu_hal`'s Vulkan `Queue::submit` already ends with `vkQueueWaitIdle`, so
rendering is provably complete before the transition is recorded. It costs
frame pipelining and buys the absence of any cross-backend `Queue::submit`
signature change for semaphore plumbing — a change that would touch all four
backends to benefit one. Revisit together with `submit`, never separately.

**6. `TextureFormat` gains `TryFrom< ash::vk::Format >`**, for the same reason
ADR-005 added the `wgpu` one: a swapchain picks its own presentation format, so
the HAL must be able to name what a driver *chose*, not only convert formats it
selected itself.

## Alternatives Considered

- **Hand-roll per-platform surface creation instead of adding `ash-window`.**
  Rejected: it duplicates a solved problem across five platform APIs, each with
  its own `unsafe` block, to avoid a dependency that is part of `ash` itself
  and carries no windowing library of its own.
- **Semaphore-based acquire/present with frames in flight.** Rejected for now:
  it requires threading wait/signal semaphores through `Queue::submit`, whose
  signature is shared by four backends, three of which have no use for them.
  The synchronous `submit` this backend already has makes the fence-only path
  correct today; both should change together or not at all.
- **Make the render pass's `final_layout` conditional on the target.**
  Rejected: it would fork `render_pass_create` and, transitively, pipeline
  creation, to avoid one barrier that costs a single one-shot command buffer
  per frame on a path that already waits for the queue to go idle.
- **Reuse `context_finish`'s physical-device selection.** Rejected: present
  support is a per-(device, queue family) property that cannot be queried
  before a surface exists, so the windowed path needs its own filter. Only the
  logical-device creation is genuinely shared, and that is the part factored
  out (`device_create`).
- **Leave the `vulkan` leg offscreen-only.** Rejected: it leaves ADR-004's
  wgpu-free objective permanently half-delivered, and leaves the four backends
  non-uniform in presentation capability with no plan to converge.

## Consequences

- **Positive**: A windowed process can now link no `wgpu` at all —
  `cargo tree -p gpu_hal_triangle_vulkan_window | grep -c wgpu` returns `0`.
  This completes ADR-004's objective, which ADR-005 could only half-deliver.
- **Positive**: The four backends are uniform in presentation capability again,
  closing the asymmetry ADR-005 recorded as an open consequence.
- **Negative**: The fence-only synchronization leaves the GPU idle at each
  frame boundary. This is a real throughput cost, deliberately accepted, and
  the reason the backend's own `submit` and this swapchain must be revisited as
  one change.
- **Negative**: `gpu_hal`'s Vulkan backend destroys no per-frame resources — a
  command pool, a render pass and a framebuffer leak per frame. Its module doc
  justified that v0 tradeoff on the grounds that *"`cargo nextest` isolates
  each test into its own process"*; a windowed loop is the first consumer that
  invalidates that reasoning, since it runs thousands of frames in one. Not
  fixed here, and now a genuine defect rather than a documented simplification.
- **Neutral**: `minvulkan` grows two dependencies (`ash-window`,
  `raw-window-handle`), both already present in the tree beneath `wgpu`.

## Related

- [005_windowed_native_presentation.md](005_windowed_native_presentation.md) —
  the `native` leg's windowed decision, which explicitly deferred this one
- [004_native_vulkan_hal_backend.md](004_native_vulkan_hal_backend.md) — the
  wgpu-free objective this ADR finishes delivering
- [002_gpu_hal_in_house.md](002_gpu_hal_in_house.md) — the enum-per-backend
  dispatch pattern the `VulkanWindow` variant follows
- [layer/001_l0_drivers.md](../layer/001_l0_drivers.md) — L0's driver contract
  the new `minvulkan` surface and swapchain API follows
- [layer/002_l1_gpu_hal.md](../layer/002_l1_gpu_hal.md) — L1's contract the
  `VulkanWindow` variant extends
