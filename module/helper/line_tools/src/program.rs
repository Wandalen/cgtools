mod private
{
  use crate::*;
  use minwebgl as gl;

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
    pub fn delete_vertex_shader( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.delete_shader( self.vertex_shader.as_ref() );
    }

    pub fn delete_fragment_shader( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.delete_shader( self.fragment_shader.as_ref() );
    }

    pub fn delete_program( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.delete_program( self.program.as_ref() );
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