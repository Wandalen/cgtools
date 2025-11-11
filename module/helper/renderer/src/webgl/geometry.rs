mod private
{
  use rustc_hash::FxHashMap;
  use mingl::geometry::BoundingBox;
  use minwebgl as gl;

  /// Represents information about a single vertex attribute.
  #[ derive( Debug, Clone ) ]
  pub struct AttributeInfo
  {
    /// The attribute slot index in the shader program.
    pub slot : u32,
    /// The WebGL buffer object containing the attribute data.
    pub buffer : gl::WebGlBuffer,
    /// Describes the layout and data type of the buffer.
    pub descriptor : gl::BufferDescriptor,
    /// The axis-aligned bounding box of the attribute data (e.g., for "positions").
    pub bounding_box : gl::geometry::BoundingBox
  }

  impl AttributeInfo
  {
    /// Configures the attribute pointer for this attribute, linking the buffer to the specified slot
    /// using the provided `WebGl2RenderingContext`.
    pub fn upload( &self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
    {
      self.descriptor.attribute_pointer( gl, self.slot, &self.buffer )?;

      Ok( () )
    }
  }

  /// Holds information about the index buffer used for indexed drawing.
  #[ derive( Debug, Clone ) ]
  pub struct IndexInfo
  {
    /// The WebGL buffer object containing the index data.
    pub buffer : gl::WebGlBuffer,
    /// The number of indices in the buffer.
    pub count : u32,
    /// The starting offset (in bytes) within the buffer.
    pub offset : u32,
    /// The data type of the indices (e.g., `gl::UNSIGNED_SHORT`, `gl::UNSIGNED_INT`).
    pub data_type : u32
  }

  /// Represents a geometric object to be rendered.
  #[ derive( Debug, Clone ) ]
  pub struct Geometry
  {
    /// The WebGL Vertex Array Object that stores the state for attribute bindings.
    pub vao : gl::WebGlVertexArrayObject,
    /// The primitive drawing mode (e.g., `gl::TRIANGLES`, `gl::LINES`).
    pub draw_mode : u32,
    /// The number of vertices in the geometry (used for non-indexed drawing).
    pub vertex_count : u32,
    /// Optional information about the index buffer, if the geometry uses indexed drawing.
    index_info : Option< IndexInfo >,
    /// A hash map storing attribute information, where the key is the attribute name (e.g., "positions", "normals").
    attributes : FxHashMap< Box< str >, AttributeInfo >
  }

  impl Geometry
  {
    /// Creates a new `Geometry` instance, initializing the VAO and other default values.
    pub fn new( gl : &gl::WebGl2RenderingContext ) -> Result< Self, gl::WebglError >
    {
      let vao = gl::vao::create( gl )?;
      let attributes = FxHashMap::default();
      let draw_mode = gl::TRIANGLES;
      let vertex_count = 0;
      let index_info = None;

      Ok
      (
        Self
        {
          vao,
          draw_mode,
          vertex_count,
          index_info,
          attributes
        }
      )
    }

    // Adds a new vertex attribute to the geometry.
    ///
    /// * `name`: The name of the attribute.
    /// * `info`: The `AttributeInfo` for the attribute.
    /// * `as_define`: A boolean indicating whether to add a `#define USE_UPPERCASE_NAME` to the `defines` string.
    ///
    /// It binds the VAO, uploads the attribute, and stores the `AttributeInfo`.
    /// It panics if an attribute with the same name already exists.
    pub fn add_attribute< Name : Into< Box< str > > >
    (
      &mut self,
      gl : &gl::WebGl2RenderingContext,
      name : Name,
      info : AttributeInfo
    ) -> Result< (), gl::WebglError >
    {
      let name = name.into();
      if !self.attributes.contains_key( &name )
      {
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

    /// Adds an index buffer to the geometry.
    ///
    /// It binds the VAO and the element array buffer, storing the information in the VAO.
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

    /// Uploads all attribute and index buffer data to the GPU.
    ///
    /// It binds the VAO and then iterates through the attributes to upload them.
    /// If an index buffer exists, it binds it as well.
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
    
    /// Returns the center point of the geometry's bounding box, assuming a "positions" attribute exists.
    ///
    /// It panics if the "positions" attribute is not found.
    pub fn center( &self ) -> gl::F32x3
    {
      self.bounding_box().center()
    }

    /// Return the bounding box of the `positions` attribute
    pub fn bounding_box( &self ) -> BoundingBox
    {
      self.attributes.get( "positions" )
      .expect( "Poisitions attribute not found on geometry")
      .bounding_box
    }

    /// Binds the geometry's Vertex Array Object.
    pub fn bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.bind_vertex_array( Some( &self.vao ) );
    }

    /// Performs the draw call for the geometry.
    ///
    /// It checks if an index buffer is present and calls `draw_elements` or `draw_arrays` accordingly.
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

    /// Returns a reference to the `FxHashMap` containing the attribute information.
    pub fn get_attributes( &self ) -> &FxHashMap< Box< str >, AttributeInfo >
    {
      &self.attributes
    }

    /// Performs the instanced draw call for the geometry.
    ///
    /// It checks if an index buffer is present and calls `draw_elements` or `draw_arrays` accordingly.
    pub fn draw_instanced( &self, gl : &gl::WebGl2RenderingContext, instance_count : i32 )
    {
      if let Some( info ) = self.index_info.as_ref()
      {
        gl.draw_elements_instanced_with_i32
        (
          self.draw_mode,
          info.count as i32,
          info.data_type,
          info.offset as i32,
          instance_count
        );
      }
      else
      {
        gl.draw_arrays_instanced
        (
          self.draw_mode,
          0,
          self.vertex_count as i32,
          instance_count
        );
      }
    }
  }

}

crate::mod_interface!
{
  orphan use
  {
    AttributeInfo,
    IndexInfo,
    Geometry
  };
}
