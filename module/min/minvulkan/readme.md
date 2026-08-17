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
Vulkan ICD (task 201). Surface/swapchain presentation and resource
construction (buffers, images, pipelines) are not yet implemented.

## Documentation

Design documentation (features) lives in [`docs/`](docs/feature/readme.md).

## Directory Layout

| Path | Responsibility |
|------|-----------------|
| `src/` | Crate source — `Context`/`ContextBuilder` and the crate's error type |
| `tests/` | Integration tests exercising `Context::builder()` against a live Vulkan ICD |
| `docs/` | Design documentation as typed doc definitions — see [docs/feature/readme.md](docs/feature/readme.md) |
| `readme.md` | This file — user-facing entry point |
