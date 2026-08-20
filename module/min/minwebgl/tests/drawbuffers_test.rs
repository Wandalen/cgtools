//! Verifies `drawbuffers`'s color-attachment index validation
//! ( `minwebgl::drawbuffers::color_attachment_index_validate` ) — the BUG-159 fix extracted the
//! bounds check into a testable function returning `Result` so an out-of-range index surfaces
//! via the function's own documented message instead of a raw, undocumented array-index panic.
//! Relocated from inline `src/drawbuffers.rs` per the all-tests-in-tests/ convention.

use minwebgl::{ drawbuffers::color_attachment_index_validate, WebglError };

#[ test ]
fn validate_color_attachment_index_accepts_in_range_values()
{
  for index in 0 .. 16
  {
    assert_eq!( color_attachment_index_validate( index ).unwrap(), index, "index {index} must be accepted" );
  }
}

// test_kind: bug_reproducer(BUG-159)
/// ## Root Cause
/// `drawbuffers` indexed a fixed `MAX_COLOR_ATTACHMENTS` ( 16 ) -element array with the
/// caller's raw attachment index and no bounds check. The only guard present ( `checked_add`
/// against `u32::MAX` when computing the attachment id ) never rejects an ordinary
/// out-of-range index like `16` — that only overflows near `u32::MAX - COLOR_ATTACHMENT0` — so
/// calling `drawbuffers(&gl, &[16])` panicked via a raw, undocumented
/// "index out of bounds: the len is 16 but the index is 16" instead of the function's own
/// documented `"Invalid color attachment {}"` message.
///
/// ## Why Not Caught
/// `drawbuffers` takes `&GL` and has no pure-logic twin to unit-test, and every current call
/// site in this repo passes only literal indices 0–3 — well within range.
///
/// ## Fix Applied
/// Extracted the bounds check into `color_attachment_index_validate`, returning
/// `Result< usize, WebglError >` ( `WebglError::IdOutOfRange` ); `drawbuffers` calls it via
/// `.expect(...)`, keeping its own already-documented "Panics if..." contract but now panicking
/// on the actual out-of-range condition with an attributable message.
///
/// ## Prevention
/// RED state (empirically confirmed): reverting this helper's body to unconditionally
/// `Ok( index )` and re-running this test genuinely fails ( no `IdOutOfRange` is ever
/// constructed ) — verified via a temporary probe before this fix was finalized.
///
/// ## Pitfall
/// `MAX_COLOR_ATTACHMENTS` bounds the ARRAY INDEX ( 0..16 ), not the attachment id after adding
/// `COLOR_ATTACHMENT0` ( 36064..36080 ) — a guard placed on the wrong quantity ( the sum,
/// checked only for u32 overflow ) can look like it's validating the right thing while actually
/// leaving the real bound ( 16 ) completely unchecked.
#[ test ]
fn validate_color_attachment_index_rejects_out_of_range_values()
{
  for index in [ 16usize, 17, 100 ]
  {
    let result = color_attachment_index_validate( index );
    assert!
    (
      matches!( &result, Err( WebglError::IdOutOfRange( _ ) ) ),
      "index {index} must be rejected with IdOutOfRange, got {result:?}"
    );
  }
}
