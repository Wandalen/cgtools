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

  node.set_local_matrix( exp );
  assert_abs_diff_eq!( node.get_local_matrix(), exp );

  let exp = math::mat4x4::identity();

  node.set_local_matrix( exp );
  assert_abs_diff_eq!( node.get_local_matrix(), exp );
}

#[ test ]
fn test_default_node()
{
  let node = Node::new();

  let exp = math::mat4x4::identity();
  let scale = math::F32x3::splat( 1.0 );
  let rotation = math::Quat::from( [ 0.0, 0.0, 0.0, 1.0 ] );
  let translation = math::F32x3::splat( 0.0 );

  assert_abs_diff_eq!( node.get_world_matrix(), exp );
  assert_abs_diff_eq!( node.get_local_matrix(), exp );
  assert_abs_diff_eq!( node.get_scale(), scale );
  assert_abs_diff_eq!( node.get_rotation(), rotation );
  assert_abs_diff_eq!( node.get_translation(), translation );
}

#[ test ]
fn test_scene_update_world_matrix_after_set_local_matrix1()
{
  let mut scene = Scene::new();
  let node_root = Rc::new( RefCell::new( Node::new() ) );
  let node1 = Rc::new( RefCell::new( Node::new() ) );
  let node2 = Rc::new( RefCell::new( Node::new() ) );
  let node11 = Rc::new( RefCell::new( Node::new() ) );

  node1.borrow_mut().add_child( node11.clone() );
  node_root.borrow_mut().add_child( node1.clone() );
  node_root.borrow_mut().add_child( node2.clone() );
  scene.add( node_root.clone() );

  let exp = math::mat3x3h::scale( [ 5.0; 3 ] );
  node_root.borrow_mut().set_local_matrix( exp );

  scene.update_world_matrix();

  assert_abs_diff_eq!( node_root.borrow().get_world_matrix(), exp );
  assert_abs_diff_eq!( node1.borrow().get_world_matrix(), exp );
  assert_abs_diff_eq!( node2.borrow().get_world_matrix(), exp );
  assert_abs_diff_eq!( node11.borrow().get_world_matrix(), exp );
}

#[ test ]
fn test_scene_update_world_matrix_after_set_local_matrix2()
{
  let mut scene = Scene::new();
  let node_root = Rc::new( RefCell::new( Node::new() ) );
  let node1 = Rc::new( RefCell::new( Node::new() ) );
  let node2 = Rc::new( RefCell::new( Node::new() ) );
  let node11 = Rc::new( RefCell::new( Node::new() ) );

  node1.borrow_mut().add_child( node11.clone() );
  node_root.borrow_mut().add_child( node1.clone() );
  node_root.borrow_mut().add_child( node2.clone() );
  scene.add( node_root.clone() );

  let exp = math::mat3x3h::scale( [ 5.0; 3 ] );
  let exp_identity = math::mat4x4::identity();
  node1.borrow_mut().set_local_matrix( exp );

  scene.update_world_matrix();

  assert_abs_diff_eq!( node_root.borrow().get_world_matrix(), exp_identity );
  assert_abs_diff_eq!( node1.borrow().get_world_matrix(), exp );
  assert_abs_diff_eq!( node2.borrow().get_world_matrix(), exp_identity );
  assert_abs_diff_eq!( node11.borrow().get_world_matrix(), exp );
}

#[ test ]
fn test_scene_update_world_matrix_after_set_local_matrix3()
{
  let mut scene = Scene::new();
  let node_root = Rc::new( RefCell::new( Node::new() ) );
  let node1 = Rc::new( RefCell::new( Node::new() ) );
  let node2 = Rc::new( RefCell::new( Node::new() ) );
  let node11 = Rc::new( RefCell::new( Node::new() ) );

  node1.borrow_mut().add_child( node11.clone() );
  node_root.borrow_mut().add_child( node1.clone() );
  node_root.borrow_mut().add_child( node2.clone() );
  scene.add( node_root.clone() );

  let mat1 = math::mat3x3h::scale( [ 5.0; 3 ] );
  let mat2 = math::mat3x3h::translation( [ 1.0; 3 ] );

  let mat12 = mat1 * mat2;

  node11.borrow_mut().set_local_matrix( mat2 );
  node2.borrow_mut().set_local_matrix( mat2 );
  node_root.borrow_mut().set_local_matrix( mat1 );

  scene.update_world_matrix();

  assert_abs_diff_eq!( node_root.borrow().get_world_matrix(), mat1 );
  assert_abs_diff_eq!( node1.borrow().get_world_matrix(), mat1 );
  assert_abs_diff_eq!( node2.borrow().get_world_matrix(), mat12 );
  assert_abs_diff_eq!( node11.borrow().get_world_matrix(), mat12 );
}

#[ test ]
fn test_set_translation()
{
  let mut node = Node::new();
  let translation = [ 1.0, 5.0, 0.0 ];

  let exp = math::mat3x3h::translation( translation );

  node.set_translation( translation );
  node.update_local_matrix();

  assert_abs_diff_eq!( exp, node.get_local_matrix() );
}

#[ test ]
fn test_set_scale()
{
  let mut node = Node::new();
  let scale = [ 1.0, 5.0, 0.0 ];

  let exp = math::mat3x3h::scale( scale );

  node.set_scale( scale );
  node.update_local_matrix();

  assert_abs_diff_eq!( exp, node.get_local_matrix() );
}

#[ test ]
fn test_set_rotation()
{
  let mut node = Node::new();
  let rotation = math::QuatF32::from_angle_y( 90f32.to_radians() );

  let exp = math::F32x4x4::from_scale_rotation_translation( [ 1.0; 3 ], rotation, [ 0.0; 3 ] );

  node.set_rotation( rotation );
  node.update_local_matrix();

  assert_abs_diff_eq!( exp, node.get_local_matrix() );
}