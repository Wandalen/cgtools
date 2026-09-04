//! `PickBuffer`'s GPU-resource teardown, against a live WebGL2 context.
//!
//! wasm32-only: every assertion here asks a real `WebGl2RenderingContext`
//! whether a GL object still exists, which a native `cargo nextest` run has no
//! way to answer. Reaches `PickBuffer`'s three private GL handles through the
//! `test_internals` feature — the handles are not surface, and the only reason
//! to hold one is exactly what this file does with it.

#![ cfg( all( test, target_arch = "wasm32", feature = "test_internals" ) ) ]

use minwebgl as gl;
use gl::GL;
use gpu_picking::PickBuffer;
use wasm_bindgen_test::wasm_bindgen_test;

// Browser, not Node: `web_sys::window()` is `None` under Node, and every test
// here needs a real WebGL2 context. Each file under `tests/` is its own
// binary, so this call belongs in each of them.
wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

fn gl_init() -> GL
{
  gl::browser::setup( gl::browser::Config::default() );
  let canvas = gl::canvas::make().unwrap();
  gl::context::from_canvas_with( &canvas, gl::context::ContextOptions::default() ).unwrap()
}

/// ## Root Cause
/// `PickBuffer` allocated a framebuffer, id texture, and depth renderbuffer in
/// `new`/`resize` but had no `impl Drop` -- every construct/drop cycle
/// permanently leaked all three for the GL context's lifetime (`resize`
/// deletes the *previous* texture/renderbuffer before replacing them, but
/// nothing ever freed the *last* one, nor the framebuffer itself, on final
/// teardown).
///
/// ## Why Not Caught
/// `gpu_picking` had zero test coverage of any kind before this sweep --
/// nothing exercised `PickBuffer`'s construction or destruction, so a missing
/// `Drop` impl produced no observable failure.
///
/// ## Fix Applied
/// Added a `gl : GL` field (an owned clone, populated in `new`, matching
/// `renderer::webgl::ShadowBaker`'s identical precedent in this workspace --
/// see BUG-432) plus `impl Drop for PickBuffer` deleting `framebuffer`,
/// `id_texture`, and `depth_renderbuffer`.
///
/// ## Prevention
/// This test captures clones of all three private GL handles right after
/// construction, asserts each is a live GL object, drops the `PickBuffer`,
/// then asserts none of the three are live any more -- the same deterministic
/// `gl.is_*` existence-check pattern used by this workspace's other
/// GPU-teardown reproducer tests (e.g. BUG-432's
/// `shadow_baker_drop_frees_framebuffer`).
///
/// ## Pitfall
/// A GPU handle wrapper (`Option< WebGlTexture >` etc.) is just a JS-object
/// reference -- letting the Rust value go out of scope does not call
/// `gl.delete*` for you; only an explicit delete call (here, via `impl Drop`)
/// reclaims the actual GPU-side allocation.
// test_kind: bug_reproducer(BUG-521)
#[ wasm_bindgen_test ]
fn pick_buffer_drop_frees_gl_resources()
{
  let gl = gl_init();
  let buffer = PickBuffer::new( &gl, 4, 4 );

  let framebuffer = buffer.framebuffer_for_test();
  let id_texture = buffer.id_texture_for_test();
  let depth_renderbuffer = buffer.depth_renderbuffer_for_test();

  // `new`'s own `resize` call already binds `framebuffer` while attaching
  // `id_texture`/`depth_renderbuffer`, so it already reports as a live GL
  // object here with no extra bind step needed (contrast BUG-432's
  // `ShadowBaker`, which only binds its framebuffer later, in `target_set`).
  assert!( gl.is_framebuffer( framebuffer.as_ref() ), "framebuffer must be a live GL object right after construction" );
  assert!( gl.is_texture( id_texture.as_ref() ), "id_texture must be a live GL object right after construction" );
  assert!( gl.is_renderbuffer( depth_renderbuffer.as_ref() ), "depth_renderbuffer must be a live GL object right after construction" );

  drop( buffer );

  assert!( !gl.is_framebuffer( framebuffer.as_ref() ), "PickBuffer::drop must delete its framebuffer" );
  assert!( !gl.is_texture( id_texture.as_ref() ), "PickBuffer::drop must delete its id_texture" );
  assert!( !gl.is_renderbuffer( depth_renderbuffer.as_ref() ), "PickBuffer::drop must delete its depth_renderbuffer" );
}
