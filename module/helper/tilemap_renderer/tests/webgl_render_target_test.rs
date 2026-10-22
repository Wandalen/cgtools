//! `TargetSlots`, the generation-checked slot table behind `RenderTargetId`.
//!
//! GL-free: the table only stores values and stamps handles, so the
//! stale-handle rule `WebGlBackend::render_targets_reset` relies on is checked
//! on the host, without a WebGL2 context.

#![ cfg( all( feature = "adapter-webgl", feature = "test_internals" ) ) ]

use tilemap_renderer::adapters::webgl::render_target::internal::TargetSlots;

/// Two pushes give two distinct handles, and each resolves to its own value
/// until the table is reset. A cache surface is drawn by handle every frame, so
/// a handle that resolved to the wrong slot would draw another layer's cache.
#[ test ]
fn target_slots_get_resolves_until_reset()
{
  let mut slots = TargetSlots::new();
  let a = slots.push( "a" ).expect( "slot" );
  let b = slots.push( "b" ).expect( "slot" );
  assert_ne!( a, b );
  assert_eq!( slots.get( a ), Some( &"a" ) );
  assert_eq!( slots.get( b ), Some( &"b" ) );
}

/// Two tables, as two backends hold, never share a generation, so a handle
/// from one is rejected by the other even at the same slot. If both started at
/// the same generation, a cache handle passed to the wrong backend would draw
/// that backend's own surface in its place.
#[ test ]
fn target_slots_get_rejects_another_tables_handle()
{
  let mut first = TargetSlots::new();
  let mut second = TargetSlots::new();
  let from_first = first.push( "first" ).expect( "slot" );
  let from_second = second.push( "second" ).expect( "slot" );
  assert_eq!( second.get( from_first ), None, "slot 0 of another table" );
  assert_eq!( first.get( from_second ), None );
  assert_eq!( first.get( from_first ), Some( &"first" ) );
}

/// After a reset the first push reuses slot 0 under a new generation, so the
/// handle from before the reset no longer resolves. Without the generation it
/// would silently address the new surface that took its slot.
#[ test ]
fn target_slots_get_rejects_stale_handle_after_slot_reuse()
{
  let mut slots = TargetSlots::new();
  let old = slots.push( "old" ).expect( "slot" );
  slots.clear();
  let new = slots.push( "new" ).expect( "slot" );
  assert_ne!( old, new, "same slot, new generation" );
  assert_eq!( slots.get( old ), None, "a handle from before the reset must not reach the new surface" );
  assert_eq!( slots.get( new ), Some( &"new" ) );
}
