//! Native tests for `Context` construction against a real `wgpu` adapter, and for
//! `texture::render_target_2d`'s actual `wgpu::Device::create_texture` call. `context_test.rs`
//! and `texture_test.rs` both deliberately stay GPU-free ( every builder there is fed
//! `Backends::empty()`, and the texture test only pins the pure zero-size precondition — see
//! their own header doc comments ); this file is the native counterpart that exercises the
//! parts of the public surface that genuinely need a live adapter/device, mirroring
//! `minvulkan/tests/context_test.rs`'s coverage shape, adapted to `wgpu`.
//!
//! Every test here needs a real adapter reachable on `wgpu::Backends::PRIMARY` ( Vulkan, Metal,
//! or DX12 -- a software ICD such as lavapipe / mesa-vulkan-drivers suffices ). When
//! `adapter_request` itself fails ( no adapter at all on this host ), `try_live_context` prints a
//! clear reason and returns `None`, and the calling test returns immediately instead of
//! panicking or failing the suite. A subsequent `context_finish` ( device request ) failure is
//! treated as a genuine failure, not a skip, since a real adapter was already found by that
//! point.

use minwgpu::{ context::Context, texture };

/// Builds a real [`Context`] against `wgpu::Backends::PRIMARY` through the crate's own public
/// builder chain ( `Context::builder()` ... `context_finish()` ) -- the finishing step
/// `context_test.rs` never reaches because every one of its builders is deliberately fed
/// `Backends::empty()`.
///
/// Returns `None` -- after printing a clear, greppable reason to stderr -- only when
/// `adapter_request` fails, i.e. no adapter is available on any backend `Backends::PRIMARY` can
/// reach. Panics ( via `expect` ) if an adapter was found but the subsequent device request
/// still failed, since that is a real defect rather than an environment limitation.
fn try_live_context( test_name : &str ) -> Option< Context >
{
  let device_stage = Context::builder()
  .backends( wgpu::Backends::PRIMARY )
  .instance_make()
  .adapter_request();

  let device_stage = match device_stage
  {
    Ok( stage ) => stage,
    Err( error ) =>
    {
      eprintln!
      (
        "skipping {test_name}: no real wgpu adapter available on Backends::PRIMARY ( {error} ) \
        -- needs a Vulkan / Metal / DX12 ICD ( a software one such as lavapipe / \
        mesa-vulkan-drivers suffices )"
      );
      return None;
    }
  };

  Some
  (
    device_stage.context_finish()
    .expect( "an adapter was found but context_finish (device request) failed" )
  )
}

/// The builder chain reaches a real `Device`/`Queue` pair that can actually do work: a
/// zero-command submission is accepted and the subsequent poll completes without error, and the
/// device's granted limits are at least the crate's default request ( `max_texture_dimension_2d`
/// = 8192, guaranteed to work on all modern backends per `wgpu::Limits::defaults` -- device
/// creation itself would have failed had the adapter been unable to satisfy it ).
#[ test ]
fn live_context_device_and_queue_are_usable()
{
  let Some( context ) = try_live_context( "live_context_device_and_queue_are_usable" ) else { return };

  let encoder = context.device_get().create_command_encoder( &wgpu::CommandEncoderDescriptor::default() );
  context.queue_get().submit( core::iter::once( encoder.finish() ) );
  context.device_get().poll( wgpu::PollType::wait_indefinitely() )
  .expect( "polling the device after a no-op submission must succeed on a real device" );

  let limits = context.device_get().limits();
  assert!
  (
    limits.max_texture_dimension_2d >= 8192,
    "expected a real device to grant at least the default max_texture_dimension_2d (8192), got {}",
    limits.max_texture_dimension_2d
  );
}

/// `render_target_2d`'s actual `wgpu::Device::create_texture` call, exercised against a real
/// device -- the resulting `wgpu::Texture`'s own queried properties ( not just the crate's
/// `Extent3d` bookkeeping ) must match what was requested.
#[ test ]
fn render_target_2d_creates_real_texture_with_expected_properties()
{
  let Some( context ) = try_live_context( "render_target_2d_creates_real_texture_with_expected_properties" ) else { return };

  let format = wgpu::TextureFormat::Rgba8Unorm;
  let target = texture::render_target_2d( context.device_get(), ( 64, 64 ), format );

  assert_eq!( target.texture.width(), 64, "the real texture must honor the requested width" );
  assert_eq!( target.texture.height(), 64, "the real texture must honor the requested height" );
  assert_eq!( target.texture.depth_or_array_layers(), 1 );
  assert_eq!( target.texture.dimension(), wgpu::TextureDimension::D2 );
  assert_eq!( target.texture.format(), format, "the real texture's format must match what was requested" );
  assert_eq!
  (
    target.texture.usage(),
    wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    "render_target_2d must request RENDER_ATTACHMENT | COPY_SRC usage from the real device"
  );
  assert_eq!( target.extend, wgpu::Extent3d { width : 64, height : 64, depth_or_array_layers : 1 } );
}
