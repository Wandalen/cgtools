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

// test_kind: bug_reproducer(BUG-290)
/// ## Root Cause
/// `context_finish`'s 3 error paths ( `Error::PhysicalDeviceEnumerate`, `Error::NoSuitableDevice`,
/// `Error::DeviceCreate` ) all occur after `instance_make` has already produced a live
/// `ash::Instance`, but each one propagated its error via a bare `?`/`.ok_or()?`, which drops
/// `instance` as an ordinary Rust value on the way out. `ash::Instance` has no `Drop` impl of its
/// own -- confirmed by reading `ash` 0.38's own source, which defines no `Drop` impl anywhere in
/// the crate for `Entry`, `Instance`, or `Device` -- Vulkan mandates explicit `vkDestroyInstance`
/// instead. So every one of these 3 error paths silently leaked the created `VkInstance` handle.
/// ## Why Not Caught
/// `context_test.rs`'s existing tests ( T01-T03 ) only exercise the success path -- none force
/// `enumerate_physical_devices`/`create_device` to fail or every physical device to lack a
/// graphics queue family, so the leaking branches were never executed by any test. Separately, a
/// resource leak produces no visible symptom in an ordinary black-box test : the function still
/// returns the correct `Err( Error::X )` either way, and nothing in the public API surfaces
/// "was `vkDestroyInstance` called." None of the 3 failure conditions are practically triggerable
/// against a real local Vulkan implementation from outside `context_finish` without faking the
/// Vulkan layer -- this is a real, driver-backed crate, so no fake/mock ICD is introduced to force
/// the branch. This regression test instead asserts the fix's presence directly in the source,
/// the same source-inspection approach BUG-287/BUG-288 used for their own hard-to-runtime-test
/// (there, doc-only ; here, cleanup-only) defects.
/// ## Fix Applied
/// Added `instance_cleanup_on_error`, a one-line `unsafe fn( &ash::Instance )` wrapping
/// `destroy_instance( None )`, called from all 3 error-producing points in `context_finish`
/// before propagating the `Err` -- via `.map_err`/`.ok_or_else`, never `.ok_or`, since `.ok_or`
/// evaluates its argument eagerly and would have destroyed the instance on the *success* path
/// too, a far worse bug than the leak it fixes.
/// ## Prevention
/// Before relying on ordinary Rust drop to clean up an FFI-owned resource on an error path,
/// verify the wrapper type actually implements `Drop` for that purpose -- `ash`'s handle wrapper
/// types deliberately do not, by Vulkan's own explicit-cleanup design.
/// ## Pitfall
/// A missing `Drop` impl doesn't announce itself -- assuming "ownership will clean this up" is
/// only true for types that actually implement `Drop` for that purpose ; FFI/driver-handle
/// wrapper types very often don't, by design, and each one must be checked individually rather
/// than assumed from Rust's ordinary RAII conventions.
#[ test ]
fn context_finish_destroys_instance_on_every_error_path()
{
  let src = include_str!( "../src/context.rs" );

  let helper_def_count = src.matches( "fn instance_cleanup_on_error" ).count();
  assert_eq!
  (
    helper_def_count, 1,
    "expected exactly one instance_cleanup_on_error helper definition (BUG-290), found {helper_def_count}"
  );

  let cleanup_call_count = src.matches( "instance_cleanup_on_error( &instance )" ).count();
  assert_eq!
  (
    cleanup_call_count, 3,
    "context_finish must call instance_cleanup_on_error( &instance ) on all 3 error-producing \
    paths -- enumerate_physical_devices failure, no suitable device, create_device failure -- \
    to avoid leaking the VkInstance handle (BUG-290) ; found {cleanup_call_count} call sites"
  );
}
