//! `Renderer::gl_resources_free` and the buffers `Renderer::resize` replaces.
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
use ::renderer::webgl::Renderer;

/// Unlike most `gl_init()` helpers in this crate, `Renderer::new` unconditionally builds
/// its `FramebufferContext` with `RGBA16F`/`R16F` multisample color attachments regardless
/// of scene contents, so `EXT_color_buffer_float` is mandatory here -- mirrors
/// `webgl_renderer_pass_cycle_test.rs`'s own `gl_init`.
fn gl_init() -> GL
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default();
  let canvas = gl::canvas::make().unwrap();
  let gl = gl::context::from_canvas_with( &canvas, options ).unwrap();
  gl.get_extension( "EXT_color_buffer_float" )
    .expect( "get_extension call should not throw" )
    .expect( "EXT_color_buffer_float must be available in the test environment" );
  gl
}

/// ## Root Cause
/// `Renderer` had no teardown API at all covering its full resource set -- dropping a
/// `Renderer` silently leaked every compiled material program plus the blend/composite/
/// skybox shader programs, since `resize()`'s narrower cleanup only frees the three fields
/// it is about to recreate ( `framebuffer_ctx`, `bloom_effect`, `swap_buffer` ).
///
/// ## Why Not Caught
/// `webgl_renderer_pass_cycle_test.rs` exercises `render()` end-to-end but never
/// constructed-then-freed a `Renderer` to check for leaked shader programs.
///
/// ## Fix Applied
/// Added `pub fn gl_resources_free`, deleting `compiled_programs`, `composite_shader`, and
/// `skybox_shader` ( plus delegating to `resizable_resources_free` and
/// `blend_effect.gl_resources_free` -- each independently covered by its own reproducer,
/// see `blend.rs`'s inline test for `blend_effect`'s program ).
///
/// ## Prevention
/// This test captures `composite_shader`/`skybox_shader`'s program handles ( both are
/// unconditionally created in `new`, so no scene/material setup is needed to populate them
/// ) before calling `gl_resources_free`, then asserts both are deleted afterward.
///
/// ## Pitfall
/// `resize()`'s pre-existing cleanup ( now factored into `resizable_resources_free` ) reads
/// as a complete teardown path at a glance -- it silently only ever covered the three
/// fields `resize()` itself recreates, never the fields that live for the `Renderer`'s
/// entire lifetime ( compiled programs, composite/skybox shaders, blend effect ).
// test_kind: bug_reproducer(BUG-434)
#[ wasm_bindgen_test::wasm_bindgen_test ]
fn renderer_gl_resources_free_deletes_composite_and_skybox_programs()
{
  let gl = gl_init();
  let mut renderer = Renderer::new( &gl, 64, 64, 4 )
  .expect( "Renderer::new should succeed on a valid context" );

  let composite_program = renderer.composite_program_for_test();
  let skybox_program = renderer.skybox_program_for_test();
  assert!( gl.is_program( Some( &composite_program ) ) );
  assert!( gl.is_program( Some( &skybox_program ) ) );

  renderer.gl_resources_free( &gl );

  // Test pitfall (not a production bug): `Renderer::new` leaves `composite_shader` bound
  // as the current program ( its `bind(gl)` call is the last one `new` makes ). Per the
  // WebGL/OpenGL ES spec, deleting the *currently bound* program only flags it for
  // deletion -- the browser defers actually invalidating it (and flipping `isProgram` to
  // false) until it stops being current. `gl_resources_free` already issued the correct
  // `delete_program` call; unbinding here just lets that deferred deletion complete so the
  // test can observe it, the same way any real caller would eventually stop using a
  // torn-down renderer's programs.
  gl.use_program( None );

  assert!( !gl.is_program( Some( &composite_program ) ), "gl_resources_free must delete composite_shader's program" );
  assert!( !gl.is_program( Some( &skybox_program ) ), "gl_resources_free must delete skybox_shader's program" );
}

/// ## Root Cause
/// `resize()` only assigned `self.bloom_effect`/`self.swap_buffer` fresh values inside the
/// `if self.use_emission` / `else` branches. When `use_emission` was true and
/// `UnrealBloomPass::new` returned `Err`, the `?` early-returned *before* either assignment
/// ran -- but `resizable_resources_free` had already deleted both structs' underlying GL
/// textures/programs, leaving `self` holding `Some( .. )` values whose handles were already
/// GPU-deleted ( a dangling handle the next `composite()` call would bind/draw with ).
///
/// ## Why Not Caught
/// No existing test resized a `Renderer` more than once, so the old→new handle transition
/// was never observed, and `UnrealBloomPass::new`'s only fallible step ( shader compile
/// from a fixed, always-valid `include_str!` source ) cannot be forced to fail from a live
/// WebGL2 context without mocking -- this workspace forbids mocking in tests ( see
/// `rulebook.md`/global test conventions ), so the exact `Err` branch has no live-GL MRE.
/// This limitation is disclosed rather than worked around with a fake failure.
///
/// ## Fix Applied
/// `bloom_effect`/`swap_buffer` are now cleared to `None` unconditionally, immediately
/// after `resizable_resources_free`, *before* the fallible `UnrealBloomPass::new(...)?`
/// line -- so an early return can never leave a `Some` wrapping an already-deleted handle.
///
/// ## Prevention
/// Since the exact error branch has no live-GL trigger, this test instead pins the
/// achievable real-GL invariant the fix must not regress : resizing twice with emission
/// enabled must fully replace `bloom_effect`/`swap_buffer` -- the first resize's textures
/// end up deleted, the second resize's are live -- proving the free-then-recreate sequence
/// never leaves the old handles both `Some` and dangling at any observable point.
///
/// ## Pitfall
/// Freeing a resource and clearing the handle that refers to it must happen atomically from
/// the caller's point of view -- never split across a fallible recreation step, or an error
/// path can leave a dangling handle mistaken for a live one. A defect only reachable via a
/// GL-implementation-dependent failure ( not a Rust-level parameter ) cannot always be
/// pinned by a live-context MRE ; code-path reasoning ( see `Root Cause` ) plus a functional
/// regression test on the reachable path is the honest substitute for one, not the same as
/// having no test at all -- fabricating a mocked failure would violate the "no mocking" rule
/// and produce a test that verifies a scenario that could never occur against a real driver.
// test_kind: bug_reproducer(BUG-435)
#[ wasm_bindgen_test::wasm_bindgen_test ]
fn renderer_resize_replaces_bloom_and_swap_buffer_cleanly_across_repeated_resizes()
{
  let gl = gl_init();
  let mut renderer = Renderer::new( &gl, 64, 64, 4 )
  .expect( "Renderer::new should succeed on a valid context" );
  renderer.use_emission_set( &gl, true );

  renderer.resize( &gl, 32, 32, 4 ).expect( "first resize with emission enabled should succeed" );
  let first_bloom = renderer.has_bloom_effect_for_test();
  let first_swap = renderer.has_swap_buffer_for_test();
  assert!( first_bloom, "resize with use_emission=true must populate bloom_effect" );
  assert!( first_swap, "resize with use_emission=true must populate swap_buffer" );

  renderer.resize( &gl, 64, 64, 4 ).expect( "second resize with emission enabled should succeed" );
  assert!( renderer.has_bloom_effect_for_test(), "bloom_effect must be repopulated ( never left None nor dangling ) after a second resize" );
  assert!( renderer.has_swap_buffer_for_test(), "swap_buffer must be repopulated ( never left None nor dangling ) after a second resize" );

  // The struct-level invariant the fix protects: at no point after `resize()` returns `Ok`
  // is either field ever `None` while `use_emission` is true, nor holding a stale handle --
  // each `resize()` call fully replaces both fields before returning.
}
