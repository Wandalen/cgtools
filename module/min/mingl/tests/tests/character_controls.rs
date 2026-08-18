use super::*;
use approx::assert_abs_diff_eq;

/// ## Root Cause
/// `CharacterControls::rotate()` mapped both the yaw delta and the pitch delta onto
/// their respective angles using the same `-=` operator, without re-deriving the sign
/// relationship separately for each axis. For yaw, `-=` is the *correct* choice: the
/// character's own "right" basis vector is `-X` (`right_xz()`'s base quaternion is
/// `[-1,0,0,0]`) while increasing `yaw` rotates `forward` toward `+X`
/// (`Quat::from_angle_y` + the Hamilton product both confirm this), so subtracting
/// `delta_x` (DOM `movementX`, positive = pointer moved right) makes a rightward mouse
/// move decrease yaw and correctly turn the character right. Pitch has no analogous
/// inversion: increasing `pitch` already rotates `forward.y` negative (down) directly
/// via `Quat::from_angle_x` (verified by hand: `from_angle_x( PI/2 )` applied to
/// `(0,0,1)` yields `(0,-1,0)`), so mapping `delta_y` (DOM `movementY`, positive =
/// pointer moved down) onto pitch needed `+=` to reproduce "mouse down -> look down".
/// The `-=` was copy-pasted from the yaw line immediately above it without re-deriving
/// this, inverting the pitch axis so mouse-down looked *up* and mouse-up looked *down*.
///
/// ## Why Not Caught
/// No test file existed for `character_controls.rs` before this one -- only its sibling
/// `camera_orbit_controls.rs` had coverage. The inversion is invisible to a cursory
/// manual smoke test of a WASD+mouselook demo unless someone explicitly checks that
/// dragging the mouse down tilts the view down rather than up.
///
/// ## Fix Applied
/// Changed `self.pitch -= delta_y * self.rotation_sensitivity;` to
/// `self.pitch += delta_y * self.rotation_sensitivity;` in
/// `CharacterControls::rotate()` (`src/controls/character_controls.rs`).
///
/// ## Prevention
/// When two sibling axes share a delta-application pattern (both written as `-=` right
/// next to each other), re-derive each axis's sign independently against its own
/// basis-vector relationship instead of copying the neighboring line's operator; a
/// regression test asserting the exact numeric `forward()` vector for a known angle is
/// what catches this class of bug (a "doesn't panic" test would not).
///
/// ## Pitfall
/// A copy-pasted `-=` sitting directly beneath a correctly-negated sibling line reads
/// as "obviously consistent" on visual review; only re-deriving the sign from the
/// underlying rotation math -- not from the sibling's already-established operator --
/// reveals the mismatch.
// test_kind: bug_reproducer(BUG-278)
#[ test ]
fn test_rotate_pitch_matches_mouse_down_looks_down_convention()
{
  let mut controls = the_module::controls::character_controls::CharacterControls::default();
  let sensitivity = controls.rotation_sensitivity;

  // Simulate a pointer-locked `mousemove` event with a purely downward movement.
  // `MouseEvent.movementY` is positive when the pointer moves down (DOM spec), and
  // `mouse_move_closure_make` forwards it into `rotate()` verbatim, unnegated.
  let delta_y = 100.0;
  controls.rotate( 0.0, delta_y );

  // Hand-computed expected pitch: `rotate()`'s own documented formula is
  // `pitch = delta_y * rotation_sensitivity` (mouse down increases pitch).
  let expected_pitch = delta_y * sensitivity;
  assert_abs_diff_eq!( controls.pitch(), expected_pitch, epsilon = 1e-9 );

  // Hand-computed expected forward direction at yaw = 0: rotating the base forward
  // vector `(0,0,1)` by `pitch` radians around the X axis (`Quat::from_angle_x`) gives
  // `(0, -sin(pitch), cos(pitch))` -- derived independently via the Hamilton product
  // in `Quat::multiply` and cross-checked numerically at pitch = PI/2 (giving `(0,-1,0)`).
  let expected_forward = the_module::F64x3::new( 0.0, -expected_pitch.sin(), expected_pitch.cos() );
  assert_abs_diff_eq!( controls.forward(), expected_forward, epsilon = 1e-9 );

  // Plain-English restatement of the same assertion: moving the mouse down must tilt
  // the view down, i.e. forward.y must go negative, not positive.
  assert!
  (
    controls.forward().y() < 0.0,
    "moving the mouse down must tilt the view down (forward.y < 0), got forward = {:?}",
    controls.forward()
  );
}

/// Sanity check for the opposite direction: moving the mouse up must tilt the view up
/// (`forward.y` positive), the mirror image of the downward case above.
#[ test ]
fn test_rotate_pitch_mouse_up_looks_up()
{
  let mut controls = the_module::controls::character_controls::CharacterControls::default();
  let sensitivity = controls.rotation_sensitivity;

  let delta_y = -100.0;
  controls.rotate( 0.0, delta_y );

  let expected_pitch = delta_y * sensitivity;
  assert_abs_diff_eq!( controls.pitch(), expected_pitch, epsilon = 1e-9 );
  assert!
  (
    controls.forward().y() > 0.0,
    "moving the mouse up must tilt the view up (forward.y > 0), got forward = {:?}",
    controls.forward()
  );
}

/// Confirms the yaw axis (already correct) is untouched by the pitch fix: a rightward
/// mouse move must still turn the character right, i.e. `forward` rotates toward the
/// character's own `right_xz()` vector.
#[ test ]
fn test_rotate_yaw_matches_mouse_right_turns_right_convention()
{
  let mut controls = the_module::controls::character_controls::CharacterControls::default();
  let sensitivity = controls.rotation_sensitivity;

  let right_before = controls.right_xz();

  let delta_x = 100.0;
  controls.rotate( delta_x, 0.0 );

  let expected_yaw = -delta_x * sensitivity;
  assert_abs_diff_eq!( controls.yaw(), expected_yaw, epsilon = 1e-9 );

  // Turning right means the new forward vector gains a component along the old
  // "right" direction (their dot product must be positive).
  let forward_after = controls.forward_xz();
  assert!
  (
    forward_after.dot( &right_before ) > 0.0,
    "moving the mouse right must turn the character toward its own right vector"
  );
}
