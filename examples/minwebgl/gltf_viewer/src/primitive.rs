use std::collections::HashMap;
use minwebgl as gl;

use crate::buffer::Buffer;

const VERTEX_SHADER : &'static str = include_str!( "../shaders/shader.vert" );

pub struct Primitive
{
  index_count : u32,
  vertex_count : u32,
  index_type : u32,
  index_offset : u32,
  draw_mode : u32,
  vao : gl::WebGlVertexArrayObject,
  vertex_shader : String,
  program : gl::WebGlProgram
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

    let mut index_type = 0;
    let mut index_offset = 0;
    // Set the index buffer
    if let Some( a ) = p.indices()
    {
      let v = a.view().expect( "Accessor is sparce" );
      let index_buffer = buffers.get( &v.index() ).expect( "Index buffer not found" );
      // Buffer should be the ELEMENT_ARRAY_BUFFERS, so binding like this should be ok
      index_buffer.bind( gl );
      index_count = a.count() as u32;
      index_type = a.data_type().as_gl_enum();
      index_offset = a.offset() as u32;
    }

    let mut vertex_count = 0;
    for ( sem, acc ) in p.attributes()
    { 
      let view = acc.view().expect( "Sparse accessors are not supported" );
      // The buffer must be the ARRAY_BUFFER, so binding like this should be ok
      let buffer = buffers.get( &view.index() ).expect( "Attribute buffer not found" );
      buffer.bind( gl );

      let slot = match sem 
      {
        gltf::Semantic::Positions => { 
          vertex_count = acc.count() as u32;
          0 
        },
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

    let vertex_shader = VERTEX_SHADER.to_string();

    let frag = include_str!( "../shaders/test/shader.frag" );
    let vert = include_str!( "../shaders/test/shader.vert" );

    let program = gl::ProgramFromSources::new( vert, frag ).compile_and_link( &gl )?;
    gl.use_program( Some( &program ) );

    Ok
    (
      Self 
      { 
        index_count,
        draw_mode,
        vao,
        vertex_shader,
        program,
        vertex_count,
        index_type,
        index_offset
      }
    )
  }

  pub fn render( &self, gl : &gl::WebGl2RenderingContext )
  {
    gl.use_program( Some( &self.program ) );
    gl.bind_vertex_array( Some( &self.vao ) );
    if self.index_count == 0
    {
      gl.draw_arrays( self.draw_mode, 0, self.vertex_count as i32 );
    }
    else 
    {
      gl.draw_elements_with_i32( self.draw_mode, self.index_count as i32, self.index_type, self.index_offset as i32 );
    }
  }
}