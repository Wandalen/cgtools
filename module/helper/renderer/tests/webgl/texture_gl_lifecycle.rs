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
