//! Verifies `CanvasRenderer`'s mesh-to-color resolution ( `canvas_renderer::renderer::mesh_colors_resolve` ) —
//! the TASK-016 fix extracted the correspondence logic into a function testable WITHOUT a
//! live WebGL context ( every `CanvasRenderer` method takes `&GL`, so nothing else here is
//! natively exercisable ). Relocated from inline `src/renderer.rs` per the
//! all-tests-in-tests/ convention; the helper is exported at the `renderer` module path
//! for exactly this purpose.

#![ cfg( feature = "enabled" ) ]

use canvas_renderer::renderer::mesh_colors_resolve;
use renderer::webgl::{ Mesh, Node, Object3D, Scene };
use minwebgl::F32x4;
use std::cell::RefCell;
use std::rc::Rc;

/// Builds a non-mesh node -- a transform-only group, matching how
/// `primitive_generation::primitives_data_to_gltf` creates "parent" nodes for
/// `PrimitiveData` entries that carry no attributes.
fn group_node() -> Rc< RefCell< Node > >
{
  Rc::new( RefCell::new( Node::new() ) )
}

/// Builds a mesh node with no primitives -- `mesh_colors_resolve` only inspects whether the
/// node is `Object3D::Mesh`, never `Mesh::primitives`.
fn mesh_node() -> Rc< RefCell< Node > >
{
  let node = Rc::new( RefCell::new( Node::new() ) );
  node.borrow_mut().object = Object3D::Mesh( Rc::new( RefCell::new( Mesh::new() ) ) );
  node
}

/// ## Root Cause
/// `CanvasRenderer::render` looked up each mesh's color using a counter that advanced once
/// per *traversed scene node* (mesh or not), while `colors` holds one entry per *mesh*, in
/// mesh-encounter order (per `render`'s own doc comment: "renders all mesh nodes with their
/// corresponding colors from the colors array"). Any non-mesh node visited before or between
/// mesh nodes -- a transform-only group being the common case in a real scene graph -- shifted
/// the counter, so every mesh after it silently read the wrong `colors` entry, or, once the
/// counter ran past the end of `colors`, fell back to the magenta default. No panic, no
/// error: just a wrong-colored mesh.
///
/// ## Why Not Caught
/// Every existing caller (the `animation_surface_rendering`, `lottie_surface_rendering`, and
/// `curve_surface_rendering` examples) happens to build scenes where every node is a mesh
/// node, so the traversal-position counter and the mesh-encounter counter were always
/// numerically identical and the desync never manifested. Nothing exercised a scene mixing
/// mesh and non-mesh nodes.
///
/// ## Fix Applied
/// Extracted the mesh-to-color resolution into `mesh_colors_resolve`, which indexes `colors`
/// by `resolved.len()` -- a count that only grows when a mesh is actually pushed -- instead
/// of a counter shared with every traversed node. `render` now calls this function once up
/// front and walks its result in lockstep with a mesh-only counter during the real
/// GL-drawing traversal.
///
/// ## Prevention
/// This test builds a scene with two top-level groups, each owning one mesh child, so a
/// non-mesh node sits between the first and second mesh in traversal order -- exactly the
/// shape that desyncs a traversal-position counter from a mesh-position counter. It fails
/// immediately if the counter regresses to counting every node again.
///
/// ## Pitfall
/// When a lookup index is shared between a filtered consumer (only meshes read it) and an
/// unfiltered traversal (every node advances it), the two stay accidentally in sync only
/// while the "skipped" case never actually occurs in test data. Count only what is actually
/// consumed, never everything visited.
#[ test ]
fn resolve_mesh_colors_stays_in_sync_across_non_mesh_siblings()
{
  // scene
  // |- group_1 (non-mesh)
  // |   `- mesh_1
  // `- group_2 (non-mesh)
  //     `- mesh_2
  let mut scene = Scene::new();

  let group_1 = group_node();
  group_1.borrow_mut().child_add( mesh_node() );

  let group_2 = group_node();
  group_2.borrow_mut().child_add( mesh_node() );

  scene.add( group_1 );
  scene.add( group_2 );

  let color_for_mesh_1 = F32x4::from_array( [ 1.0, 0.0, 0.0, 1.0 ] );
  let color_for_mesh_2 = F32x4::from_array( [ 0.0, 1.0, 0.0, 1.0 ] );
  let colors = [ color_for_mesh_1, color_for_mesh_2 ];

  let resolved = mesh_colors_resolve( &scene, &colors );

  assert_eq!( resolved.len(), 2, "expected exactly one resolved color per mesh" );
  assert_eq!
  (
    resolved[ 0 ], color_for_mesh_1,
    "first mesh encountered must get colors[0], not a color shifted by the preceding non-mesh group"
  );
  assert_eq!
  (
    resolved[ 1 ], color_for_mesh_2,
    "second mesh encountered must get colors[1], not fall back to the default color"
  );
}

/// Extracts `CanvasRenderer::render`'s body verbatim from the crate's own `src/renderer.rs`, via
/// brace counting from the `pub fn render` signature's opening `{` to its matching closing `}` --
/// robust to whitespace/formatting drift, unlike an offset- or next-`pub fn`-based cut. No live
/// `WebGl2RenderingContext` exists in this crate's test environment (see the reproducer below),
/// so structural inspection of the real source is the only way to regression-test this function
/// without adding disproportionate new browser-test infrastructure.
fn render_fn_body() -> &'static str
{
  const SRC : &str = include_str!( "../src/renderer.rs" );

  let sig_pos = SRC.find( "pub fn render" )
  .expect( "`pub fn render` not found in src/renderer.rs -- has `CanvasRenderer::render` been renamed or moved?" );
  let after_sig = &SRC[ sig_pos.. ];
  let open_brace = after_sig.find( '{' )
  .expect( "no opening brace found after the `pub fn render` signature" );

  let mut depth = 0_i32;
  let mut close = None;
  for ( i, ch ) in after_sig[ open_brace.. ].char_indices()
  {
    match ch
    {
      '{' => depth += 1,
      '}' =>
      {
        depth -= 1;
        if depth == 0
        {
          close = Some( open_brace + i + 1 );
          break;
        }
      },
      _ => {}
    }
  }
  let close = close.expect( "unbalanced braces while scanning `render`'s body -- extraction logic assumption broke" );

  &after_sig[ ..close ]
}

// test_kind: bug_reproducer(BUG-342)
/// ## Root Cause
/// `render` binds `self.framebuffer` (`gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &self.framebuffer ) )`)
/// but never rebinds the default (`None`) framebuffer before returning -- unlike its siblings
/// `framebuffer_create` and `texture_set`, which both explicitly restore `None` at the end.
/// WebGL's `bindFramebuffer` state persists on the context until explicitly changed, so any GL
/// call issued after `render()` returns, by code that doesn't itself rebind first, silently
/// targets the internal offscreen texture instead of the intended target.
/// ## Why Not Caught
/// All 3 real call sites (`animation_surface_rendering`, `curve_surface_rendering`,
/// `lottie_surface_rendering`) immediately follow `canvas_renderer.render(...)` with a
/// *different* renderer's `.render(...)` (`renderer::webgl::Renderer::render`), which explicitly
/// rebinds its own target first -- masking the leak by luck, not by any restore `render` itself
/// performs. No live `WebGl2RenderingContext` test infrastructure exists in this crate (no
/// `wasm-bindgen-test` dev-dependency; same limitation BUG-227 already documented for this exact
/// crate), so no behavioral test could have caught this either.
/// ## Fix Applied
/// Added `gl.bind_framebuffer( GL::FRAMEBUFFER, None );` at the end of `render`, mirroring
/// `framebuffer_create`/`texture_set`'s existing convention. See `src/renderer.rs`.
/// ## Prevention
/// No live-context behavioral test is feasible here (see Why Not Caught and BUG-227's own
/// Prevention section for this crate's precedent). This is a structural/source-inspection
/// regression test instead: it extracts `render`'s body verbatim from the real `src/renderer.rs`
/// at test-run time (so it always exercises the current implementation, never a copy that could
/// drift stale) and asserts a `bind_framebuffer( ..., None )` restore call appears after the
/// `Some( &self.framebuffer )` bind -- it fails if that restore is ever removed again.
/// ## Pitfall
/// A passing structural check here proves only that the *text* of the restore call is present in
/// `render`'s body, not that it runs on every code path (e.g. it would not catch the restore
/// being added before an early `return`/`?` that skips it). Read the diff, not just this test's
/// PASS, when touching `render` again.
#[ test ]
fn render_restores_default_framebuffer_binding_before_returning()
{
  let body = render_fn_body();

  let self_bind_pos = body.find( "Some( &self.framebuffer )" )
  .expect( "test setup: expected render() to bind self.framebuffer at some point" );

  let after_self_bind = &body[ self_bind_pos + "Some( &self.framebuffer )".len().. ];
  let next_bind_call = after_self_bind.find( "bind_framebuffer" )
  .map( | offset | &after_self_bind[ offset..( offset + 60 ).min( after_self_bind.len() ) ] );

  assert!
  (
    matches!( next_bind_call, Some( call ) if call.contains( "None )" ) ),
    "render() must call `bind_framebuffer( GL::FRAMEBUFFER, None )` after binding self.framebuffer and before returning (matching framebuffer_create/texture_set's own restore convention) -- next bind_framebuffer call found after the self.framebuffer bind was: {next_bind_call:?}"
  );
}

// test_kind: bug_reproducer(BUG-493)
/// ## Root Cause
/// `render` unconditionally sets 4 pieces of global GL state -- `DEPTH_TEST`/`BLEND` enable
/// flags (`gl.enable`/`gl.disable`), `depth_mask( true )`, and `front_face( gl::CCW )` -- but,
/// unlike the framebuffer binding restored just above (BUG-342), never restored any of them
/// before returning. WebGL enable-flag/mask/winding state persists on the context until
/// explicitly changed, so a caller that had `BLEND` enabled for its own transparent pass, or
/// `CW` winding for its own meshes, silently had that state overwritten by `render()` and left
/// overwritten after it returned, with no error or indication anywhere.
/// ## Why Not Caught
/// Same structural gap as BUG-342: no live `WebGl2RenderingContext` test infrastructure exists
/// in this crate (no `wasm-bindgen-test` dev-dependency), and all 3 real call sites happen to
/// only ever need `DEPTH_TEST` enabled / `BLEND` disabled / CCW winding for their own
/// subsequent draws, so the leaked state never visibly broke anything downstream -- masking by
/// luck, not by any restore `render` itself performs.
/// ## Fix Applied
/// `render` now snapshots each of the 4 state bits (`gl.is_enabled( gl::DEPTH_TEST )`,
/// `gl.is_enabled( gl::BLEND )`, `gl.get_parameter( gl::DEPTH_WRITEMASK )`,
/// `gl.get_parameter( gl::FRONT_FACE )`) before overwriting them, and restores all 4 in the
/// same restore block as the existing BUG-342 framebuffer restore, right before returning. See
/// `src/renderer.rs`.
/// ## Prevention
/// Structural/source-inspection regression test, same technique as BUG-342's own test above (no
/// live-context behavioral test is feasible here -- see that test's own Why Not Caught): asserts
/// all 4 snapshot reads are present in `render`'s current body, and that all 4 corresponding
/// restore calls appear after the BUG-342 framebuffer restore point, in the same trailing
/// restore block.
/// ## Pitfall
/// `render()` already restored one piece of global state it mutates (the framebuffer binding,
/// per BUG-342) -- fixing that one restore did not guarantee the other 4 pieces of state this
/// same function mutates were also restored. Each piece of global GL state a function changes
/// has to be individually audited for its own snapshot/restore; a passing test for one piece of
/// leaked state is not evidence about any other piece.
#[ test ]
fn render_restores_depth_test_blend_depth_mask_and_front_face_before_returning()
{
  let body = render_fn_body();

  // Snapshot reads must exist -- these are what make a restore possible at all.
  for snapshot in
  [
    "is_enabled( gl::DEPTH_TEST )",
    "is_enabled( gl::BLEND )",
    "get_parameter( gl::DEPTH_WRITEMASK )",
    "get_parameter( gl::FRONT_FACE )",
  ]
  {
    assert!
    (
      body.contains( snapshot ),
      "render() must snapshot its prior GL state via `{snapshot}` before overwriting it -- \
      snapshot call not found in render()'s current body"
    );
  }

  // The framebuffer restore (BUG-342) anchors "near the end, before returning" -- the 4
  // state-bit restores added by this fix must appear after it, in the same restore block.
  let framebuffer_restore_pos = body.find( "bind_framebuffer( GL::FRAMEBUFFER, None )" )
  .expect( "test setup: expected render() to still restore the default framebuffer binding (BUG-342)" );
  let after_framebuffer_restore = &body[ framebuffer_restore_pos.. ];

  for restore in
  [
    "depth_test_was_enabled",
    "blend_was_enabled",
    "gl.depth_mask( depth_mask_was_enabled )",
    "gl.front_face( front_face_was )",
  ]
  {
    assert!
    (
      after_framebuffer_restore.contains( restore ),
      "render() must restore its snapshotted GL state (via `{restore}`) after the framebuffer \
      restore and before returning -- restore not found after the BUG-342 framebuffer restore \
      point in render()'s current body"
    );
  }
}
