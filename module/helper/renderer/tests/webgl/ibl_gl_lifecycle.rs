//! `IBL`'s three textures: freed on drop, and not double-freed when a clone outlives the original.
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
use ::renderer::webgl::IBL;

fn gl_init() -> gl::WebGl2RenderingContext
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default();
  let canvas = gl::canvas::make().unwrap();
  gl::context::from_canvas_with( &canvas, options ).unwrap()
}

// Test pitfall (not a production bug): `gl.create_texture()` alone allocates a texture
// *name*, but per the WebGL/OpenGL ES spec, `isTexture` only recognizes an object once it
// has been bound at least once via `bindTexture` -- an unbound name is correctly reported
// as "not a texture" even though `create_texture` succeeded. Every real allocation path in
// this crate binds before use ( `gl::TEXTURE_2D` is fine here regardless of the texture's
// real target -- only "has this name ever been bound" affects `isTexture` ), so this helper
// does the same one-time bind to make the constructed `IBL` observably real for the tests
// below, matching what `loaders::ibl`/`loaders::pmrem` always do before this code ever runs.
fn ibl_with_real_textures( gl : &gl::WebGl2RenderingContext ) -> IBL
{
  let diffuse_texture = gl.create_texture();
  let specular_1_texture = gl.create_texture();
  let specular_2_texture = gl.create_texture();
  for texture in [ &diffuse_texture, &specular_1_texture, &specular_2_texture ]
  {
    gl.bind_texture( gl::TEXTURE_2D, texture.as_ref() );
  }
  gl.bind_texture( gl::TEXTURE_2D, None );

  IBL::new_owning_for_test( diffuse_texture, specular_1_texture, specular_2_texture, 10, gl )
}

/// ## Root Cause
/// `IBL` allocated three cubemap/2D textures ( via its loaders ) but had no way to free
/// them -- `Renderer::ibl_set` replacing an already-set `self.ibl` ( e.g. an application
/// swapping environment maps at runtime ) silently leaked the previous `IBL`'s three
/// textures every time, and nothing freed them even when the owning `Renderer` was dropped.
///
/// ## Why Not Caught
/// `webgl/ibl.rs`'s existing test only covers `ibl_texture_parameters_apply`'s mip-range
/// targeting -- no test previously constructed-then-dropped an `IBL` to check for leaks.
///
/// ## Fix Applied
/// Added a `pub(crate) gl : Option< gl::WebGl2RenderingContext >` field ( populated by both
/// loaders ) and `impl Drop for IBL`, deleting all three textures when `gl` is populated.
///
/// ## Prevention
/// Constructs an `IBL` directly via struct literal with all three textures pre-populated
/// and `gl` set, then asserts all three handles are freed after drop.
///
/// ## Pitfall
/// `IBL` is a plain "loose bag of `pub` shared texture handles" with no allocation-time
/// hook of its own ( unlike `TransformsData`/`DisplacementsData`, which allocate inside
/// their own `upload()` method ) -- its textures are always populated by an external loader
/// function, which is easy to overlook when auditing "does this type manage its own
/// cleanup?", since the type itself never calls `gl.create_texture()`.
// test_kind: bug_reproducer(BUG-440)
#[ wasm_bindgen_test::wasm_bindgen_test ]
fn ibl_drop_frees_all_three_textures_when_gl_populated()
{
  let gl = gl_init();
  let ibl = ibl_with_real_textures( &gl );

  let diffuse = ibl.diffuse_texture.clone();
  let specular_1 = ibl.specular_1_texture.clone();
  let specular_2 = ibl.specular_2_texture.clone();
  assert!( gl.is_texture( diffuse.as_ref() ) );
  assert!( gl.is_texture( specular_1.as_ref() ) );
  assert!( gl.is_texture( specular_2.as_ref() ) );

  drop( ibl );

  assert!( !gl.is_texture( diffuse.as_ref() ), "IBL::drop must delete diffuse_texture" );
  assert!( !gl.is_texture( specular_1.as_ref() ), "IBL::drop must delete specular_1_texture" );
  assert!( !gl.is_texture( specular_2.as_ref() ), "IBL::drop must delete specular_2_texture" );
}

/// ## Root Cause
/// `IBL` previously derived `Clone`, which would have copied the texture handles by
/// reference ( aliasing the same GPU textures across instances ) with no
/// reallocation-on-clone mechanism -- adding `Drop` on top of that derive would let either
/// copy free textures the other still relies on, a double-free/dangling-handle risk.
///
/// ## Why Not Caught
/// N/A -- this is a safety margin added alongside the BUG-440 fix itself, not a
/// previously-observed failure ( no caller in this workspace clones an `IBL` today,
/// confirmed by grep across `module/` and `examples/` ).
///
/// ## Fix Applied
/// Replaced `#[ derive( Clone ) ]` with a manual `impl Clone` that copies the three texture
/// handles field-for-field but always resets the clone's `gl` to `None` -- only the
/// original loader-populated instance ever frees; every `Clone` is a permanently
/// non-owning view.
///
/// ## Prevention
/// This test clones a real, `gl`-populated `IBL`, asserts the clone's textures alias the
/// same GL objects ( same handle values, both still valid ), drops the CLONE first, and
/// asserts the original's textures remain valid ( the clone's `Drop` was a no-op since its
/// `gl` is `None` ) -- proving the clone cannot double-free the original's textures.
///
/// ## Pitfall
/// `#[ derive( Clone ) ]` on a struct holding shared GPU handles is only safe if the type
/// either deep-copies the underlying resource or guarantees at most one copy ever frees it
/// -- a derived field-for-field clone of a `gl`-populated `Drop` type silently creates two
/// owners of the same GPU object.
// test_kind: bug_reproducer(BUG-440)
#[ wasm_bindgen_test::wasm_bindgen_test ]
fn ibl_clone_does_not_double_free_original_textures()
{
  let gl = gl_init();
  let original = ibl_with_real_textures( &gl );
  let diffuse = original.diffuse_texture.clone();

  let clone = original.clone();
  assert!( !clone.owns_textures_for_test(), "a Clone of IBL must never take on ownership of the handles it copies" );
  assert!( gl.is_texture( clone.diffuse_texture.as_ref() ), "Clone must still alias a live GL texture object" );

  drop( clone );
  assert!( gl.is_texture( diffuse.as_ref() ), "dropping a non-owning Clone must not free the original's texture" );

  drop( original );
  assert!( !gl.is_texture( diffuse.as_ref() ), "dropping the original ( gl-populated ) IBL must still free its texture" );
}
