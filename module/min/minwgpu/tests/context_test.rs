//! Native tests for the `Context` type-state builder (established by task 070). None of
//! these need a GPU: a `wgpu::Instance` created with `Backends::empty()` deterministically
//! has no adapter to offer, so every adapter request below must resolve to
//! `Err( Error::RequestAdapterError )` — never panic — regardless of the host's hardware.
//! Builder state accumulation is pinned through the `get_*` getters: the instance and
//! adapter stages through the public chain, the device stage through the `doc( hidden )`
//! `Context::device_builder_for_tests` constructor ( that state is unreachable without a
//! real adapter ).

use minwgpu::{ context::Context, Error };

/// A builder chained from `Context::builder()` through `make_instance` with zero backends
/// must surface the adapter failure as the crate's own error type, not a panic.
#[ test ]
fn empty_backends_request_adapter_errors_without_panicking()
{
  let result = Context::builder()
  .backends( wgpu::Backends::empty() )
  .make_instance()
  .request_adapter();

  // `ContextBuilder` is not `Debug` (it holds a boxed selector closure), so destructure
  // instead of formatting the whole `Result`.
  let Err( error ) = result else { panic!( "empty backends must not yield an adapter" ) };
  assert!
  (
    matches!( &error, Error::RequestAdapterError( _ ) ),
    "expected Error::RequestAdapterError, got {error:?}"
  );
}

/// The instance stage accumulates `backends` into the `wgpu::InstanceDescriptor`,
/// observable through `get_instance_descriptor`.
#[ test ]
fn instance_builder_sets_backends()
{
  let builder = Context::builder().backends( wgpu::Backends::VULKAN );
  assert_eq!( builder.get_instance_descriptor().backends, wgpu::Backends::VULKAN );
}

/// The instance stage accumulates `flags` into the `wgpu::InstanceDescriptor`.
#[ test ]
fn instance_builder_sets_flags()
{
  let flags = wgpu::InstanceFlags::VALIDATION;
  let builder = Context::builder().flags( flags );
  assert_eq!( builder.get_instance_descriptor().flags, flags );
}

/// The adapter stage ( reached through the real `make_instance` chain — headless-safe
/// with zero backends ) accumulates `power_preference` into the request options.
#[ test ]
fn adapter_builder_sets_power_preference()
{
  let builder = Context::builder()
  .backends( wgpu::Backends::empty() )
  .make_instance()
  .power_preference( wgpu::PowerPreference::HighPerformance );
  assert_eq!( builder.get_request_adapter_options().power_preference, wgpu::PowerPreference::HighPerformance );
}

/// The adapter stage accumulates `force_fallback_adapter` into the request options.
#[ test ]
fn adapter_builder_sets_force_fallback()
{
  let builder = Context::builder()
  .backends( wgpu::Backends::empty() )
  .make_instance()
  .force_fallback_adapter( true );
  assert!( builder.get_request_adapter_options().force_fallback_adapter );
}

/// Providing a custom selector is recorded ( `has_adapter_selector` ) without invoking it.
#[ test ]
fn adapter_builder_sets_selector()
{
  let builder = Context::builder()
  .backends( wgpu::Backends::empty() )
  .make_instance()
  .adapter_selector( | _ | panic!( "should not be called" ) );
  assert!( builder.has_adapter_selector() );
}

/// The device stage accumulates `label` into the `wgpu::DeviceDescriptor`.
#[ test ]
fn device_builder_sets_label()
{
  let label = String::from( "test_device" );
  let builder = Context::device_builder_for_tests().label( &label );
  assert_eq!( builder.get_device_descriptor().label, Some( "test_device" ) );
}

/// The device stage accumulates `required_features`.
#[ test ]
fn device_builder_sets_features()
{
  let features = wgpu::Features::TEXTURE_COMPRESSION_BC;
  let builder = Context::device_builder_for_tests().required_features( features );
  assert_eq!( builder.get_device_descriptor().required_features, features );
}

/// The device stage accumulates `required_limits`.
#[ test ]
fn device_builder_sets_limits()
{
  let limits = wgpu::Limits { max_bind_groups : 4, ..wgpu::Limits::downlevel_webgl2_defaults() };
  let builder = Context::device_builder_for_tests().required_limits( limits.clone() );
  assert_eq!( builder.get_device_descriptor().required_limits, limits );
}

/// The device stage accumulates `memory_hints` ( `wgpu::MemoryHints` has no `PartialEq`,
/// so the stored variant is compared by discriminant ).
#[ test ]
fn device_builder_sets_memory_hints()
{
  let hints = wgpu::MemoryHints::MemoryUsage;
  let builder = Context::device_builder_for_tests().memory_hints( hints.clone() );

  assert_eq!
  (
    core::mem::discriminant( &builder.get_device_descriptor().memory_hints ),
    core::mem::discriminant( &hints )
  );
}

/// A custom `adapter_selector` takes priority over the request options and its error
/// propagates out of `request_adapter` unchanged in kind. The closure flips a flag so the
/// test proves the selector was genuinely invoked, not bypassed.
#[ test ]
fn adapter_selector_is_invoked_and_its_error_propagates()
{
  // Harvest a real `Error` value first — `wgpu::RequestAdapterError` is not constructible
  // directly, but an empty-backends request produces one deterministically.
  let harvest = Context::builder()
  .backends( wgpu::Backends::empty() )
  .make_instance()
  .request_adapter();
  let Err( harvested ) = harvest else { panic!( "empty backends must not yield an adapter" ) };

  let called = core::cell::Cell::new( false );
  let mut err_slot = Some( harvested );

  let result = Context::builder()
  .backends( wgpu::Backends::empty() )
  .make_instance()
  .adapter_selector
  (
    | _instance |
    {
      called.set( true );
      Err( err_slot.take().expect( "selector must be called exactly once" ) )
    }
  )
  .request_adapter();

  assert!( called.get(), "custom adapter_selector must be invoked" );
  let Err( error ) = result else { panic!( "selector error must propagate as Err" ) };
  assert!
  (
    matches!( &error, Error::RequestAdapterError( _ ) ),
    "selector error must propagate, got {error:?}"
  );
}

/// `Context::from_instance` enters the builder at the adapter stage: the returned builder
/// accepts adapter-stage configuration and resolves the request against the provided
/// instance — here one with no backends, so the request must error, not panic.
#[ test ]
fn from_instance_supports_adapter_stage_configuration()
{
  let descriptor = wgpu::InstanceDescriptor
  {
    backends : wgpu::Backends::empty(),
    ..Default::default()
  };
  let instance = wgpu::Instance::new( &descriptor );

  let result = Context::from_instance( instance )
  .power_preference( wgpu::PowerPreference::HighPerformance )
  .force_fallback_adapter( false )
  .request_adapter();

  let Err( error ) = result else { panic!( "empty backends must not yield an adapter" ) };
  assert!
  (
    matches!( &error, Error::RequestAdapterError( _ ) ),
    "expected Error::RequestAdapterError, got {error:?}"
  );
}
