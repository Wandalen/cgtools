
//!
//! # `minvulkan`
//!
//! Minimal, opinionated native Vulkan toolkit via `ash`. It is the
//! `wgpu`-free counterpart to `minwgpu` — the same thin, backend-faithful
//! wrapper role at L0 (see `docs/layer/001_l0_drivers.md`), targeting raw
//! Vulkan directly instead of through `wgpu`.
//!
//! Introduced by ADR-004 (`docs/adr/004_native_vulkan_hal_backend.md`);
//! the initial `Context`/`ContextBuilder` slice is tracked by task 201.
//!

#![ allow( unsafe_code, reason = "native Vulkan FFI driver crate -- every `ash` call that touches \
the Vulkan API is inherently unsafe ; each call site carries its own `// SAFETY:` comment rather \
than repeating this justification at every one of them" ) ]

use mingl::mod_interface;

mod private {}

mod_interface!
{
  /// The raw window handle traits, re-exported so a consumer reaches them
  /// through this driver rather than naming a second, independently versioned
  /// copy in its own manifest — the same reason `minwgpu` re-exports `wgpu`.
  own use ::raw_window_handle;

  layer context;
  layer error;
  layer surface;
  layer swapchain;
}
