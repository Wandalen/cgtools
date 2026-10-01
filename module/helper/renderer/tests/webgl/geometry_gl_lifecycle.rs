//! `Geometry` VAO teardown and the buffers it must leave alone.
//!
//! wasm32-only: every assertion asks a real `WebGl2RenderingContext` whether a
//! GL object still exists (`gl.is_vertex_array`, `gl.is_buffer`), which only
//! the headless-browser runner in `.cargo/config.toml` can answer. Handle
//! clones are held across the drop for the same reason as in
//! `skeleton_gl_lifecycle.rs`: dropping a handle wrapper frees nothing by
//! itself, only the context can tell a released object from a forgotten one.

use std::{ cell::RefCell, rc::Rc };
use minwebgl as gl;
use gl::GL;
use mingl::geometry::BoundingBox;
use ::renderer::webgl::{ AttributeInfo, Geometry, IndexInfo, Material, Primitive, material::PbrMaterial };

fn gl_init() -> GL
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default();
  let canvas = gl::canvas::make().unwrap();
  gl::context::from_canvas_with( &canvas, options ).unwrap()
}

/// A buffer `gl.is_buffer` recognises: WebGL only reports a buffer name as a
/// buffer once it has been bound, which every real upload path does.
fn bound_buffer( gl : &GL, target : u32 ) -> gl::WebGlBuffer
{
  let buffer = gl.create_buffer().unwrap();
  gl.bind_buffer( target, Some( &buffer ) );
  gl.bind_buffer( target, None );
  buffer
}

fn position_info( buffer : &gl::WebGlBuffer ) -> AttributeInfo
{
  let attr = mingl::VertexAttribute::new( 0, mingl::VectorDataType::new( mingl::DataType::F32, 3, 1 ), 0 );
  AttributeInfo
  {
    slot : 0,
    buffer : buffer.clone(),
    descriptor : gl::BufferDescriptor::from_vector( attr.vector ).offset( attr.offset ).stride( 0 ),
    bounding_box : BoundingBox::default(),
  }
}

/// Two geometries built over the same vertex and index buffers, the way the
/// glTF loader builds every primitive that reads one bufferView. Dropping one
/// must delete its own VAO and nothing else: the survivor's VAO and both
/// shared buffers stay alive, so the survivor can still be re-uploaded and the
/// buffers can still be attached or written to.
#[ wasm_bindgen_test::wasm_bindgen_test ]
fn geometry_drop_deletes_own_vao_but_not_shared_buffers()
{
  let gl = gl_init();
  let vertices = bound_buffer( &gl, gl::ARRAY_BUFFER );
  let indices = bound_buffer( &gl, gl::ELEMENT_ARRAY_BUFFER );
  let index_info = IndexInfo { buffer : indices.clone(), count : 3, offset : 0, data_type : gl::UNSIGNED_SHORT };

  let mut dropped = Geometry::new( &gl ).unwrap();
  dropped.attribute_add( &gl, "positions", position_info( &vertices ) ).unwrap();
  dropped.index_add( &gl, index_info.clone() ).unwrap();
  let mut survivor = Geometry::new( &gl ).unwrap();
  survivor.attribute_add( &gl, "positions", position_info( &vertices ) ).unwrap();
  survivor.index_add( &gl, index_info ).unwrap();
  gl.bind_vertex_array( None );

  let dropped_vao = dropped.vao().clone();
  assert!( gl.is_vertex_array( Some( &dropped_vao ) ) );

  drop( dropped );

  assert!( !gl.is_vertex_array( Some( &dropped_vao ) ), "Geometry::drop must delete its own VAO" );
  assert!( gl.is_vertex_array( Some( survivor.vao() ) ), "the other geometry's VAO must survive" );
  assert!( gl.is_buffer( Some( &vertices ) ), "a shared attribute buffer must survive the drop" );
  assert!( gl.is_buffer( Some( &indices ) ), "a shared index buffer must survive the drop" );

  // `upload` only fails on a data-type conversion; binding a deleted buffer is a
  // silent GL error, so the context's error flag is what shows the buffers are usable.
  while gl.get_error() != gl::NO_ERROR {}
  survivor.upload( &gl ).expect( "the survivor's attribute descriptors are valid" );
  assert_eq!( gl.get_error(), gl::NO_ERROR, "the survivor must still re-upload against the shared buffers" );
}

/// `Primitive::clone` shares its geometry instead of copying it (`Geometry` is
/// not `Clone`): both primitives hold the same `Rc`, so the VAO outlives the
/// first primitive dropped and is deleted once, with the last.
#[ wasm_bindgen_test::wasm_bindgen_test ]
fn primitive_clone_shares_geometry()
{
  let gl = gl_init();
  let geometry = Rc::new( RefCell::new( Geometry::new( &gl ).unwrap() ) );
  // WebGL only reports a VAO name as a vertex array once it has been bound.
  geometry.borrow().bind( &gl );
  let vao = geometry.borrow().vao().clone();
  assert!( gl.is_vertex_array( Some( &vao ) ) );
  let material : Rc< RefCell< Box< dyn Material > > > = Rc::new( RefCell::new( Box::new( PbrMaterial::new( &gl ) ) ) );
  let original = Primitive { geometry, material };
  let clone = original.clone();
  assert!( Rc::ptr_eq( &original.geometry, &clone.geometry ), "a cloned primitive must share its geometry" );
  gl.bind_vertex_array( None );

  drop( original );
  assert!( gl.is_vertex_array( Some( &vao ) ), "the shared VAO must outlive the first primitive dropped" );

  drop( clone );
  assert!( !gl.is_vertex_array( Some( &vao ) ), "the last primitive sharing the geometry deletes its VAO" );
}
