//!
//! Fluent builder producing a native Vulkan `Context` — instance, physical
//! device, logical device, and a graphics-capable queue — built directly on
//! `ash`, with no `wgpu` dependency. Mirrors the shape of
//! `minwgpu::Context::builder()` for the `wgpu`-backed sibling crate.
//!

use mingl::mod_interface;

mod private
{
  use core::marker::PhantomData;
  use crate::Error;

  /// A native Vulkan graphics context : an instance, a selected physical device, a logical
  /// device, and one graphics-capable queue retrieved from it.
  pub struct Context
  {
    pub( super ) entry : ash::Entry,
    pub( super ) instance : ash::Instance,
    pub( super ) physical_device : ash::vk::PhysicalDevice,
    pub( super ) device : ash::Device,
    pub( super ) queue : ash::vk::Queue,
    pub( super ) queue_family_index : u32,
  }

  impl core::fmt::Debug for Context
  {
    /// Hand-written rather than derived : `ash::Entry` carries no `Debug`, and
    /// the raw handles that do would print as bare pointers with no meaning to
    /// a reader anyway.
    fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
    {
      f.debug_struct( "Context" )
      .field( "queue_family_index", &self.queue_family_index )
      .finish_non_exhaustive()
    }
  }

  impl Context
  {
    /// Starts the fluent builder used to create a `Context`.
    #[ must_use ]
    pub fn builder() -> ContextBuilder< InstanceBuilder >
    {
      ContextBuilder
      {
        _state : PhantomData,
        instance_create_flags : ash::vk::InstanceCreateFlags::empty(),
        entry : None,
        instance : None,
      }
    }

    /// Returns a reference to the loaded `ash::Entry`.
    #[ must_use ]
    pub fn entry_get( &self ) -> &ash::Entry
    {
      &self.entry
    }

    /// Returns a reference to the `ash::Instance`.
    #[ must_use ]
    pub fn instance_get( &self ) -> &ash::Instance
    {
      &self.instance
    }

    /// Returns the selected `ash::vk::PhysicalDevice`.
    #[ must_use ]
    pub fn physical_device_get( &self ) -> ash::vk::PhysicalDevice
    {
      self.physical_device
    }

    /// Returns a reference to the logical `ash::Device`.
    #[ must_use ]
    pub fn device_get( &self ) -> &ash::Device
    {
      &self.device
    }

    /// Returns the graphics-capable `ash::vk::Queue`.
    #[ must_use ]
    pub fn queue_get( &self ) -> ash::vk::Queue
    {
      self.queue
    }

    /// Returns the queue family index `queue` was retrieved from.
    #[ must_use ]
    pub fn queue_family_index_get( &self ) -> u32
    {
      self.queue_family_index
    }
  }

  // Fix(BUG-290): `context_finish`'s 3 error-return paths ( enumeration failure, no suitable
  // device, device-creation failure ) all occur after `instance` is already live, but each one
  // previously propagated its error via a bare `?`/`.ok_or()?`, silently dropping `instance` as
  // an ordinary value on the way out instead of destroying it.
  // Root cause: `ash::Instance` has no `Drop` impl of its own ( confirmed against `ash` 0.38's
  // own source -- Vulkan mandates explicit `vkDestroyInstance`, so relying on ordinary Rust drop
  // to clean it up was never correct ), leaking the `VkInstance` handle on every one of those 3
  // paths.
  // Pitfall: a missing `Drop` impl doesn't announce itself -- verify an FFI handle wrapper type
  // actually implements `Drop` for cleanup before relying on ordinary ownership to provide it ;
  // `ash::Entry` incidentally self-cleans via an internal `Arc<Library>`, which does not
  // generalize to `Instance`/`Device`.
  /// Destroys `instance` -- called from `context_finish`'s error paths, where an early return
  /// must destroy the already-created instance instead of leaking it ( see `Fix(BUG-290)` above ).
  fn instance_cleanup_on_error( instance : &ash::Instance )
  {
    // SAFETY: called only from error paths -- `context_finish`'s directly, and
    // `windowed`'s through `InstanceGuard`'s `Drop` -- where `instance` is still live
    // and is about to be abandoned without being handed off to a `Context` : no other
    // code holds or will use this handle afterward.
    unsafe { instance.destroy_instance( None ); }
  }

  impl Drop for Context
  {
    fn drop( &mut self )
    {
      // SAFETY: `device` and `instance` are owned exclusively by this `Context` ( it is
      // never `Clone`, so no other value can hold the same handles ) and neither has been
      // destroyed before this point, so destroying device-then-instance here exactly once,
      // in dependency order, upholds Vulkan's single-destruction requirement.
      unsafe
      {
        self.device.destroy_device( None );
        self.instance.destroy_instance( None );
      }
    }
  }

  /// Type-state marker : the builder is configuring instance creation ( the state returned
  /// by [`Context::builder`] ).
  pub struct InstanceBuilder;

  /// Type-state marker : the builder holds a live instance and is ready to select a
  /// physical device and create the logical device ( entered via `instance_make` ).
  pub struct DeviceBuilder;

  /// A type-state fluent builder for a native Vulkan `Context`.
  pub struct ContextBuilder< State >
  {
    pub( super ) _state : PhantomData< State >,

    pub( super ) instance_create_flags : ash::vk::InstanceCreateFlags,

    pub( super ) entry : Option< ash::Entry >,
    pub( super ) instance : Option< ash::Instance >,
  }

  impl ContextBuilder< InstanceBuilder >
  {
    /// Sets the `ash::vk::InstanceCreateFlags` used for instance creation.
    #[ must_use ]
    pub fn flags( mut self, value : ash::vk::InstanceCreateFlags ) -> Self
    {
      self.instance_create_flags = value;
      self
    }

    /// Loads the Vulkan loader library at runtime and creates the `ash::Instance`,
    /// transitioning the builder into the device-creation state.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EntryLoad`] when the Vulkan loader library ( e.g. `libvulkan.so.1` )
    /// cannot be found, or [`Error::InstanceCreate`] when `vkCreateInstance` fails.
    pub fn instance_make( self ) -> Result< ContextBuilder< DeviceBuilder >, Error >
    {
      // SAFETY: loading the Vulkan loader library and binding its exported functions has no
      // preconditions beyond the library itself being a well-formed Vulkan loader, which is
      // outside this call's control ; a malformed loader is reported back as `LoadingError`
      // rather than silently miscompiled into this call.
      let entry = unsafe { ash::Entry::load() }.map_err( Error::EntryLoad )?;

      let create_info = ash::vk::InstanceCreateInfo::default().flags( self.instance_create_flags );
      // SAFETY: `create_info` is a fully-initialized `InstanceCreateInfo` with no pointer
      // fields left dangling and no extension chain ; `entry` was just loaded above and no
      // custom allocator is supplied, so passing `None` for the allocation callbacks is valid.
      let instance = unsafe { entry.create_instance( &create_info, None ) }.map_err( Error::InstanceCreate )?;

      Ok
      (
        ContextBuilder
        {
          _state : PhantomData,
          instance_create_flags : self.instance_create_flags,
          entry : Some( entry ),
          instance : Some( instance ),
        }
      )
    }
  }

  impl ContextBuilder< DeviceBuilder >
  {
    /// Selects the first enumerated physical device exposing a graphics-capable queue
    /// family, creates the logical `ash::Device` with a single graphics queue requested on
    /// that family, and retrieves the resulting `ash::vk::Queue` — consuming the builder to
    /// produce the final `Context`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PhysicalDeviceEnumerate`] when physical device enumeration fails,
    /// [`Error::NoSuitableDevice`] when no enumerated device exposes a graphics-capable
    /// queue family, or [`Error::DeviceCreate`] when `vkCreateDevice` fails.
    ///
    /// # Panics
    ///
    /// Never in practice : `entry`/`instance` are always `Some` here, since `DeviceBuilder`
    /// is only reachable via `instance_make`, which populates both before returning it.
    pub fn context_finish( self ) -> Result< Context, Error >
    {
      // Both fields are always `Some` here : this state is only reachable via
      // `instance_make`, which populates both before returning `ContextBuilder< DeviceBuilder >`.
      let entry = self.entry.expect( "entry is set by instance_make before DeviceBuilder is reachable" );
      let instance = self.instance.expect( "instance is set by instance_make before DeviceBuilder is reachable" );

      // SAFETY: `instance` was created by `instance_make` immediately prior and is still
      // live ; enumeration performs no writes through caller-supplied pointers.
      let physical_devices = unsafe { instance.enumerate_physical_devices() }
      .map_err( | e | { instance_cleanup_on_error( &instance ); Error::PhysicalDeviceEnumerate( e ) } )?;

      let ( physical_device, queue_family_index ) = physical_devices
      .into_iter()
      .find_map
      (
        | candidate |
        {
          // SAFETY: `candidate` was just returned by `enumerate_physical_devices` on this
          // same, still-live `instance` — it is a valid handle for this query.
          let properties = unsafe { instance.get_physical_device_queue_family_properties( candidate ) };
          properties
          .iter()
          .position( | family | family.queue_flags.contains( ash::vk::QueueFlags::GRAPHICS ) )
          .map( | index | ( candidate, u32::try_from( index ).expect( "queue family index fits u32" ) ) )
        }
      )
      // `ok_or_else`, not `ok_or` : the latter evaluates its argument eagerly, which would run
      // `instance_cleanup_on_error` ( and so destroy `instance` ) on the success path too.
      .ok_or_else( || { instance_cleanup_on_error( &instance ); Error::NoSuitableDevice } )?;

      let ( device, queue ) = device_create( &instance, physical_device, queue_family_index, &[] )
      .inspect_err( | _ | instance_cleanup_on_error( &instance ) )?;

      Ok( Context { entry, instance, physical_device, device, queue, queue_family_index } )
    }
  }

  /// Creates a logical device with one graphics queue on `queue_family_index`,
  /// enabling `extensions`, and retrieves that queue.
  ///
  /// Shared by the headless path ( `context_finish`, no extensions ) and the
  /// windowed one ( [`windowed`], `VK_KHR_swapchain` ), so the two never drift
  /// apart in how the device and its single queue are set up.
  ///
  /// # Errors
  ///
  /// Returns [`Error::DeviceCreate`] when `vkCreateDevice` fails.
  fn device_create
  (
    instance : &ash::Instance,
    physical_device : ash::vk::PhysicalDevice,
    queue_family_index : u32,
    extensions : &[ *const core::ffi::c_char ],
  )
  -> Result< ( ash::Device, ash::vk::Queue ), Error >
  {
    let queue_priorities = [ 1.0_f32 ];
    let queue_create_info = ash::vk::DeviceQueueCreateInfo::default()
    .queue_family_index( queue_family_index )
    .queue_priorities( &queue_priorities );
    let queue_create_infos = [ queue_create_info ];
    let device_create_info = ash::vk::DeviceCreateInfo::default()
    .queue_create_infos( &queue_create_infos )
    .enabled_extension_names( extensions );

    // SAFETY: `physical_device` was selected from this same `instance`'s own
    // enumeration by the caller ; `device_create_info` and the slices it borrows are
    // all stack-local and outlive this call.
    let device = unsafe { instance.create_device( physical_device, &device_create_info, None ) }
    .map_err( Error::DeviceCreate )?;

    // SAFETY: `device` was just created above with exactly one queue ( index 0 )
    // requested at `queue_family_index` via `device_create_info`, so that ( family,
    // index ) pair is guaranteed valid for this retrieval.
    let queue = unsafe { device.get_device_queue( queue_family_index, 0 ) };
    Ok( ( device, queue ) )
  }

  /// Creates a context, a surface for `window`, and a swapchain at `size` — the
  /// windowed counterpart of the [`Context::builder`] chain, and the Vulkan
  /// counterpart of `minwgpu::context::windowed`.
  ///
  /// Performs the four steps in the order Vulkan requires, which is why this is
  /// one function rather than four builder calls : the instance must be created
  /// with the platform's surface extensions, the surface must exist before a
  /// physical device can be checked for present support, and the swapchain needs
  /// both. Prefer [`crate::surface::Windowed::new`], which wraps the result.
  ///
  /// # Errors
  ///
  /// Returns [`Error::WindowHandle`] or [`Error::SurfaceCreate`] when the window
  /// yields no usable handle, [`Error::EntryLoad`] / [`Error::InstanceCreate`] as
  /// [`ContextBuilder::instance_make`] does, [`Error::NoPresentDevice`] when no
  /// device can both render and present to the surface, [`Error::DeviceCreate`]
  /// when `vkCreateDevice` fails, or any swapchain error from
  /// [`crate::swapchain::Swapchain::new`].
  pub fn windowed
  (
    window : &( impl raw_window_handle::HasDisplayHandle + raw_window_handle::HasWindowHandle ),
    size : ( u32, u32 ),
  )
  -> Result< ( Context, crate::surface::Surface, crate::swapchain::Swapchain ), Error >
  {
    let instance_extensions = crate::surface::required_instance_extensions( window )?;

    // SAFETY: as `instance_make` -- loading the Vulkan loader library has no
    // preconditions beyond the library being a well-formed loader, reported back as
    // `LoadingError` rather than miscompiled into this call.
    let entry = unsafe { ash::Entry::load() }.map_err( Error::EntryLoad )?;
    let create_info = ash::vk::InstanceCreateInfo::default()
    .enabled_extension_names( instance_extensions );
    // SAFETY: `create_info` borrows `instance_extensions`, a `'static` slice from
    // `ash-window`, and no other pointer field is set ; `entry` was just loaded.
    let instance = unsafe { entry.create_instance( &create_info, None ) }
    .map_err( Error::InstanceCreate )?;

    // Four fallible steps follow with `instance` already live. Declared here, the
    // guard drops *after* `surface` below ( locals drop in reverse declaration
    // order ) — which is the order Vulkan requires, and the reason the guard is
    // not simply an `instance_cleanup_on_error` call per error path as in
    // `context_finish`, where nothing outlives the instance.
    let mut instance_guard = InstanceGuard( Some( instance ) );

    let surface = crate::surface::Surface::from_window( &entry, instance_guard.get(), window )?;
    let ( physical_device, queue_family_index ) = present_device_select( instance_guard.get(), &surface )?;

    let extensions = [ ash::khr::swapchain::NAME.as_ptr() ];
    let ( device, queue ) = device_create( instance_guard.get(), physical_device, queue_family_index, &extensions )?;

    // The `Context` takes over destroying the instance from here on, so the guard
    // is disarmed rather than left to destroy it a second time.
    let instance = instance_guard.disarm();
    let context = Context { entry, instance, physical_device, device, queue, queue_family_index };

    match crate::swapchain::Swapchain::new( &context, &surface, size )
    {
      Ok( swapchain ) => Ok( ( context, surface, swapchain ) ),
      Err( e ) =>
      {
        // Explicit, and in this order : `context` is declared after `surface`, so
        // letting them fall out of scope would destroy the instance first and leave
        // the surface dangling.
        drop( surface );
        drop( context );
        Err( e )
      }
    }
  }

  /// Destroys the instance it holds on drop, unless [`InstanceGuard::disarm`] has
  /// handed it off first — the RAII form of [`instance_cleanup_on_error`].
  ///
  /// [`windowed`] runs several fallible steps between creating an instance and
  /// the `Context` that finally owns it, and one of them ( the surface ) must
  /// itself be destroyed before the instance. Per-error-path cleanup calls, as in
  /// `context_finish`, cannot express that ordering ; drop order can.
  struct InstanceGuard( Option< ash::Instance > );

  impl InstanceGuard
  {
    /// Borrows the guarded instance.
    fn get( &self ) -> &ash::Instance
    {
      self.0.as_ref().expect( "the guard is disarmed only by consuming its instance, never before" )
    }

    /// Takes the instance out, so dropping the guard no longer destroys it.
    fn disarm( &mut self ) -> ash::Instance
    {
      self.0.take().expect( "disarmed exactly once, on the single success path" )
    }
  }

  impl Drop for InstanceGuard
  {
    fn drop( &mut self )
    {
      if let Some( instance ) = self.0.as_ref()
      {
        instance_cleanup_on_error( instance );
      }
    }
  }

  /// Finds the first physical device exposing a queue family that is both
  /// graphics-capable and able to present to `surface`.
  ///
  /// Present support is a per-( device, family ) property, so it cannot be
  /// checked before the surface exists — which is why the windowed path has its
  /// own selection rather than reusing `context_finish`'s graphics-only one.
  fn present_device_select
  (
    instance : &ash::Instance,
    surface : &crate::surface::Surface,
  )
  -> Result< ( ash::vk::PhysicalDevice, u32 ), Error >
  {
    // SAFETY: `instance` is live and enumeration performs no writes through
    // caller-supplied pointers.
    let physical_devices = unsafe { instance.enumerate_physical_devices() }
    .map_err( Error::PhysicalDeviceEnumerate )?;

    for candidate in physical_devices
    {
      // SAFETY: `candidate` was just returned by `enumerate_physical_devices` on
      // this same, still-live `instance`.
      let families = unsafe { instance.get_physical_device_queue_family_properties( candidate ) };
      for ( index, family ) in families.iter().enumerate()
      {
        if !family.queue_flags.contains( ash::vk::QueueFlags::GRAPHICS )
        {
          continue;
        }
        let index = u32::try_from( index ).expect( "queue family index fits u32" );
        if surface.present_supported( candidate, index )?
        {
          return Ok( ( candidate, index ) );
        }
      }
    }
    Err( Error::NoPresentDevice )
  }
}

mod_interface!
{
  own use Context;
  own use ContextBuilder;
  own use InstanceBuilder;
  own use DeviceBuilder;
  own use windowed;
}
