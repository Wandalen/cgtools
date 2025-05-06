use std::collections::HashMap;
use minwebgl as gl;

use crate::buffer::Buffer;

pub struct Primitive
{
  pub id : uuid::Uuid,
  pub vs_defines : String,
  index_count : u32,
  vertex_count : u32,
  index_type : u32,
  index_offset : u32,
  draw_mode : u32,
  vao : gl::WebGlVertexArrayObject,
  material_id : Option< usize >
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
    let mut vs_defines = String::new();
    let id = uuid::Uuid::new_v4();
    let mut index_count = 0;
    let draw_mode = p.mode().as_gl_enum();
    let material_id = p.material().index();

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
      if acc.sparse().is_some()
      {
        gl::log::info!( "Sparce accessors are not supported yet" );
        continue;
      }
      
      let slot = match sem 
      {
        gltf::Semantic::Positions => { 
          vertex_count = acc.count() as u32;
          0 
        },
        gltf::Semantic::Normals => 1,
        gltf::Semantic::TexCoords( i ) => 
        {
          assert!( i < 5, "Only 5 types of texture coordinates are supported" );
          2 + i
        },
        gltf::Semantic::Colors( i ) => 
        {
          assert!( i < 2, "Only 2 types of color coordinates are supported" );
          7 + i
        },
        gltf::Semantic::Tangents => 
        {
          vs_defines.push_str( "#define USE_TANGENTS\n" );
          9
        },
        a => { gl::warn!( "Unsupported attribute: {:?}", a ); continue; }
      };

      let view = acc.view().expect( "Sparse accessors are not supported" );
      // The buffer must be the ARRAY_BUFFER, so binding like this should be ok
      let buffer = buffers.get( &view.index() ).expect( "Attribute buffer not found" );
      buffer.bind( gl );

      let size = acc.dimensions().multiplicity();
      assert!( size <= 4, "Vertex attribute has more than 4 elements" );

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

      //gl.vertex_attrib_divisor( slot, 1 );
      gl.enable_vertex_attrib_array( slot );
    }
    Ok
    (
      Self 
      { 
        id,
        vs_defines,
        index_count,
        draw_mode,
        vao,
        vertex_count,
        index_type,
        index_offset,
        material_id
      }
    )
  }

  pub fn get_vertex_defines( &self ) -> &str
  {
    &self.vs_defines    
  }

  pub fn get_material_id( &self ) -> Option< usize >
  {
    self.material_id
  }

  pub fn apply( &self, gl : &gl::WebGl2RenderingContext )
  {
    gl.bind_vertex_array( Some( &self.vao ) );
  }

  pub fn draw( &self, gl : &gl::WebGl2RenderingContext )
  {
    if self.index_count == 0
    {
      gl.draw_arrays( self.draw_mode, 0, self.vertex_count as i32 );
    }
    else 
    {
      gl.draw_elements_with_i32( self.draw_mode, self.index_count as i32, self.index_type, self.index_offset as i32 );
    }
  }

  pub fn set_material_id( &mut self, id : usize )
  {
    self.material_id = Some( id );
  }
}