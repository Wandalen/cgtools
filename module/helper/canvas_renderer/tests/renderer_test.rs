//! Verifies `CanvasRenderer`'s mesh-to-color resolution ( `canvas_renderer::renderer::resolve_mesh_colors` ) —
//! the TASK-016 fix extracted the correspondence logic into a function testable WITHOUT a
//! live WebGL context ( every `CanvasRenderer` method takes `&GL`, so nothing else here is
//! natively exercisable ). Relocated from inline `src/renderer.rs` per the
//! all-tests-in-tests/ convention; the helper is exported at the `renderer` module path
//! for exactly this purpose.

#![ cfg( feature = "enabled" ) ]

use canvas_renderer::renderer::resolve_mesh_colors;
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

/// Builds a mesh node with no primitives -- `resolve_mesh_colors` only inspects whether the
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
/// Extracted the mesh-to-color resolution into `resolve_mesh_colors`, which indexes `colors`
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
  group_1.borrow_mut().add_child( mesh_node() );

  let group_2 = group_node();
  group_2.borrow_mut().add_child( mesh_node() );

  scene.add( group_1 );
  scene.add( group_2 );

  let color_for_mesh_1 = F32x4::from_array( [ 1.0, 0.0, 0.0, 1.0 ] );
  let color_for_mesh_2 = F32x4::from_array( [ 0.0, 1.0, 0.0, 1.0 ] );
  let colors = [ color_for_mesh_1, color_for_mesh_2 ];

  let resolved = resolve_mesh_colors( &scene, &colors );

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
