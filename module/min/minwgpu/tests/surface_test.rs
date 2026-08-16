//! Native tests for the pure format-selection logic behind `surface::surface_configure`.
//!
//! The GPU half of surface configuration ( `get_default_config`, `configure` ) needs a real
//! adapter, device, and window-backed surface, and is exercised by the native
//! `examples/minwgpu/flecs_bouncing_circles` binary; the format-selection logic is pure and
//! is pinned here.

use minwgpu::{ surface::{ preferred_format, validate_size }, Error };

/// Given a mix of sRGB and non-sRGB formats, the first sRGB-encoded one wins regardless of
/// its position in the list.
#[ test ]
fn preferred_format_picks_first_srgb_when_present()
{
  let available =
  [
    wgpu::TextureFormat::Bgra8Unorm,
    wgpu::TextureFormat::Bgra8UnormSrgb,
    wgpu::TextureFormat::Rgba8UnormSrgb,
  ];
  assert_eq!( preferred_format( &available ), wgpu::TextureFormat::Bgra8UnormSrgb );
}

/// With no sRGB format reported at all, the first format in the list is used as a fallback
/// rather than panicking or picking arbitrarily.
#[ test ]
fn preferred_format_falls_back_to_first_when_no_srgb_present()
{
  let available = [ wgpu::TextureFormat::Bgra8Unorm, wgpu::TextureFormat::Rgba8Unorm ];
  assert_eq!( preferred_format( &available ), wgpu::TextureFormat::Bgra8Unorm );
}

/// A single-format list ( the minimum `wgpu` ever reports for a real surface ) returns that
/// one format, sRGB or not.
#[ test ]
fn preferred_format_single_element_returns_that_element()
{
  let available = [ wgpu::TextureFormat::Rgba16Float ];
  assert_eq!( preferred_format( &available ), wgpu::TextureFormat::Rgba16Float );
}

// test_kind: bug_reproducer(BUG-165)
/// ## Root Cause
/// `surface_configure` forwarded its `size` argument straight to `wgpu::Surface::configure`
/// with no precondition check, panicking ( `wgpu-core`'s `ConfigureSurfaceError::ZeroArea`,
/// surfaced through `wgpu`'s default uncaptured-error handler since this crate never installs
/// a custom one ) whenever either dimension was `0` -- a normal, reachable resize outcome
/// ( e.g. a minimized window ), not a caller bug.
/// ## Why Not Caught
/// No test called `surface_configure` ( or any precondition check backing it ) with a zero
/// size; the only prior defense against this was a hand-written `width == 0 || height == 0`
/// guard in `examples/minwgpu/flecs_bouncing_circles`'s own resize handler, which worked
/// around the panic at the call site instead of the library ever asserting it can't happen.
/// ## Fix Applied
/// `surface_configure` now calls a new `validate_size` precondition check before touching
/// `wgpu` at all, returning `Error::ZeroSizeSurface` instead of forwarding a zero size.
/// `validate_size` is split out specifically so this precondition is unit-testable without a
/// real GPU adapter/device/surface, matching this file's own pure-logic-only testing scope.
/// ## Prevention
/// This test calls `validate_size` directly with every zero-containing combination of a
/// `( width, height )` pair and asserts each returns `Err( Error::ZeroSizeSurface )`.
/// ## Pitfall
/// An "idempotent-safe, call again on every resize" contract invites exactly the kind of
/// resize input ( a transient zero size ) that the underlying GPU API panics on -- a
/// resize-shaped function must validate the resize size itself, not assume every caller will
/// independently discover and guard the same edge case.
#[ test ]
fn validate_size_rejects_zero_width_or_height()
{
  assert!( matches!( validate_size( ( 0, 512 ) ), Err( Error::ZeroSizeSurface( 0, 512 ) ) ) );
  assert!( matches!( validate_size( ( 512, 0 ) ), Err( Error::ZeroSizeSurface( 512, 0 ) ) ) );
  assert!( matches!( validate_size( ( 0, 0 ) ), Err( Error::ZeroSizeSurface( 0, 0 ) ) ) );
}

/// A non-zero size in both dimensions passes validation, matching the ordinary case
/// `surface_configure` handles on every normal ( non-minimized ) resize.
#[ test ]
fn validate_size_accepts_nonzero_width_and_height()
{
  assert!( validate_size( ( 512, 512 ) ).is_ok() );
  assert!( validate_size( ( 1, 1 ) ).is_ok() );
}
