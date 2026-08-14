use super::*;
use the_module::{ Node, Scene };
use std::rc::Rc;
use std::cell::RefCell;
use approx::assert_abs_diff_eq;
use mingl::geometry::BoundingBox;

/// Manually recomputes the same hierarchical combination `Scene::compute_bounding_box`
/// caches, for independent comparison in tests.
fn hierarchical_bounding_box( children : &[ Rc< RefCell< Node > > ] ) -> BoundingBox
{
  let mut bbox = BoundingBox::default();

  for child in children
  {
    bbox.combine_mut( &child.borrow().bounding_box_hierarchical() );
  }

  bbox
}

#[ test ]
fn test_set_local_matrix()
{
  let mut scene = Scene::new();

  let exp = math::mat3x3h::scale( [ 5.0; 3 ] );

  scene.local_matrix_set( exp );
  assert_abs_diff_eq!( scene.local_matrix_get(), exp );

  let exp = math::mat4x4::identity();

  scene.local_matrix_set( exp );
  assert_abs_diff_eq!( scene.local_matrix_get(), exp );
}

#[ test ]
fn test_default()
{
  let scene = Scene::new();

  let mat = math::mat4x4::identity();
  let scale = math::F32x3::splat( 1.0 );
  let rotation = math::Quat::from( [ 0.0, 0.0, 0.0, 1.0 ] );
  let translation = math::F32x3::splat( 0.0 );

  assert_abs_diff_eq!( scene.local_matrix_get(), mat );
  assert_abs_diff_eq!( scene.scale_get(), scale );
  assert_abs_diff_eq!( scene.rotation_get(), rotation );
  assert_abs_diff_eq!( scene.translation_get(), translation );
}

#[ test ]
fn test_scene_update_world_matrix_after_set_local_matrix1()
{
  let mut scene = Scene::new();
  let node_root = Rc::new( RefCell::new( Node::new() ) );
  let node1 = Rc::new( RefCell::new( Node::new() ) );
  let node2 = Rc::new( RefCell::new( Node::new() ) );
  let node11 = Rc::new( RefCell::new( Node::new() ) );

  node1.borrow_mut().child_add( node11.clone() );
  node_root.borrow_mut().child_add( node1.clone() );
  node_root.borrow_mut().child_add( node2.clone() );
  scene.add( node_root.clone() );

  let exp = math::mat3x3h::scale( [ 5.0; 3 ] );
  scene.local_matrix_set( exp );
  scene.world_matrix_update();

  assert_abs_diff_eq!( node_root.borrow().world_matrix_get(), exp );
  assert_abs_diff_eq!( node1.borrow().world_matrix_get(), exp );
  assert_abs_diff_eq!( node2.borrow().world_matrix_get(), exp );
  assert_abs_diff_eq!( node11.borrow().world_matrix_get(), exp );
}

#[ test ]
fn test_scene_update_world_matrix_after_set_local_matrix2()
{
  let mut scene = Scene::new();
  let node_root = Rc::new( RefCell::new( Node::new() ) );
  let node1 = Rc::new( RefCell::new( Node::new() ) );
  let node2 = Rc::new( RefCell::new( Node::new() ) );
  let node11 = Rc::new( RefCell::new( Node::new() ) );

  node1.borrow_mut().child_add( node11.clone() );
  node_root.borrow_mut().child_add( node1.clone() );
  node_root.borrow_mut().child_add( node2.clone() );
  scene.add( node_root.clone() );

  let mat1 = math::mat3x3h::scale( [ 5.0; 3 ] );
  let mat2 = math::mat3x3h::translation( [ 1.0, 1.0, 10. ] );
  let mat_exp = mat2 * mat1;

  node1.borrow_mut().local_matrix_set( mat1 );
  scene.local_matrix_set( mat2 );
  scene.world_matrix_update();

  assert_abs_diff_eq!( node_root.borrow().world_matrix_get(), mat2 );
  assert_abs_diff_eq!( node1.borrow().world_matrix_get(), mat_exp );
  assert_abs_diff_eq!( node2.borrow().world_matrix_get(), mat2 );
  assert_abs_diff_eq!( node11.borrow().world_matrix_get(), mat_exp );
}

#[ test ]
fn test_set_translation()
{
  let mut scene = Scene::new();
  let translation = [ 1.0, 5.0, 0.0 ];

  let exp = math::mat3x3h::translation( translation );

  scene.translation_set( translation );
  scene.local_matrix_update();

  assert_abs_diff_eq!( exp, scene.local_matrix_get() );
}

#[ test ]
fn test_set_scale()
{
  let mut scene = Scene::new();
  let scale = [ 1.0, 5.0, 0.0 ];

  let exp = math::mat3x3h::scale( scale );

  scene.scale_set( scale );
  scene.local_matrix_update();

  assert_abs_diff_eq!( exp, scene.local_matrix_get() );
}

#[ test ]
fn test_set_rotation()
{
  let mut scene = Scene::new();
  let rotation = math::QuatF32::from_angle_y( 90f32.to_radians() );

  let exp = math::F32x4x4::from_scale_rotation_translation( [ 1.0; 3 ], rotation, [ 0.0; 3 ] );

  scene.rotation_set( rotation );
  scene.local_matrix_update();

  assert_abs_diff_eq!( exp, scene.local_matrix_get() );
}

#[ test ]
fn test_bounding_box_cached_single_root()
{
  let mut scene = Scene::new();
  let node_root = Rc::new( RefCell::new( Node::new() ) );
  scene.add( node_root.clone() );

  scene.world_matrix_update();

  // Computed while no borrow is held: a live tree-walking `bounding_box()`
  // would need to `.borrow()` every node again on each call, so proving it
  // instead requires none of the tree's `RefCell`s to be free.
  let exp = hierarchical_bounding_box( std::slice::from_ref( &node_root ) );

  // Hold a live mutable borrow on the root node: a `bounding_box()` that
  // still walked the tree on every call would panic here (`RefCell` already
  // mutably borrowed). The cached implementation reads only `Scene`'s own
  // `bounding_box` field and never touches `node_root`'s `RefCell`, so this
  // proves the value below comes from the cache, not a fresh recompute.
  let guard = node_root.borrow_mut();
  let got = scene.bounding_box();
  drop( guard );

  assert_abs_diff_eq!( got.min, exp.min );
  assert_abs_diff_eq!( got.max, exp.max );
}

#[ test ]
fn test_bounding_box_cached_three_level_chain()
{
  let mut scene = Scene::new();
  let node_root = Rc::new( RefCell::new( Node::new() ) );
  let node_child = Rc::new( RefCell::new( Node::new() ) );
  let node_grandchild = Rc::new( RefCell::new( Node::new() ) );

  node_child.borrow_mut().child_add( node_grandchild );
  node_root.borrow_mut().child_add( node_child );
  scene.add( node_root.clone() );

  scene.world_matrix_update();

  let exp = hierarchical_bounding_box( &[ node_root ] );
  let got = scene.bounding_box();
  assert_abs_diff_eq!( got.min, exp.min );
  assert_abs_diff_eq!( got.max, exp.max );
}

#[ test ]
fn test_bounding_box_empty_scene_is_default()
{
  let mut scene = Scene::new();

  scene.world_matrix_update();

  let exp = BoundingBox::default();
  let got = scene.bounding_box();
  assert_abs_diff_eq!( got.min, exp.min );
  assert_abs_diff_eq!( got.max, exp.max );
}
