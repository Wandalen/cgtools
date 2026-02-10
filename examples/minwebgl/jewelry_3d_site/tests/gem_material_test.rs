#![ allow( missing_docs ) ]
#![ allow( clippy::float_cmp ) ]

use jewelry_3d_site_lib::gem::GemMaterial;
use wasm_bindgen_test::*;
use minwebgl as gl;
use gl::{ GL, JsCast };
use mingl::web::canvas;
use renderer::webgl::Material;

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

#[ wasm_bindgen_test ]
fn test_gem_material_clone()
{
  let gl = create_test_gl_context();
  let mut material = GemMaterial::new( &gl );

  // Modify some properties
  material.ray_bounces = 10;
  material.color = gl::F32x3::from_array([ 1.0, 0.0, 0.0 ] );
  material.env_map_intensity = 2.0;
  material.radius = 500.0;
  material.needs_update = false;
  material.n2 = 3.0;
  material.rainbow_delta = 0.05;
  material.distance_attenuation_speed = 0.2;

  let cloned = material.clone();

  // Verify cloned properties match
  assert_eq!( cloned.ray_bounces, 10 );
  assert_eq!( cloned.color, gl::F32x3::from_array( [ 1.0, 0.0, 0.0 ] ) );
  assert_eq!( cloned.env_map_intensity, 2.0 );
  assert_eq!( cloned.radius, 500.0 );
  assert!( !cloned.needs_update );
  assert_eq!( cloned.n2, 3.0 );
  assert_eq!( cloned.rainbow_delta, 0.05 );
  assert_eq!( cloned.distance_attenuation_speed, 0.2 );

  // Verify IDs are different (clone should have new ID)
  assert_ne!( material.get_id(), cloned.get_id() );
}

#[ wasm_bindgen_test ]
fn test_gem_material_type_name()
{
  let gl = create_test_gl_context();
  let material = GemMaterial::new( &gl );

  assert_eq!( material.type_name(), "GemMaterial" );
}

#[ wasm_bindgen_test ]
fn test_gem_material_needs_update()
{
  let gl = create_test_gl_context();
  let mut material = GemMaterial::new( &gl );
  assert!( material.needs_update() );

  material.needs_update = false;
  assert!( !material.needs_update() );
}

#[ wasm_bindgen_test ]
fn test_gem_material_dyn_clone()
{
  let gl = create_test_gl_context();
  let material = GemMaterial::new( &gl );

  let boxed_original : Box< dyn Material > = Box::new( material );
  let boxed_clone = boxed_original.dyn_clone();

  assert_eq!( boxed_clone.type_name(), "GemMaterial" );
  assert_ne!( boxed_original.get_id(), boxed_clone.get_id() );
}
