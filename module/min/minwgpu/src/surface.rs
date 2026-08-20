//! This module provides helpers for configuring a `wgpu::Surface` for presentation to a
//! window — creating one from a window handle, picking a presentation format, building the
//! `wgpu::SurfaceConfiguration` applied on first setup and again on every resize, and
//! driving the per-frame acquire/present cycle.

use mingl::mod_interface;

mod private
{
  /// Creates a presentation surface for `window`.
  ///
  /// `window` is anything `wgpu` accepts as a surface target. In practice that is any type
  /// implementing both `raw_window_handle::HasWindowHandle` and `HasDisplayHandle`, which
  /// `wgpu` blanket-converts into a `wgpu::SurfaceTarget` — notably
  /// `Arc< winit::window::Window >`. Taking the handle traits rather than a concrete window
  /// type is what keeps this crate independent of any particular windowing library : `winit`
  /// is never a dependency here, and a consumer may use `sdl2`, `glfw`, or a raw handle
  /// instead.
  ///
  /// The returned surface is not yet configured — pass it to [`surface_configure`] before
  /// acquiring frames from it, or use [`crate::context::windowed`], which performs both
  /// steps together in the required order.
  ///
  /// # Errors
  /// Returns [`crate::Error::CreateSurfaceError`] when the platform declines to create a
  /// surface for this window ( an unsupported handle type, or a resource shortage ).
  pub fn from_window< 'w >
  (
    instance : &wgpu::Instance,
    window : impl Into< wgpu::SurfaceTarget< 'w > >,
  )
  -> Result< wgpu::Surface< 'w >, crate::Error >
  {
    Ok( instance.create_surface( window )? )
  }

  /// The outcome of one [`frame_acquire`] attempt.
  ///
  /// Collapses `wgpu::CurrentSurfaceTexture`'s seven arms into the three actions a render
  /// loop actually takes, so every consumer does not re-derive the same matching. `Suboptimal`
  /// folds into [`Frame::Ready`] rather than [`Frame::Reconfigure`] : the texture it carries is
  /// valid and drawable, so discarding it would drop a frame that a normal resize event will
  /// correct anyway.
  ///
  /// Deliberately exhaustive, unlike [`crate::Error`] : the whole purpose of this type is to
  /// be a closed, stable simplification of `wgpu`'s open-ended acquisition status, so a
  /// consumer gets a compile error rather than a silent wildcard if the set ever changes.
  #[ derive( Debug ) ]
  pub enum Frame
  {
    /// A drawable frame. Render into `view`, then hand `texture` to [`frame_present`].
    Ready
    {
      /// The acquired swapchain texture, presented by [`frame_present`].
      texture : wgpu::SurfaceTexture,
      /// A default view over `texture`, ready to use as a color attachment.
      view : wgpu::TextureView,
    },
    /// Nothing to draw right now — the window is occluded or acquisition timed out. Skip
    /// this frame and try again on the next tick; no reconfiguration is needed.
    Skip,
    /// The surface configuration is stale. Re-apply it with [`surface_configure`] ( or
    /// `Windowed::resize` ) and acquire again.
    Reconfigure,
  }

  /// Acquires the next frame from `surface`.
  ///
  /// # Errors
  /// Returns [`crate::Error::SurfaceAcquire`] when acquisition raised a validation error.
  /// The transient and stale outcomes are not errors — they are reported as [`Frame::Skip`]
  /// and [`Frame::Reconfigure`] respectively, because a render loop handles them by
  /// continuing rather than by failing.
  pub fn frame_acquire( surface : &wgpu::Surface< '_ > ) -> Result< Frame, crate::Error >
  {
    let ready = | texture : wgpu::SurfaceTexture |
    {
      let view = texture.texture.create_view( &wgpu::TextureViewDescriptor::default() );
      Frame::Ready { texture, view }
    };

    match surface.get_current_texture()
    {
      wgpu::CurrentSurfaceTexture::Success( texture )
      | wgpu::CurrentSurfaceTexture::Suboptimal( texture ) => Ok( ready( texture ) ),
      wgpu::CurrentSurfaceTexture::Timeout
      | wgpu::CurrentSurfaceTexture::Occluded => Ok( Frame::Skip ),
      wgpu::CurrentSurfaceTexture::Outdated
      | wgpu::CurrentSurfaceTexture::Lost => Ok( Frame::Reconfigure ),
      wgpu::CurrentSurfaceTexture::Validation => Err( crate::Error::SurfaceAcquire ),
    }
  }

  /// Presents a drawn frame to the screen.
  ///
  /// Call after submitting the commands that render into the frame's view. Named as the
  /// counterpart of [`frame_acquire`] so the pair reads as one lifecycle at the call site.
  ///
  /// Presentation is a `wgpu::Queue` operation in wgpu 30 ( it was `SurfaceTexture::present`
  /// in earlier releases ), so the queue is required here; taking it as a parameter keeps
  /// that migration inside this crate rather than at every call site.
  #[ inline ]
  pub fn frame_present( queue : &wgpu::Queue, texture : wgpu::SurfaceTexture )
  {
    queue.present( texture );
  }

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

  /// A `Context` bound to a window surface and its current configuration.
  ///
  /// Owns the three pieces that windowed rendering always needs together — context, surface,
  /// configuration — so a consumer holds one value instead of keeping them in sync by hand,
  /// and reaches the whole per-frame lifecycle through methods rather than through raw `wgpu`
  /// calls. Build it with [`Windowed::new`]; take the pieces back out with
  /// [`Windowed::into_parts`] when a lower-level path ( e.g. an L1 HAL backend ) needs them
  /// separately.
  #[ derive( Debug ) ]
  pub struct Windowed< 'w >
  {
    pub( super ) context : crate::context::Context,
    pub( super ) surface : wgpu::Surface< 'w >,
    pub( super ) config : wgpu::SurfaceConfiguration,
  }

  impl< 'w > Windowed< 'w >
  {
    /// Creates a context and a configured surface for `window`, on the primary backends.
    ///
    /// # Errors
    /// See [`crate::context::windowed`].
    #[ inline ]
    pub fn new( window : impl Into< wgpu::SurfaceTarget< 'w > >, size : ( u32, u32 ) )
    -> Result< Self, crate::Error >
    {
      Self::new_with( wgpu::Backends::PRIMARY, window, size )
    }

    /// Creates a context and a configured surface for `window`, on the given backends.
    ///
    /// # Errors
    /// See [`crate::context::windowed_with`].
    #[ inline ]
    pub fn new_with
    (
      backends : wgpu::Backends,
      window : impl Into< wgpu::SurfaceTarget< 'w > >,
      size : ( u32, u32 ),
    )
    -> Result< Self, crate::Error >
    {
      let ( context, surface, config ) = crate::context::windowed_with( backends, window, size )?;
      Ok( Self { context, surface, config } )
    }

    /// Returns a reference to the underlying `Context`.
    #[ inline ]
    #[ must_use ]
    pub fn context_get( &self ) -> &crate::context::Context
    {
      &self.context
    }

    /// Returns a reference to the `wgpu::Device`.
    #[ inline ]
    #[ must_use ]
    pub fn device_get( &self ) -> &wgpu::Device
    {
      self.context.device_get()
    }

    /// Returns a reference to the `wgpu::Queue`.
    #[ inline ]
    #[ must_use ]
    pub fn queue_get( &self ) -> &wgpu::Queue
    {
      self.context.queue_get()
    }

    /// Returns a reference to the presentation surface.
    #[ inline ]
    #[ must_use ]
    pub fn surface_get( &self ) -> &wgpu::Surface< 'w >
    {
      &self.surface
    }

    /// Returns the surface's current configuration.
    #[ inline ]
    #[ must_use ]
    pub fn config_get( &self ) -> &wgpu::SurfaceConfiguration
    {
      &self.config
    }

    /// Returns the surface's presentation format — the format a render pipeline's color
    /// target must match.
    #[ inline ]
    #[ must_use ]
    pub fn format( &self ) -> wgpu::TextureFormat
    {
      self.config.format
    }

    /// Returns the current drawable size as `( width, height )`.
    #[ inline ]
    #[ must_use ]
    pub fn size( &self ) -> ( u32, u32 )
    {
      ( self.config.width, self.config.height )
    }

    /// Re-applies the surface configuration at a new drawable size.
    ///
    /// # Errors
    /// Returns [`crate::Error::ZeroSizeSurface`] for a transient zero size ( e.g. reported
    /// while the window is minimized ), leaving the existing configuration untouched so the
    /// caller may simply skip the resize and keep rendering once the window is restored.
    #[ inline ]
    pub fn resize( &mut self, size : ( u32, u32 ) ) -> Result< (), crate::Error >
    {
      self.config = surface_configure
      (
        self.context.device_get(),
        self.context.adapter_get(),
        &self.surface,
        size,
      )?;
      Ok( () )
    }

    /// Acquires the next frame.
    ///
    /// # Errors
    /// See [`frame_acquire`].
    #[ inline ]
    pub fn frame_acquire( &self ) -> Result< Frame, crate::Error >
    {
      frame_acquire( &self.surface )
    }

    /// Presents a drawn frame to the screen.
    ///
    /// Counterpart of [`Windowed::frame_acquire`]; supplies this context's queue to
    /// [`frame_present`].
    #[ inline ]
    pub fn frame_present( &self, texture : wgpu::SurfaceTexture )
    {
      frame_present( self.context.queue_get(), texture );
    }

    /// Consumes this value, returning the context, surface and configuration separately.
    #[ inline ]
    #[ must_use ]
    pub fn into_parts( self )
    -> ( crate::context::Context, wgpu::Surface< 'w >, wgpu::SurfaceConfiguration )
    {
      let Self { context, surface, config } = self;
      ( context, surface, config )
    }
  }
}

mod_interface!
{
  own use preferred_format;
  own use validate_size;
  own use surface_configure;
  own use from_window;
  own use frame_acquire;
  own use frame_present;
  own use Frame;
  own use Windowed;
}
