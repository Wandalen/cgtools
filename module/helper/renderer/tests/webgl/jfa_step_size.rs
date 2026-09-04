//! Regression coverage for BUG-180: `WideOutlinePass::jfa_step_pass` scaled the JFA step's
//! horizontal jump distance by `width / height` before uploading it as the `stepSize` uniform --
//! but `jfa_step.frag` already converts `stepSize` from pixels to normalized UV space per-axis
//! (`ceil( dir * stepSize ) / resolution`, dividing each axis by its own resolution component),
//! which alone already produces a uniform real-pixel jump on a non-square canvas. The extra
//! Rust-side scaling double-applied that correction, stretching the JFA search radius --
//! and therefore the rendered outline -- wider than tall on any non-square canvas.
//!
//! `jfa_step_pass` is private and GL-embedded (no pure function to extract without adding new
//! public API surface for a purely internal computation), so -- following this crate's own
//! `white_balance.rs` precedent for logic with no test-reachable execution path -- the functions
//! below are a line-for-line Rust port of the fixed Rust-side computation plus the ( unchanged,
//! already-correct ) shader-side conversion, kept close to both sources so the mapping stays
//! auditable.

/// Port of the fixed `jfa_step_pass` step-size computation (`wide_outline.rs`):
/// `let step_size = outline_thickness / 2f32.powf( i ); [ step_size, step_size ]`.
fn jfa_step_size( outline_thickness : f32, i : u32 ) -> [ f32; 2 ]
{
  let step_size = outline_thickness / ( 2.0_f32 ).powf( i as f32 );
  [ step_size, step_size ]
}

/// Port of `jfa_step.frag`'s per-neighbor offset computation, converted back to real screen
/// pixels ( `offset_uv * resolution` ) so it's directly comparable across a non-square canvas:
/// `vec2 offset = ceil( dir * stepSize ) / resolution;`.
fn jfa_step_pixel_jump( dir : [ f32; 2 ], step_size : [ f32; 2 ], resolution : [ f32; 2 ] ) -> [ f32; 2 ]
{
  let offset_uv =
  [
    ( dir[ 0 ] * step_size[ 0 ] ).ceil() / resolution[ 0 ],
    ( dir[ 1 ] * step_size[ 1 ] ).ceil() / resolution[ 1 ],
  ];
  [ offset_uv[ 0 ] * resolution[ 0 ], offset_uv[ 1 ] * resolution[ 1 ] ]
}

/// ## Root Cause
/// `jfa_step_pass` scaled `stepSize.x` ( but not `.y` ) by `width / height` before upload, on
/// top of `jfa_step.frag`'s own per-axis `/ resolution` conversion, which already accounts for a
/// non-square canvas. Applying both meant the real per-axis pixel jump worked out to
/// `step_size * aspect_ratio` horizontally vs. just `step_size` vertically.
/// ## Why Not Caught
/// No test exercised `jfa_step_pass` prior to this bug, and the distortion is a subtle
/// elliptical stretch rather than a crash or missing pixel -- easy to miss on visual inspection
/// of a demo scene, especially since outlines are inherently a bit soft already.
/// ## Fix Applied
/// Removed the `* aspect_ratio` scaling; both `stepSize` components now carry the same pixel
/// distance, relying entirely on the shader's own per-axis `/ resolution` division for correct
/// non-square-canvas normalization.
/// ## Prevention
/// This test computes the real per-axis pixel jump ( Rust step-size port + shader offset port,
/// composed exactly as production code composes them ) for a non-square resolution and asserts
/// both axes agree -- the pre-fix formula would have failed this by a factor of `width / height`.
/// ## Pitfall
/// When a value is converted from pixel space to normalized UV space via a component-wise
/// `/ resolution` on a non-square texture, that division has ALREADY corrected for aspect ratio
/// per axis -- scaling the pre-division value by `width / height` first double-corrects rather
/// than compensating for anything still missing.
// test_kind: bug_reproducer(BUG-180)
#[ test ]
fn jfa_step_pixel_jump_is_isotropic_on_a_non_square_canvas()
{
  let resolution = [ 1920.0, 1080.0 ];
  let outline_thickness = 64.0;

  for i in 0 .. 3
  {
    let step_size = jfa_step_size( outline_thickness, i );
    let jump = jfa_step_pixel_jump( [ 1.0, 1.0 ], step_size, resolution );
    assert!
    (
      ( jump[ 0 ] - jump[ 1 ] ).abs() < 1e-4,
      "JFA step {i}: real pixel jump must be equal in both axes on a non-square canvas, got {jump:?}"
    );
  }
}

#[ test ]
fn jfa_step_pixel_jump_matches_configured_thickness_at_step_zero()
{
  // A non-square, non-16:9 canvas as well, to rule out a coincidental cancellation specific to
  // one aspect ratio.
  let resolution = [ 800.0, 600.0 ];
  let outline_thickness = 40.0;

  let step_size = jfa_step_size( outline_thickness, 0 );
  let jump = jfa_step_pixel_jump( [ 1.0, 1.0 ], step_size, resolution );

  assert!( ( jump[ 0 ] - outline_thickness ).abs() < 1e-4, "expected x jump == outline_thickness, got {jump:?}" );
  assert!( ( jump[ 1 ] - outline_thickness ).abs() < 1e-4, "expected y jump == outline_thickness, got {jump:?}" );
}
