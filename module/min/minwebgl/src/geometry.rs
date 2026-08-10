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

  /// Checks whether `natoms` is a vector arity currently supported for
  /// vertex attribute upload by [ `Positions::new` ].
  ///
  /// # Errors
  /// Returns `WebglError::NotSupportedForType` if `natoms` is not currently
  /// supported ( only `2` is supported at the moment ).
  // Fix(BUG-052): was `_ => panic!( "Unsapported buffer descriptor" )` — an
  // unsupported natoms value ( e.g. loading geometry with 3 or 4 components
  // per vertex ) crashed the whole process instead of giving the caller a
  // `Result` to handle.
  // Root cause: `Positions::new` already returns `Result< Self, WebglError >`
  // and uses `?` for every other fallible step, but this one arm was written
  // as a `panic!` instead of returning through the existing error type.
  // Pitfall: a function that already returns `Result` is exactly where a
  // stray `panic!`/`unwrap`/`expect` is easiest to miss in review — grep for
  // those macros in any function whose signature already promises `Result`.
  fn validate_natoms( natoms : i32 ) -> Result< (), WebglError >
  {
    match natoms
    {
      2 => Ok( () ),
      _ => Err( WebglError::NotSupportedForType( "natoms other than 2 is not supported by Positions::new" ) ),
    }
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
      validate_natoms( natoms )?;
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
        // natoms is already validated by `validate_natoms` above, so any value
        // other than 2 already returned early via `?` — this arm can't run.
        _ => unreachable!( "natoms already validated by validate_natoms" ),
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

  #[ cfg( test ) ]
  mod tests
  {
    use super::*;

    #[ test ]
    fn validate_natoms_accepts_supported_value()
    {
      assert!( validate_natoms( 2 ).is_ok() );
    }

    /// RED state (empirically confirmed): reverting this helper's body to the pre-fix
    /// `panic!( "Unsapported buffer descriptor" )` and marking this test `#[should_panic]`
    /// genuinely panics — verified via a temporary probe before this fix was finalized.
    #[ test ]
    fn validate_natoms_rejects_unsupported_value()
    {
      let result = validate_natoms( 3 );
      assert!( matches!( result, Err( WebglError::NotSupportedForType( _ ) ) ) );
    }
  }

}

crate::mod_interface!
{
  reuse ::mingl::geometry;

  own use
  {
    Positions,
  };

}
