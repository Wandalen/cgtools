# minvulkan

Minimal, opinionated native Vulkan toolkit via `ash` — the `wgpu`-free
counterpart to [`minwgpu`](../minwgpu/).

## Overview

`minvulkan` is L0's fourth native driver: a thin, backend-faithful wrapper
over raw Vulkan (via [`ash`](https://docs.rs/ash)), exposing real Vulkan
objects and concepts rather than a cross-backend abstraction — the same
role [`minwebgl`](../minwebgl/), [`minwebgpu`](../minwebgpu/), and
[`minwgpu`](../minwgpu/) already occupy for their own backends. Unlike
`minwgpu`, it does not depend on `wgpu` at all: it exists so `gpu_hal`
(L1) can offer a Vulkan backend that stays genuinely `wgpu`-free, for
consumers such as `examples/orrery/flexible` that must not link `wgpu`
except through an explicit `wgpu`-backend selection. See
[ADR-004](../../../docs/adr/004_native_vulkan_hal_backend.md).

**Status:** `Context::builder()` produces a real `ash::Instance`,
`PhysicalDevice`, `Device`, and graphics `Queue` — tested against a live
Vulkan ICD (task 201). `context::windowed` additionally produces a
`VkSurfaceKHR` over a caller-supplied window and a real `VK_KHR_swapchain`
over it, with a per-frame acquire/present pair and resize-driven rebuild
(see [ADR-006](../../../docs/adr/006_vulkan_windowed_presentation.md)) —
the only route in this workspace to a windowed process that links no
`wgpu` at all. Resource construction (buffers, images, pipelines) is not
implemented here; reach it through `gpu_hal`'s Vulkan backend.

A window enters as `raw_window_handle` traits only — this crate depends on
no windowing library, and re-exports those traits as
`minvulkan::raw_window_handle` so a consumer need not name them itself.

## Documentation

Design documentation (features) lives in [`docs/`](docs/feature/readme.md).

## Directory Layout

| Path | Responsibility |
|------|-----------------|
| `src/` | Crate source — `Context`/`ContextBuilder`, window surface, swapchain, and the crate's error type |
| `tests/` | Integration tests exercising `Context::builder()` against a live Vulkan ICD, plus the surface's pure edges and swapchain rebuild's error-path cleanup |
| `docs/` | Design documentation as typed doc definitions — see [docs/feature/readme.md](docs/feature/readme.md) |
| `readme.md` | This file — user-facing entry point |
