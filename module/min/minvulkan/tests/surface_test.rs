//!
//! Tests for the parts of the windowed presentation path reachable without a
//! real window : `surface::preferred_format`'s pure selection logic, and the
//! drop-order invariant `Windowed`'s field declaration order encodes.
//!
//! Everything else in `surface.rs`/`swapchain.rs` — `Surface::from_window`,
//! `Swapchain::new`/`frame_acquire`/`frame_present`, `Windowed`'s accessors —
//! needs a live `VkSurfaceKHR`, which needs a real window handle this crate
//! deliberately cannot produce ( it depends on no windowing library, per
//! `docs/adr/005_windowed_native_presentation.md` ). That half is covered
//! end-to-end by `examples/gpu_hal/triangle_vulkan_window`, the same way
//! `minwgpu`'s own windowed path is covered by
//! `examples/minwgpu/flecs_bouncing_circles` rather than by its unit tests.
//!

use minvulkan::{ surface::preferred_format, Error };

/// Builds a surface format pair with the color space every swapchain uses.
fn format_of( format : ash::vk::Format ) -> ash::vk::SurfaceFormatKHR
{
  ash::vk::SurfaceFormatKHR
  {
    format,
    color_space : ash::vk::ColorSpaceKHR::SRGB_NONLINEAR,
  }
}

/// T01 : given a mix of sRGB and non-sRGB formats, the first sRGB-encoded one
/// wins regardless of its position in the list.
#[ test ]
fn preferred_format_picks_first_srgb_when_present()
{
  let available =
  [
    format_of( ash::vk::Format::B8G8R8A8_UNORM ),
    format_of( ash::vk::Format::B8G8R8A8_SRGB ),
    format_of( ash::vk::Format::R8G8B8A8_SRGB ),
  ];
  let picked = preferred_format( &available ).expect( "a non-empty list always yields a format" );
  assert_eq!( picked.format, ash::vk::Format::B8G8R8A8_SRGB );
}

/// T02 : with no sRGB format reported at all, the first format in the list is
/// used as a fallback rather than panicking or picking arbitrarily.
#[ test ]
fn preferred_format_falls_back_to_first_when_no_srgb_present()
{
  let available =
  [
    format_of( ash::vk::Format::B8G8R8A8_UNORM ),
    format_of( ash::vk::Format::R8G8B8A8_UNORM ),
  ];
  let picked = preferred_format( &available ).expect( "a non-empty list always yields a format" );
  assert_eq!( picked.format, ash::vk::Format::B8G8R8A8_UNORM );
}

/// T03 : a single-format list ( the minimum a real surface ever reports )
/// returns that one format, sRGB or not.
#[ test ]
fn preferred_format_single_element_returns_that_element()
{
  let available = [ format_of( ash::vk::Format::R16G16B16A16_SFLOAT ) ];
  let picked = preferred_format( &available ).expect( "a non-empty list always yields a format" );
  assert_eq!( picked.format, ash::vk::Format::R16G16B16A16_SFLOAT );
}

/// T04 : an empty list is an error rather than a panic or a silent default —
/// it means the driver query itself returned nothing, which no format choice
/// can paper over.
#[ test ]
fn preferred_format_errors_on_empty_list()
{
  let result = preferred_format( &[] );
  assert!
  (
    matches!( result, Err( Error::NoSurfaceFormat ) ),
    "an empty format list must report NoSurfaceFormat, got {result:?}"
  );
}

/// T05 : the color space of the chosen pair is carried through untouched —
/// selection is on `format` alone, and swapping in the wrong color space would
/// silently double-apply ( or skip ) gamma correction on every presented frame.
#[ test ]
fn preferred_format_carries_color_space_through()
{
  let available =
  [
    ash::vk::SurfaceFormatKHR
    {
      format : ash::vk::Format::B8G8R8A8_SRGB,
      color_space : ash::vk::ColorSpaceKHR::DISPLAY_P3_NONLINEAR_EXT,
    },
  ];
  let picked = preferred_format( &available ).expect( "a non-empty list always yields a format" );
  assert_eq!( picked.color_space, ash::vk::ColorSpaceKHR::DISPLAY_P3_NONLINEAR_EXT );
}

/// T06 : `Windowed` declares its fields swapchain-before-surface-before-context.
///
/// Rust drops struct fields in declaration order, and Vulkan requires a
/// swapchain destroyed before the surface it presents to, and that surface
/// destroyed before the instance that created it. Reordering these three
/// fields is a one-line, innocuous-looking edit that breaks nothing at compile
/// time and produces a validation error only once a real window is attached —
/// which no test in this crate can attach. Asserting the order at the source
/// level is what makes the invariant enforceable here at all, the same
/// approach `context_test.rs`'s BUG-290 regression test takes for its own
/// cleanup-only defect.
#[ test ]
fn windowed_field_order_enforces_vulkan_destruction_order()
{
  let src = include_str!( "../src/surface.rs" );
  let ( _, after_declaration ) = src.split_once( "pub struct Windowed" )
  .expect( "surface.rs must declare `pub struct Windowed`" );
  let ( body, _ ) = after_declaration.split_once( '}' )
  .expect( "the `Windowed` declaration must have a closing brace" );

  let swapchain = body.find( "swapchain :" ).expect( "`Windowed` must have a `swapchain` field" );
  let surface = body.find( "surface :" ).expect( "`Windowed` must have a `surface` field" );
  let context = body.find( "context :" ).expect( "`Windowed` must have a `context` field" );

  assert!
  (
    swapchain < surface,
    "`swapchain` must be declared before `surface` -- Rust drops fields in declaration order, \
     and vkDestroySwapchainKHR must precede vkDestroySurfaceKHR"
  );
  assert!
  (
    surface < context,
    "`surface` must be declared before `context` -- vkDestroySurfaceKHR must precede the \
     vkDestroyInstance that `Context`'s own Drop performs"
  );
}
