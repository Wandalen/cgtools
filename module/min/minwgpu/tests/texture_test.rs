//! Native tests for the pure size-validation logic behind `texture::render_target_2d`.
//!
//! The GPU half of render-target creation ( `create_texture`, `create_view`, `create_sampler` )
//! needs a real device and is exercised by the native `examples/minwgpu` binaries; the
//! zero-size precondition check is pure and is pinned here.

use minwgpu::texture::is_nonzero_size;

/// A non-zero size in both dimensions passes validation, matching the ordinary case
/// `render_target_2d` handles for any real render target.
#[ test ]
fn is_nonzero_size_accepts_nonzero_width_and_height()
{
  assert!( is_nonzero_size( ( 512, 512 ) ) );
  assert!( is_nonzero_size( ( 1, 1 ) ) );
}

// test_kind: bug_reproducer(BUG-276)
/// ## Root Cause
/// `render_target_2d` forwarded its `size` argument straight to `wgpu::Device::create_texture`
/// with no precondition check, panicking ( `wgpu-core`'s `CreateTextureError::InvalidDimension(
/// TextureDimensionError::Zero(..))`, surfaced through `wgpu`'s default uncaptured-error handler
/// -- `panic!("wgpu error: {err}")` in `wgpu-core`'s `backend/wgpu_core.rs`, confirmed by reading
/// that source directly, since this crate never installs a custom handler -- whenever either
/// dimension was `0`. Same defect class BUG-165 fixed for `surface::surface_configure`, but via
/// a separate call path BUG-165's own fix never covered.
/// ## Why Not Caught
/// No test called `render_target_2d` ( or any precondition check backing it ) with a zero size;
/// BUG-165's fix and tests were scoped to `surface_configure` only, and no follow-up swept
/// sibling call paths taking the same shape of caller-supplied `( u32, u32 )` size.
/// ## Fix Applied
/// `render_target_2d` now asserts a new `is_nonzero_size` precondition check before touching
/// `wgpu` at all, panicking immediately with a clear, crate-authored message instead of letting
/// the caller hit `wgpu-core`'s opaque validation panic several layers down. `is_nonzero_size`
/// is split out specifically so this precondition is unit-testable without a real GPU device,
/// matching this file's own pure-logic-only testing scope ( see module doc comment ). Full
/// recoverable-`Result` treatment ( matching `surface_configure`'s `Error::ZeroSizeSurface` )
/// would need a new `crate::Error` variant in `error.rs`, out of scope for this fix.
/// ## Prevention
/// This test calls `is_nonzero_size` directly with every zero-containing combination of a
/// `( width, height )` pair and asserts each returns `false`.
/// ## Pitfall
/// Fixing one unguarded call path into a `wgpu` API that panics on zero-sized input does not
/// protect a sibling call path taking the same shape of input -- `surface_configure` (BUG-165)
/// and `render_target_2d` both accept a caller-supplied `( u32, u32 )` size with no shared
/// validation chokepoint between them, so each needed its own guard.
#[ test ]
fn is_nonzero_size_rejects_zero_width_or_height()
{
  assert!( !is_nonzero_size( ( 0, 512 ) ) );
  assert!( !is_nonzero_size( ( 512, 0 ) ) );
  assert!( !is_nonzero_size( ( 0, 0 ) ) );
}
