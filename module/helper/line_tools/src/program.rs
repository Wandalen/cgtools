mod private
{
  use crate::*;
  use minwebgl::{self as gl, JsCast, JsValue };

  #[ derive( Clone, Debug, Default ) ]
  pub struct Program
  {
    pub vertex_shader : Option< gl::WebGlShader >,
    pub fragment_shader : Option< gl::WebGlShader >,
    pub program :Option< gl::WebGlProgram >,
    pub vao : Option< gl::web_sys::WebGlVertexArrayObject >,
    pub draw_mode : u32,
    pub instance_count : Option< u32 >,
    pub index_count : Option< u32 >,
    pub vertex_count : u32
  }

  impl Program 
  {
    pub fn delete_vertex_shader( &mut self, gl : &gl::WebGl2RenderingContext )
    {
      gl.delete_shader( self.vertex_shader.as_ref() );
      self.vertex_shader = None;
    }

    pub fn delete_fragment_shader( &mut self, gl : &gl::WebGl2RenderingContext )
    {
      gl.delete_shader( self.fragment_shader.as_ref() );
      self.fragment_shader = None;
    }

    pub fn delete_program( &mut self, gl : &gl::WebGl2RenderingContext )
    {
      gl.delete_program( self.program.as_ref() );
      self.program = None;
    }

    pub fn delete_vao( &mut self, gl : &gl::WebGl2RenderingContext )
    {
      gl.delete_vertex_array( self.vao.as_ref() );
      self.vao = None;
    }

    pub fn copy_uniforms_to( &self, gl : &gl::WebGl2RenderingContext, program : &gl::WebGlProgram ) -> Result< (), gl::WebglError >
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
            => gl::uniform::upload( gl, location, gl::js_sys::Float32Array::from( value ).to_vec().as_slice() )?,
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

    pub fn upload< D >( &self, gl : &gl::WebGl2RenderingContext, name : &str, data : &D  ) -> Result< (), gl::WebglError >
    where 
      D : gl::UniformUpload + ?Sized
    {
      gl.use_program( self.program.as_ref() );
      gl::uniform::upload( gl, gl.get_uniform_location( self.program.as_ref().expect( "Cannot upload, because the program is not set" ), name ), data )?;

      Ok( () )
    }

    pub fn upload_matrix< D >( &self, gl : &gl::WebGl2RenderingContext, name : &str, data : &D  ) -> Result< (), gl::WebglError >
    where 
      D : gl::UniformMatrixUpload + ?Sized
    {
      gl.use_program( self.program.as_ref() );
      gl::uniform::matrix_upload( gl, gl.get_uniform_location( self.program.as_ref().expect( "Cannot upload, because the program is not set" ), name ), data, true )?;

      Ok( () )
    }
  
    pub fn bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.use_program( self.program.as_ref() );
      gl.bind_vertex_array( self.vao.as_ref() );
    }

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
  }
    
}

crate::mod_interface!
{
  orphan use 
  {
    Program
  };
}