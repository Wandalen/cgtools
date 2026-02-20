use super::*;
use the_module::{ Node, Scene };
use std::rc::Rc;
use std::cell::RefCell;
use approx::assert_abs_diff_eq;

#[ test ]
fn test_set_local_matrix()
{
  let mut scene = Scene::new();

  let exp = math::mat3x3h::scale( [ 5.0; 3 ] );

  scene.set_local_matrix( exp );
  assert_abs_diff_eq!( scene.get_local_matrix(), exp );

  let exp = math::mat4x4::identity();

  scene.set_local_matrix( exp );
  assert_abs_diff_eq!( scene.get_local_matrix(), exp );
}

#[ test ]
fn test_default()
{
  let scene = Scene::new();

  let mat = math::mat4x4::identity();
  let scale = math::F32x3::splat( 1.0 );
  let rotation = math::Quat::from( [ 0.0, 0.0, 0.0, 1.0 ] );
  let translation = math::F32x3::splat( 0.0 );

  assert_abs_diff_eq!( scene.get_local_matrix(), mat );
  assert_abs_diff_eq!( scene.get_scale(), scale );
  assert_abs_diff_eq!( scene.get_rotation(), rotation );
  assert_abs_diff_eq!( scene.get_translation(), translation );
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
  scene.set_local_matrix( exp );
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

  let mat1 = math::mat3x3h::scale( [ 5.0; 3 ] );
  let mat2 = math::mat3x3h::translation( [ 1.0, 1.0, 10. ] );
  let mat_exp = mat2 * mat1;

  node1.borrow_mut().set_local_matrix( mat1 );
  scene.set_local_matrix( mat2 );
  scene.update_world_matrix();

  assert_abs_diff_eq!( node_root.borrow().get_world_matrix(), mat2 );
  assert_abs_diff_eq!( node1.borrow().get_world_matrix(), mat_exp );
  assert_abs_diff_eq!( node2.borrow().get_world_matrix(), mat2 );
  assert_abs_diff_eq!( node11.borrow().get_world_matrix(), mat_exp );
}

#[ test ]
fn test_set_translation()
{
  let mut scene = Scene::new();
  let translation = [ 1.0, 5.0, 0.0 ];

  let exp = math::mat3x3h::translation( translation );

  scene.set_translation( translation );
  scene.update_local_matrix();

  assert_abs_diff_eq!( exp, scene.get_local_matrix() );
}

#[ test ]
fn test_set_scale()
{
  let mut scene = Scene::new();
  let scale = [ 1.0, 5.0, 0.0 ];

  let exp = math::mat3x3h::scale( scale );

  scene.set_scale( scale );
  scene.update_local_matrix();

  assert_abs_diff_eq!( exp, scene.get_local_matrix() );
}

#[ test ]
fn test_set_rotation()
{
  let mut scene = Scene::new();
  let rotation = math::QuatF32::from_angle_y( 90f32.to_radians() );

  let exp = math::F32x4x4::from_scale_rotation_translation( [ 1.0; 3 ], rotation, [ 0.0; 3 ] );

  scene.set_rotation( rotation );
  scene.update_local_matrix();

  assert_abs_diff_eq!( exp, scene.get_local_matrix() );
}
