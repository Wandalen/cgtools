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
use rustc_hash::FxHashMap;
use ::renderer::webgl::{ DisplacementsData, Skeleton, TransformsData };

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

/// ## Root Cause
/// `TransformsData`'s `Clone` copied the `global_texture` / `inverse_texture` handles together
/// with `gl`, so a clone dropped before its own first `upload()` ran the BUG-437 `Drop` and
/// deleted the original's live textures.
///
/// ## Why Not Caught
/// The BUG-437 reproducers above only drop an owning value; nothing cloned an uploaded
/// `TransformsData` and dropped the clone first.
///
/// ## Fix Applied
/// `Clone` resets both texture handles (and `gl`) to `None`: the clone holds none of the
/// original's textures, and its own first `upload()` allocates fresh ones.
///
/// ## Prevention
/// Clones an owning `TransformsData`, asserts the clone holds no texture handle, drops only
/// the clone, and asserts both of the original's textures are still live GL objects, then
/// drops the original and asserts they are freed exactly by it.
///
/// ## Pitfall
/// `Clone` must never copy a GPU handle that `Drop` deletes.
// test_kind: bug_reproducer(BUG-533)
#[ wasm_bindgen_test::wasm_bindgen_test ]
fn transforms_data_clone_drop_keeps_original_textures()
{
  let gl = gl_init();
  let global_texture = gl.create_texture();
  let inverse_texture = gl.create_texture();
  gl.bind_texture( gl::TEXTURE_2D, global_texture.as_ref() );
  gl.bind_texture( gl::TEXTURE_2D, inverse_texture.as_ref() );
  gl.bind_texture( gl::TEXTURE_2D, None );

  let original = TransformsData::new_owning_for_test( global_texture.clone(), inverse_texture.clone(), &gl );
  let clone = original.clone();
  assert_eq!( clone.textures_for_test(), ( None, None ), "a clone must not hold the original's textures" );
  drop( clone );

  assert!
  (
    gl.is_texture( global_texture.as_ref() ),
    "dropping a clone must not delete the original's global_texture",
  );
  assert!
  (
    gl.is_texture( inverse_texture.as_ref() ),
    "dropping a clone must not delete the original's inverse_texture",
  );

  drop( original );
  assert!( !gl.is_texture( global_texture.as_ref() ), "the original still frees global_texture" );
  assert!( !gl.is_texture( inverse_texture.as_ref() ), "the original still frees inverse_texture" );
}

/// ## Root Cause
/// Same as `transforms_data_clone_drop_keeps_original_textures`: `DisplacementsData`'s `Clone`
/// copied the texture handle together with `gl`, so a clone dropped before its first
/// `upload()` deleted the original's `displacements_texture`.
///
/// ## Why Not Caught
/// The BUG-437 reproducer never cloned an owning `DisplacementsData`.
///
/// ## Fix Applied
/// `Clone` resets `displacements_texture` (and `gl`) to `None`.
///
/// ## Prevention
/// Clones an owning `DisplacementsData`, asserts the clone holds no texture handle, drops the
/// clone, and asserts the original's texture survives until the original itself is dropped.
///
/// ## Pitfall
/// See `transforms_data_clone_drop_keeps_original_textures`.
// test_kind: bug_reproducer(BUG-533)
#[ wasm_bindgen_test::wasm_bindgen_test ]
fn displacements_data_clone_drop_keeps_original_texture()
{
  let gl = gl_init();
  let displacements_texture = gl.create_texture();
  gl.bind_texture( gl::TEXTURE_2D, displacements_texture.as_ref() );
  gl.bind_texture( gl::TEXTURE_2D, None );

  let original = DisplacementsData::new_owning_for_test( displacements_texture.clone(), &gl );
  let clone = original.clone();
  assert_eq!( clone.texture_for_test(), None, "a clone must not hold the original's texture" );
  drop( clone );

  assert!
  (
    gl.is_texture( displacements_texture.as_ref() ),
    "dropping a clone must not delete the original's displacements_texture",
  );

  drop( original );
  assert!
  (
    !gl.is_texture( displacements_texture.as_ref() ),
    "the original still frees displacements_texture",
  );
}

/// Every uniform name the skinning and morph-target uploads look up, with no
/// location: the uploads still bind and fill their textures, which is all
/// these tests observe.
fn skeleton_locations() -> FxHashMap< String, Option< gl::WebGlUniformLocation > >
{
  [
    "globalJointTransformMatricesTexture",
    "inverseBindMatricesTexture",
    "skinMatricesTextureSize",
    "morphTargetsDisplacementsTexture",
    "morphWeights",
  ]
  .into_iter()
  .map( | name | ( name.to_owned(), None ) )
  .collect()
}

/// Uploads `transforms` through a `Skeleton` and hands it back with the
/// textures that upload allocated.
fn transforms_upload( gl : &GL, transforms : TransformsData ) -> TransformsData
{
  let mut skeleton = Skeleton::new();
  *skeleton.transforms_as_mut() = Some( transforms );
  skeleton.upload( gl, &skeleton_locations() );
  skeleton.transforms_as_mut().take().unwrap()
}

/// Same as [`transforms_upload`] for morph-target displacements.
fn displacements_upload( gl : &GL, displacements : DisplacementsData ) -> DisplacementsData
{
  let mut skeleton = Skeleton::new();
  *skeleton.displacements_as_mut() = Some( displacements );
  skeleton.upload( gl, &skeleton_locations() );
  skeleton.displacements_as_mut().take().unwrap()
}

/// A `TransformsData` clone gets its own textures on its first `upload()`, in
/// either drop order: a clone dropped first leaves the original's textures, and
/// an original dropped first leaves an uploaded clone's. Neither upload raises
/// a GL error, so no upload goes through a deleted handle.
#[ wasm_bindgen_test::wasm_bindgen_test ]
fn transforms_data_clone_uploads_its_own_textures_in_either_drop_order()
{
  let gl = gl_init();
  let live = | t : &gl::web_sys::WebGlTexture | gl.is_texture( Some( t ) );
  let original = transforms_upload( &gl, TransformsData::new( vec![] ) );
  let ( og, oi ) = original.textures_for_test();
  let ( og, oi ) = ( og.unwrap(), oi.unwrap() );

  while gl.get_error() != gl::NO_ERROR {}
  let first = transforms_upload( &gl, original.clone() );
  let second = transforms_upload( &gl, original.clone() );
  assert_eq!( gl.get_error(), gl::NO_ERROR, "a clone's upload must not touch a deleted texture" );
  let ( fg, fi ) = first.textures_for_test();
  let ( fg, fi ) = ( fg.unwrap(), fi.unwrap() );
  assert!( fg != og && fi != oi, "an uploaded clone must hold textures of its own" );

  // Clone dropped first: the original keeps its textures.
  drop( first );
  assert!( !live( &fg ) && !live( &fi ), "a dropped clone frees its own textures" );
  assert!( live( &og ) && live( &oi ), "the original's textures survive the clone" );

  // Original dropped first: the other clone keeps its textures and still uploads.
  drop( original );
  assert!( !live( &og ) && !live( &oi ), "the original frees its own textures" );
  let ( sg, si ) = second.textures_for_test();
  let ( sg, si ) = ( sg.unwrap(), si.unwrap() );
  assert!( live( &sg ) && live( &si ), "a clone's textures survive the original" );
  let second = transforms_upload( &gl, second );
  assert_eq!( gl.get_error(), gl::NO_ERROR, "the surviving clone must still upload" );
  drop( second );
}

/// The `DisplacementsData` counterpart of
/// `transforms_data_clone_uploads_its_own_textures_in_either_drop_order`.
#[ wasm_bindgen_test::wasm_bindgen_test ]
fn displacements_data_clone_uploads_its_own_texture_in_either_drop_order()
{
  let gl = gl_init();
  let ot = gl.create_texture().unwrap();
  gl.bind_texture( gl::TEXTURE_2D, Some( &ot ) );
  gl.bind_texture( gl::TEXTURE_2D, None );
  let original = DisplacementsData::new_owning_for_test( Some( ot.clone() ), &gl );

  while gl.get_error() != gl::NO_ERROR {}
  let first = displacements_upload( &gl, original.clone() );
  let second = displacements_upload( &gl, original.clone() );
  assert_eq!( gl.get_error(), gl::NO_ERROR, "a clone's upload must not touch a deleted texture" );
  let ft = first.texture_for_test().unwrap();
  assert!( ft != ot, "an uploaded clone must hold a texture of its own" );

  drop( first );
  assert!( !gl.is_texture( Some( &ft ) ), "a dropped clone frees its own texture" );
  assert!( gl.is_texture( Some( &ot ) ), "the original's texture survives the clone" );

  drop( original );
  assert!( !gl.is_texture( Some( &ot ) ), "the original frees its own texture" );
  let st = second.texture_for_test().unwrap();
  assert!( gl.is_texture( Some( &st ) ), "a clone's texture survives the original" );
  let second = displacements_upload( &gl, second );
  assert_eq!( gl.get_error(), gl::NO_ERROR, "the surviving clone must still upload" );
  drop( second );
}
