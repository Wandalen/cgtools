# Feature: Surface Configuration

`minwgpu` provides two small helpers (`preferred_format` / `surface_configure`) for configuring a `wgpu::Surface` used to present to a window — the piece of setup every native, windowed `wgpu` application needs once at startup and again on every resize, on top of the `Context` built by [feature/001_context_builder.md](001_context_builder.md).

### Scope

- **Purpose**: Pick a sensible presentation format and build/apply a `wgpu::SurfaceConfiguration` without hand-rolling the full descriptor at each call site.
- **Responsibility**: Cross-reference the source and tests that make up surface format selection and configuration.
- **In Scope**: `preferred_format`, `surface_configure`.
- **Out of Scope**: Window/event-loop creation (owned by the consuming native binary, e.g. via `winit` — `minwgpu` itself has no `winit` dependency); `wgpu` context/device/queue setup (see [feature/001_context_builder.md](001_context_builder.md)).

### Design

`preferred_format( available )` picks the first sRGB-encoded format out of a surface's reported supported formats, falling back to the first format when none are sRGB. This matters because `wgpu::Surface::get_default_config`'s own format pick is only "the first format the backend reports" — not guaranteed to be sRGB — so a shader written to output linear-space color (as if targeting an `Rgba8UnormSrgb` offscreen texture) needs the caller to override that default explicitly to get the same gamma-corrected result on screen. Panics if `available` is empty, since `wgpu` never reports an empty format list for a real adapter/surface pair — an empty slice indicates a caller bug, not a recoverable runtime condition.

`surface_configure( device, adapter, surface, size )` starts from `wgpu::Surface::get_default_config` (usage, color space, present mode, frame latency, alpha mode, and view formats all left at `wgpu`'s own defaults), substitutes `preferred_format`'s pick for the format field, applies the result via `surface.configure(..)`, and returns the built `wgpu::SurfaceConfiguration` so the caller can retain it (e.g. to know the current format when building a render pipeline's color target). Intended to be called once at startup and again on every resize — `wgpu` requires a fresh `configure` any time the drawable size changes, and the function is deliberately idempotent-safe for that purpose. Panics if `surface` is incompatible with `adapter` (`get_default_config` returns `None`), which indicates the surface was never checked against this adapter (e.g. via `compatible_surface` during adapter selection) rather than a recoverable runtime condition.

### Sources

| File | Relationship |
|------|--------------|
| `src/surface.rs` | `preferred_format` / `surface_configure` implementation |

### Tests

| File | Relationship |
|------|--------------|
| `tests/surface_test.rs` | `preferred_format`'s sRGB-preference and fallback logic across sRGB-present, sRGB-absent, and single-element format lists — the GPU half of `surface_configure` needs a real adapter/device/window-backed surface and is exercised by the native `examples/minwgpu/flecs_bouncing_circles` binary instead |
