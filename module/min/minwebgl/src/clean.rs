/// Internal namespace.
mod private
{
  #[ allow( clippy::wildcard_imports, reason = "crate-root prelude from mod_interface!; enumerating would break on every layer change" ) ]
  use crate::*;
  /// A type alias for the WebGL2 rendering context.
  type GL = WebGl2RenderingContext;

  /// Converts an attachment id into a `u32`, returning `WebglError::IdOutOfRange` instead of
  /// panicking when the id does not fit into `u32`. An id computed at runtime (e.g. while
  /// iterating a dynamically sized framebuffer configuration) can legitimately be out of
  /// range, and callers should be able to recover from that instead of the process crashing.
  // Fix(TASK-011): `framebuffer_texture_2d_array`/`framebuffer_renderbuffer_array` used to
  // convert each attachment id via `i.try_into().expect( "Attachment id is out of range" )`,
  // panicking the whole process on a dynamically computed id that doesn't fit into `u32`.
  // Root cause: both functions accept any `IntoIterator`, so a caller-supplied id is not
  // guaranteed to be a compile-time-known-good literal, yet the conversion used `.expect()`
  // instead of propagating through the `Result` a caller could otherwise recover from.
  // Pitfall: `.expect()`/`.unwrap()` inside a loop body over caller-supplied data is easy to
  // miss in review since the surrounding function's own (pre-fix) signature gave no hint that
  // a panic was possible inside.
  fn convert_attachment_id< I, E >( id : I ) -> Result< u32, WebglError >
  where
    E : std::fmt::Debug,
    I : TryInto< u32, Error = E >
  {
    id.try_into().map_err( | e | WebglError::IdOutOfRange( format!( "Attachment id is out of range : {e:?}" ) ) )
  }

  /// Unbind the currently bound 2D texture.
  pub fn texture_2d( gl : &GL )
  {
    gl.bind_texture( GL::TEXTURE_2D, None );
  }

  /// Unbind the 2D texture from a specific texture unit.
  pub fn texture_2d_active( gl : &GL, active : u32 )
  {
    gl.active_texture( GL::TEXTURE0 + active );
    texture_2d( gl );
  }

  /// Unbind 2D textures from multiple texture units.
  ///
  /// # Panics
  /// Panics if any `active` item fails to convert into a `u32` texture unit id.
  pub fn texture_2d_array< T, E >( gl : &GL, active : T )
  where 
    T : IntoIterator,
    E : std::fmt::Debug,
    T::Item : TryInto< u32, Error = E >
  {
    for i in active
    {
      texture_2d_active( gl, i.try_into().expect( "Active id is out of range" ) );
    }
  }

  /// Unbind the currently bound framebuffer.
  pub fn framebuffer( gl : &GL )
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );
  }

  /// Detach a 2D texture from a specific framebuffer attachment.
  pub fn framebuffer_texture_2d_attachment( gl : &GL, attachment : u32 )
  {
    gl.framebuffer_texture_2d
    (
      GL::FRAMEBUFFER, 
      GL::COLOR_ATTACHMENT0 + attachment, 
      GL::TEXTURE_2D, 
      None, 
      0
    );
  } 

  /// Detach the 2D texture from framebuffer attachment 0.
  pub fn framebuffer_texture_2d( gl : &GL )
  {
    framebuffer_texture_2d_attachment( gl, 0 );
  } 

  /// Detach 2D textures from multiple framebuffer attachments.
  ///
  /// # Errors
  /// Returns `WebglError::IdOutOfRange` if any attachment id does not fit into `u32`.
  pub fn framebuffer_texture_2d_array< T, E >( gl : &GL, attachments : T ) -> Result< (), WebglError >
  where
    T : IntoIterator,
    E : std::fmt::Debug,
    T::Item : TryInto< u32, Error = E >
  {
    for i in attachments
    {
      framebuffer_texture_2d_attachment( gl, convert_attachment_id( i )? );
    }
    Ok( () )
  }

  /// Detaches a renderbuffer from a specific color attachment point of the currently bound framebuffer.
  pub fn framebuffer_renderbuffer_attachment( gl : &GL, attachment : u32 )
  {
    gl.framebuffer_texture_2d
    (
      GL::FRAMEBUFFER, 
      GL::COLOR_ATTACHMENT0 + attachment, 
      GL::RENDERBUFFER, 
      None, 
      0
    );
  } 

  /// Detach the renderbuffer from framebuffer attachment 0.
  pub fn framebuffer_renderbuffer( gl : &GL )
  {
    framebuffer_renderbuffer_attachment( gl, 0 );
  } 

  /// Detach renderbuffers from multiple framebuffer attachments.
  ///
  /// # Errors
  /// Returns `WebglError::IdOutOfRange` if any attachment id does not fit into `u32`.
  pub fn framebuffer_renderbuffer_array< T, E >( gl : &GL, attachments : T ) -> Result< (), WebglError >
  where
    T : IntoIterator,
    E : std::fmt::Debug,
    T::Item : TryInto< u32, Error = E >
  {
    for i in attachments
    {
      framebuffer_renderbuffer_attachment( gl, convert_attachment_id( i )? );
    }
    Ok( () )
  }

  // Documented exception (task 069) to the all-tests-in-tests/ convention: these tests stay
  // inline because `convert_attachment_id` is a private helper by design -- extracting it INTO
  // a testable private function returning `Result` was the TASK-011 fix; publishing it solely
  // for test placement would widen the API for no caller. Native `tests/` coverage of the
  // crate's public pure-logic surface lives in `tests/` (see the readme's Testing section for
  // the full runnability story).
  #[ cfg( test ) ]
  mod tests
  {
    use super::*;

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
    /// Extracted the conversion into a private `convert_attachment_id` helper returning
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
      let result = convert_attachment_id( bad_id );
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
      assert_eq!( convert_attachment_id( good_id ).unwrap(), 3u32 );
    }
  }

}

crate::mod_interface!
{
  own use
  {
    framebuffer,
    framebuffer_renderbuffer,
    framebuffer_renderbuffer_array,
    framebuffer_renderbuffer_attachment,
    framebuffer_texture_2d,
    framebuffer_texture_2d_array,
    framebuffer_texture_2d_attachment,
    texture_2d,
    texture_2d_array,
    texture_2d_active
  };
}
