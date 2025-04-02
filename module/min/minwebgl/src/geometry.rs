/// Internal namespace.
mod private
{
  use crate::*;

  /// Represents the vertices geometry, including its vertex array object (VAO)
  /// and the number of vertices.
  pub struct Positions
  {
    /// Graphical context.
    pub gl : GL,
    /// The WebGL Vertex Array Object.
    pub vao : WebGlVertexArrayObject,
    /// Vector descriptor.
    pub typ : VectorDataType,
    /// The number of vertices contained in the geometry.
    pub nvertices : i32,
  }

  impl Positions
  {
    /// Creates a new `Positions` for a 2D shape from a list of vertex positions.
    ///
    /// # Parameters
    /// - `gl`: The WebGL context.
    /// - `positions`: A slice of f32 representing the 2D vertex positions.
    ///
    /// # Returns
    /// A `Result` which is:
    /// - `Ok(Positions)` containing the created VAO and vertex count if successful.
    /// - `Err(WebglError)` if there is an issue creating buffers, VAOs, or uploading the geometry data.
    ///
    /// # Example
    ///
    /// ```
    /// # use minwebgl::{ GL, geometry::Positions, WebglError };
    /// # fn example( gl : GL ) -> Result< (), WebglError >
    /// {
    /// let positions = vec![ 0.0, 0.0, 1.0, 1.0, -1.0, 1.0 ];
    /// let geometry = Positions::new( gl, &positions, 2 )?;
    /// // Use `geometry.vao` for rendering and `geometry.nvertices` for the vertex count.
    /// # Ok(())
    /// # }
    /// ```
    pub fn new( gl : GL, positions : &[ f32 ], natoms : i32 ) -> Result< Self, WebglError >
    {
      let position_buffer = buffer::create( &gl )?;
      let typ = VectorDataType::new( DataType::F32, natoms, 1 );
      buffer::upload( &gl, &position_buffer, positions, GL::STATIC_DRAW );
      let vao = vao::create( &gl )?;
      gl.bind_vertex_array( Some( &vao ) );

      // qqq : xxx : move out switch and make it working for all types
      match typ.natoms
      {
        2 =>
        {
          BufferDescriptor::new::< [ f32; 2 ] >()
          .stride( 0 )
          .offset( 0 )
          .divisor( 0 )
          .attribute_pointer( &gl, 0, &position_buffer )?;
        },
        _ => { panic!( "Unsapported buffer descriptor" ) }
      }

      let nvertices = positions.len() as i32 / natoms;
      Ok( Positions { vao, typ, nvertices, gl } )
    }

    /// Activates the vertex array object (VAO) associated with this shader program.
    ///
    /// This method binds the VAO stored in the `vao` field to the current WebGL context
    /// by calling `bind_vertex_array`. Binding the VAO ensures that subsequent rendering operations,
    /// such as draw calls, will use the correct vertex attribute configurations defined within this VAO.
    ///
    /// # Note
    /// Ensure that the VAO has been properly initialized before calling this method.
    pub fn activate( &self )
    {
      self.gl.bind_vertex_array( Some( &self.vao ) );
    }

  }

}

crate::mod_interface!
{

  own use
  {
    Positions,
  };

}
