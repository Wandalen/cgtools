//! `Texture` GPU ownership: owning textures vs non-owning views.
//!
//! wasm32-only, like the other `*_gl_lifecycle.rs` suites: whether a texture
//! was deleted is only observable by asking a live context (`gl.is_texture`),
//! so the raw handle is held across every drop.

use minwebgl as gl;
use gl::GL;
use ::renderer::webgl::{ Sampler, Texture };

fn gl_init() -> GL
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default();
  let canvas = gl::canvas::make().unwrap();
  gl::context::from_canvas_with( &canvas, options ).unwrap()
}

/// A texture `gl.is_texture` recognises: WebGL only reports a texture name as
/// a texture once it has been bound, which every real upload path does.
fn bound_texture( gl : &GL ) -> gl::web_sys::WebGlTexture
{
  let texture = gl.create_texture().unwrap();
  gl.bind_texture( gl::TEXTURE_2D, Some( &texture ) );
  gl.bind_texture( gl::TEXTURE_2D, None );
  texture
}

/// A `Texture::owning` texture is shared, not duplicated, by `Clone`: dropping
/// one clone must keep the GPU texture alive for the other, and dropping the
/// last must delete it exactly once.
#[ wasm_bindgen_test::wasm_bindgen_test ]
fn owning_texture_is_deleted_after_its_last_clone_drops()
{
  let gl = gl_init();
  let source = bound_texture( &gl );

  let texture = Texture::owning( &gl, gl::TEXTURE_2D, source.clone(), Sampler::default() );
  let clone = texture.clone();
  assert!( texture.is_owning() && clone.is_owning() );

  drop( texture );
  assert!( gl.is_texture( Some( &source ) ), "a surviving clone must keep the GPU texture alive" );

  drop( clone );
  assert!( !gl.is_texture( Some( &source ) ), "the last owning clone must delete the GPU texture" );
}

/// A `Texture` built with the `Former` builder is a view onto a texture owned
/// elsewhere (a framebuffer attachment, a shared glTF image): dropping it and
/// its clones must never delete the source.
#[ wasm_bindgen_test::wasm_bindgen_test ]
fn view_texture_drop_leaves_source_alive()
{
  let gl = gl_init();
  let source = bound_texture( &gl );

  let view = Texture::former().target( gl::TEXTURE_2D ).source( source.clone() ).form();
  let clone = view.clone();
  assert!( !view.is_owning() );

  drop( view );
  drop( clone );
  assert!( gl.is_texture( Some( &source ) ), "a view must not delete a texture it does not own" );
}

/// A 1x1 opaque red PNG.
const RED_PIXEL_PNG : &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg==";

/// Resolves after `ms` milliseconds, long enough for a data-URI image to load.
async fn sleep_ms( ms : i32 )
{
  let promise = gl::web_sys::js_sys::Promise::new( &mut | resolve, _ |
  {
    gl::web_sys::window().unwrap()
    .set_timeout_with_callback_and_timeout_and_arguments_0( &resolve, ms )
    .unwrap();
  });
  wasm_bindgen_futures::JsFuture::from( promise ).await.unwrap();
}

/// A 1x1 texture filled with `rgba`.
fn solid_texture( gl : &GL, rgba : [ u8; 4 ] ) -> gl::web_sys::WebGlTexture
{
  let texture = gl.create_texture().unwrap();
  gl.bind_texture( gl::TEXTURE_2D, Some( &texture ) );
  gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
  (
    gl::TEXTURE_2D, 0, gl::RGBA as i32, 1, 1, 0, gl::RGBA, gl::UNSIGNED_BYTE, Some( &rgba )
  ).unwrap();
  texture
}

/// The 1x1 texel of `texture`, read back through a framebuffer.
fn texel( gl : &GL, texture : &gl::web_sys::WebGlTexture ) -> [ u8; 4 ]
{
  let framebuffer = gl.create_framebuffer().unwrap();
  gl.bind_framebuffer( gl::FRAMEBUFFER, Some( &framebuffer ) );
  gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, Some( texture ), 0 );
  let mut pixel = [ 0_u8; 4 ];
  gl.read_pixels_with_opt_u8_array( 0, 0, 1, 1, gl::RGBA, gl::UNSIGNED_BYTE, Some( &mut pixel ) ).unwrap();
  gl.bind_framebuffer( gl::FRAMEBUFFER, None );
  gl.delete_framebuffer( Some( &framebuffer ) );
  pixel
}

/// `load_from_path` uploads its image when it arrives: the texel is the image's.
#[ wasm_bindgen_test::wasm_bindgen_test ]
async fn load_from_path_uploads_into_its_own_texture()
{
  let gl = gl_init();
  let texture = Texture::load_from_path( &gl, RED_PIXEL_PNG, false );
  assert!( texture.is_owning(), "load_from_path creates its texture for this Texture alone" );
  sleep_ms( 200 ).await;
  let source = texture.source.clone().unwrap();
  assert_eq!( texel( &gl, &source ), [ 255, 0, 0, 255 ] );
}

/// A `load_from_path` texture dropped before its image arrives must not upload
/// anywhere. Its GPU texture is gone by then, so binding it fails with
/// `INVALID_OPERATION` and leaves the previous binding in place: an upload
/// that still ran would write the image into whatever texture is bound.
#[ wasm_bindgen_test::wasm_bindgen_test ]
async fn load_from_path_dropped_before_load_writes_nowhere()
{
  let gl = gl_init();
  let texture = Texture::load_from_path( &gl, RED_PIXEL_PNG, false );
  drop( texture );

  let bystander = solid_texture( &gl, [ 0, 0, 255, 255 ] );
  gl.bind_texture( gl::TEXTURE_2D, Some( &bystander ) );
  while gl.get_error() != gl::NO_ERROR {}

  sleep_ms( 200 ).await;
  assert_eq!( gl.get_error(), gl::NO_ERROR, "the pending load must not touch the deleted texture" );
  assert_eq!( texel( &gl, &bystander ), [ 0, 0, 255, 255 ], "the image must not land in the bound texture" );
}
