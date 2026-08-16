//! This module provides helpers for configuring a `wgpu::Surface` for presentation to a
//! window — picking a presentation format and building the `wgpu::SurfaceConfiguration`
//! applied on first setup and again on every resize.

use mingl::mod_interface;

mod private
{
  /// Picks the preferred surface format from a surface's reported supported formats.
  ///
  /// Prefers the first sRGB-encoded format so a shader that writes linear-space color (as if
  /// targeting an `Rgba8UnormSrgb` offscreen texture) is displayed with the same gamma
  /// correction applied on present; `wgpu::Surface::get_default_config`'s own format pick is
  /// only "the first format the backend reports," which is not guaranteed to be sRGB. Falls
  /// back to the first supported format when none of the reported formats are sRGB-encoded.
  ///
  /// # Panics
  /// Panics if `available` is empty. `wgpu` never reports an empty format list for a real
  /// adapter/surface pair, so an empty slice here indicates a caller bug, not a recoverable
  /// runtime condition.
  #[ must_use ]
  pub fn preferred_format( available : &[ wgpu::TextureFormat ] ) -> wgpu::TextureFormat
  {
    available.iter().copied().find( wgpu::TextureFormat::is_srgb )
    .unwrap_or_else( || available[ 0 ] )
  }

  // Fix(BUG-165): `surface_configure` used to forward `size` straight to `wgpu` with no
  // precondition check, panicking when either dimension was `0`.
  // Root cause: `wgpu::Surface::configure` panics on a zero-area configuration
  // (`wgpu-core`'s `ConfigureSurfaceError::ZeroArea`, surfaced through `wgpu`'s default
  // uncaptured-error handler since this crate never installs a custom one) -- a `0×0` resize is
  // a normal, reachable event (e.g. a minimized window), not a caller bug. Discovered because
  // the crate's own `examples/minwgpu/flecs_bouncing_circles` already had to hand-write a
  // `width == 0 || height == 0` guard before its own resize call site to avoid this exact panic.
  // Pitfall: an "idempotent-safe, call again on every resize" contract invites exactly the kind
  // of resize input (a transient zero size) that the underlying GPU API panics on -- a
  // resize-shaped function must validate the resize size itself, not assume every caller
  // will independently discover and guard the same edge case.
  /// Validates that `size` is non-zero in both dimensions.
  ///
  /// Split out of [`surface_configure`] so this precondition is unit-testable without a real
  /// GPU adapter/device/surface -- see `tests/surface_test.rs`.
  ///
  /// # Errors
  /// Returns [`crate::Error::ZeroSizeSurface`] when either `width` or `height` is `0`.
  pub fn validate_size( size : ( u32, u32 ) ) -> Result< (), crate::Error >
  {
    let ( width, height ) = size;
    if width == 0 || height == 0
    {
      return Err( crate::Error::ZeroSizeSurface( width, height ) );
    }
    Ok( () )
  }

  /// Builds a `wgpu::SurfaceConfiguration` for `size`, applies it via `surface.configure`,
  /// and returns it.
  ///
  /// Starts from `wgpu::Surface::get_default_config` (usage, color space, present mode,
  /// frame latency, alpha mode, view formats all left at `wgpu`'s own sensible defaults), then
  /// substitutes [`preferred_format`]'s sRGB pick for the format field.
  ///
  /// Call again on every resize: `wgpu` requires a fresh `configure` any time the drawable
  /// size changes, and this function is deliberately idempotent-safe for that purpose — the
  /// same call that performs first-time setup also performs a resize, just with a new `size`.
  ///
  /// # Errors
  /// Returns [`crate::Error::ZeroSizeSurface`] for a transient `(0, height)`/`(width, 0)` size
  /// (e.g. reported while a window is minimized) instead of forwarding it to `wgpu`, which
  /// would otherwise panic (BUG-165).
  ///
  /// # Panics
  /// Panics if `surface` is incompatible with `adapter` (`get_default_config` returns `None`) —
  /// this indicates a caller bug (the surface was never checked against this adapter, e.g. via
  /// `compatible_surface` during adapter selection) rather than a recoverable runtime condition.
  pub fn surface_configure
  (
    device : &wgpu::Device,
    adapter : &wgpu::Adapter,
    surface : &wgpu::Surface< '_ >,
    size : ( u32, u32 ),
  ) -> Result< wgpu::SurfaceConfiguration, crate::Error >
  {
    validate_size( size )?;
    let ( width, height ) = size;
    let mut config = surface.get_default_config( adapter, width, height )
    .expect( "surface must be compatible with the adapter used to configure it" );
    config.format = preferred_format( &surface.get_capabilities( adapter ).formats );
    surface.configure( device, &config );
    Ok( config )
  }
}

mod_interface!
{
  own use preferred_format;
  own use validate_size;
  own use surface_configure;
}
