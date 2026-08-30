//! `ShadowBaker` framebuffer teardown, and the cull-face state `shadow_map_render` must restore.
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
use ::renderer::webgl::{ Scene, SpotLight };
use ::renderer::webgl::shadow::{ Light, ShadowBaker, ShadowMap };
use wasm_bindgen_test::wasm_bindgen_test;

fn gl_init() -> GL
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default();
  let canvas = gl::canvas::make().unwrap();
  gl::context::from_canvas_with( &canvas, options ).unwrap()
}

fn spot_light_make() -> SpotLight
{
  SpotLight
  {
    position : gl::F32x3::from_array( [ 0.0, 5.0, 0.0 ] ),
    direction : gl::F32x3::from_array( [ 0.0, -1.0, 0.0 ] ),
    color : gl::F32x3::from_array( [ 1.0, 1.0, 1.0 ] ),
    strength : 1.0,
    range : 10.0,
    inner_cone_angle : 0.1,
    outer_cone_angle : 0.5,
    use_light_map : false,
  }
}

/// ## Root Cause
/// `ShadowBaker` allocated a `WebGlFramebuffer` in `new` but had no `impl Drop` -- every
/// construct/drop cycle permanently leaked one framebuffer for the GL context's lifetime.
///
/// ## Why Not Caught
/// `ShadowBaker` had zero prior test coverage of any kind -- nothing exercised its
/// construction or destruction, so a missing `Drop` impl produced no observable failure.
///
/// ## Fix Applied
/// Added `impl Drop for ShadowBaker`, calling `gl.delete_framebuffer` on `self.framebuffer`
/// ( matching the sibling `ShadowMap`'s pre-existing `impl Drop`, right above it in this
/// file, which this new impl was modeled on ).
///
/// ## Prevention
/// This test captures a clone of the private `framebuffer` handle before drop ( `mod tests`
/// is a descendant of `mod private`, so the field is directly visible here ), then asserts
/// `gl.is_framebuffer` flips from `true` to `false` once the `ShadowBaker` is dropped --
/// the same deterministic existence-check pattern used by this crate's other GPU-teardown
/// tests ( see `gpu_resource_leak_test` siblings in `gbuffer.rs`, `unreal_bloom.rs`, etc. ).
///
/// ## Pitfall
/// A GPU handle wrapper ( `Option< WebGlFramebuffer >` ) is just a JS-object reference --
/// letting the Rust value go out of scope does not call `gl.delete*` for you; only an
/// explicit delete call ( here, via `impl Drop` ) reclaims the actual GPU-side allocation.
// test_kind: bug_reproducer(BUG-432)
#[ wasm_bindgen_test ]
fn shadow_baker_drop_frees_framebuffer()
{
  let gl = gl_init();
  let baker = ShadowBaker::new( &gl ).expect( "ShadowBaker::new should succeed on a valid context" );

  let framebuffer = baker.framebuffer_for_test();
  // Test pitfall (not a production bug): `ShadowBaker::new` calls `create_framebuffer()`
  // but only ever binds it later, inside `target_set` ( called from `soft_shadow_render`,
  // not from `new` ) -- an unbound name is correctly reported as "not a framebuffer" by
  // `isFramebuffer` per the WebGL/OpenGL ES spec until bound at least once. This one-time
  // bind/unbind reproduces that precondition without needing a real `target_set` call
  // ( which additionally attaches a texture and checks completeness -- more than this test
  // needs ).
  gl.bind_framebuffer( gl::FRAMEBUFFER, framebuffer.as_ref() );
  gl.bind_framebuffer( gl::FRAMEBUFFER, None );
  assert!( gl.is_framebuffer( framebuffer.as_ref() ), "framebuffer must be a live GL object right after construction" );

  drop( baker );

  assert!( !gl.is_framebuffer( framebuffer.as_ref() ), "ShadowBaker::drop must delete its framebuffer" );
}

/// ## Root Cause
/// `ShadowMap::bind()` sets `cull_face( FRONT )` as a peter-panning mitigation for the
/// depth-only shadow pass. `render()` previously restored the framebuffer binding at its
/// end but left `cull_face` at `FRONT`, so any subsequent draw call issued without going
/// through `Renderer::render()`'s own per-material face-property setup would silently
/// inherit front-face culling.
///
/// ## Why Not Caught
/// Existing `ShadowMap::render` tests ( `fbo_pass_cycle_test.rs` ) only assert `Result::is_ok`
/// -- they never inspect GL state left behind after the call returns.
///
/// ## Fix Applied
/// `render()` now calls `self.gl.cull_face( gl::BACK )` immediately before returning,
/// restoring the renderer-wide default face mode ( `CULL_FACE` enable state and the
/// viewport are deliberately left unrestored -- see the `Fix(BUG-439)` comment above for
/// why neither has a single correct default from this scope ).
///
/// ## Prevention
/// This test reads back `gl::CULL_FACE_MODE` via `gl.get_parameter` after a real
/// `ShadowMap::render` call on an empty scene, asserting it is `gl::BACK` -- the general,
/// state-agnostic invariant the fix restores, not a pinned per-scene expectation.
///
/// ## Pitfall
/// GL state ( as opposed to GL objects/resources ) has no "drop" mechanism at all -- a pass
/// that mutates global context state ( `cull_face`, blend mode, depth func, etc. ) must
/// explicitly restore whatever contract it promises callers, since nothing in the type
/// system enforces symmetric enable/restore the way `Drop` does for owned GPU objects.
// test_kind: bug_reproducer(BUG-439)
#[ wasm_bindgen_test ]
fn shadow_map_render_restores_cull_face_to_back()
{
  let gl = gl_init();
  let shadow_map = ShadowMap::new( &gl, 64 ).expect( "ShadowMap::new should succeed on a valid context" );
  let scene = Scene::new();
  let light = Light::from( spot_light_make() );

  let result = shadow_map.render( &scene, light );
  assert!( result.is_ok(), "ShadowMap::render should succeed on an empty scene -- got {:?}", result.err() );

  let mode = gl.get_parameter( gl::CULL_FACE_MODE ).expect( "CULL_FACE_MODE must be readable" );
  let mode = mode.as_f64().expect( "CULL_FACE_MODE must be numeric" ) as u32;
  assert_eq!( mode, gl::BACK, "ShadowMap::render must restore cull_face to BACK before returning" );
}
