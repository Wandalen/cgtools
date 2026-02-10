#![ allow( missing_docs ) ]
#![ allow( clippy::std_instead_of_alloc ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::float_cmp ) ]

use jewelry_3d_site_lib::cube_normal_map_generator::*;
use mingl::web::canvas;
use wasm_bindgen_test::*;
use minwebgl as gl;
use gl::{ GL, JsCast };
use renderer::webgl::Node;
use std::rc::Rc;
use std::cell::RefCell;

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

/// Helper function to create a GL context for testing
fn create_test_gl_context( canvas : &canvas::HtmlCanvasElement ) -> GL
{
  let options = gl::context::ContextOptions::default().antialias( false );
  return gl::context::from_canvas_with( canvas, options ).expect( "should create GL context" )
}


#[ wasm_bindgen_test ]
fn test_cube_normal_map_generator_creation()
{
  let canvas = create_test_canvas();
  let gl = create_test_gl_context( &canvas );
  let result = CubeNormalMapGenerator::new( &gl );
  assert!( result.is_ok(), "CubeNormalMapGenerator should be created successfully" );

  let generator = result.unwrap();
  assert_eq!( generator.texture_width, 512 );
  assert_eq!( generator.texture_height, 512 );
}

#[ wasm_bindgen_test ]
fn test_cube_normal_map_generator_set_texture_size()
{
  let canvas = create_test_canvas();
  let gl = create_test_gl_context( &canvas );
  let mut generator = CubeNormalMapGenerator::new( &gl ).expect( "should create generator" );

  generator.set_texture_size( &gl, 256, 256 );
  assert_eq!( generator.texture_width, 256 );
  assert_eq!( generator.texture_height, 256 );

  generator.set_texture_size( &gl, 1024, 512 );
  assert_eq!( generator.texture_width, 1024 );
  assert_eq!( generator.texture_height, 512 );
}


#[ wasm_bindgen_test ]
fn test_cube_normal_map_generator_with_empty_node()
{
  let canvas = create_test_canvas();
  let gl = create_test_gl_context( &canvas );
  let generator = CubeNormalMapGenerator::new( &gl ).expect( "should create generator" );

  // Create an empty node without mesh
  let empty_node = Rc::new( RefCell::new( Node::new() ) );

  let result = generator.generate( &gl, &empty_node );
  assert!( result.is_ok(), "Generate should handle empty nodes" );

  let cube_normal_data = result.unwrap();
  assert!( cube_normal_data.texture.is_none(), "Empty node should have no texture" );
  assert_eq!( cube_normal_data.max_distance, 0.0, "Empty node should have zero max distance" );
}
