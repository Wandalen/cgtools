//!
//! Integration tests for `minvulkan::Context` construction against a real local Vulkan
//! implementation. See `tests/manual/readme.md` for the runtime this exercises.
//!

#![ allow( unsafe_code, reason = "exercises raw `ash` FFI calls against a live Vulkan device ; \
each call site carries its own `// SAFETY:` comment rather than repeating this justification at \
every one of them" ) ]

use ash::vk::Handle;
use minvulkan::context::Context;

/// T01 : the builder chain completes with `Ok`, and the resulting `Context` carries
/// non-null physical-device and queue handles.
#[ test ]
fn context_builder_produces_valid_handles()
{
  let context = Context::builder()
  .instance_make()
  .expect( "instance_make should succeed against the local Vulkan loader" )
  .context_finish()
  .expect( "context_finish should succeed against the local Vulkan loader" );

  assert_ne!( context.physical_device_get().as_raw(), 0, "physical device handle must not be VK_NULL_HANDLE" );
  assert_ne!( context.queue_get().as_raw(), 0, "queue handle must not be VK_NULL_HANDLE" );
}

/// T02 : the logical device is genuinely live — `vkDeviceWaitIdle` succeeds on it.
#[ test ]
fn context_device_is_live()
{
  let context = Context::builder()
  .instance_make()
  .expect( "instance_make should succeed against the local Vulkan loader" )
  .context_finish()
  .expect( "context_finish should succeed against the local Vulkan loader" );

  // SAFETY: `context.device_get()` returns the live `ash::Device` owned by `context`,
  // which is not dropped until after this call returns.
  let result = unsafe { context.device_get().device_wait_idle() };
  assert!( result.is_ok(), "device_wait_idle should succeed on a freshly-created, idle device" );
}

/// T03 : the queue family index `Context` selected is independently re-derived here, via a
/// fresh enumeration against the same instance/physical device, rather than re-asserting the
/// builder's own internal choice.
#[ test ]
fn context_queue_family_index_matches_independent_derivation()
{
  let context = Context::builder()
  .instance_make()
  .expect( "instance_make should succeed against the local Vulkan loader" )
  .context_finish()
  .expect( "context_finish should succeed against the local Vulkan loader" );

  let instance = context.instance_get();
  let physical_device = context.physical_device_get();

  // SAFETY: `instance` is the live instance owned by `context`, and `physical_device` was
  // enumerated from that same instance by `context_finish` ; querying its queue family
  // properties performs no writes through caller-supplied pointers.
  let properties = unsafe { instance.get_physical_device_queue_family_properties( physical_device ) };
  let expected_index = properties
  .iter()
  .position( | family | family.queue_flags.contains( ash::vk::QueueFlags::GRAPHICS ) )
  .expect( "the physical device Context selected must itself expose a graphics-capable queue family" );

  assert_eq!( context.queue_family_index_get(), u32::try_from( expected_index ).expect( "index fits u32" ) );
}
