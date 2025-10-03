mod private
{
  use minwebgl as gl;
  use crate::*;

  /// Represents a complete WebGL shader program and its drawing configuration.
  #[ derive( Clone, Debug, Default ) ]
  pub struct Program
  {
    /// The WebGL handle for the vertex shader. `None` if not yet compiled or deleted.
    pub vertex_shader : Option< gl::WebGlShader >,
    /// The WebGL handle for the fragment shader. `None` if not yet compiled or deleted.
    pub fragment_shader : Option< gl::WebGlShader >,
    /// The WebGL handle for the linked shader program. `None` if not yet linked or deleted.
    pub program :Option< gl::WebGlProgram >,
    /// The WebGL Vertex Array Object handle. `None` if not yet created or deleted.
    pub vao : Option< gl::web_sys::WebGlVertexArrayObject >,
    /// The primitive type for the draw call (e.g., `gl::TRIANGLES`, `gl::LINES`).
    pub draw_mode : u32,
    /// The number of instances to draw. `None` for non-instanced drawing.
    pub instance_count : Option< u32 >,
    /// The number of indices to draw. `None` for non-indexed drawing.
    pub index_count : Option< u32 >,
    /// The number of vertices to draw.
    pub vertex_count : u32,
    /// The WebGL buffer for indices. `None` for non-indexed drawing.
    pub index_buffer : Option< gl::WebGlBuffer >,
    /// Uniforms belonging to this program
    pub uniforms : UniformStorage
  }

  impl Program 
  {
    /// Deletes the vertex shader from the WebGL context and sets the internal handle to `None`.
    pub fn vertex_shader_delete( &mut self, gl : &gl::WebGl2RenderingContext )
    {
      gl.delete_shader( self.vertex_shader.as_ref() );
      self.vertex_shader = None;
    }

    /// Deletes the fragment shader from the WebGL context and sets the internal handle to `None`.
    pub fn fragment_shader_delete( &mut self, gl : &gl::WebGl2RenderingContext )
    {
      gl.delete_shader( self.fragment_shader.as_ref() );
      self.fragment_shader = None;
    }

    /// Deletes the main shader program from the WebGL context and sets the internal handle to `None`.
    pub fn program_delete( &mut self, gl : &gl::WebGl2RenderingContext )
    {
      gl.delete_program( self.program.as_ref() );
      self.program = None;
    }

    /// Deletes the Vertex Array Object from the WebGL context and sets the internal handle to `None`.
    pub fn vao_delete( &mut self, gl : &gl::WebGl2RenderingContext )
    {
      gl.delete_vertex_array( self.vao.as_ref() );
      self.vao = None;
    }

    /// Copies all active uniform values from this program to another program.
    pub fn uniforms_copy_to_gl( &self, gl : &gl::WebGl2RenderingContext, program : &gl::WebGlProgram ) -> Result< (), gl::WebglError >
    {
      if let Some( own_program ) = self.program.as_ref()
      {
        let active_uniforms_count = gl.get_program_parameter( &own_program, gl::ACTIVE_UNIFORMS ).as_f64().unwrap() as u32;

        for i in 0..active_uniforms_count
        {
          let active_uniform = gl.get_active_uniform( own_program, i ).unwrap();
          let uniform_name = active_uniform.name();
          let value = gl.get_uniform( own_program, &gl.get_uniform_location( own_program, uniform_name.as_str() ).unwrap() );

          let location = gl.get_uniform_location( program, uniform_name.as_str() );

          gl.use_program( Some( &program ) );

          match active_uniform.type_()
          {
            // Scalars
            gl::FLOAT 
            =>  gl::uniform::upload( gl, location, &( value.as_f64().unwrap() as f32 )  )?,
            gl::INT 
            =>  gl::uniform::upload( gl, location, &( value.as_f64().unwrap() as i32 )  )?,
            gl::UNSIGNED_INT 
            =>  gl::uniform::upload( gl, location, &( value.as_f64().unwrap() as u32 )  )?,

            // Vectors
            gl::FLOAT_VEC2 | gl::FLOAT_VEC3 | gl::FLOAT_VEC4 
            => { gl::uniform::upload( gl, location, gl::js_sys::Float32Array::from( value ).to_vec().as_slice() )? },
            gl::INT_VEC2 | gl::INT_VEC3 | gl::INT_VEC4 
            => gl::uniform::upload( gl, location, gl::js_sys::Int32Array::from( value ).to_vec().as_slice() )?,
            gl::UNSIGNED_INT_VEC2 | gl::UNSIGNED_INT_VEC3 | gl::UNSIGNED_INT_VEC4 
            => gl::uniform::upload( gl, location, gl::js_sys::Uint32Array::from( value ).to_vec().as_slice() )?,

            // Matrices
            gl::FLOAT_MAT2 | gl::FLOAT_MAT3 | gl::FLOAT_MAT4
            => gl::uniform::matrix_upload( gl, location, gl::js_sys::Float32Array::from( value ).to_vec().as_slice(), true )?,
            _ => { gl::info!( "Unsupported uniform type for copy" ) }
          }

        }
      }

      Ok( () )
    }

    /// Copies uniforms from the current program to the other
    pub fn uniforms_copy_to( &self, gl : &gl::WebGl2RenderingContext, program : &mut Self ) -> Result< (), gl::WebglError >
    {
      self.uniforms.copy_to( &mut program.uniforms );
      program.all_uniforms_upload( gl )?;
      Ok( () )
    }

    /// Uploads a uniform value to the current program.
    pub fn upload< D : Into< Uniform > + Copy >( &mut self, gl : &gl::WebGl2RenderingContext, name : &str, data : &D  ) -> Result< (), gl::WebglError >
    {
      self.uniforms.uniform_set( name, data.into() );
      if let Some( program ) = self.program.as_ref()
      {
        self.uniforms.upload( gl, program, name, true )?;
      }

      Ok( () )
    }

    /// Binds the program, VAO, and index buffer to the WebGL context.
    pub fn bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.use_program( self.program.as_ref() );
      gl.bind_vertex_array( self.vao.as_ref() );
      gl.bind_buffer( gl::ELEMENT_ARRAY_BUFFER, self.index_buffer.as_ref() );
    }

    /// Executes the draw call for the program's geometry.
    pub fn draw( &self, gl : &gl::WebGl2RenderingContext )
    {
      self.bind( gl );

      if let Some( index_count ) = self.index_count
      {
        if let Some( instance_count ) = self.instance_count
        {
          gl.draw_elements_instanced_with_i32( self.draw_mode, index_count as i32, gl::UNSIGNED_INT, 0, instance_count as i32 );
        }
        else 
        {
          gl.draw_elements_with_i32( self.draw_mode, index_count as i32, gl::UNSIGNED_INT, 0 );
        }
      }
      else 
      {
        if let Some( instance_count ) = self.instance_count
        {
          gl.draw_arrays_instanced( self.draw_mode, 0, self.vertex_count as i32, instance_count as i32 );
        }
        else 
        {
          gl.draw_arrays( self.draw_mode, 0, self.vertex_count as i32 );
        }
      }
    }

    /// Clear saved uniforms locations
    pub fn uniform_locations_clear( &mut self )
    {
      self.uniforms.clear_locations();
    }

    /// Clear saved uniforms
    pub fn uniforms_clear( &mut self )
    {
      self.uniforms.clear_uniforms();
    }

    /// Upload the current set of uniform to the program
    pub fn all_uniforms_upload( &mut self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
    {
      if let Some( program ) = self.program.as_ref()
      {
        self.uniforms.all_upload( gl, program, true )?;
      }

      Ok( () )
    }
  }
    
}

crate::mod_interface!
{
  orphan use 
  {
    Program
  };
}