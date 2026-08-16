//! Native tests for the pure format-selection logic behind `surface::surface_configure`.
//!
//! The GPU half of surface configuration ( `get_default_config`, `configure` ) needs a real
//! adapter, device, and window-backed surface, and is exercised by the native
//! `examples/minwgpu/flecs_bouncing_circles` binary; the format-selection logic is pure and
//! is pinned here.

use minwgpu::surface::preferred_format;

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
