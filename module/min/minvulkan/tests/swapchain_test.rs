//!
//! Regression test for `minvulkan::swapchain::Swapchain::rebuild`'s error-path handle/view
//! cleanup. Source-inspection only, mirroring `context_test.rs`'s BUG-290 test : this crate
//! deliberately introduces no mock/fake Vulkan layer ( real implementations only ), and
//! `surface_test.rs`'s own header already documents that `Swapchain::new`/`rebuild` need a live
//! `VkSurfaceKHR`, which needs a real window handle this crate cannot produce in a native test —
//! so there is no way to runtime-trigger `get_swapchain_images`/`image_view_create` failing
//! after a real `vkCreateSwapchainKHR` has already succeeded, from this test suite at all.
//!

// test_kind: bug_reproducer(BUG-424)
/// ## Root Cause
/// `Swapchain::rebuild` creates a new `VkSwapchainKHR` via `create_swapchain`, then calls
/// `get_swapchain_images` and, for each image, `image_view_create` — both behind `?` — before
/// ever assigning the new handle/views into `self`. Before this fix, a failure in either step
/// left the just-created `VkSwapchainKHR` handle with no path to destruction : `?` propagates
/// the error immediately, and nothing had assigned `handle` into `self.handle` yet for `Drop`/
/// `destroy_chain` to find it. View creation made this worse : it used
/// `.map( image_view_create ).collect::< Result< Vec< _ >, Error > >()`, which, on the Nth
/// image's view failing, discards the first N-1 already-created `VkImageView`s as an ordinary
/// `Vec` alongside the collected `Err` — those handles were real, driver-allocated resources,
/// not plain Rust values, so discarding the `Vec` leaked every one of them too. Structurally
/// identical to `Fix(BUG-290)`'s `Context` instance leak ( `context.rs`, `context_test.rs` ) :
/// a live handle created successfully, then abandoned on a sibling step's failure with no
/// cleanup.
/// ## Why Not Caught
/// `context_test.rs`'s and `surface_test.rs`'s existing tests only exercise `Swapchain`
/// indirectly, if at all, and none force `get_swapchain_images` or `create_image_view` to fail
/// after a real `create_swapchain` has already succeeded — this crate is a real, driver-backed
/// wrapper with no mock ICD to make that failure deterministically triggerable, and (per
/// `surface_test.rs`'s own header) `Swapchain::new`/`rebuild` need a live `VkSurfaceKHR`, which
/// needs a real window handle this crate deliberately cannot produce at all in a native test —
/// so the leaking branches were structurally unreachable from this test suite, not merely
/// untested by it. A resource leak also produces no visible symptom in an ordinary black-box
/// test even where reachable : the function still returns the correct `Err`, and nothing in the
/// public API surfaces "was `vkDestroySwapchainKHR`/`vkDestroyImageView` called."
/// ## Fix Applied
/// Added `SwapchainGuard`, an RAII guard ( mirroring `Fix(BUG-290)`'s `InstanceGuard` ) that
/// takes ownership of the new `VkSwapchainKHR` handle immediately after creation and
/// accumulates each `VkImageView` one at a time via `view_push` as `rebuild`'s `for` loop
/// creates them — deliberately not `.map().collect::< Result< Vec< _ >, _ > >()`, which would
/// still discard already-created views on a later failure. `SwapchainGuard`'s `Drop` destroys
/// every held view then the handle ; `rebuild`'s only success path calls `guard.disarm()`,
/// which hands both out without destroying them, right before they're assigned into `self`.
/// ## Prevention
/// Before relying on ordinary Rust drop, or `.collect::< Result< Vec< _ >, _ > >()`'s
/// discard-on-`Err` behavior, to clean up FFI-owned resources on an error path, verify the
/// items involved don't need their own explicit destruction call — `ash`'s handle wrapper types
/// carry no `Drop` impl of their own, by Vulkan's own explicit-cleanup design, and neither does
/// discarding a `Vec` of them replace that call.
/// ## Pitfall
/// A `.collect::< Result< Vec< _ >, _ > >()` turning "many fallible steps" into "one fallible
/// step" is exactly the right tool when every already-`Ok` item is a pure Rust value ; the
/// moment those items are FFI handles needing their own explicit destruction, the same pattern
/// silently converts "N-1 successes, 1 failure" into "N-1 leaks, 1 correctly-reported error" —
/// the `Err` propagates faithfully, so nothing about the function's return value hints anything
/// was lost.
#[ test ]
fn swapchain_rebuild_guards_handle_and_views_on_error_path()
{
  let src = include_str!( "../src/swapchain.rs" );

  let guard_struct_count = src.matches( "struct SwapchainGuard" ).count();
  assert_eq!
  (
    guard_struct_count, 1,
    "expected exactly one SwapchainGuard definition (BUG-424), found {guard_struct_count}"
  );

  let guard_drop_count = src.matches( "impl Drop for SwapchainGuard" ).count();
  assert_eq!
  (
    guard_drop_count, 1,
    "expected exactly one Drop impl for SwapchainGuard (BUG-424), found {guard_drop_count}"
  );

  let guard_construct_count = src.matches( "SwapchainGuard::new( &self.loader, &self.device, handle )" ).count();
  assert_eq!
  (
    guard_construct_count, 1,
    "rebuild must construct SwapchainGuard immediately after create_swapchain succeeds \
    (BUG-424), found {guard_construct_count} call sites"
  );

  let guard_disarm_count = src.matches( "guard.disarm()" ).count();
  assert_eq!
  (
    guard_disarm_count, 1,
    "rebuild's success path must call guard.disarm() exactly once, right before committing \
    handle/views into self (BUG-424), found {guard_disarm_count} call sites"
  );

  // The leak-prone collect -- discarding already-created views alongside the first Err -- must
  // be gone, not merely supplemented by the guard.
  let leaky_collect_count = src.matches( ".collect::< Result< Vec< _ >, Error > >()" ).count();
  assert_eq!
  (
    leaky_collect_count, 0,
    "the leak-prone `.map().collect::< Result< Vec< _ >, Error > >()` view-creation pattern \
    (BUG-424) must be fully replaced by the incremental guard.view_push loop, found \
    {leaky_collect_count} remaining occurrences"
  );

  // SwapchainGuard's Drop must destroy both artifact kinds it can hold -- a guard that only
  // cleaned up one of the two would still leak the other.
  assert!
  (
    src.contains( "self.device.destroy_image_view( view, None )" ),
    "SwapchainGuard::drop (BUG-424) must destroy every held view via destroy_image_view"
  );
  assert!
  (
    src.contains( "self.loader.destroy_swapchain( handle, None )" ),
    "SwapchainGuard::drop (BUG-424) must destroy the held handle via destroy_swapchain"
  );
}
