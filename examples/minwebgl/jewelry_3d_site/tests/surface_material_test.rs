#![ allow( missing_docs ) ]

use jewelry_3d_site_lib::surface_material::SurfaceMaterial;
use wasm_bindgen_test::*;
use minwebgl as gl;
use gl::{ GL, F32x3, JsCast };
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
fn test_surface_material_clone()
{
  let gl = create_test_gl_context();
  let mut material = SurfaceMaterial::new( &gl );
  // Modify properties
  material.color = F32x3::from_array( [ 0.5, 0.5, 0.5 ] );
  material.needs_update = false;

  let cloned = material.clone();

  // Verify cloned properties match
  assert_eq!( cloned.color, F32x3::from_array( [ 0.5, 0.5, 0.5 ] ) );
  assert!( !cloned.needs_update );
  assert!( cloned.texture.is_none() );

  // Verify IDs are different
  assert_ne!( material.get_id(), cloned.get_id() );
}

#[ wasm_bindgen_test ]
fn test_surface_material_type_name()
{
  let gl = create_test_gl_context();
  let material = SurfaceMaterial::new( &gl );
  assert_eq!( material.type_name(), "SurfaceMaterial" );
}

#[ wasm_bindgen_test ]
fn test_surface_material_dyn_clone()
{
  let gl = create_test_gl_context();
  let material = SurfaceMaterial::new( &gl );

  let boxed_original: Box<dyn Material> = Box::new( material );
  let boxed_clone = boxed_original.dyn_clone();

  assert_eq!( boxed_clone.type_name(), "SurfaceMaterial" );
  assert_ne!( boxed_original.get_id(), boxed_clone.get_id() );
}
