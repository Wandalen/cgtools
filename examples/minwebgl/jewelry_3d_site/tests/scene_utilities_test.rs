#![ allow( missing_docs ) ]
#![ allow( clippy::std_instead_of_alloc ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::implicit_return ) ]

use jewelry_3d_site_lib::scene_utilities::*;
use wasm_bindgen_test::*;
use minwebgl as gl;
use gl::{ GL, JsCast };
use mingl::web::canvas;
use renderer::webgl::{ Node, Scene };
use std::rc::Rc;
use std::cell::RefCell;

/// Helper function to create a test canvas
fn create_test_canvas() -> canvas::HtmlCanvasElement
{
  let window = gl::web_sys::window().expect( "should have a window" );
  let document = window.document().expect( "should have a document" );
  let canvas_element = document
  .create_element( "canvas" )
  .expect( "should create canvas" )
  .dyn_into::< gl::web_sys::HtmlCanvasElement >()
  .expect( "should be canvas" );

  canvas_element.set_width( 800 );
  canvas_element.set_height( 600 );

  return canvas_element
}

/// Helper function to create a GL context for testing
fn create_test_gl_context() -> GL
{
  let canvas = create_test_canvas();
  let options = gl::context::ContextOptions::default().antialias( false );
  return gl::context::from_canvas_with( &canvas, options ).expect( "should create GL context" )
}

/// Helper function to create a test scene with named nodes
fn create_test_scene() -> Rc< RefCell< Scene > >
{
  let scene = Rc::new( RefCell::new( Scene::new() ) );

  // Create nodes with different names
  let mut gem_node = Node::new();
  gem_node.set_name( "gem_1" );
  scene.borrow_mut().add(Rc::new( RefCell::new( gem_node ) ) );

  let mut diamond_node = Node::new();
  diamond_node.set_name( "diamond_main" );
  scene.borrow_mut().add(Rc::new( RefCell::new( diamond_node ) ) );

  let mut crystal_node = Node::new();
  crystal_node.set_name( "Crystal_Top" );
  scene.borrow_mut().add( Rc::new( RefCell::new( crystal_node ) ) );

  let mut metal_node = Node::new();
  metal_node.set_name( "metal_ring" );
  scene.borrow_mut().add( Rc::new( RefCell::new( metal_node ) ) );

  let unnamed_node = Node::new();
  scene.borrow_mut().add( Rc::new( RefCell::new( unnamed_node ) ) );

  scene
}

#[ wasm_bindgen_test ]
async fn test_create_empty_texture()
{
  let gl = create_test_gl_context();
  let result = create_empty_texture( &gl ).await;

  assert!( result.is_some(), "create_empty_texture should return Some" );

  let texture_info = result.unwrap();
  assert_eq!( texture_info.uv_position, 0 );
  assert!( texture_info.texture.borrow().source.is_some() );
}

#[ wasm_bindgen_test ]
fn test_filter_nodes_case_sensitive()
{
  let scene = create_test_scene();

  // Test case-sensitive search for "gem"
  let filtered = filter_nodes( &scene, "gem".to_string(), true );
  assert_eq!( filtered.len(), 1, "Should find exactly 1 gem node (case-sensitive)" );
  assert!( filtered.contains_key( "gem_1" ) );

  // Test case-sensitive search for "diamond"
  let filtered = filter_nodes( &scene, "diamond".to_string(), true );
  assert_eq!( filtered.len(), 1, "Should find exactly 1 diamond node" );
  assert!( filtered.contains_key( "diamond_main" ) );
}

#[ wasm_bindgen_test ]
fn test_filter_nodes_case_insensitive()
{
  let scene = create_test_scene();

  // Test case-insensitive search for "crystal"
  let filtered = filter_nodes( &scene, "crystal".to_string(), false );
  assert_eq!( filtered.len(), 1, "Should find crystal node (case-insensitive)" );
  assert!( filtered.contains_key( "Crystal_Top" ) );

  // Test case-insensitive search for "DIAMOND"
  let filtered = filter_nodes( &scene, "DIAMOND".to_string(), false );
  assert_eq!( filtered.len(), 1, "Should find diamond with uppercase search" );
  assert!( filtered.contains_key( "diamond_main" ) );
}

#[ wasm_bindgen_test ]
fn test_filter_nodes_no_matches()
{
  let scene = create_test_scene();

  // Test search with no matches
  let filtered = filter_nodes( &scene, "nonexistent".to_string(), false );
  assert_eq!( filtered.len(), 0, "Should find no nodes for non-existent substring" );
}

#[ wasm_bindgen_test ]
fn test_filter_nodes_partial_match()
{
  let scene = create_test_scene();

  // Test partial match
  let filtered = filter_nodes( &scene, "al".to_string(), false );
  assert!( filtered.len() >= 2, "Should find nodes with 'al' substring (metal, Crystal)" );
  assert!( filtered.contains_key( "metal_ring" ) );
  assert!( filtered.contains_key( "Crystal_Top" ) );
}

#[ wasm_bindgen_test ]
fn test_add_resize_callback()
{
  let result = add_resize_callback();
  assert!( result.is_some() );
}
