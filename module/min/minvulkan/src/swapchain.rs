//!
//! `VK_KHR_swapchain` presentation : the chain of presentable images a window
//! surface hands out, the per-frame acquire/present cycle over them, and the
//! recreation that a resize requires.
//!
//! Counterpart of `minwgpu::surface`'s frame lifecycle, one layer lower : where
//! `wgpu` hides the image ring behind `get_current_texture`, Vulkan exposes it,
//! so this module owns the images, their views, and the fence acquisition
//! signals.
//!

use mingl::mod_interface;

mod private
{
  use crate::Error;

  /// A presentable image chain over a window surface, plus the views and the
  /// acquisition fence a render loop needs to drive it.
  ///
  /// Destroy before the [`crate::surface::Surface`] it was created for, and
  /// before the [`crate::context::Context`] that owns the device — [`Drop`]
  /// handles this crate's own ordering, but a caller holding all three
  /// separately ( e.g. after [`crate::surface::Windowed::into_parts`] ) is
  /// responsible for dropping them swapchain-first.
  pub struct Swapchain
  {
    pub( super ) loader : ash::khr::swapchain::Device,
    pub( super ) device : ash::Device,
    pub( super ) handle : ash::vk::SwapchainKHR,
    pub( super ) views : Vec< ash::vk::ImageView >,
    pub( super ) images : Vec< ash::vk::Image >,
    pub( super ) format : ash::vk::Format,
    pub( super ) extent : ash::vk::Extent2D,
    pub( super ) acquire_fence : ash::vk::Fence,
  }

  impl core::fmt::Debug for Swapchain
  {
    fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
    {
      f.debug_struct( "Swapchain" )
      .field( "format", &self.format )
      .field( "extent", &self.extent )
      .field( "images", &self.images.len() )
      .finish_non_exhaustive()
    }
  }

  /// The outcome of one [`Swapchain::frame_acquire`] attempt.
  ///
  /// Two arms rather than `minwgpu::surface::Frame`'s three : acquisition here
  /// uses an infinite timeout, so "no image available yet" blocks instead of
  /// returning, and there is no `Skip` outcome to report.
  ///
  /// Deliberately exhaustive, for the same reason as its `minwgpu` counterpart :
  /// it is a closed simplification of Vulkan's open-ended `VkResult`, so a
  /// consumer gets a compile error rather than a silent wildcard if the set
  /// ever changes.
  #[ derive( Debug, Clone, Copy ) ]
  pub enum Frame
  {
    /// A drawable frame. Render into `view`, then hand `index` back to
    /// [`Swapchain::frame_present`].
    Ready
    {
      /// Index of the acquired image within the chain — what present takes.
      index : u32,
      /// The acquired presentable image, for layout transitions.
      image : ash::vk::Image,
      /// Full color view of `image`, usable as a render pass attachment.
      view : ash::vk::ImageView,
      /// Pixel dimensions of `image`.
      extent : ash::vk::Extent2D,
    },
    /// The swapchain no longer matches its surface. Rebuild it with
    /// [`Swapchain::resize`] and acquire again.
    Reconfigure,
  }

  impl Swapchain
  {
    /// Creates a swapchain over `surface` at `size`.
    ///
    /// Picks the presentation format via [`crate::surface::preferred_format`],
    /// `FIFO` present mode ( the only mode Vulkan guarantees is supported ), and
    /// an image count one above the surface's stated minimum so the driver has a
    /// spare to hand out while one is on screen.
    ///
    /// # Errors
    /// Returns [`Error::SurfaceCapabilities`] when the surface's capabilities,
    /// formats, or present modes cannot be queried, [`Error::SwapchainCreate`]
    /// when `vkCreateSwapchainKHR` fails, or [`Error::SwapchainImages`] when the
    /// resulting images or their views cannot be retrieved.
    pub fn new
    (
      context : &crate::context::Context,
      surface : &crate::surface::Surface,
      size : ( u32, u32 ),
    )
    -> Result< Self, Error >
    {
      let loader = ash::khr::swapchain::Device::new( context.instance_get(), context.device_get() );
      let fence_info = ash::vk::FenceCreateInfo::default();
      // SAFETY: `fence_info` is a stack-local, fully-initialized create info with no
      // extension chain, and `context`'s device is live for the duration of this call.
      let acquire_fence = unsafe { context.device_get().create_fence( &fence_info, None ) }
      .map_err( Error::SwapchainCreate )?;

      let mut swapchain = Self
      {
        loader,
        device : context.device_get().clone(),
        handle : ash::vk::SwapchainKHR::null(),
        views : Vec::new(),
        images : Vec::new(),
        format : ash::vk::Format::UNDEFINED,
        extent : ash::vk::Extent2D::default(),
        acquire_fence,
      };
      swapchain.rebuild( context, surface, size )?;
      Ok( swapchain )
    }

    /// Rebuilds the chain at a new drawable size, replacing whatever it held.
    ///
    /// Waits for the device to go idle first — the images and views being
    /// destroyed may still be referenced by in-flight work.
    ///
    /// # Errors
    /// Same as [`Swapchain::new`], plus [`Error::DeviceWaitIdle`] when the
    /// device cannot be brought to idle before the old chain is destroyed.
    pub fn resize
    (
      &mut self,
      context : &crate::context::Context,
      surface : &crate::surface::Surface,
      size : ( u32, u32 ),
    )
    -> Result< (), Error >
    {
      // SAFETY: `device` is the live logical device this swapchain's images and
      // views were created on ; waiting for it to go idle is what makes destroying
      // them inside `rebuild` below safe.
      unsafe { self.device.device_wait_idle() }.map_err( Error::DeviceWaitIdle )?;
      self.rebuild( context, surface, size )
    }

    /// Creates the new chain, then destroys whatever the old one held.
    ///
    /// The old `VkSwapchainKHR` is passed as `oldSwapchain`, letting the driver
    /// reuse its resources, and is destroyed only after the new one exists.
    fn rebuild
    (
      &mut self,
      context : &crate::context::Context,
      surface : &crate::surface::Surface,
      size : ( u32, u32 ),
    )
    -> Result< (), Error >
    {
      let physical_device = context.physical_device_get();
      let capabilities = surface.capabilities( physical_device )?;
      let format = crate::surface::preferred_format( &surface.formats( physical_device )? )?;
      let extent = extent_clamp( &capabilities, size );
      // A minimized window reports a zero `currentExtent`, which `vkCreateSwapchainKHR`
      // rejects as a validation error rather than something diagnosable. Bailing here
      // -- before anything is created or destroyed -- leaves the existing chain intact,
      // so rendering resumes on its own once the window comes back.
      if extent.width == 0 || extent.height == 0
      {
        return Err( Error::ZeroExtent( extent.width, extent.height ) );
      }
      let image_count = image_count_pick( &capabilities );

      let create_info = ash::vk::SwapchainCreateInfoKHR::default()
      .surface( surface.handle_get() )
      .min_image_count( image_count )
      .image_format( format.format )
      .image_color_space( format.color_space )
      .image_extent( extent )
      .image_array_layers( 1 )
      // COLOR_ATTACHMENT to render into, TRANSFER_DST so a consumer may also
      // blit or clear into a presentable image without a render pass.
      .image_usage( ash::vk::ImageUsageFlags::COLOR_ATTACHMENT | ash::vk::ImageUsageFlags::TRANSFER_DST )
      // EXCLUSIVE : this crate creates one queue, so graphics and present are
      // always the same family and no concurrent sharing is needed.
      .image_sharing_mode( ash::vk::SharingMode::EXCLUSIVE )
      .pre_transform( capabilities.current_transform )
      .composite_alpha( ash::vk::CompositeAlphaFlagsKHR::OPAQUE )
      .present_mode( ash::vk::PresentModeKHR::FIFO )
      .clipped( true )
      .old_swapchain( self.handle );

      // SAFETY: `create_info` borrows only stack-local values that outlive this call,
      // and `surface.handle_get()` belongs to the same instance `loader` was created
      // from. `old_swapchain` is either null ( first build ) or this swapchain's own
      // still-live handle, both of which the spec accepts.
      let handle = unsafe { self.loader.create_swapchain( &create_info, None ) }
      .map_err( Error::SwapchainCreate )?;

      // Fix(BUG-424): `handle`, and each view created below, are now tracked by `guard`
      // from this point on -- every fallible step between here and `disarm()` goes through
      // it, so an early `?` return destroys whatever was already created instead of
      // leaking it.
      // Root cause: `get_swapchain_images` and per-image `image_view_create` both sit
      // behind `?` after `handle` already exists, and view creation previously used
      // `.map().collect::< Result< Vec< _ >, _ > >()`, which -- on the Nth image's view
      // failing -- discards the first N-1 already-created `VkImageView`s as an ordinary
      // `Vec` alongside the collected `Err`, along with `handle` itself. Structurally
      // identical to `Fix(BUG-290)`'s `Context` instance leak : a live handle created
      // successfully, then abandoned on a sibling step's failure with no cleanup.
      // Pitfall: `.collect::< Result< Vec< _ >, _ > >()` is exactly the tool for turning
      // "many fallible steps" into "one fallible step" -- but it silently discards every
      // already-`Ok` item on the first `Err`, which is only safe when those `Ok` items are
      // pure Rust values. The moment they are FFI handles needing their own explicit
      // destruction, the discarded partial `Vec` becomes a leak, not a no-op.
      let mut guard = SwapchainGuard::new( &self.loader, &self.device, handle );

      // SAFETY: `handle` was just created by this same loader.
      let images = unsafe { self.loader.get_swapchain_images( handle ) }
      .map_err( Error::SwapchainImages )?;
      for image in &images
      {
        guard.view_push( image_view_create( &self.device, *image, format.format )? );
      }

      let ( handle, views ) = guard.disarm();
      self.destroy_chain();
      self.handle = handle;
      self.images = images;
      self.views = views;
      self.format = format.format;
      self.extent = extent;
      Ok( () )
    }

    /// Destroys the current views and swapchain handle, leaving the struct
    /// holding a null chain. Callers must have ensured the device is idle.
    fn destroy_chain( &mut self )
    {
      // SAFETY: every view in `views` was created by `rebuild` on `device` and has
      // not been destroyed since ; `handle` is either null ( which
      // `destroy_swapchain` accepts ) or this swapchain's own live handle. The
      // device is idle — `resize` waits for it, and `Drop` waits for it — so no
      // in-flight work still references them.
      unsafe
      {
        for view in self.views.drain( .. )
        {
          self.device.destroy_image_view( view, None );
        }
        self.loader.destroy_swapchain( self.handle, None );
      }
      self.handle = ash::vk::SwapchainKHR::null();
      self.images.clear();
    }

    /// Acquires the next presentable image.
    ///
    /// Blocks until one is available ( infinite timeout ) and until the
    /// acquisition fence signals, so the returned image is ready to render into
    /// with no further synchronization. A `suboptimal` acquisition is still
    /// reported as [`Frame::Ready`] : the image is drawable, and the next resize
    /// event corrects the mismatch anyway.
    ///
    /// # Errors
    /// Returns [`Error::SwapchainAcquire`] when acquisition fails for any reason
    /// other than an out-of-date chain, which is reported as
    /// [`Frame::Reconfigure`] rather than as an error.
    pub fn frame_acquire( &self ) -> Result< Frame, Error >
    {
      // SAFETY: `acquire_fence` was created by `new` on this same device and is
      // only ever used by this method, which waits for it before returning — so it
      // is never reset while a pending acquisition still references it.
      unsafe { self.device.reset_fences( &[ self.acquire_fence ] ) }
      .map_err( Error::SwapchainAcquire )?;

      // SAFETY: `handle` is this swapchain's live chain, and the fence was just
      // reset above. A null semaphore with a non-null fence is the spec's
      // fence-only acquisition form.
      let acquired = unsafe
      {
        self.loader.acquire_next_image
        ( self.handle, u64::MAX, ash::vk::Semaphore::null(), self.acquire_fence )
      };

      let index = match acquired
      {
        Ok( ( index, _suboptimal ) ) => index,
        Err( ash::vk::Result::ERROR_OUT_OF_DATE_KHR ) => return Ok( Frame::Reconfigure ),
        Err( e ) => return Err( Error::SwapchainAcquire( e ) ),
      };

      // SAFETY: the acquisition above submitted `acquire_fence` ; waiting for it is
      // what makes the returned image safe to render into immediately.
      unsafe { self.device.wait_for_fences( &[ self.acquire_fence ], true, u64::MAX ) }
      .map_err( Error::SwapchainAcquire )?;

      let slot = index as usize;
      Ok( Frame::Ready
      {
        index,
        image : self.images[ slot ],
        view : self.views[ slot ],
        extent : self.extent,
      } )
    }

    /// Presents the image at `index`, previously returned by
    /// [`Swapchain::frame_acquire`].
    ///
    /// No wait semaphore is supplied : the caller is required to have made the
    /// rendering into that image complete before calling — which the whole of
    /// this crate's synchronous, queue-idle-after-submit posture already
    /// guarantees.
    ///
    /// Returns `true` when the chain should be rebuilt ( the surface reported
    /// suboptimal or out-of-date ). That is a normal, expected outcome of a
    /// resize, not a failure.
    ///
    /// # Errors
    /// Returns [`Error::SwapchainPresent`] when presentation fails for any
    /// reason other than an out-of-date chain.
    pub fn frame_present( &self, queue : ash::vk::Queue, index : u32 ) -> Result< bool, Error >
    {
      let swapchains = [ self.handle ];
      let indices = [ index ];
      let present_info = ash::vk::PresentInfoKHR::default()
      .swapchains( &swapchains )
      .image_indices( &indices );

      // SAFETY: `present_info` borrows only stack-local arrays that outlive this
      // call ; `queue` belongs to the same device this chain was created on, and
      // `index` came from this same chain's own acquisition.
      match unsafe { self.loader.queue_present( queue, &present_info ) }
      {
        Ok( suboptimal ) => Ok( suboptimal ),
        Err( ash::vk::Result::ERROR_OUT_OF_DATE_KHR ) => Ok( true ),
        Err( e ) => Err( Error::SwapchainPresent( e ) ),
      }
    }

    /// The format the chain's images were created with.
    #[ inline ]
    #[ must_use ]
    pub fn format( &self ) -> ash::vk::Format
    {
      self.format
    }

    /// The pixel dimensions of the chain's images.
    #[ inline ]
    pub fn extent( &self ) -> ash::vk::Extent2D
    {
      self.extent
    }

    /// The chain's presentable images, in index order.
    #[ inline ]
    #[ must_use ]
    pub fn images_get( &self ) -> &[ ash::vk::Image ]
    {
      &self.images
    }

    /// The chain's image views, in index order.
    #[ inline ]
    #[ must_use ]
    pub fn views_get( &self ) -> &[ ash::vk::ImageView ]
    {
      &self.views
    }
  }

  impl Drop for Swapchain
  {
    fn drop( &mut self )
    {
      // SAFETY: `device` is the live logical device every handle here was created
      // on. Waiting for it to go idle first is what makes destroying the views,
      // chain and fence safe — no in-flight work can still reference them.
      unsafe
      {
        let _ = self.device.device_wait_idle();
        self.destroy_chain();
        self.device.destroy_fence( self.acquire_fence, None );
      }
    }
  }

  // Fix(BUG-424): `SwapchainGuard` is the RAII form of the same pattern `Fix(BUG-290)`
  // established for `crate::context::Context`'s instance ( see `InstanceGuard` there ),
  // applied here to the two artifacts `rebuild` creates before it can commit them to
  // `self` -- see the longer root-cause note at `rebuild`'s own guard construction below.
  /// Destroys the swapchain handle and any image views it still holds on drop, unless
  /// [`SwapchainGuard::disarm`] has handed them off first.
  struct SwapchainGuard< 'a >
  {
    loader : &'a ash::khr::swapchain::Device,
    device : &'a ash::Device,
    handle : Option< ash::vk::SwapchainKHR >,
    views : Vec< ash::vk::ImageView >,
  }

  impl< 'a > SwapchainGuard< 'a >
  {
    fn new( loader : &'a ash::khr::swapchain::Device, device : &'a ash::Device, handle : ash::vk::SwapchainKHR ) -> Self
    {
      Self { loader, device, handle : Some( handle ), views : Vec::new() }
    }

    /// Records a newly created view as guard-owned, so a later failure destroys it
    /// along with the handle instead of leaking it.
    fn view_push( &mut self, view : ash::vk::ImageView )
    {
      self.views.push( view );
    }

    /// Takes the handle and views out, so dropping the guard no longer destroys them.
    fn disarm( mut self ) -> ( ash::vk::SwapchainKHR, Vec< ash::vk::ImageView > )
    {
      ( self.handle.take().expect( "disarmed exactly once, on the single success path" ), core::mem::take( &mut self.views ) )
    }
  }

  impl Drop for SwapchainGuard< '_ >
  {
    fn drop( &mut self )
    {
      // SAFETY: every view in `views` was created on `device` by this same `rebuild`
      // call and has not been destroyed since ; `handle`, if still `Some`, was just
      // created by `loader` and has not been handed off to the owning `Swapchain` yet --
      // both are being abandoned here without any other code holding or using them
      // afterward.
      unsafe
      {
        for view in self.views.drain( .. )
        {
          self.device.destroy_image_view( view, None );
        }
        if let Some( handle ) = self.handle.take()
        {
          self.loader.destroy_swapchain( handle, None );
        }
      }
    }
  }

  /// Clamps a requested drawable size into the range the surface accepts.
  ///
  /// A `currentExtent` of `u32::MAX` is the spec's "the swapchain picks"
  /// signal ( Wayland reports it ); anything else is the size the platform
  /// dictates, and the request is ignored.
  fn extent_clamp
  (
    capabilities : &ash::vk::SurfaceCapabilitiesKHR,
    size : ( u32, u32 ),
  )
  -> ash::vk::Extent2D
  {
    if capabilities.current_extent.width != u32::MAX
    {
      return capabilities.current_extent;
    }
    let ( width, height ) = size;
    ash::vk::Extent2D
    {
      width : width.clamp( capabilities.min_image_extent.width, capabilities.max_image_extent.width ),
      height : height.clamp( capabilities.min_image_extent.height, capabilities.max_image_extent.height ),
    }
  }

  /// One image above the surface's stated minimum, so the driver always has a
  /// spare to hand out while another is on screen — capped at `maxImageCount`
  /// when the surface states one ( `0` means unlimited ).
  fn image_count_pick( capabilities : &ash::vk::SurfaceCapabilitiesKHR ) -> u32
  {
    let desired = capabilities.min_image_count + 1;
    if capabilities.max_image_count > 0
    {
      return desired.min( capabilities.max_image_count );
    }
    desired
  }

  /// Creates a full, single-mip, single-layer color view of a swapchain image.
  fn image_view_create
  (
    device : &ash::Device,
    image : ash::vk::Image,
    format : ash::vk::Format,
  )
  -> Result< ash::vk::ImageView, Error >
  {
    let subresource_range = ash::vk::ImageSubresourceRange::default()
    .aspect_mask( ash::vk::ImageAspectFlags::COLOR )
    .base_mip_level( 0 )
    .level_count( 1 )
    .base_array_layer( 0 )
    .layer_count( 1 );
    let create_info = ash::vk::ImageViewCreateInfo::default()
    .image( image )
    .view_type( ash::vk::ImageViewType::TYPE_2D )
    .format( format )
    .subresource_range( subresource_range );
    // SAFETY: `create_info` borrows only stack-local values that outlive this call,
    // and `image` was returned by `get_swapchain_images` on this same device.
    unsafe { device.create_image_view( &create_info, None ) }
    .map_err( Error::SwapchainImages )
  }
}

mod_interface!
{
  own use Swapchain;
  own use Frame;
}
