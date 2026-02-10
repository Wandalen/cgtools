#![ allow( missing_docs ) ]
#![ allow( clippy::std_instead_of_alloc ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::float_cmp ) ]
#![ allow( clippy::bool_assert_comparison ) ]

use jewelry_3d_site_lib::
{
  configurator::*,
  scene_utilities::filter_nodes,
  gem::GemMaterial,
};
use wasm_bindgen_test::*;
use minwebgl as gl;
use gl::{ GL, JsCast };
use mingl::web::canvas;
use renderer::webgl::
{
  Node,
  Scene,
  Material,
  material::PbrMaterial
};
use web_sys::HtmlCanvasElement;
use std::rc::Rc;
use std::cell::RefCell;

/// Helper function to create a GL context for testing
fn create_test_gl_context( canvas: &HtmlCanvasElement ) -> GL
{
  let options = gl::context::ContextOptions::default().antialias( false );
  return gl::context::from_canvas_with( canvas, options ).expect( "should create GL context" )
}

/// Helper function to create a test canvas
fn create_test_canvas() -> canvas::HtmlCanvasElement
{
  let window = gl::web_sys::window().expect( "should have a window" );
  let document = window.document().expect( "should have a document" );
  let canvas = document
  .create_element( "canvas" )
  .expect( "should create canvas" )
  .dyn_into::< gl::web_sys::HtmlCanvasElement >()
  .expect( "should be canvas" );

  canvas.set_width( 800 );
  canvas.set_height( 600 );

  return canvas
}

/// Helper function to create a simple test scene
fn create_simple_test_scene() -> Rc< RefCell< Scene > >
{
  let scene = Rc::new( RefCell::new( Scene::new() ) );

  // Create a gem node
  let mut gem_node = Node::new();
  gem_node.set_name( "gem_1" );
  scene.borrow_mut().add( Rc::new( RefCell::new( gem_node ) ) );

  return scene
}

#[ test ]
fn test_remove_numbers()
{
  assert_eq!( remove_numbers( "gem123" ), "gem" );
  assert_eq!( remove_numbers( "diamond_456" ), "diamond_" );
  assert_eq!( remove_numbers( "crystal789stone" ), "crystalstone" );
  assert_eq!( remove_numbers( "no_numbers" ), "no_numbers" );
  assert_eq!( remove_numbers( "123456" ), "" );
  assert_eq!( remove_numbers( "" ), "" );
}

#[ test ]
fn test_ring_colors_default()
{
  let colors = RingColors::default();
  assert_eq!( colors.gem, "white" );
  assert_eq!( colors.metal, "silver" );
}

#[ test ]
fn test_ring_colors_clone()
{
  let mut colors = RingColors::default();
  colors.gem = "red".to_string();
  colors.metal = "gold".to_string();

  let cloned = colors.clone();
  assert_eq!( cloned.gem, "red" );
  assert_eq!( cloned.metal, "gold" );
}

#[ wasm_bindgen_test ]
fn test_animation_state_new()
{
  let state = AnimationState::new();
  assert_eq!( state.materials.len(), 0 );
  assert_eq!( state.material_callbacks.len(), 0 );
}

#[ wasm_bindgen_test ]
fn test_animation_state_update()
{
  let mut state = AnimationState::new();

  // Update with no animations should not panic
  state.update( 16.0 );
}

#[ wasm_bindgen_test ]
fn test_get_color_pbr_material()
{
  let canvas = create_test_canvas();
  let gl = create_test_gl_context( &canvas );
  let mut pbr_material = PbrMaterial::new( &gl );
  pbr_material.base_color_factor = gl::F32x3::from_array( [ 0.5, 0.6, 0.7 ] ).to_homogenous();

  let material: Rc< RefCell< Box< dyn Material > > > = Rc::new( RefCell::new( Box::new( pbr_material ) ) );
  let color = get_color( &material );

  assert_eq!( color.0, [ 0.5, 0.6, 0.7 ] );
}

#[ wasm_bindgen_test ]
fn test_get_color_gem_material()
{
  let canvas = create_test_canvas();
  let gl = create_test_gl_context( &canvas );
  let mut gem_material = GemMaterial::new( &gl );
  gem_material.color = gl::F32x3::from_array([ 0.8, 0.2, 0.3 ] );

  let material: Rc< RefCell< Box< dyn Material > > > = Rc::new( RefCell::new( Box::new( gem_material ) ) );
  let color = get_color( &material );

  assert_eq!( color.0, [ 0.8, 0.2, 0.3 ] );
}

#[ wasm_bindgen_test ]
fn test_setup_camera()
{
  let canvas = create_test_canvas();
  let camera = setup_camera( &canvas );

  // Verify controls are configured
  let controls = camera.get_controls();
  assert_eq!( controls.borrow().pan.enabled, false );
  assert_eq!( controls.borrow().rotation.movement_smoothing_enabled, true );
  assert_eq!( controls.borrow().rotation.speed, 50.0 );
}

#[ wasm_bindgen_test ]
fn test_create_shadow_texture()
{
  let canvas = create_test_canvas();
  let gl = create_test_gl_context( &canvas );
  let texture = create_shadow_texture( &gl, 512, 5 );

  assert!( texture.is_some(), "Shadow texture should be created" );
}

# [ wasm_bindgen_test ]
fn test_filter_nodes_with_gems()
{
  let scene = create_simple_test_scene();

  // Add more gem nodes
  let mut diamond_node = Node::new();
  diamond_node.set_name( "diamond_main" );
  scene.borrow_mut().add(Rc::new( RefCell::new( diamond_node ) ) );

  let mut crystal_node = Node::new();
  crystal_node.set_name( "crystal_top" );
  scene.borrow_mut().add( Rc::new( RefCell::new( crystal_node ) ) );

  // Filter for gems
  let gems = filter_nodes( &scene, "gem".to_string(), false );
  assert_eq!( gems.len(), 1 );
  assert!( gems.contains_key( "gem_1" ) );

  // Filter for diamonds
  let diamonds = filter_nodes( &scene, "diamond".to_string(), false );
  assert_eq!( diamonds.len(), 1 );
  assert!( diamonds.contains_key( "diamond_main" ) );

  // Filter for crystals
  let crystals = filter_nodes( &scene, "crystal".to_string(), false );
  assert_eq!( crystals.len(), 1 );
  assert!( crystals.contains_key( "crystal_top" ) );
}
