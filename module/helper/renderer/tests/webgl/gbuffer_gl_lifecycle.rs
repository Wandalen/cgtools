//! `GBuffer`'s VAO, framebuffer, depth buffer and textures, all freed on drop.
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
use gl::web_sys::{ WebGlBuffer, WebGlTexture };
use rustc_hash::FxHashMap;
use ::renderer::webgl::post_processing::{ GBuffer, GBufferAttachment };

fn gl_init() -> GL
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default();
  let canvas = gl::canvas::make().unwrap();
  gl::context::from_canvas_with( &canvas, options ).unwrap()
}

/// ## Root Cause
/// `GBuffer::new` created a depth `WebGlRenderbuffer` ( local `depthbuffer` binding ) but
/// never stored it on the struct, so nothing could ever delete it -- and the struct had no
/// `impl Drop` at all, so the VAO, color framebuffer, and every attachment texture leaked
/// too on every construct/drop cycle ( e.g. a canvas resize rebuilding the geometry pass ).
///
/// ## Why Not Caught
/// `webgl/gbuffer.rs`'s existing test only covers `GBufferAttachment::define_const`/
/// `attribute_info` mapping -- no test previously constructed or dropped a real `GBuffer`.
///
/// ## Fix Applied
/// `depthbuffer` is now a stored field, and `impl Drop for GBuffer` deletes the VAO,
/// framebuffer, depthbuffer, and every texture in `textures`.
///
/// ## Prevention
/// This test captures clones of all four handle families from the private fields before
/// drop, then asserts each `gl.is_*` check flips from `true` to `false` afterward -- the
/// same deterministic existence-check pattern used by this crate's other GPU-teardown
/// reproducer tests ( `shadow.rs`, `unreal_bloom.rs`, `wide_outline.rs`, `skeleton.rs` ).
///
/// ## Pitfall
/// A local variable holding a GPU handle wrapper going out of scope without ever being
/// stored on the struct is doubly invisible -- neither a compiler warning nor a runtime
/// signal indicates the allocation was never reachable for cleanup in the first place.
// test_kind: bug_reproducer(BUG-433)
#[ wasm_bindgen_test::wasm_bindgen_test ]
fn gbuffer_drop_frees_vao_framebuffer_depthbuffer_and_textures()
{
  let gl = gl_init();

  let mut attachment_buffers : FxHashMap< GBufferAttachment, Vec< WebGlBuffer > > = FxHashMap::default();
  attachment_buffers.insert( GBufferAttachment::Albedo, vec![] );
  attachment_buffers.insert( GBufferAttachment::PbrInfo, vec![] );
  attachment_buffers.insert( GBufferAttachment::Uv1, vec![] );

  let gbuffer = GBuffer::new( &gl, 64, 64, attachment_buffers )
  .expect( "GBuffer::new should succeed on a valid context with a minimal attachment set" );

  let vao = gbuffer.vao_for_test();
  let framebuffer = gbuffer.framebuffer_for_test();
  let depthbuffer = gbuffer.depthbuffer_for_test();
  let textures : Vec< WebGlTexture > = gbuffer.textures_for_test();
  assert!( !textures.is_empty(), "minimal attachment set must still allocate at least one texture" );

  assert!( gl.is_vertex_array( Some( &vao ) ) );
  assert!( gl.is_framebuffer( Some( &framebuffer ) ) );
  assert!( gl.is_renderbuffer( Some( &depthbuffer ) ) );
  for texture in &textures
  {
    assert!( gl.is_texture( Some( texture ) ) );
  }

  drop( gbuffer );

  assert!( !gl.is_vertex_array( Some( &vao ) ), "GBuffer::drop must delete its VAO" );
  assert!( !gl.is_framebuffer( Some( &framebuffer ) ), "GBuffer::drop must delete its framebuffer" );
  assert!( !gl.is_renderbuffer( Some( &depthbuffer ) ), "GBuffer::drop must delete its depthbuffer" );
  for texture in &textures
  {
    assert!( !gl.is_texture( Some( texture ) ), "GBuffer::drop must delete every attachment texture" );
  }
}
