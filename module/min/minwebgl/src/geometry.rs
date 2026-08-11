/// Internal namespace.
mod private
{
  use crate::{ GL, WebGlVertexArrayObject, VectorDataType, WebglError, buffer, DataType, vao, BufferDescriptor, AsBytes };

  /// Represents the vertices geometry, including its vertex array object (VAO)
  /// and the number of vertices.
  #[ non_exhaustive ]
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
  /// Returns `WebglError::NotSupportedForType` if `natoms` is outside `1 ..= 4` —
  /// the size range `vertex_attrib_pointer` accepts for a single attribute slot.
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
      1 ..= 4 => Ok( () ),
      _ => Err( WebglError::NotSupportedForType( "natoms outside 1..=4 is not supported by Positions::new" ) ),
    }
  }

  impl Positions
  {
    /// Creates a new `Positions` from a flat list of vertex positions with `natoms`
    /// components per vertex ( any arity in `1 ..= 4` — 2D, 3D, and 4D positions alike ).
    ///
    /// # Parameters
    /// - `gl`: The WebGL context.
    /// - `positions`: A flat slice of f32 vertex positions, `natoms` components per vertex.
    /// - `natoms`: Components per vertex, `1 ..= 4`.
    ///
    /// # Returns
    /// A `Result` which is:
    /// - `Ok(Positions)` containing the created VAO and vertex count if successful.
    /// - `Err(WebglError)` if there is an issue creating buffers, VAOs, or uploading the geometry data.
    ///
    /// # Errors
    /// Returns `WebglError::NotSupportedForType` if `natoms` is outside `1 ..= 4`, and
    /// `WebglError::Other` if `positions.len()` does not fit into `i32` ( the vertex count
    /// a WebGL `vertexAttribPointer` call can address ). Also propagates any `WebglError`
    /// returned while creating buffers, VAOs, or uploading the geometry data.
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
    #[ inline ]
    pub fn new( gl : GL, positions : &[ f32 ], natoms : i32 ) -> Result< Self, WebglError >
    {
      validate_natoms( natoms )?;
      let position_buffer = buffer::create( &gl )?;
      let typ = VectorDataType::new( DataType::F32, natoms, 1 );
      buffer::upload( &gl, &position_buffer, positions, GL::STATIC_DRAW );
      let vao = vao::create( &gl )?;
      gl.bind_vertex_array( Some( &vao ) );

      // `BufferDescriptor` is fully data-driven — `attribute_pointer` reads only the
      // runtime `VectorDataType` — so no natoms-to-compile-time-type dispatch is
      // needed: build the descriptor from `typ` directly. This is what generalizes
      // `Positions::new` to every arity `validate_natoms` accepts.
      let descriptor = BufferDescriptor
      {
        vector : typ,
        offset : 0,
        stride : 0,
        divisor : 0,
        normalized : false,
      };
      descriptor.attribute_pointer( &gl, 0, &position_buffer )?;

      let nvertices : i32 = positions.len().try_into()
      .map_err( | _ | WebglError::Other( "positions length exceeds i32::MAX" ) )?;
      let nvertices = nvertices / natoms;
      Ok( Positions { gl, vao, typ, nvertices } )
    }

    /// Activates the vertex array object (VAO) associated with this shader program.
    ///
    /// This method binds the VAO stored in the `vao` field to the current WebGL context
    /// by calling `bind_vertex_array`. Binding the VAO ensures that subsequent rendering operations,
    /// such as draw calls, will use the correct vertex attribute configurations defined within this VAO.
    ///
    /// # Note
    /// Ensure that the VAO has been properly initialized before calling this method.
    #[ inline ]
    pub fn activate( &self )
    {
      self.gl.bind_vertex_array( Some( &self.vao ) );
    }

  }

  // Documented exception (task 069) to the all-tests-in-tests/ convention: these tests stay
  // inline because `validate_natoms` is a private helper by design -- extracting it INTO a
  // testable private function was the BUG-052 fix, and it is deliberately absent from the
  // `mod_interface` exports; publishing it solely for test placement would widen the API for
  // no caller. Native `tests/` coverage of the crate's public pure-logic surface lives in
  // `tests/` (see the readme's Testing section for the full runnability story).
  #[ cfg( test ) ]
  mod tests
  {
    use super::*;

    #[ test ]
    fn validate_natoms_accepts_supported_values()
    {
      for natoms in 1 ..= 4
      {
        assert!( validate_natoms( natoms ).is_ok(), "natoms {natoms} must be supported" );
      }
    }

    // test_kind: bug_reproducer(BUG-052)
    /// RED state (empirically confirmed): reverting this helper's body to the pre-fix
    /// `panic!( "Unsapported buffer descriptor" )` and marking this test `#[should_panic]`
    /// genuinely panics — verified via a temporary probe before this fix was finalized.
    /// The original probe value was `3`; task 062's switch removal made `1 ..= 4`
    /// supported, so the unsupported probes moved outside that range. The BUG-052
    /// contract under test is unchanged: unsupported `natoms` returns `Err`, never panics.
    #[ test ]
    fn validate_natoms_rejects_unsupported_value()
    {
      for natoms in [ 0, 5, -1 ]
      {
        let result = validate_natoms( natoms );
        assert!
        (
          matches!( result, Err( WebglError::NotSupportedForType( _ ) ) ),
          "natoms {natoms} must be rejected with NotSupportedForType"
        );
      }
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
