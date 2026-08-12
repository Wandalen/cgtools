//! This module provides a fluent, type-state builder for setting up a `wgpu` context,
//! which includes the `Instance`, `Adapter`, `Device`, and `Queue`. This pattern guides
//! the user through the required steps of `wgpu` initialization in the correct order.

use mingl::mod_interface;

mod private
{
  use core::marker::PhantomData;

  /// A container for the core `wgpu` components, representing a complete graphics context.
  ///
  /// An instance of `Context` holds everything needed to start creating resources and rendering.
  #[ derive( Debug, Clone ) ]
  pub struct Context
  {
    pub( super ) instance : wgpu::Instance,
    pub( super ) adapter : wgpu::Adapter,
    pub( super ) device : wgpu::Device,
    pub( super ) queue : wgpu::Queue,
  }

  impl Context
  {
    /// Creates a new `ContextBuilder` to start the setup process.
    ///
    /// This is the entry point for the fluent builder pattern.
    #[ inline ]
    #[ must_use ]
    pub fn builder() -> ContextBuilder< 'static, 'static, 'static, 'static, InstanceBuilder >
    {
      ContextBuilder
      {
        _state : PhantomData,
        instance_descriptor : wgpu::InstanceDescriptor::default(),
        request_adapter_options : wgpu::RequestAdapterOptionsBase::default(),
        device_descriptor : wgpu::wgt::DeviceDescriptor::default(),
        instance : None,
        adapter : None,
        adapter_selector : None
      }
    }

    /// Test-support constructor: a `DeviceBuilder`-state builder with default descriptors
    /// and no instance or adapter. Public only so the device-descriptor setter tests can
    /// live in `tests/`; the type-state invariant ( an adapter is present in this state
    /// when it is reached through the public chain ) is deliberately not upheld, so
    /// calling `context_finish` on the result panics. Not part of the supported API.
    #[ doc( hidden ) ]
    #[ inline ]
    #[ must_use ]
    pub fn device_builder_for_tests() -> ContextBuilder< 'static, 'static, 'static, 'static, DeviceBuilder >
    {
      ContextBuilder
      {
        _state : PhantomData,
        instance_descriptor : wgpu::InstanceDescriptor::default(),
        request_adapter_options : wgpu::RequestAdapterOptionsBase::default(),
        device_descriptor : wgpu::wgt::DeviceDescriptor::default(),
        instance : None,
        adapter : None,
        adapter_selector : None
      }
    }

    /// Returns a reference to the `wgpu::Instance`.
    #[ inline ]
    #[ must_use ]
    pub fn instance_get( &self ) -> &wgpu::Instance
    {
      &self.instance
    }

    /// Returns a reference to the `wgpu::Adapter`.
    #[ inline ]
    #[ must_use ]
    pub fn adapter_get( &self ) -> &wgpu::Adapter
    {
      &self.adapter
    }

    /// Returns a reference to the `wgpu::Device`.
    #[ inline ]
    #[ must_use ]
    pub fn device_get( &self ) -> &wgpu::Device
    {
      &self.device
    }

    /// Returns a reference to the `wgpu::Queue`.
    #[ inline ]
    #[ must_use ]
    pub fn queue_get( &self ) -> &wgpu::Queue
    {
      &self.queue
    }
  }

  impl From< wgpu::Instance > for ContextBuilder< 'static, 'static, 'static, 'static, AdapterBuilder >
  {
    /// Creates a new `ContextBuilder` with a provided `wgpu::Instance`.
    ///
    /// This is the entry point for the fluent builder pattern.
    #[ inline ]
    fn from( instance : wgpu::Instance ) -> Self
    {
      Self
      {
        _state : PhantomData,
        instance_descriptor : wgpu::InstanceDescriptor::default(),
        request_adapter_options : wgpu::RequestAdapterOptionsBase::default(),
        device_descriptor : wgpu::wgt::DeviceDescriptor::default(),
        instance : Some( instance ),
        adapter : None,
        adapter_selector : None
      }
    }
  }

  impl AsRef< wgpu::Instance > for Context
  {
    #[ inline ]
    fn as_ref( &self ) -> &wgpu::Instance
    {
      &self.instance
    }
  }

  impl AsRef< wgpu::Adapter > for Context
  {
    #[ inline ]
    fn as_ref( &self ) -> &wgpu::Adapter
    {
      &self.adapter
    }
  }

  impl AsRef< wgpu::Device > for Context
  {
    #[ inline ]
    fn as_ref( &self ) -> &wgpu::Device
    {
      &self.device
    }
  }

  impl AsRef< wgpu::Queue > for Context
  {
    #[ inline ]
    fn as_ref( &self ) -> &wgpu::Queue
    {
      &self.queue
    }
  }

  /// Creates a ready-to-use headless `Context` on the primary backends.
  ///
  /// Equivalent to [`headless_with`] with [`wgpu::Backends::PRIMARY`] : an instance on the
  /// primary backends, a high-performance adapter, and a device with the default
  /// descriptor. Suited for offscreen rendering where no window surface is involved;
  /// use [`Context::builder`] directly when extra features or limits are required.
  ///
  /// # Errors
  ///
  /// Returns an error when no suitable adapter is found or the device request fails.
  #[ inline ]
  pub fn headless() -> Result< Context, crate::Error >
  {
    headless_with( wgpu::Backends::PRIMARY )
  }

  /// Creates a ready-to-use headless `Context` on the given backends.
  ///
  /// Builds an instance restricted to `backends`, requests a high-performance adapter,
  /// and requests a device with the default descriptor.
  ///
  /// # Errors
  ///
  /// Returns an error when no suitable adapter is found on the given backends or the
  /// device request fails.
  #[ inline ]
  pub fn headless_with( backends : wgpu::Backends ) -> Result< Context, crate::Error >
  {
    Context::builder()
    .backends( backends )
    .instance_make()
    .power_preference( wgpu::PowerPreference::HighPerformance )
    .adapter_request()?
    .context_finish()
  }

  pub type AdapterSelector< 's > = Box< dyn FnMut( &wgpu::Instance ) -> Result< wgpu::Adapter, crate::Error > + 's >;

  /// Type-state marker: the builder is configuring the `wgpu::Instance` ( the state
  /// returned by [`Context::builder`] ).
  pub struct InstanceBuilder;

  /// Type-state marker: the builder is selecting a `wgpu::Adapter` ( entered via
  /// `instance_make` or the `From< wgpu::Instance >` impl ).
  pub struct AdapterBuilder;

  /// Type-state marker: the builder is configuring the `wgpu::Device` request ( entered
  /// via `adapter_request` / `adapter_request_async` ).
  pub struct DeviceBuilder;

  /// A type-state builder for creating a `wgpu` `Context`.
  ///
  /// This builder guides the user through the sequential process of creating an instance,
  /// selecting an adapter, and requesting a device.
  pub struct ContextBuilder< 'a, 'b, 'l, 's, S >
  {
    pub( super ) _state : PhantomData< S >,

    pub( super ) instance_descriptor : wgpu::InstanceDescriptor,
    pub( super ) request_adapter_options : wgpu::RequestAdapterOptions< 'a, 'b >,
    pub( super ) device_descriptor : wgpu::DeviceDescriptor< 'l >,

    pub( super ) instance : Option< wgpu::Instance >,
    pub( super ) adapter :  Option< wgpu::Adapter >,

    pub( super ) adapter_selector : Option< AdapterSelector< 's > >
  }

  impl< 'a, 'b, 'l, S > ContextBuilder< 'a, 'b, 'l, '_, S >
  {
    /// Returns the accumulated `wgpu::InstanceDescriptor`.
    #[ inline ]
    #[ must_use ]
    pub fn instance_descriptor_get( &self ) -> &wgpu::InstanceDescriptor
    {
      &self.instance_descriptor
    }

    /// Returns the accumulated `wgpu::RequestAdapterOptions`.
    #[ inline ]
    #[ must_use ]
    pub fn request_adapter_options_get( &self ) -> &wgpu::RequestAdapterOptions< 'a, 'b >
    {
      &self.request_adapter_options
    }

    /// Returns the accumulated `wgpu::DeviceDescriptor`.
    #[ inline ]
    #[ must_use ]
    pub fn device_descriptor_get( &self ) -> &wgpu::DeviceDescriptor< 'l >
    {
      &self.device_descriptor
    }

    /// Returns `true` once a custom adapter selector has been provided.
    #[ inline ]
    #[ must_use ]
    pub fn has_adapter_selector( &self ) -> bool
    {
      self.adapter_selector.is_some()
    }
  }

  impl< 'a, 'b, 'l, 's > ContextBuilder< 'a, 'b, 'l, 's, InstanceBuilder >
  {
    /// Sets the graphics backends to be used.
    #[ inline ]
    #[ must_use ]
    pub fn backends( mut self, value : wgpu::Backends ) -> Self
    {
      self.instance_descriptor.backends = value;
      self
    }

    /// Sets the instance flags.
    #[ inline ]
    #[ must_use ]
    pub fn flags( mut self, value : wgpu::InstanceFlags ) -> Self
    {
      self.instance_descriptor.flags = value;
      self
    }

    /// Sets the memory budget thresholds for the instance.
    #[ inline ]
    #[ must_use ]
    pub fn memory_budget_thresholds( mut self, value : wgpu::MemoryBudgetThresholds ) -> Self
    {
      self.instance_descriptor.memory_budget_thresholds = value;
      self
    }

    /// Sets backend-specific options.
    #[ inline ]
    #[ must_use ]
    pub fn backend_options( mut self, value : wgpu::BackendOptions ) -> Self
    {
      self.instance_descriptor.backend_options = value;
      self
    }

    /// Creates the `wgpu::Instance` and transitions the builder to the next state for adapter selection.
    #[ inline ]
    #[ must_use ]
    pub fn instance_make( mut self ) -> ContextBuilder< 'a, 'b, 'l, 's, AdapterBuilder >
    {
      self.instance = Some( wgpu::Instance::new( &self.instance_descriptor ) );

      let Self
      {
        instance_descriptor,
        request_adapter_options,
        device_descriptor,
        instance,
        adapter,
        adapter_selector,
        ..
      } = self;

      ContextBuilder
      {
        _state : PhantomData,
        instance_descriptor,
        request_adapter_options,
        device_descriptor,
        instance,
        adapter,
        adapter_selector,
      }
    }
  }

  impl< 'a, 'b, 'l, 's > ContextBuilder< 'a, 'b, 'l, 's, AdapterBuilder >
  {
    /// Sets the power preference for the adapter.
    #[ inline ]
    #[ must_use ]
    pub fn power_preference( mut self, value : wgpu::PowerPreference ) -> Self
    {
      self.request_adapter_options.power_preference = value;
      self
    }

    /// Forces the use of a fallback adapter if a suitable one is not found.
    #[ inline ]
    #[ must_use ]
    pub fn force_fallback_adapter( mut self, value : bool ) -> Self
    {
      self.request_adapter_options.force_fallback_adapter = value;
      self
    }

    /// Specifies a surface that the adapter must be compatible with.
    #[ inline ]
    #[ must_use ]
    pub fn compatible_surface( mut self, value : &'a wgpu::Surface< 'b > ) -> Self
    {
      self.request_adapter_options.compatible_surface = Some( value );
      self
    }

    /// Provides a custom closure to select a `wgpu::Adapter`.
    /// If the closure is provided it will be used to select an adapter in first place,
    /// selected `wgpu::RequestAdapterOptions` will be ignored.
    #[ inline ]
    #[ must_use ]
    pub fn adapter_selector< F >( mut self, value : F ) -> Self
    where
      F : FnMut( &wgpu::Instance ) -> Result< wgpu::Adapter, crate::Error > + 's
    {
      self.adapter_selector = Some( Box::new( value ) );
      self
    }

    /// Asynchronously requests a `wgpu::Adapter` and transitions the builder to the device creation state.
    ///
    /// # Errors
    ///
    /// Return error in case of `Instance::request_adapter` returns error.
    ///
    /// # Panics
    ///
    /// Panics if the instance was never set. This cannot happen through the public API: the
    /// `AdapterBuilder` state is only reachable via `instance_make` or the `From< wgpu::Instance >`
    /// impl, both of which populate `instance` before this method becomes callable.
    #[ inline ]
    pub async fn adapter_request_async( mut self ) -> Result< ContextBuilder< 'a, 'b, 'l, 's, DeviceBuilder >, crate::Error >
    {
      let adapter = if let Some( adapter_selector ) = &mut self.adapter_selector
      {
        adapter_selector( self.instance.as_ref().unwrap() )?
      }
      else
      {
        self.instance.as_ref().unwrap().request_adapter( &self.request_adapter_options ).await?
      };

      self.adapter = Some( adapter );

      let Self
      {
        instance_descriptor,
        request_adapter_options,
        device_descriptor,
        instance,
        adapter,
        adapter_selector,
        ..
      } = self;

      Ok
      (
        ContextBuilder
        {
          _state : PhantomData,
          instance_descriptor,
          request_adapter_options,
          device_descriptor,
          instance,
          adapter,
          adapter_selector,
        }
      )
    }

    /// Synchronously requests a `wgpu::Adapter` and transitions the builder.
    ///
    /// # Errors
    ///
    /// Return error in case of `Instance::request_adapter` returns error.
    #[ inline ]
    pub fn adapter_request( self ) -> Result< ContextBuilder< 'a, 'b, 'l, 's, DeviceBuilder >, crate::Error >
    {
      pollster::block_on( self.adapter_request_async() )
    }
  }

  impl< 'l > ContextBuilder< '_, '_, 'l, '_, DeviceBuilder >
  {
    /// Sets a debug label for the `wgpu::Device`.
    #[ inline ]
    #[ must_use ]
    pub fn label( mut self, value : &'l str ) -> Self
    {
      self.device_descriptor.label = Some( value );
      self
    }

    /// Specifies the features that the `wgpu::Device` must support.
    #[ inline ]
    #[ must_use ]
    pub fn required_features( mut self, value : wgpu::Features ) -> Self
    {
      self.device_descriptor.required_features = value;
      self
    }

    /// Specifies the limits that the `wgpu::Device` must support.
    #[ inline ]
    #[ must_use ]
    pub fn required_limits( mut self, value : wgpu::Limits ) -> Self
    {
      self.device_descriptor.required_limits = value;
      self
    }

    /// Provides memory usage hints to the driver.
    #[ inline ]
    #[ must_use ]
    pub fn memory_hints( mut self, value : wgpu::MemoryHints ) -> Self
    {
      self.device_descriptor.memory_hints = value;
      self
    }

    /// Enables a `wgpu` trace to be captured from this device.
    #[ inline ]
    #[ must_use ]
    pub fn trace( mut self, value : wgpu::Trace ) -> Self
    {
      self.device_descriptor.trace = value;
      self
    }

    /// Asynchronously requests the `wgpu::Device` and `wgpu::Queue`,
    /// consuming the builder to produce the final `Context`.
    ///
    /// # Errors
    ///
    /// Returns error in case of `Adapter::request_device` returns error.
    ///
    /// # Panics
    ///
    /// Panics if the adapter was never set. This cannot happen through the public API: the
    /// `DeviceBuilder` state is only reachable via `adapter_request`/`adapter_request_async`,
    /// which populate `adapter` before this method becomes callable.
    #[ inline ]
    pub async fn context_finish_async( self ) -> Result< Context, crate::Error >
    {
      let ( device, queue ) = self.adapter.as_ref().unwrap().request_device( &self.device_descriptor ).await?;
      let Self {  instance, adapter, .. } = self;
      let instance = instance.unwrap();
      let adapter = adapter.unwrap();

      Ok
      (
        Context { instance, adapter, device, queue }
      )
    }

    /// Synchronously requests the `wgpu::Device` and `wgpu::Queue`, producing the final `Context`.
    ///
    /// # Errors
    ///
    /// Returns error in case of `Adapter::request_device` returns error.
    #[ inline ]
    pub fn context_finish( self ) -> Result< Context, crate::Error >
    {
      pollster::block_on( self.context_finish_async() )
    }
  }
}

mod_interface!
{
  own use Context;
  own use ContextBuilder;
  own use InstanceBuilder;
  own use AdapterBuilder;
  own use DeviceBuilder;
  own use headless;
  own use headless_with;
}
