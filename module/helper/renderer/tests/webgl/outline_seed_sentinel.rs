//! Regression coverage for BUG-182: `outline.frag`'s seed-validity check compared
//! `seedCoord.x != 0.0 && seedCoord.y != 0.0` against the JFA sentinel, when the code's own
//! comment states the sentinel is `-1.0` -- an inequality-with-zero test, not the sign check the
//! actual sentinel value requires. This both incorrectly accepted the real `(-1, -1)` sentinel as
//! a "valid" seed ( `-1.0 != 0.0` is true ) and incorrectly rejected any genuinely-found seed
//! coordinate that happened to land exactly on `0.0` on either axis.
//!
//! GLSL ES 3.00 has no native/offline execution path in this crate (see
//! `shader_validation_tests.rs`'s own scope note: naga's `glsl-in` front end parses desktop
//! GLSL, not the ES profile these `.frag` files use), so `seed_is_valid` below is a
//! line-for-line Rust port of the fixed shader check, kept deliberately close to the GLSL
//! source so the mapping stays auditable.

/// Port of the fixed seed-validity check in `outline.frag`'s `main()`
/// (`if ( seedCoord.x >= 0.0 && seedCoord.y >= 0.0 )`).
fn seed_is_valid( seed_coord : [ f32; 2 ] ) -> bool
{
  seed_coord[ 0 ] >= 0.0 && seed_coord[ 1 ] >= 0.0
}

/// ## Root Cause
/// `outline.frag` checked `seedCoord.x != 0.0 && seedCoord.y != 0.0` to decide whether the JFA
/// texture held a real seed coordinate or the `jfa_init.frag`/`jfa_step.frag` sentinel -- but the
/// actual sentinel written on a no-seed-found pixel is `vec4(-1.0, -1.0, -1.0, 1.0)`, and real
/// seed coordinates are always non-negative UV values ( `vec4(vUv, 0.0, 1.0)` ). An
/// inequality-with-zero test is neither necessary nor sufficient for a sign-based sentinel.
/// ## Why Not Caught
/// No test exercised this check prior to this bug. The sentinel `(-1,-1)` incorrectly passing as
/// "valid" produces a distance against a bogus far-off position that usually exceeds any
/// realistic `outlineThickness`, so the resulting mis-draw was visually masked in typical scenes
/// rather than producing an obviously-wrong image.
/// ## Fix Applied
/// Changed the comparison from `!= 0.0` to `>= 0.0` on both components -- matching the sign-based
/// sentinel the code's own comment already named.
/// ## Prevention
/// This test asserts the real `(-1,-1)` sentinel is rejected, that a legitimately-found seed
/// sitting exactly on `0.0` on one axis is accepted, and that an ordinary positive seed
/// coordinate is unaffected -- covering both failure modes the old `!= 0.0` check produced.
/// ## Pitfall
/// An inequality-with-zero test ( `!= 0.0` ) can look like a "not the default/empty value" check,
/// but it silently assumes zero is both the only sentinel value and never a legitimate value --
/// neither holds here: the real sentinel is `-1.0`, and `0.0` is a perfectly valid coordinate.
// test_kind: bug_reproducer(BUG-182)
#[ test ]
fn background_sentinel_seed_is_rejected()
{
  assert!( !seed_is_valid( [ -1.0, -1.0 ] ), "the ( -1, -1 ) JFA sentinel must not be treated as a valid seed" );
}

#[ test ]
fn seed_coordinate_exactly_on_zero_is_still_accepted()
{
  assert!( seed_is_valid( [ 0.0, 0.3 ] ), "a legitimately-found seed with x == 0.0 must still be accepted, not rejected" );
  assert!( seed_is_valid( [ 0.3, 0.0 ] ), "a legitimately-found seed with y == 0.0 must still be accepted, not rejected" );
}

#[ test ]
fn ordinary_positive_seed_is_accepted()
{
  assert!( seed_is_valid( [ 0.5, 0.7 ] ), "an ordinary positive seed coordinate must be accepted" );
}
