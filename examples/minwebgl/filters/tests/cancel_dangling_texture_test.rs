//! Regression tests verifying `filters::renderer::Renderer::image_texture_set`'s aliasing guard
//! never deletes a GL texture handle it is simultaneously being asked to keep.
//!
//! `filters` is a binary-only example crate (no `[lib]` target) whose `Renderer` needs a real
//! `web_sys::WebGlTexture` -- a JS-interop handle only constructible inside an actual browser --
//! so, like `hsl_wraparound_test.rs`/`blur_kernel_test.rs`, this test reads `renderer.rs`'s own
//! real source text via `include_str!` (to anchor against regression of the actual guard fix) and
//! exercises a hand-ported pure-Rust mirror of its handle-bookkeeping algorithm, using plain `u32`
//! IDs standing in for opaque, comparison-only `WebGlTexture` handles (`WebGlTexture`'s own
//! `PartialEq` is reference equality, exactly what `u32` equality mirrors here).

const RENDERER_RS : &str = include_str!( "../src/renderer.rs" );

/// Pure-Rust mirror of `Renderer`'s texture-handle bookkeeping (`image_texture`,
/// `original_texture`, `previous_texture`, and the `image_texture_set`/`previous_texture_save`/
/// `previous_texture_restore` methods), standing in for real `WebGlTexture`s with comparable
/// `u32` IDs and recording every simulated `gl.delete_texture` call instead of issuing one.
#[ derive( Default ) ]
struct TextureState
{
  image_texture : Option< u32 >,
  original_texture : Option< u32 >,
  previous_texture : Option< u32 >,
  deleted : Vec< u32 >,
}

impl TextureState
{
  /// Mirrors the FIXED `image_texture_set`: skips the delete when `old` aliases
  /// `original_texture` OR when the incoming value is `old` itself (self-assignment).
  fn image_texture_set( &mut self, image_texture : Option< u32 > )
  {
    if let Some( old ) = self.image_texture.take()
    {
      let aliases_original = self.original_texture == Some( old );
      let is_self_assign = image_texture == Some( old );
      if !aliases_original && !is_self_assign
      {
        self.deleted.push( old );
      }
    }
    self.image_texture = image_texture;
  }

  fn previous_texture_save( &mut self )
  {
    self.previous_texture = self.image_texture;
  }

  fn previous_texture_restore( &mut self )
  {
    if let Some( previous ) = self.previous_texture.take()
    {
      self.image_texture_set( Some( previous ) );
    }
  }
}

/// ## Root Cause
/// `Renderer::image_texture_set`'s BUG-463 aliasing guard only ever compared the outgoing texture
/// (`old`) against `original_texture` before deciding to `gl.delete_texture` it — it never checked
/// whether the *incoming* replacement value was `old` itself. `previous_texture_restore` always
/// calls this setter with a clone of whatever `previous_texture_save` cloned out of
/// `self.image_texture` earlier, and nothing in between ever mutates `image_texture` (every
/// `Filter::draw` takes `&impl FilterRenderer`, an immutable borrow with no field-mutating access
/// — a filter preview only draws to the canvas/framebuffer, it never replaces this field). So on
/// every real Cancel, `old` and the incoming `image_texture` argument are the exact same handle —
/// a self-assignment through a setter whose whole job is "delete the thing being replaced". Once
/// `original_texture` no longer happened to alias that same handle too (true the moment at least
/// one "Apply" click has already baked a new base texture earlier in the session), the guard's one
/// check no longer protected it, and the setter deleted the very texture `image_texture` was about
/// to be reassigned to.
///
/// ## Why Not Caught
/// BUG-463's own verification hand-traced Cancel's call path, but only against a fresh upload
/// (`image_texture` still aliasing `original_texture`, where the guard's existing check already
/// prevents deletion by coincidence) — it never traced the sequence of "Apply once, then select a
/// filter and Cancel", the one where `image_texture` has since decoupled from `original_texture`
/// and the self-assignment case becomes live. Visually the symptom is a broken/blank canvas after
/// Cancel (WebGL renders a deleted texture as empty), easy to misread as "Cancel does nothing"
/// rather than "Cancel corrupts the very texture it restores".
///
/// ## Fix Applied
/// `image_texture_set` (and, mirrored defensively, `original_texture_set`) now also skips the
/// delete when the incoming replacement value is identical to the outgoing one, alongside the
/// pre-existing `original_texture`/`image_texture` sibling-aliasing check.
///
/// ## Prevention
/// This test asserts the real source now carries the self-assignment check (catching a regression
/// back to the single-condition guard), and exercises a pure-Rust mirror of the fixed algorithm
/// across the exact reachable sequence (upload → filter → Apply → filter → Cancel) that makes the
/// self-assignment case live, asserting the restored handle is never recorded as deleted.
///
/// ## Pitfall
/// An aliasing guard that special-cases one *other* field can still miss the simplest alias of
/// all — the incoming value aliasing the very value it's replacing. Always check self-assignment
/// first, independent of whatever other sibling-field reasoning a setter also needs.
#[ test ]
fn bug_reproducer_bug_503_cancel_does_not_delete_the_texture_it_restores()
{
  assert!
  (
    RENDERER_RS.contains( "is_self_assign = image_texture.as_ref() == Some( &old )" ),
    "Renderer::image_texture_set should skip deleting `old` when the incoming replacement is the \
    same handle as `old` (self-assignment) -- without this check, `previous_texture_restore` \
    deletes the very texture it is restoring `image_texture` to (BUG-503)"
  );

  let mut state = TextureState::default();

  // Fresh upload: image_texture and original_texture alias the same handle (1).
  state.original_texture = Some( 1 );
  state.image_texture = Some( 1 );

  // First filter session: select a filter (saves baseline 1), preview (no mutation), then Apply
  // bakes a new base texture (2) -- decoupling image_texture from original_texture.
  state.previous_texture_save();
  state.image_texture_set( Some( 2 ) );
  state.previous_texture = None; // mirrors `apply_button_setup`'s own `previous_state_clear()`

  assert!
  (
    !state.deleted.contains( &2 ),
    "sanity: handle 2 was just created by Apply, it must not already be recorded as deleted"
  );

  // Second filter session: select a filter again -- saves the now-decoupled baseline (2), then
  // preview only (still no mutation of image_texture).
  state.previous_texture_save();
  assert_eq!( state.previous_texture, Some( 2 ) );
  assert_eq!( state.image_texture, Some( 2 ) );

  // Cancel: restores `image_texture` to `previous_texture`, which is the same handle (2) it
  // already holds.
  state.previous_texture_restore();

  assert!
  (
    !state.deleted.contains( &2 ),
    "Cancel deleted handle 2 -- the very texture it was simultaneously restoring `image_texture` \
    to, leaving `image_texture` pointing at a texture that was just freed (BUG-503)"
  );
  assert_eq!
  (
    state.image_texture, Some( 2 ),
    "image_texture should still be the restored handle after Cancel"
  );
  assert_eq!( state.previous_texture, None, "previous_texture should be consumed by restore" );
}
