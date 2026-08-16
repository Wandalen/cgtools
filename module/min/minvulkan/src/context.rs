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
      .map_err( Error::PhysicalDeviceEnumerate )?;

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
      .ok_or( Error::NoSuitableDevice )?;

      let queue_priorities = [ 1.0_f32 ];
      let queue_create_info = ash::vk::DeviceQueueCreateInfo::default()
      .queue_family_index( queue_family_index )
      .queue_priorities( &queue_priorities );
      let queue_create_infos = [ queue_create_info ];
      let device_create_info = ash::vk::DeviceCreateInfo::default().queue_create_infos( &queue_create_infos );

      // SAFETY: `physical_device` was just selected from this same `instance`'s own
      // enumeration above ; `device_create_info` and the slices it borrows are all
      // stack-local and outlive this call.
      let device = unsafe { instance.create_device( physical_device, &device_create_info, None ) }
      .map_err( Error::DeviceCreate )?;

      // SAFETY: `device` was just created above with exactly one queue ( index 0 )
      // requested at `queue_family_index` via `device_create_info`, so that ( family,
      // index ) pair is guaranteed valid for this retrieval.
      let queue = unsafe { device.get_device_queue( queue_family_index, 0 ) };

      Ok( Context { entry, instance, physical_device, device, queue, queue_family_index } )
    }
  }
}

mod_interface!
{
  own use Context;
  own use ContextBuilder;
  own use InstanceBuilder;
  own use DeviceBuilder;
}
