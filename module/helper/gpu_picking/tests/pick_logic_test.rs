//! The crate's pure, context-free logic — the sentinel mapping, the bounds
//! check, and the pick-id validation.
//!
//! `IdProgram`/`PickBuffer`'s own methods all require a live
//! `WebGl2RenderingContext` to construct or call (framebuffers, textures,
//! shader compilation), which a native `cargo nextest` run cannot provide —
//! the same Wasm Native-Check Blind Spot already established in this workspace
//! (see `primitive_generation/tests/geometry_normal_attribute_test.rs`). These
//! three functions are the only logic this crate can test natively;
//! live-context teardown checks live in `pick_buffer_drop_test.rs` instead.
//!
//! All three are private, reached here through the `test_internals` feature —
//! they are deliberately not surface, since a caller drives them through
//! `PickBuffer::pick` and `IdProgram::draw_part` rather than directly.

#![ cfg( feature = "test_internals" ) ]

use gpu_picking::internal::{ assert_pick_id_valid, pick_in_bounds, readback_to_pick_id };

#[ test ]
fn background_sentinel_maps_to_none()
{
  assert_eq!( readback_to_pick_id( -1 ), None, "-1 is the documented background sentinel" );
}

#[ test ]
fn zero_and_positive_ids_map_to_some()
{
  assert_eq!( readback_to_pick_id( 0 ), Some( 0 ), "id 0 is a valid, pickable id, not background" );
  assert_eq!( readback_to_pick_id( 7 ), Some( 7 ) );
  assert_eq!( readback_to_pick_id( i32::MAX ), Some( i32::MAX ) );
}

/// ## Root Cause
/// `PickBuffer::pick` passed `(x, y)` straight to `read_pixels` with no
/// validation against the buffer's own `(width, height)` -- an out-of-range
/// coordinate's outcome was left entirely to driver-specific `read_pixels`
/// behavior instead of being handled by this crate, and a freshly-created
/// buffer's `readback` starts zero-filled (JS `TypedArray`s always
/// zero-initialize), so an out-of-range read that leaves it untouched reads
/// back as a false `Some(0)` instead of `None`.
///
/// ## Why Not Caught
/// `gpu_picking` had zero test coverage of any kind before this sweep --
/// nothing exercised out-of-range pick coordinates, and the bug only manifests
/// as a wrong *value* (not a panic or compile error), so it would silently
/// misreport picks near/past the canvas edge.
///
/// ## Fix Applied
/// Added `pick_in_bounds`, called at the top of `PickBuffer::pick` -- returns
/// `None` immediately for any `(x, y)` outside `[0, width) x [0, height)`,
/// before ever touching the GPU.
///
/// ## Prevention
/// These cases cover all four boundary edges (`x`/`y` each at `-1` and at
/// exactly `width`/`height`, the classic off-by-one edge) plus the degenerate
/// `0x0` buffer, so a regression re-widening or dropping the check trips at
/// least one boundary case immediately.
///
/// ## Pitfall
/// `x < width` (not `x <= width`) is the correct upper check -- valid columns
/// are `0..width`, so `x == width` is one column past the last valid one and
/// must be rejected, not accepted.
// test_kind: bug_reproducer(BUG-530)
#[ test ]
fn pick_in_bounds_rejects_out_of_range_coordinates()
{
  assert!( pick_in_bounds( 0, 0, 4, 4 ), "(0,0) is the first valid pixel" );
  assert!( pick_in_bounds( 3, 3, 4, 4 ), "(3,3) is the last valid pixel of a 4x4 buffer" );

  assert!( !pick_in_bounds( -1, 0, 4, 4 ), "negative x must be rejected" );
  assert!( !pick_in_bounds( 0, -1, 4, 4 ), "negative y must be rejected" );
  assert!( !pick_in_bounds( 4, 0, 4, 4 ), "x == width is one past the last valid column" );
  assert!( !pick_in_bounds( 0, 4, 4, 4 ), "y == height is one past the last valid row" );
  assert!( !pick_in_bounds( 0, 0, 0, 0 ), "a 0x0 buffer has no valid pixel at all" );
}

/// ## Root Cause
/// `Pickable::pick_id`'s doc comment never stated the `>= 0` constraint
/// implied by `readback_to_pick_id` treating `-1` as the reserved background
/// sentinel -- a part using `-1` (or any negative id) rendered successfully
/// but could never be read back by `PickBuffer::pick`, silently making it
/// permanently unpickable.
///
/// ## Why Not Caught
/// `gpu_picking` had zero test coverage of any kind before this sweep, and the
/// failure mode is silent (no panic, no error -- the part just never gets
/// picked), so nothing would surface it short of a user noticing an object
/// simply doesn't respond to clicks.
///
/// ## Fix Applied
/// Added `assert_pick_id_valid`, called from `IdProgram::draw_part` for every
/// part drawn -- panics loudly (instead of silently producing an unpickable
/// part) the moment a negative `pick_id` is used.
///
/// ## Prevention
/// Covers the exact boundary (`-1`, the reserved sentinel itself) plus the
/// valid boundary (`0`) and two representative valid values, so a regression
/// loosening the check to allow negatives trips immediately.
///
/// ## Pitfall
/// `id >= 0` (not `id > 0`) is the correct check -- `0` is a valid, pickable id
/// (see `zero_and_positive_ids_map_to_some` above); only negative values are
/// reserved.
// test_kind: bug_reproducer(BUG-513)
#[ test ]
#[ should_panic( expected = "pick ids must be >= 0" ) ]
fn negative_pick_id_panics()
{
  assert_pick_id_valid( -1 );
}

#[ test ]
fn zero_and_positive_pick_ids_are_accepted()
{
  assert_pick_id_valid( 0 );
  assert_pick_id_valid( 42 );
  assert_pick_id_valid( i32::MAX );
}
