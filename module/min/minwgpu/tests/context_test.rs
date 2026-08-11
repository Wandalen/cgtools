//! Native tests for the `Context` type-state builder's public error surface (established by
//! task 070). None of these need a GPU: a `wgpu::Instance` created with `Backends::empty()`
//! deterministically has no adapter to offer, so every adapter request below must resolve to
//! `Err( Error::RequestAdapterError )` — never panic — regardless of the host's hardware.
//! (The builders' state-accumulation logic is covered by the documented-exception inline
//! tests in `src/`, which read internal fields no public getter exposes.)

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
