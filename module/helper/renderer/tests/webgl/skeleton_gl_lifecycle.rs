//! `TransformsData` / `DisplacementsData` texture teardown.
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
use ::renderer::webgl::{ DisplacementsData, TransformsData };

fn gl_init() -> GL
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default();
  let canvas = gl::canvas::make().unwrap();
  gl::context::from_canvas_with( &canvas, options ).unwrap()
}

/// ## Root Cause
/// `TransformsData` allocated `global_texture`/`inverse_texture` via `gl.create_texture()`
/// inside `upload()` but never freed them anywhere -- dropping a `TransformsData` ( e.g.
/// when its owning `Skeleton`/`Mesh`/`Node` is discarded ) silently leaked both textures.
///
/// ## Why Not Caught
/// `skeleton_tests.rs` and `gltf_skeleton_displacements_test.rs` exercise upload/animation
/// logic but never construct-then-drop a `TransformsData` to check for leaked GL objects.
///
/// ## Fix Applied
/// Added a `gl : Option< GL >` field ( populated on first `upload()` call ) and
/// `impl Drop for TransformsData`, deleting `global_texture`/`inverse_texture` when `gl` is
/// populated.
///
/// ## Prevention
/// Constructs a `TransformsData` directly via struct literal with `global_texture`/
/// `inverse_texture` pre-populated and `gl` set ( bypassing `upload()`'s real allocation
/// path, which is exercised separately by `skeleton_tests.rs` ), then asserts both handles
/// are freed after drop -- the same deterministic existence-check pattern used by this
/// crate's other GPU-teardown reproducer tests.
///
/// ## Pitfall
/// A struct whose GPU-resource-owning fields are only populated lazily ( on first `upload`,
/// not in `new` ) is easy to reason about as "doesn't own anything yet" and skip when
/// auditing for missing `Drop` impls -- the fields are still owned once populated, on
/// whichever call path first fills them in.
// test_kind: bug_reproducer(BUG-437)
#[ wasm_bindgen_test::wasm_bindgen_test ]
fn transforms_data_drop_frees_global_and_inverse_textures()
{
  let gl = gl_init();
  let global_texture = gl.create_texture();
  let inverse_texture = gl.create_texture();
  // Test pitfall (not a production bug): `create_texture()` alone allocates a name, but
  // `isTexture` only recognizes it once bound at least once via `bindTexture` -- every real
  // `upload()` call binds before use, so this one-time bind reproduces that precondition.
  gl.bind_texture( gl::TEXTURE_2D, global_texture.as_ref() );
  gl.bind_texture( gl::TEXTURE_2D, inverse_texture.as_ref() );
  gl.bind_texture( gl::TEXTURE_2D, None );
  assert!( gl.is_texture( global_texture.as_ref() ) );
  assert!( gl.is_texture( inverse_texture.as_ref() ) );

  let transforms_data = TransformsData::new_owning_for_test
  (
    global_texture.clone(),
    inverse_texture.clone(),
    &gl,
  );

  drop( transforms_data );

  assert!( !gl.is_texture( global_texture.as_ref() ), "TransformsData::drop must delete global_texture" );
  assert!( !gl.is_texture( inverse_texture.as_ref() ), "TransformsData::drop must delete inverse_texture" );
}

/// ## Root Cause
/// `DisplacementsData` allocated `displacements_texture` via `gl.create_texture()` inside
/// `upload()` but never freed it anywhere -- dropping a `DisplacementsData` silently leaked
/// the GPU texture every time.
///
/// ## Why Not Caught
/// Same gap as `TransformsData` above -- no test previously constructed-then-dropped a
/// `DisplacementsData` to check for a leaked GL object.
///
/// ## Fix Applied
/// Added a `gl : Option< GL >` field ( populated on first `upload()` call ) and
/// `impl Drop for DisplacementsData`, deleting `displacements_texture` when `gl` is
/// populated.
///
/// ## Prevention
/// Constructs a `DisplacementsData` directly via struct literal with `displacements_texture`
/// pre-populated and `gl` set, then asserts the handle is freed after drop.
///
/// ## Pitfall
/// Same as `TransformsData` above -- lazily-populated GPU fields are still owned once
/// populated, regardless of which call path first fills them in.
// test_kind: bug_reproducer(BUG-437)
#[ wasm_bindgen_test::wasm_bindgen_test ]
fn displacements_data_drop_frees_displacements_texture()
{
  let gl = gl_init();
  let displacements_texture = gl.create_texture();
  // Test pitfall (not a production bug): see the identical comment on
  // `transforms_data_drop_frees_global_and_inverse_textures` above.
  gl.bind_texture( gl::TEXTURE_2D, displacements_texture.as_ref() );
  gl.bind_texture( gl::TEXTURE_2D, None );
  assert!( gl.is_texture( displacements_texture.as_ref() ) );

  let displacements_data = DisplacementsData::new_owning_for_test( displacements_texture.clone(), &gl );

  drop( displacements_data );

  assert!( !gl.is_texture( displacements_texture.as_ref() ), "DisplacementsData::drop must delete displacements_texture" );
}
