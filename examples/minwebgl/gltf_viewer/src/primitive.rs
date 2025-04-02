use std::collections::HashMap;
use minwebgl as gl;

use crate::buffer::Buffer;


pub struct Primitive
{
  index_count : u32,
  draw_mode : u32,
  vao : gl::WebGlVertexArrayObject
}

impl Primitive
{
  pub fn new
  ( 
    gl : &gl::WebGl2RenderingContext, 
    p : &gltf::Primitive, 
    buffers : &HashMap< usize, Buffer >
  ) -> Result< Self, gl::WebglError >
  {
    let mut index_count = 0;
    let draw_mode = p.mode().as_gl_enum();

    let vao = gl::vao::create( gl )?;
    gl.bind_vertex_array( Some( &vao ) );

    // Set the index buffer
    if let Some( a ) = p.indices()
    {
      let v = a.view().expect( "Accessor is sparce" );
      let index_buffer = buffers.get( &v.index() ).expect( "Index buffer not found" );
      // Buffer should be the ELEMENT_ARRAY_BUFFERS, so binding like this should be ok
      index_buffer.bind( gl );
      index_count = a.count() as u32;
    }

    for ( sem, acc ) in p.attributes()
    { 
      let view = acc.view().expect( "Sparse accessors are not supported" );
      // The buffer must be the ARRAY_BUFFER, so binding like this should be ok
      let buffer = buffers.get( &view.index() ).expect( "Attribute buffer not found" );
      buffer.bind( gl );

      let slot = match sem 
      {
        gltf::Semantic::Positions => 0,
        gltf::Semantic::Normals => 1,
        gltf::Semantic::TexCoords( i ) => {
          if i > 1 { panic!( "Only 2 types of texture coordinates are supported") }
          2 + i
        },
        gltf::Semantic::Colors( i ) => {
          if i > 1 { panic!( "Only 2 types of color coordinates are supported") }
          4 + i
        },
        a => { gl::warn!( "Unsupported attribute: {:?}", a ); continue; }
      };

      let size = acc.dimensions().multiplicity();
      if size > 4 { panic!( "Attribute size bigger than 4 is not supported" ); }

      let type_ = acc.data_type().as_gl_enum();
      let stride = view.stride().unwrap_or( 0 );

      gl.vertex_attrib_pointer_with_i32
      ( 
        slot, 
        size as i32, 
        type_, 
        acc.normalized(), 
        stride as i32, 
        acc.offset() as i32 
      );
    }

    Ok
    (
      Self 
      { 
        index_count,
        draw_mode,
        vao
      }
    )
  }
}