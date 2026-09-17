//! `WideOutlinePass::gl_resources_free` freeing what it owns and leaving what it borrows.
//!
//! wasm32-only: every assertion asks a real `WebGl2RenderingContext` whether a
//! GL object still exists, which a native `cargo nextest` run cannot answer —
//! constructing the subject at all needs a live context. The headless-browser
//! runner in `.cargo/config.toml` is what executes these.
//!
//! Freeing a GPU resource is invisible from the Rust side: a handle wrapper is
//! a JS-object reference, so letting it go out of scope reclaims nothing. Only
//! asking the context — `gl.is_texture`, `gl.is_framebuffer` — distinguishes a
//! resource that was released from one that merely stopped being referenced,
//! which is why these tests hold handle clones across the drop.

use minwebgl as gl;
use gl::GL;
use gl::web_sys::{ WebGlFramebuffer, WebGlTexture };
use ::renderer::webgl::post_processing::outline::WideOutlinePass;

fn gl_init() -> GL
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default();
  let canvas = gl::canvas::make().unwrap();
  gl::context::from_canvas_with( &canvas, options ).unwrap()
}

fn texture_make( gl : &GL, width : i32, height : i32 ) -> WebGlTexture
{
  let texture = gl.create_texture().unwrap();
  gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) );
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, gl::RGBA8, width, height );
  texture
}

/// ## Root Cause
/// `WideOutlinePass::new` allocated 4 framebuffers and 4 intermediate textures ( JFA init,
/// two JFA step ping-pong buffers, and the final outline framebuffer/texture ) but had no
/// cleanup path at all -- neither a manual `gl_resources_free` nor an `impl Drop` -- so
/// every construct/drop cycle ( e.g. a canvas resize rebuilding the outline pipeline )
/// permanently leaked all 8 objects.
///
/// ## Why Not Caught
/// `tests/webgl/wide_outline.rs`'s existing coverage only asserts `render()` completes
/// without error -- it never constructs-then-drops a pass to check for leaked GL objects.
///
/// ## Fix Applied
/// Added `pub fn gl_resources_free`, deleting all 4 framebuffers and every texture in
/// `textures` EXCEPT `object_color` ( supplied by and still owned by the caller ), plus
/// `impl Drop` calling it automatically.
///
/// ## Prevention
/// This test captures clones of all 4 framebuffer handles and all 5 texture handles
/// ( the 4 owned intermediates plus `object_color` ) before calling `gl_resources_free`,
/// then asserts every owned handle is deleted while `object_color` remains a live GL object
/// -- the same deterministic existence-check pattern used by this crate's other
/// GPU-teardown reproducer tests, extended to also guard the caller-ownership boundary.
///
/// ## Pitfall
/// `object_color` living in the same `textures` map as the 4 owned intermediate textures
/// makes "delete everything in `textures`" the wrong rule -- a blanket-delete would free a
/// texture the caller still holds a handle to and may still use, a use-after-free from the
/// caller's perspective the moment its own copy of the handle is next bound.
// test_kind: bug_reproducer(BUG-436)
#[ wasm_bindgen_test::wasm_bindgen_test ]
fn wide_outline_pass_gl_resources_free_frees_owned_resources_but_not_object_color()
{
  let gl = gl_init();
  let width = 8;
  let height = 8;
  let object_color = texture_make( &gl, width, height );

  let mut pass = WideOutlinePass::new( &gl, object_color.clone(), 3.0, width as u32, height as u32 )
  .expect( "WideOutlinePass construction should succeed" );

  let framebuffers : Vec< WebGlFramebuffer > = pass.framebuffers_for_test();
  let owned_textures : Vec< WebGlTexture > = pass.textures_for_test().into_iter()
  .filter( | ( name, _ ) | name.as_str() != "object_color" )
  .map( | ( _, texture ) | texture )
  .collect();
  assert_eq!( framebuffers.len(), 4, "WideOutlinePass must own exactly 4 framebuffers" );
  assert_eq!( owned_textures.len(), 4, "WideOutlinePass must own exactly 4 intermediate textures" );

  for framebuffer in &framebuffers
  {
    assert!( gl.is_framebuffer( Some( framebuffer ) ) );
  }
  for texture in &owned_textures
  {
    assert!( gl.is_texture( Some( texture ) ) );
  }
  assert!( gl.is_texture( Some( &object_color ) ) );

  pass.gl_resources_free( &gl );

  for framebuffer in &framebuffers
  {
    assert!( !gl.is_framebuffer( Some( framebuffer ) ), "gl_resources_free must delete every owned framebuffer" );
  }
  for texture in &owned_textures
  {
    assert!( !gl.is_texture( Some( texture ) ), "gl_resources_free must delete every owned intermediate texture" );
  }
  assert!( gl.is_texture( Some( &object_color ) ), "gl_resources_free must NOT delete the caller-owned object_color texture" );
}
