//! Unit-level validation tests for `renderer::webgl::loaders::gltf::{ nodes_create,
//! skeletons_attach, scenes_create }` -- the node-hierarchy / skeleton-attach / scene-assembly
//! stages of the glTF loader. All three are pure data transforms over an already-parsed
//! `gltf::Gltf` document plus pre-built placeholder `Mesh` / `Node` values ( via their GL-free
//! `Default` impls ) -- no `gl` / `GL` / `WebGl` calls anywhere in their bodies. Originally
//! private, made `pub` alongside these tests per task 299.
#![ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]

use renderer::webgl::loaders::gltf::{ nodes_create, skeletons_attach, scenes_create };
use renderer::webgl::{ Mesh, Node, Object3D, Scene };
use std::{ cell::RefCell, rc::Rc };

/// A root with two children : one carrying a mesh reference and a translation, one plain.
const NODE_HIERARCHY_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "scene": 0,
  "scenes": [ { "nodes": [ 0 ] } ],
  "nodes":
  [
    { "name": "root", "children": [ 1, 2 ] },
    { "name": "mesh_child", "mesh": 0, "translation": [ 1.0, 2.0, 3.0 ] },
    { "name": "other_child" }
  ],
  "meshes": [ { "primitives": [] } ]
}
"#;

#[ test ]
fn nodes_create_wires_hierarchy_transform_and_mesh_assignment()
{
  let gltf = gltf::Gltf::from_slice( NODE_HIERARCHY_FIXTURE.as_bytes() ).unwrap();
  let mesh = Rc::new( RefCell::new( Mesh::default() ) );
  let meshes = [ mesh.clone() ];

  let result = nodes_create( &gltf, &meshes );

  assert_eq!( result.nodes.len(), 3, "one Node per glTF node" );
  assert!( result.lights.is_empty(), "fixture has no KHR_lights_punctual nodes" );

  let root = result.nodes[ 0 ].borrow();
  assert_eq!( root.children_get().len(), 2 );
  assert!( Rc::ptr_eq( &root.children_get()[ 0 ], &result.nodes[ 1 ] ), "root's first child must be the mesh_child node, in document order" );
  assert!( Rc::ptr_eq( &root.children_get()[ 1 ], &result.nodes[ 2 ] ), "root's second child must be the other_child node, in document order" );
  assert!( matches!( root.object, Object3D::Other ), "root has neither a mesh nor a light reference" );
  drop( root );

  let mesh_child = result.nodes[ 1 ].borrow();
  assert_eq!( mesh_child.translation_get(), [ 1.0, 2.0, 3.0 ].into() );
  if let Object3D::Mesh( m ) = &mesh_child.object
  {
    assert!( Rc::ptr_eq( m, &mesh ), "must reference the same mesh Rc passed in via `meshes`, not a clone" );
  }
  else
  {
    panic!( "expected mesh_child.object to be Object3D::Mesh" );
  }
  drop( mesh_child );

  assert!( matches!( result.nodes[ 2 ].borrow().object, Object3D::Other ), "other_child has no mesh reference" );
}

#[ test ]
fn skeletons_attach_leaves_an_unrigged_mesh_node_untouched()
{
  let mesh = Rc::new( RefCell::new( Mesh::default() ) );
  let mut node = Node::default();
  node.object = Object3D::Mesh( mesh.clone() );
  let node = Rc::new( RefCell::new( node ) );

  let nodes = [ node.clone() ];
  let rigged_nodes = vec![ ( node.clone(), None, None, None ) ];

  skeletons_attach( &nodes, rigged_nodes, &[] );

  assert!( mesh.borrow().skeleton.is_none(), "a rigged node with no skin and no morph targets must not get a skeleton attached" );
}

/// Two scenes : one referencing the sole node, one empty.
const TWO_SCENE_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "scene": 0,
  "scenes": [ { "nodes": [ 0 ] }, { "nodes": [] } ],
  "nodes": [ { "name": "solo" } ]
}
"#;

#[ test ]
fn scenes_create_builds_one_scene_per_document_scene_with_correct_membership()
{
  let gltf = gltf::Gltf::from_slice( TWO_SCENE_FIXTURE.as_bytes() ).unwrap();
  let node : Rc< RefCell< Node > > = Rc::new( RefCell::new( Node::default() ) );
  let nodes = [ node.clone() ];

  let scenes : Vec< Rc< RefCell< Scene > > > = scenes_create( &gltf, &nodes );

  assert_eq!( scenes.len(), 2, "one Scene per glTF scene" );
  assert_eq!( scenes[ 0 ].borrow().children.len(), 1 );
  assert!( Rc::ptr_eq( &scenes[ 0 ].borrow().children[ 0 ], &node ) );
  assert!( scenes[ 1 ].borrow().children.is_empty(), "the second scene declares no nodes" );
}
