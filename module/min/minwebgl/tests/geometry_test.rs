//! Verifies `geometry`'s atom-count validation ( `minwebgl::geometry::natoms_validate` ) —
//! the BUG-052 fix extracted the check into a testable function returning `Result` so an
//! unsupported `natoms` surfaces as a recoverable error instead of a panic. Relocated from
//! inline `src/geometry.rs` per the all-tests-in-tests/ convention; the helper is exported
//! at the `geometry` module path for exactly this purpose.

use minwebgl::{ geometry::natoms_validate, WebglError };

#[ test ]
fn validate_natoms_accepts_supported_values()
{
  for natoms in 1 ..= 4
  {
    assert!( natoms_validate( natoms ).is_ok(), "natoms {natoms} must be supported" );
  }
}

// test_kind: bug_reproducer(BUG-052)
/// ## Root Cause
/// `natoms_validate` (relocated from inline `src/geometry.rs`) panicked via
/// `panic!( "Unsapported buffer descriptor" )` on an unsupported `natoms` value
/// instead of returning a recoverable error.
///
/// ## Why Not Caught
/// The check lived inline in `src/geometry.rs` with no standalone function to call
/// directly, so nothing could unit-test the unsupported-`natoms` path in isolation
/// before this fix extracted it.
///
/// ## Fix Applied
/// Extracted the check into `natoms_validate`, returning `Result< (), WebglError >`
/// (`WebglError::NotSupportedForType`) instead of panicking; the BUG-052 contract —
/// unsupported `natoms` returns `Err`, never panics — is otherwise unchanged.
///
/// ## Prevention
/// RED state (empirically confirmed): reverting this helper's body to the pre-fix
/// `panic!( "Unsapported buffer descriptor" )` and marking this test `#[should_panic]`
/// genuinely panics — verified via a temporary probe before this fix was finalized.
///
/// ## Pitfall
/// The probe values are relative to the currently-supported range, not fixed: the
/// original probe value was `3`; task 062's switch removal made `1 ..= 4` supported,
/// so the unsupported probes moved outside that range to `[ 0, 5, -1 ]`. A future
/// change to the supported range must re-check these probes still fall outside it.
#[ test ]
fn validate_natoms_rejects_unsupported_value()
{
  for natoms in [ 0, 5, -1 ]
  {
    let result = natoms_validate( natoms );
    assert!
    (
      matches!( result, Err( WebglError::NotSupportedForType( _ ) ) ),
      "natoms {natoms} must be rejected with NotSupportedForType"
    );
  }
}
