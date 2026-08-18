//! Sprite animation example — plays a 2D sprite-sheet animation from a texture with WebGL2.

use minwebgl as gl;

fn main()
{
  gl::spawn_local( async move { app_run().await.unwrap() } );
}

async fn app_run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let gl = gl::context::retrieve_or_make()?;

  let vert_shader = include_str!( "../shaders/main.vert" );
  let frag_shader = include_str!( "../shaders/main.frag" );
  let program = gl::ProgramFromSources::new( vert_shader, frag_shader ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  // Settings for `rock.png` sprite sheets
  let path = "static/rock.png";
  let image_element = gl::dom::image_element_create( path )
  .expect( "Failed to create image element" );
  let sprite_sheet = gl::texture::d2::SpriteSheet
  {
    sprites_in_row: 8,
    sprite_width: 128,
    sprite_height: 128,
    amount: 64,
  };

  gl::texture::d2::sprite_upload( &gl, &image_element, &sprite_sheet ).await?;

  let update_and_draw =
  {
    let mut step = 0.0;
    let frame_rate = 24.0;
    let hold_ticks = sprite_sheet.amount as f32 - 1.0;
    let sprite_count = sprite_sheet.amount as f32;

    move | _ |
    {
      // BUG-313 task/bug/313_sprite_animation_modulus_skips_last_frame.md --
      // Fix(BUG-313): wraparound modulus must be `sprite_count`, not `sprite_count - 1`.
      // Root cause: `frame % (sprite_count - 1)` can never equal `sprite_count - 1`, so the
      // sprite sheet's last frame was permanently unreachable.
      // Pitfall: `hold_ticks` (how long each frame is held) and the wraparound range (how
      // many distinct frames exist) are different quantities that must not share one value.
      gl.vertex_attrib1f( 0, sprite_frame_index( step, hold_ticks, sprite_count ) );
      gl.draw_arrays( gl::GL::TRIANGLE_STRIP, 0, 4 );

      step += frame_rate;

      true
    }
  };

  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

/// Computes the sprite-sheet frame index to display for a given animation `step`.
///
/// `hold_ticks` controls how many `step` units each frame is held before advancing.
/// `sprite_count` is the total number of distinct frames in the sheet -- the wraparound
/// range must equal this exactly, so the returned index cycles through `[0, sprite_count)`
/// inclusive of the last frame.
fn sprite_frame_index( step : f32, hold_ticks : f32, sprite_count : f32 ) -> f32
{
  let frame = ( step / hold_ticks ).floor();
  frame % sprite_count
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  /// ## Root Cause
  /// `update_and_draw`'s frame-index computation used the same `amount` value
  /// (`sprite_sheet.amount as f32 - 1.0`, i.e. `63.0` for a 64-frame sheet) for two
  /// unrelated purposes: the divisor controlling how long each frame is held (a pacing
  /// choice, fine as `count - 1`), and the modulus controlling which frame index range is
  /// valid (must be exactly `count` -- `frame % (count - 1)` can only ever produce values
  /// in `[0, count - 1)`, permanently excluding index `count - 1`, the sheet's last frame).
  ///
  /// ## Why Not Caught
  /// No test file existed for this crate -- it is a `fn main()`-only WebGL demo binary,
  /// verified only by running it in a browser. Skipping 1 frame out of 64 in a continuously
  /// looping animation is easy to miss by eye, especially since the animation still loops
  /// smoothly (just one frame short) with no crash or visible glitch.
  ///
  /// ## Fix Applied
  /// Split the single `amount` variable into `hold_ticks` (unchanged, `count - 1`, pacing
  /// only) and `sprite_count` (`count`, the true wraparound range), and extracted the
  /// computation into `sprite_frame_index()` so the two roles can never be silently
  /// conflated again.
  ///
  /// ## Prevention
  /// This test picks the exact `step` that makes `frame == sprite_count - 1` (the last
  /// valid index) via closed-form arithmetic (`step = (sprite_count - 1) * hold_ticks`),
  /// then asserts the fixed call reaches it while the pre-fix buggy call (passing
  /// `hold_ticks` as the modulus too) silently wraps back to `0.0` instead.
  ///
  /// ## Pitfall
  /// A modulus base and a divisor that happen to start from the same source count are not
  /// automatically the same value -- `count` (a size) and `count - 1` (a maximum valid
  /// index) serve different roles and must not be collapsed into one shared variable.
  // BUG-313 task/bug/313_sprite_animation_modulus_skips_last_frame.md -- reproducer for the
  // sprite sheet's last frame being permanently unreachable due to an off-by-one modulus.
  // test_kind: bug_reproducer(BUG-313)
  #[ test ]
  fn test_sprite_frame_index_reaches_last_frame()
  {
    let sprite_count = 64.0_f32;
    let hold_ticks = sprite_count - 1.0;

    // Closed-form `step` making `frame == sprite_count - 1.0` (63.0) exactly:
    // `step / hold_ticks == 63.0` when `step == 63.0 * hold_ticks`.
    let step = ( sprite_count - 1.0 ) * hold_ticks;

    // Frame indices are conceptually integers (`sprite_frame_index` returns `f32` only
    // because `vertex_attrib1f` requires it); compare as `i32` -- exact for these
    // whole-number inputs and avoids a raw-float equality comparison.
    let correct = sprite_frame_index( step, hold_ticks, sprite_count );
    assert_eq!( correct as i32, ( sprite_count - 1.0 ) as i32, "the last sprite frame must be reachable" );

    // The example's pre-fix expression used `hold_ticks` as BOTH the hold-divisor and the
    // modulus base -- at this exact `step`, `frame % hold_ticks == 63.0 % 63.0 == 0.0`,
    // silently wrapping back to frame 0 instead of showing the last frame.
    let buggy = sprite_frame_index( step, hold_ticks, hold_ticks );
    assert_eq!( buggy as i32, 0 );
    assert_ne!( buggy as i32, correct as i32 );
  }
}
