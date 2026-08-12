//! Verifies `clean`'s attachment-id conversion ( `minwebgl::clean::attachment_id_convert` ) —
//! the TASK-011 fix extracted the conversion into a testable function returning `Result`
//! so out-of-range ids surface as recoverable errors instead of panics. Relocated from
//! inline `src/clean.rs` per the all-tests-in-tests/ convention; the helper is exported
//! at the `clean` module path for exactly this purpose.

use minwebgl::{ clean::attachment_id_convert, WebglError };

/// bug_reproducer(TASK-011)
///
/// ## Root Cause
/// `framebuffer_texture_2d_array`/`framebuffer_renderbuffer_array` converted each
/// caller-supplied attachment id via `TryInto< u32 >` then `.expect()` the conversion —
/// a dynamically computed id that does not fit into `u32` panicked the whole program
/// instead of letting the caller recover, even though this is a realistically
/// recoverable, expected failure mode (ids can come from runtime iteration, not just
/// compile-time-known-good literals).
///
/// ## Why Not Caught
/// `minwebgl` had zero pre-existing tests (no `tests/` directory, no other
/// `#[ cfg( test ) ]` module) before this task, so nothing exercised either function with
/// an out-of-range id.
///
/// ## Fix Applied
/// Extracted the conversion into a `attachment_id_convert` helper returning
/// `Result< u32, WebglError >` (new `WebglError::IdOutOfRange` variant), called via `?`
/// from both functions, which now return `Result< (), WebglError >` instead of `()`.
///
/// ## Prevention
/// RED state (empirically confirmed): reverting this helper's body to the pre-fix
/// `.expect( "Attachment id is out of range" )` and marking this test `#[should_panic]`
/// genuinely panics — verified via a temporary probe before this fix was finalized.
///
/// ## Pitfall
/// `.expect()`/`.unwrap()` inside a loop body over caller-supplied data is easy to miss
/// in review since the surrounding function's own (pre-fix) signature gave no hint that a
/// panic was possible inside.
#[ test ]
fn convert_attachment_id_rejects_out_of_range_input()
{
  let bad_id : i64 = -1;
  let result = attachment_id_convert( bad_id );
  assert!
  (
    matches!( &result, Err( WebglError::IdOutOfRange( _ ) ) ),
    "expected Err( WebglError::IdOutOfRange ), got {result:?}"
  );
}

/// Companion happy-path case: an in-range id still converts successfully.
#[ test ]
fn convert_attachment_id_accepts_in_range_input()
{
  let good_id : i64 = 3;
  assert_eq!( attachment_id_convert( good_id ).unwrap(), 3u32 );
}
