mod private
{
  use std::collections::HashMap;

use minwebgl as gl;

  #[ derive( Default, Clone, Copy ) ]
  pub struct BoundingBox
  {
    pub min : gl::F32x3,
    pub max : gl::F32x3
  }

  impl BoundingBox
  {
    pub fn new< T : Into< gl::F32x3 > >( min : T, max : T ) -> Self
    {
      Self
      {
        min : min.into(),
        max : max.into()
      }
    }

    pub fn center( &self ) -> gl::F32x3
    {
      ( self.max + self.min ) / 2.0
    }
  }

  pub struct AttributeInfo
  {
    pub slot : u32,
    pub buffer : gl::WebGlBuffer,
    pub descriptor : gl::BufferDescriptor,
    pub bounding_box : BoundingBox
  }

  impl AttributeInfo 
  {
    pub fn upload( &self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
    {
      self.descriptor.attribute_pointer( gl, self.slot, &self.buffer )?;

      Ok( () )
    }    
  }

  pub struct IndexInfo
  {
    pub buffer : gl::WebGlBuffer,
    pub count : u32,
    pub offset : u32,
    pub data_type : u32
  }

  pub struct Geometry
  {
    pub defines : String,
    pub vao : gl::WebGlVertexArrayObject,
    pub draw_mode : u32,
    pub vertex_count : u32,
    index_info : Option< IndexInfo >,
    attributes : HashMap< String, AttributeInfo >
  }

  impl Geometry 
  {
    pub fn new( gl : &gl::WebGl2RenderingContext ) -> Result< Self, gl::WebglError >
    {
      let vao = gl::vao::create( gl )?;
      let attributes = HashMap::new();
      let draw_mode = gl::TRIANGLES;
      let vertex_count = 0;
      let index_info = None;
      let defines = String::new();

      Ok
      (
        Self
        {
          defines,
          vao,
          draw_mode,
          vertex_count,
          index_info,
          attributes
        }
      )
    }

    pub fn add_attribute< Name : Into< String > >
    ( 
      &mut self, 
      gl : &gl::WebGl2RenderingContext,
      name : Name, 
      info : AttributeInfo, 
      as_define : bool 
    ) -> Result< (), gl::WebglError >
    {
      let name = name.into();
      if !self.attributes.contains_key( name.as_str() )
      {
        if as_define
        {
          self.defines.push_str( &format!( "#define USE_{}\n", name.to_uppercase() ) );
        }
        self.bind( gl );
        info.upload( gl )?;
        self.attributes.insert( name, info );
      }
      else 
      {
        panic!( "An attribute {} already exists", name );
      }

      Ok( () ) 
    }

    pub fn add_index
    ( 
      &mut self, 
      gl : &gl::WebGl2RenderingContext,
      info : IndexInfo,
    ) -> Result< (), gl::WebglError >
    {
      self.bind( gl );
      gl.bind_buffer( gl::ELEMENT_ARRAY_BUFFER, Some( &info.buffer ) );
      self.index_info = Some( info );
      Ok( () ) 
    }

    pub fn upload( &self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
    {
      self.bind( gl );

      for info in self.attributes.values()
      {
        info.upload( gl )?;
      }

      if let Some( info ) = self.index_info.as_ref()
      {
        gl.bind_buffer( gl::ELEMENT_ARRAY_BUFFER, Some( &info.buffer ) );
      }

      Ok( () )
    }

    pub fn get_defines( &self ) -> &str
    {
      &self.defines
    }

    pub fn center( &self ) -> gl::F32x3
    {
      self.attributes.get( "positions" )
      .expect( "Poisitions attribute not found on geometry")
      .bounding_box.center()
    }

    pub fn bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.bind_vertex_array( Some( &self.vao ) );
    }

    pub fn draw( &self, gl : &gl::WebGl2RenderingContext )
    {
      if let Some( info ) = self.index_info.as_ref()
      {
        gl.draw_elements_with_i32( self.draw_mode, info.count as i32, info.data_type, info.offset as i32 );
      }
      else 
      {
        gl.draw_arrays( self.draw_mode, 0, self.vertex_count as i32 );
      }
    }
  }

}

crate::mod_interface!
{
  orphan use
  {
    BoundingBox,
    AttributeInfo,
    IndexInfo,
    Geometry
  };
}