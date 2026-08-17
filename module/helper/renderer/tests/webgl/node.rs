use super::*;
use the_module::{ Node, Scene };
use std::rc::Rc;
use std::cell::RefCell;
use approx::assert_abs_diff_eq;

#[ test ]
fn test_set_local_matrix()
{
  let mut node = Node::new();

  let exp = math::mat3x3h::scale( [ 5.0; 3 ] );

  node.local_matrix_set( exp );
  assert_abs_diff_eq!( node.local_matrix_get(), exp );

  let exp = math::mat4x4::identity();

  node.local_matrix_set( exp );
  assert_abs_diff_eq!( node.local_matrix_get(), exp );
}

#[ test ]
fn test_default_node()
{
  let node = Node::new();

  let exp = math::mat4x4::identity();
  let scale = math::F32x3::splat( 1.0 );
  let rotation = math::Quat::from( [ 0.0, 0.0, 0.0, 1.0 ] );
  let translation = math::F32x3::splat( 0.0 );

  assert_abs_diff_eq!( node.world_matrix_get(), exp );
  assert_abs_diff_eq!( node.local_matrix_get(), exp );
  assert_abs_diff_eq!( node.scale_get(), scale );
  assert_abs_diff_eq!( node.rotation_get(), rotation );
  assert_abs_diff_eq!( node.translation_get(), translation );
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
  node_root.borrow_mut().local_matrix_set( exp );

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

  let exp = math::mat3x3h::scale( [ 5.0; 3 ] );
  let exp_identity = math::mat4x4::identity();
  node1.borrow_mut().local_matrix_set( exp );

  scene.world_matrix_update();

  assert_abs_diff_eq!( node_root.borrow().world_matrix_get(), exp_identity );
  assert_abs_diff_eq!( node1.borrow().world_matrix_get(), exp );
  assert_abs_diff_eq!( node2.borrow().world_matrix_get(), exp_identity );
  assert_abs_diff_eq!( node11.borrow().world_matrix_get(), exp );
}

#[ test ]
fn test_scene_update_world_matrix_after_set_local_matrix3()
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
  let mat2 = math::mat3x3h::translation( [ 1.0; 3 ] );

  let mat1_mul_mat2 = mat1 * mat2;

  node11.borrow_mut().local_matrix_set( mat2 );
  node2.borrow_mut().local_matrix_set( mat2 );
  node_root.borrow_mut().local_matrix_set( mat1 );

  scene.world_matrix_update();

  assert_abs_diff_eq!( node_root.borrow().world_matrix_get(), mat1 );
  assert_abs_diff_eq!( node1.borrow().world_matrix_get(), mat1 );
  assert_abs_diff_eq!( node2.borrow().world_matrix_get(), mat1_mul_mat2 );
  assert_abs_diff_eq!( node11.borrow().world_matrix_get(), mat1_mul_mat2 );
}

#[ test ]
fn test_set_translation()
{
  let mut node = Node::new();
  let translation = [ 1.0, 5.0, 0.0 ];

  let exp = math::mat3x3h::translation( translation );

  node.translation_set( translation );
  node.local_matrix_update();

  assert_abs_diff_eq!( exp, node.local_matrix_get() );
}

#[ test ]
fn test_set_scale()
{
  let mut node = Node::new();
  let scale = [ 1.0, 5.0, 0.0 ];

  let exp = math::mat3x3h::scale( scale );

  node.scale_set( scale );
  node.local_matrix_update();

  assert_abs_diff_eq!( exp, node.local_matrix_get() );
}

#[ test ]
fn test_set_rotation()
{
  let mut node = Node::new();
  let rotation = math::QuatF32::from_angle_y( 90f32.to_radians() );

  let exp = math::F32x4x4::from_scale_rotation_translation( [ 1.0; 3 ], rotation, [ 0.0; 3 ] );

  node.rotation_set( rotation );
  node.local_matrix_update();

  assert_abs_diff_eq!( exp, node.local_matrix_get() );
}

#[ test ]
fn test_zero_scale_node_does_not_panic_on_singular_matrix_paths()
{
  // BUG-171: a node whose accumulated scale has a zero on one axis produces a singular
  // linear part. `world_matrix_set` (reached every frame via `world_matrix_update`) and
  // `local_bounding_box_hierarchical` both used to panic via `.inverse().unwrap()` on this
  // input; both now fall back to identity instead of unwrapping `None`.
  //
  // `local_matrix_set` is not used here: it round-trips the input through `decompose()`,
  // which returns `None` for a singular matrix, silently no-op-ing the whole call and never
  // reaching the buggy code path -- `scale_set` + `local_matrix_update` (as `test_set_scale`
  // above already does) builds the matrix directly instead.
  let mut scene = Scene::new();
  let node_root = Rc::new( RefCell::new( Node::new() ) );
  scene.add( node_root.clone() );

  let degenerate_scale = [ 1.0, 0.0, 1.0 ];
  let exp = math::mat3x3h::scale( degenerate_scale );
  node_root.borrow_mut().scale_set( degenerate_scale );
  node_root.borrow_mut().local_matrix_update();

  scene.world_matrix_update();
  assert_abs_diff_eq!( node_root.borrow().world_matrix_get(), exp );

  let _bbox = node_root.borrow().local_bounding_box_hierarchical();
}
