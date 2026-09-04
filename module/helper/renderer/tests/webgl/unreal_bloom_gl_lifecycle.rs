//! `UnrealBloomPass` freeing every texture and program on drop, with no explicit free call.
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
use gl::web_sys::WebGlProgram;
use ::renderer::webgl::post_processing::UnrealBloomPass;

// The mip-chain length every target/program count below is checked against. It
// is `UnrealBloomPass`'s own constant, not a number restated here — a test that
// hard-coded `5` would keep passing if the pass silently started allocating a
// different number of levels, which is exactly the leak this file is about.
const MIPS : usize = UnrealBloomPass::MIPS_FOR_TEST;

fn gl_init() -> gl::GL
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default();
  let canvas = gl::canvas::make().unwrap();
  gl::context::from_canvas_with( &canvas, options ).unwrap()
}

/// ## Root Cause
/// `UnrealBloomPass` already had a manual `gl_resources_free` method ( deleting its 10 mip
/// textures and all blur/composite shader programs ) but no `impl Drop` backstop -- any
/// caller that dropped the pass without first remembering to call `gl_resources_free`
/// ( e.g. on an error path, or simply forgetting ) leaked every one of those GPU resources.
///
/// ## Why Not Caught
/// `unreal_bloom_tests.rs` exercises `render()` end-to-end but never drops a pass without
/// first calling `gl_resources_free` to check whether `Drop` alone was sufficient.
///
/// ## Fix Applied
/// Added a `gl : gl::GL` field ( populated in `new` ) and `impl Drop for UnrealBloomPass`,
/// calling the pre-existing `gl_resources_free` automatically.
///
/// ## Prevention
/// This test captures clones of all 10 mip-texture handles and both shader-program handles
/// from the private fields before drop, then asserts every one of the 12 GL objects is
/// deleted afterward -- the same deterministic existence-check pattern used by this crate's
/// other GPU-teardown reproducer tests, without calling `gl_resources_free` explicitly
/// first ( proving `Drop` alone, not just the manual method, does the job ).
///
/// ## Pitfall
/// A manual `gl_resources_free`-only cleanup method is opt-in -- it only helps callers who
/// remember to call it, and does nothing on a panic-unwind or an early `?`-return before
/// the call site is reached. A stored `gl` field plus `impl Drop` makes cleanup
/// unconditional regardless of how the value's last use ends.
// test_kind: bug_reproducer(BUG-438)
#[ wasm_bindgen_test::wasm_bindgen_test ]
fn unreal_bloom_pass_drop_frees_all_textures_and_programs_without_explicit_free_call()
{
  let gl = gl_init();
  let pass = UnrealBloomPass::new( &gl, 64, 64, gl::RGBA16F )
  .expect( "UnrealBloomPass construction should succeed on a valid context" );

  let horizontal = pass.horizontal_targets_for_test();
  let vertical = pass.vertical_targets_for_test();
  assert_eq!( horizontal.len(), MIPS, "all {MIPS} horizontal mip textures must have allocated" );
  assert_eq!( vertical.len(), MIPS, "all {MIPS} vertical mip textures must have allocated" );

  let blur_programs : Vec< WebGlProgram > = pass.blur_programs_for_test();
  let composite_program = pass.composite_program_for_test();
  assert_eq!( blur_programs.len(), MIPS, "one compiled blur program per mip level" );

  for texture in horizontal.iter().chain( vertical.iter() )
  {
    assert!( gl.is_texture( Some( texture ) ) );
  }
  for program in &blur_programs
  {
    assert!( gl.is_program( Some( program ) ) );
  }
  assert!( gl.is_program( Some( &composite_program ) ) );

  // No explicit `gl_resources_free()` call -- this is the exact regression BUG-438 covers:
  // `Drop` alone, with no caller-remembered cleanup call, must still free everything.
  drop( pass );

  // Test pitfall (not a production bug): `UnrealBloomPass::new`'s last bind call is
  // `composite_material.bind(gl)`, leaving it the currently-bound program. Per the
  // WebGL/OpenGL ES spec, deleting a currently-bound program only flags it for deletion --
  // see the identical comment on `renderer_gl_resources_free_deletes_composite_and_skybox_programs`.
  gl.use_program( None );

  for texture in horizontal.iter().chain( vertical.iter() )
  {
    assert!( !gl.is_texture( Some( texture ) ), "UnrealBloomPass::drop must delete every mip texture" );
  }
  for program in &blur_programs
  {
    assert!( !gl.is_program( Some( program ) ), "UnrealBloomPass::drop must delete every blur program" );
  }
  assert!( !gl.is_program( Some( &composite_program ) ), "UnrealBloomPass::drop must delete the composite program" );
}
