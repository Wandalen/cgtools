//! Verifies the legacy `renderer::webgl::Renderer::render()` path's
//! drawbuffers attachment-selection logic
//! ( `renderer::webgl::frame_attachments` ) — the pure function of
//! `( has_transparent, has_emissive )` that picks which color attachments
//! `render()` enables for a frame — with zero `WebGl2RenderingContext`/`gl::`
//! calls anywhere in its body. Mirrors `native_render_test.rs`'s coverage of
//! the canonical webgpu path's frame shape, at the orchestration level
//! rather than the pixel level.

use renderer::webgl::frame_attachments;

#[ test ]
fn no_transparent_no_emissive_yields_main_color_only()
{
  assert_eq!( frame_attachments( false, false ), &[ 0 ] );
}

#[ test ]
fn no_transparent_with_emissive_yields_main_and_emission()
{
  assert_eq!( frame_attachments( false, true ), &[ 0, 1 ] );
}

#[ test ]
fn transparent_no_emissive_yields_main_and_accumulate_revealage()
{
  assert_eq!( frame_attachments( true, false ), &[ 0, 2, 3 ] );
}

#[ test ]
fn transparent_and_emissive_yields_all_four_attachments()
{
  assert_eq!( frame_attachments( true, true ), &[ 0, 1, 2, 3 ] );
}
