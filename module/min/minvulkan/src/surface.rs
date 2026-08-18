//!
//! `VK_KHR_surface` : the presentation surface of a window, the queries a
//! swapchain is built from, and the owning [`Windowed`] value that binds a
//! context, a surface and a swapchain together.
//!
//! A window enters through the raw handle traits and nothing else — this crate
//! never depends on `winit`, `sdl2` or `glfw`, exactly as `minwgpu` does not
//! ( see `docs/adr/005_windowed_native_presentation.md` ).
//!

use mingl::mod_interface;

mod private
{
  use crate::Error;

  /// A `VkSurfaceKHR` over a window, plus the `VK_KHR_surface` entry points
  /// needed to query and destroy it.
  ///
  /// Destroy after every [`crate::swapchain::Swapchain`] built over it and
  /// before the [`crate::context::Context`] whose instance created it.
  pub struct Surface
  {
    pub( super ) loader : ash::khr::surface::Instance,
    pub( super ) handle : ash::vk::SurfaceKHR,
  }

  impl core::fmt::Debug for Surface
  {
    fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
    {
      f.debug_struct( "Surface" ).finish_non_exhaustive()
    }
  }

  impl Surface
  {
    /// Creates a presentation surface for `window`.
    ///
    /// `window` is anything implementing both `raw_window_handle::HasWindowHandle`
    /// and `HasDisplayHandle` — notably `winit::window::Window`. Taking the handle
    /// traits rather than a concrete window type is what keeps this crate
    /// independent of any particular windowing library.
    ///
    /// The instance must already have been created with the extensions
    /// [`required_instance_extensions`] reports for the same display, which
    /// [`crate::context::windowed`] does.
    ///
    /// # Errors
    /// Returns [`Error::WindowHandle`] when the window declines to hand out a
    /// raw handle, or [`Error::SurfaceCreate`] when the platform's own
    /// `vkCreate*SurfaceKHR` fails.
    pub fn from_window
    (
      entry : &ash::Entry,
      instance : &ash::Instance,
      window : &( impl raw_window_handle::HasDisplayHandle + raw_window_handle::HasWindowHandle ),
    )
    -> Result< Self, Error >
    {
      let display_handle = window.display_handle().map_err( Error::WindowHandle )?.as_raw();
      let window_handle = window.window_handle().map_err( Error::WindowHandle )?.as_raw();

      // SAFETY: `entry` and `instance` are live and belong together, and the two
      // handles were just obtained from `window`, which outlives this call. The
      // resulting surface must not outlive `window` — a requirement this crate
      // discharges by tying it to the caller-held window through `Windowed`.
      let handle = unsafe
      {
        ash_window::create_surface( entry, instance, display_handle, window_handle, None )
      }
      .map_err( Error::SurfaceCreate )?;

      Ok( Self { loader : ash::khr::surface::Instance::new( entry, instance ), handle } )
    }

    /// The raw `VkSurfaceKHR`.
    #[ inline ]
    #[ must_use ]
    pub fn handle_get( &self ) -> ash::vk::SurfaceKHR
    {
      self.handle
    }

    /// The surface's capabilities on `physical_device` — extent bounds, image
    /// count bounds, and the transform a swapchain must declare.
    ///
    /// # Errors
    /// Returns [`Error::SurfaceCapabilities`] when the query fails.
    pub fn capabilities( &self, physical_device : ash::vk::PhysicalDevice )
    -> Result< ash::vk::SurfaceCapabilitiesKHR, Error >
    {
      // SAFETY: `handle` is this surface's live handle and `physical_device` was
      // enumerated from the same instance `loader` was created from.
      unsafe { self.loader.get_physical_device_surface_capabilities( physical_device, self.handle ) }
      .map_err( Error::SurfaceCapabilities )
    }

    /// The ( format, color space ) pairs the surface supports on
    /// `physical_device`.
    ///
    /// # Errors
    /// Returns [`Error::SurfaceCapabilities`] when the query fails.
    pub fn formats( &self, physical_device : ash::vk::PhysicalDevice )
    -> Result< Vec< ash::vk::SurfaceFormatKHR >, Error >
    {
      // SAFETY: as `capabilities` above.
      unsafe { self.loader.get_physical_device_surface_formats( physical_device, self.handle ) }
      .map_err( Error::SurfaceCapabilities )
    }

    /// Whether `queue_family_index` on `physical_device` can present to this
    /// surface — the filter a windowed context's device selection must apply on
    /// top of the graphics-capability one.
    ///
    /// # Errors
    /// Returns [`Error::SurfaceCapabilities`] when the query fails.
    pub fn present_supported
    (
      &self,
      physical_device : ash::vk::PhysicalDevice,
      queue_family_index : u32,
    )
    -> Result< bool, Error >
    {
      // SAFETY: as `capabilities` above ; `queue_family_index` is bounds-checked by
      // the caller against this device's own reported family count.
      unsafe
      {
        self.loader.get_physical_device_surface_support( physical_device, queue_family_index, self.handle )
      }
      .map_err( Error::SurfaceCapabilities )
    }
  }

  impl Drop for Surface
  {
    fn drop( &mut self )
    {
      // SAFETY: `handle` is owned exclusively by this value ( `Surface` is not
      // `Clone` ), has not been destroyed before this point, and every swapchain
      // built over it is destroyed first — `Windowed`'s field order enforces that,
      // and `into_parts`' doc comment states it for a caller holding the parts
      // separately.
      unsafe { self.loader.destroy_surface( self.handle, None ); }
    }
  }

  /// The instance extensions a surface for `window`'s display requires —
  /// `VK_KHR_surface` plus the platform-specific one.
  ///
  /// Must be passed to `vkCreateInstance`, which is why this is a free function
  /// queried from a display handle alone : the instance does not exist yet.
  ///
  /// # Errors
  /// Returns [`Error::WindowHandle`] when the window declines to hand out a raw
  /// display handle, or [`Error::SurfaceCreate`] when the platform is one
  /// `ash-window` has no surface extension for.
  pub fn required_instance_extensions
  (
    window : &impl raw_window_handle::HasDisplayHandle,
  )
  -> Result< &'static [ *const core::ffi::c_char ], Error >
  {
    let display_handle = window.display_handle().map_err( Error::WindowHandle )?.as_raw();
    ash_window::enumerate_required_extensions( display_handle )
    .map_err( Error::SurfaceCreate )
  }

  /// Picks the preferred surface format from a surface's reported supported
  /// formats.
  ///
  /// Prefers the first sRGB-encoded format, so a shader writing linear-space
  /// color is gamma-corrected on present — the same choice
  /// `minwgpu::surface::preferred_format` makes, for the same reason. Falls back
  /// to the first reported format when none is sRGB-encoded.
  ///
  /// # Errors
  /// Returns [`Error::NoSurfaceFormat`] when `available` is empty. Unlike its
  /// `minwgpu` counterpart this is an error rather than a panic : the list comes
  /// straight from a driver query here, not from an infallible `wgpu` capability
  /// struct.
  pub fn preferred_format( available : &[ ash::vk::SurfaceFormatKHR ] )
  -> Result< ash::vk::SurfaceFormatKHR, Error >
  {
    available
    .iter()
    .copied()
    .find( | format | format_is_srgb( format.format ) )
    .or_else( || available.first().copied() )
    .ok_or( Error::NoSurfaceFormat )
  }

  /// Whether a `VkFormat` carries sRGB encoding — the 8-bit color formats a
  /// swapchain realistically reports.
  fn format_is_srgb( format : ash::vk::Format ) -> bool
  {
    matches!
    (
      format,
      ash::vk::Format::R8G8B8A8_SRGB
      | ash::vk::Format::B8G8R8A8_SRGB
      | ash::vk::Format::A8B8G8R8_SRGB_PACK32
      | ash::vk::Format::R8G8B8_SRGB
      | ash::vk::Format::B8G8R8_SRGB
    )
  }

  /// A [`crate::context::Context`] bound to a window surface and its swapchain.
  ///
  /// Owns the three pieces windowed rendering always needs together, so a
  /// consumer holds one value instead of keeping them in sync by hand — the
  /// direct counterpart of `minwgpu::surface::Windowed`.
  ///
  /// Field order is load-bearing : Rust drops fields in declaration order, and
  /// Vulkan requires the swapchain destroyed before the surface and the surface
  /// before the instance that created it.
  #[ derive( Debug ) ]
  pub struct Windowed
  {
    pub( super ) swapchain : crate::swapchain::Swapchain,
    pub( super ) surface : Surface,
    pub( super ) context : crate::context::Context,
  }

  impl Windowed
  {
    /// Creates a context, a surface for `window`, and a swapchain at `size`.
    ///
    /// # Errors
    /// See [`crate::context::windowed`].
    #[ inline ]
    pub fn new
    (
      window : &( impl raw_window_handle::HasDisplayHandle + raw_window_handle::HasWindowHandle ),
      size : ( u32, u32 ),
    )
    -> Result< Self, Error >
    {
      let ( context, surface, swapchain ) = crate::context::windowed( window, size )?;
      Ok( Self { swapchain, surface, context } )
    }

    /// Returns a reference to the underlying `Context`.
    #[ inline ]
    #[ must_use ]
    pub fn context_get( &self ) -> &crate::context::Context
    {
      &self.context
    }

    /// Returns a reference to the presentation surface.
    #[ inline ]
    #[ must_use ]
    pub fn surface_get( &self ) -> &Surface
    {
      &self.surface
    }

    /// Returns a reference to the swapchain.
    #[ inline ]
    #[ must_use ]
    pub fn swapchain_get( &self ) -> &crate::swapchain::Swapchain
    {
      &self.swapchain
    }

    /// The swapchain's presentation format — the format a render pipeline's
    /// color attachment must match.
    #[ inline ]
    #[ must_use ]
    pub fn format( &self ) -> ash::vk::Format
    {
      self.swapchain.format()
    }

    /// The current drawable size as `( width, height )`.
    #[ inline ]
    #[ must_use ]
    pub fn size( &self ) -> ( u32, u32 )
    {
      let extent = self.swapchain.extent();
      ( extent.width, extent.height )
    }

    /// Rebuilds the swapchain at a new drawable size.
    ///
    /// # Errors
    /// See [`crate::swapchain::Swapchain::resize`].
    #[ inline ]
    pub fn resize( &mut self, size : ( u32, u32 ) ) -> Result< (), Error >
    {
      self.swapchain.resize( &self.context, &self.surface, size )
    }

    /// Acquires the next frame.
    ///
    /// # Errors
    /// See [`crate::swapchain::Swapchain::frame_acquire`].
    #[ inline ]
    pub fn frame_acquire( &self ) -> Result< crate::swapchain::Frame, Error >
    {
      self.swapchain.frame_acquire()
    }

    /// Presents the frame at `index` on this context's own graphics queue.
    ///
    /// # Errors
    /// See [`crate::swapchain::Swapchain::frame_present`].
    #[ inline ]
    pub fn frame_present( &self, index : u32 ) -> Result< bool, Error >
    {
      self.swapchain.frame_present( self.context.queue_get(), index )
    }

    /// Consumes this value, returning the swapchain, surface and context
    /// separately.
    ///
    /// The caller takes over the destruction ordering `Windowed`'s own field
    /// order otherwise enforces : drop the swapchain first, then the surface,
    /// then the context. Returning them in that order makes the correct
    /// discipline the natural one — binding the tuple to three locals drops them
    /// in exactly that sequence.
    #[ inline ]
    #[ must_use ]
    pub fn into_parts( self ) -> ( crate::swapchain::Swapchain, Surface, crate::context::Context )
    {
      let Self { swapchain, surface, context } = self;
      ( swapchain, surface, context )
    }
  }
}

mod_interface!
{
  own use Surface;
  own use Windowed;
  own use required_instance_extensions;
  own use preferred_format;
}
