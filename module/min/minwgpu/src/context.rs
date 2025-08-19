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

    /// Returns a reference to the `wgpu::Instance`.
    #[ inline ]
    #[ must_use ]
    pub fn instance( &self ) -> &wgpu::Instance
    {
      &self.instance
    }

    /// Returns a reference to the `wgpu::Adapter`.
    #[ inline ]
    #[ must_use ]
    pub fn adapter( &self ) -> &wgpu::Adapter
    {
      &self.adapter
    }

    /// Returns a reference to the `wgpu::Device`.
    #[ inline ]
    #[ must_use ]
    pub fn device( &self ) -> &wgpu::Device
    {
      &self.device
    }

    /// Returns a reference to the `wgpu::Queue`.
    #[ inline ]
    #[ must_use ]
    pub fn queue( &self ) -> &wgpu::Queue
    {
      &self.queue
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

  pub type AdapterSelector< 's > = Box< dyn FnMut( &wgpu::Instance ) -> Result< wgpu::Adapter, crate::Error > + 's >;

  pub struct InstanceBuilder;

  pub struct AdapterBuilder;

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
    pub fn make_instance( mut self ) -> ContextBuilder< 'a, 'b, 'l, 's, AdapterBuilder >
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
    #[ inline ]
    #[ allow( clippy::missing_panics_doc ) ]
    pub async fn request_adapter_async( mut self ) -> Result< ContextBuilder< 'a, 'b, 'l, 's, DeviceBuilder >, crate::Error >
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
    pub fn request_adapter( self ) -> Result< ContextBuilder< 'a, 'b, 'l, 's, DeviceBuilder >, crate::Error >
    {
      pollster::block_on( self.request_adapter_async() )
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
    #[ inline ]
    #[ allow( clippy::missing_panics_doc ) ]
    pub async fn finish_context_async( self ) -> Result< Context, crate::Error >
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
    pub fn finish_context( self ) -> Result< Context, crate::Error >
    {
      pollster::block_on( self.finish_context_async() )
    }
  }
}

#[ cfg( test ) ]
mod tests
{
  use super::private::*;
  use core::marker::PhantomData;
  use wgpu::{ Backends, InstanceFlags, PowerPreference, Features, Limits, MemoryHints };

  // Helper to construct a builder in the AdapterBuilder state for testing
  fn adapter_builder_state() -> ContextBuilder< 'static, 'static, 'static, 'static, AdapterBuilder >
  {
    ContextBuilder
    {
      _state: PhantomData,
      instance_descriptor: wgpu::InstanceDescriptor::default(),
      request_adapter_options: wgpu::RequestAdapterOptions::default(),
      device_descriptor: wgpu::DeviceDescriptor::default(),
      instance: None, // In a real scenario, this would be Some(wgpu::Instance)
      adapter: None,
      adapter_selector: None,
    }
  }

  // Helper to construct a builder in the DeviceBuilder state for testing
  fn device_builder_state() -> ContextBuilder< 'static, 'static, 'static, 'static, DeviceBuilder >
  {
    ContextBuilder
    {
      _state: PhantomData,
      instance_descriptor: wgpu::InstanceDescriptor::default(),
      request_adapter_options: wgpu::RequestAdapterOptions::default(),
      device_descriptor: wgpu::DeviceDescriptor::default(),
      instance: None,
      adapter: None, // In a real scenario, this would be Some(wgpu::Adapter)
      adapter_selector: None,
    }
  }

  #[ test ]
  fn instance_builder_sets_backends()
  {
    let builder = Context::builder().backends( Backends::VULKAN );
    assert_eq!( builder.instance_descriptor.backends, Backends::VULKAN );
  }

  #[ test ]
  fn instance_builder_sets_flags()
  {
    let flags = InstanceFlags::VALIDATION;
    let builder = Context::builder().flags( flags );
    assert_eq!( builder.instance_descriptor.flags, flags );
  }

  #[ test ]
  fn adapter_builder_sets_power_preference()
  {
    let builder = adapter_builder_state().power_preference( PowerPreference::HighPerformance );
    assert_eq!( builder.request_adapter_options.power_preference, PowerPreference::HighPerformance );
  }

  #[ test ]
  fn adapter_builder_sets_force_fallback()
  {
    let builder = adapter_builder_state().force_fallback_adapter( true );
    assert!( builder.request_adapter_options.force_fallback_adapter );
  }

  #[ test ]
  fn adapter_builder_sets_selector()
  {
    let builder = adapter_builder_state().adapter_selector( |_| panic!( "should not be called" ) );
    assert!( builder.adapter_selector.is_some() );
  }

  #[ test ]
  fn device_builder_sets_label()
  {
    let label = String::from( "test_device" );
    let builder = device_builder_state().label( &label );
    assert_eq!( builder.device_descriptor.label, Some( "test_device" ) );
  }

  #[ test ]
  fn device_builder_sets_features()
  {
    let features = Features::TEXTURE_COMPRESSION_BC;
    let builder = device_builder_state().required_features( features );
    assert_eq!( builder.device_descriptor.required_features, features );
  }

  #[ test ]
  fn device_builder_sets_limits()
  {
    let limits = Limits { max_bind_groups: 4, ..Limits::downlevel_webgl2_defaults() };
    let builder = device_builder_state().required_limits( limits.clone() );
    assert_eq!( builder.device_descriptor.required_limits, limits );
  }

  #[ test ]
  fn device_builder_sets_memory_hints()
  {
    let hints = MemoryHints::MemoryUsage;
    let builder = device_builder_state().memory_hints( hints.clone() );

    assert_eq!
    (
      core::mem::discriminant( &builder.device_descriptor.memory_hints ),
      core::mem::discriminant( &hints )
    );
  }
}

mod_interface!
{
  own use Context;
  own use ContextBuilder;
}
