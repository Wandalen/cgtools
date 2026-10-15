//! Regression coverage for BUG-181 and BUG-193: `jfa_init.frag` and `outline.frag` each
//! independently checked `objectColorTexture.r > 0.01` to decide whether a pixel belongs to an
//! object, which only matched objects whose red channel happened to be close to 1.0 -- any other
//! object color ( pure green/blue/cyan, or even ordinary black ) was silently treated as
//! background in both places: `jfa_init.frag`'s copy meant the object never seeded the JFA (no
//! outline at all), and `outline.frag`'s copy meant the object's own pixels drew the plain
//! background/source color instead of the object's actual appearance.
//!
//! GLSL ES 3.00 has no native/offline execution path in this crate (see
//! `shader_validation_tests.rs`'s own scope note: naga's `glsl-in` front end parses desktop
//! GLSL, not the ES profile these `.frag` files use), so `object_is_present` below is a
//! line-for-line Rust port of the fixed shader check -- identical in both files, since both
//! read the same `OBJECT_COLOR` GBuffer attachment under the same clear/write contract -- kept
//! deliberately close to the GLSL source so the mapping stays auditable. Covered once here
//! rather than duplicated per call site (`jfa_init.frag`'s and `outline.frag`'s checks are
//! textually identical post-fix), with test names distinguishing which file's regression each
//! case guards.

/// Port of the fixed silhouette check shared by `jfa_init.frag`'s and `outline.frag`'s `main()`
/// (`float objectColorR = texture( objectColorTexture, vUv ).r; if ( objectColorR >= 0.0 )`).
pub( crate ) fn object_is_present( object_color_r : f32 ) -> bool
{
  object_color_r >= 0.0
}

/// ## Root Cause
/// `jfa_init.frag` checked `objectColorTexture.r > 0.01` to decide whether a pixel belongs to an
/// object -- but `gbuffer.rs`'s `GBuffer::render` clears the OBJECT_COLOR attachment to
/// `( -1, -1, -1, 1 )`, and `gbuffer.frag` writes the real, caller-supplied object color to every
/// rasterized object pixel. The `> 0.01` threshold only matched objects whose red channel was
/// close to 1.0 -- true only by coincidence of the one caller that existed at the time always
/// using red -- so any object with a different color had its silhouette silently dropped.
/// ## Why Not Caught
/// No test exercised the silhouette check prior to this bug, and the one real caller
/// (`renderer_with_outlines`) hardcodes every object's color to red via `object_colors_generate`,
/// so the defect was invisible until a caller used a non-red object color.
/// ## Fix Applied
/// Changed the comparison from `> 0.01` to `>= 0.0` -- any non-negative red channel can only come
/// from a real ( always non-negative ) object color, never from the negative sentinel clear
/// value, so this correctly detects objects of any color while still excluding the sentinel.
/// ## Prevention
/// This test asserts a black ( r = 0.0 ) object -- the color furthest from the old `r > 1.0`-ish
/// assumption while still being a legitimate, common object color -- is detected as present, that
/// the actual `-1.0` sentinel is not, and that the original red ( r = 1.0 ) case still works.
/// ## Pitfall
/// A magnitude threshold ( `> 0.01` ) silently encodes an assumption about which channel and
/// which value range "presence" looks like. When the actual discriminant is a sign difference
/// against a sentinel ( non-negative real color vs. a negative marker ), a sign check ( `>= 0.0` )
/// is what the data model actually guarantees -- a magnitude threshold picked to fit one caller's
/// current values will silently break for any other legitimate value in-between.
// test_kind: bug_reproducer(BUG-181)
#[ test ]
fn non_red_object_colors_are_still_detected_as_present()
{
  assert!( object_is_present( 0.0 ), "a black ( r = 0.0 ) object must be detected as present, got treated as background" );
}

#[ test ]
fn background_sentinel_is_not_detected_as_present()
{
  assert!( !object_is_present( -1.0 ), "the ( -1, -1, -1 ) background clear sentinel must not be treated as an object" );
}

#[ test ]
fn red_objects_remain_detected_as_present()
{
  assert!( object_is_present( 1.0 ), "a fully red object ( the pre-fix caller's only color ) must remain detected" );
}

/// ## Root Cause
/// `outline.frag` duplicated the same `objectColorTexture.r > 0.01` check independently of
/// `jfa_init.frag` ( BUG-181 ), to decide whether to draw the current pixel with its own
/// source/object color rather than following the JFA outline-distance path. The same GBuffer
/// OBJECT_COLOR contract applies here: `GBuffer::render` clears to `( -1, -1, -1, 1 )` and
/// `gbuffer.frag` writes the caller's arbitrary `objectColor` uniform verbatim, so any object
/// color with `r <= 0.01` was silently treated as background in this second location too --
/// drawing the plain source color over that object's own pixels instead of its actual rendered
/// appearance.
/// ## Why Not Caught
/// Same root cause as BUG-181: the one real caller (`renderer_with_outlines`) hardcodes every
/// object's color to red via `object_colors_generate`. This copy was independently missed even
/// after BUG-181's fix landed, since the two checks live in different files with no shared code
/// path between them.
/// ## Fix Applied
/// Changed `outline.frag`'s comparison from `> 0.01` to `>= 0.0`, identical to BUG-181's fix --
/// reuses the shared `object_is_present` port above rather than a second copy, since the check
/// is textually identical in both files.
/// ## Prevention
/// Exercises the shared `object_is_present` port against the same non-red / sentinel / red cases
/// BUG-181 already covers, so both call sites are backed by one tested formula instead of two
/// independently-verified copies.
/// ## Pitfall
/// A fix applied to one occurrence of a duplicated check does not fix the duplicate -- searching
/// for other copies of the same defect pattern ( here, the same `> 0.01` literal ) after fixing
/// the first is what surfaced this second, independent defect in `outline.frag`.
// test_kind: bug_reproducer(BUG-193)
#[ test ]
fn outline_frag_object_presence_check_matches_jfa_init_frag()
{
  assert!( object_is_present( 0.0 ), "outline.frag: a black ( r = 0.0 ) object must be detected as present, got treated as background" );
  assert!( !object_is_present( -1.0 ), "outline.frag: the ( -1, -1, -1 ) background clear sentinel must not be treated as an object" );
  assert!( object_is_present( 1.0 ), "outline.frag: a fully red object ( the pre-fix caller's only color ) must remain detected" );
}
