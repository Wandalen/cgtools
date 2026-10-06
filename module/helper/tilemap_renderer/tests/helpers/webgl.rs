//! Live-context fixtures shared by the WebGL2 browser suites
//! ( `webgl_context_loss_test.rs`, `webgl_pending_image_test.rs`,
//! `webgl_premultiplied_test.rs` ).
//!
//! Gated on wasm32 and `adapter-webgl` in `helpers/mod.rs`, since the native
//! suites pull in the same `helpers` module and have no browser to run these in.

use minwebgl as gl;

/// Builds a real, live WebGL2 context via a headless browser canvas. Same helper shape as
/// `renderer`'s `tests/webgl/*.rs::gl_init()` ( `gl::browser::setup` / `gl::canvas::make` /
/// `gl::context::from_canvas_with` ).
pub fn gl_init() -> gl::GL
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default();
  let canvas = gl::canvas::make().unwrap();
  gl::context::from_canvas_with( &canvas, options ).unwrap()
}

/// Resolves after `ms` milliseconds, giving the browser a task to run an image
/// decode or dispatch context events in.
pub async fn sleep( ms : i32 )
{
  let promise = gl::js_sys::Promise::new( &mut | resolve, _reject |
  {
    web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0( &resolve, ms ).unwrap();
  });
  gl::JsFuture::from( promise ).await.unwrap();
}

/// The RGBA bytes of pixel `( 0, 0 )` of the default framebuffer.
pub fn pixel_read( gl : &gl::GL ) -> [ u8; 4 ]
{
  let mut pixel = [ 0_u8; 4 ];
  gl.read_pixels_with_opt_u8_array( 0, 0, 1, 1, gl::RGBA, gl::UNSIGNED_BYTE, Some( &mut pixel ) ).unwrap();
  pixel
}

/// Little-endian `f32` bytes for a geometry `Source::Bytes` buffer.
pub fn f32_bytes( values : &[ f32 ] ) -> Vec< u8 >
{
  values.iter().flat_map( | v | v.to_le_bytes() ).collect()
}
