mod private
{
  use crate::*;
  use minwebgl as gl;
  use std::collections::HashMap;

  #[ derive( Default, Debug, Clone ) ]
  pub struct Mesh
  {
    pub program_map : HashMap< Box< str >, Program >,
    pub buffers : HashMap< Box< str >, gl::WebGlBuffer >
  }

  impl Mesh 
  {
    pub fn upload< D >( &self, gl : &gl::WebGl2RenderingContext, uniform_name : &str, data : &D ) -> Result< (), gl::WebglError >
    where 
      D : gl::UniformUpload + ?Sized
    {
      for p in self.program_map.values()
      {
        p.upload( gl, uniform_name, data )?;
      }

      Ok( () )
    }

    pub fn upload_to< D >( &self, gl : &gl::WebGl2RenderingContext, program_name : &str, uniform_name : &str, data : &D ) -> Result< (), gl::WebglError >
    where 
      D : gl::UniformUpload + ?Sized
    {
      self.program_map.get( program_name ).expect( "Program with a specified name does not exist" )
      .upload( gl, uniform_name, data )?;

      Ok( () )
    }

    pub fn upload_matrix< D >( &self, gl : &gl::WebGl2RenderingContext, uniform_name : &str, data : &D ) -> Result< (), gl::WebglError >
    where 
      D : gl::UniformMatrixUpload + ?Sized
    {
      for p in self.program_map.values()
      {
        p.upload_matrix( gl, uniform_name, data )?;
      }

      Ok( () )
    }

    pub fn upload_matrix_to< D >( &self, gl : &gl::WebGl2RenderingContext, program_name : &str, uniform_name : &str, data : &D ) -> Result< (), gl::WebglError >
    where 
      D : gl::UniformMatrixUpload + ?Sized
    {
      self.program_map.get( program_name ).expect( "Program with a specified name does not exist" )
      .upload_matrix( gl, uniform_name, data )?;
    
      Ok( () )
    }

    pub fn add_program< T : Into< Box< str > > >( &mut self, name : T, program : Program )
    {
      self.program_map.insert( name.into(), program );
    }

    pub fn get_program( &self, name : &str ) -> &Program
    {
      self.program_map.get( name ).expect( "Program with the specified name does not exist" )
    }

    pub fn get_program_mut( &mut self, name : &str ) -> &mut Program
    {
      self.program_map.get_mut( name ).expect( "Program with the specified name does not exist" )
    }

    pub fn draw( &self, gl : &gl::WebGl2RenderingContext, name : &str )
    {
      if let Some( p ) = self.program_map.get( name )
      {
        p.draw( gl );
      }
    }

    pub fn draw_all( &self, gl : &gl::WebGl2RenderingContext )
    {
      for p in self.program_map.values()
      {
        p.draw( gl );
      }
    }

    pub fn get_buffer( &self, name : &str ) -> &gl::WebGlBuffer
    {
      self.buffers.get( name ).expect( "Buffer with the specified name does not exist" )
    }

    pub fn add_buffer< T : Into< Box< str > > >( &mut self, name : T, buffer : gl::WebGlBuffer )
    {
      self.buffers.insert( name.into(), buffer );
    }
  }
    
}

crate::mod_interface!
{
  orphan use 
  {
    Mesh
  };
}