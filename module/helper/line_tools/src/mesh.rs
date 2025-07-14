mod private
{
  use crate::*;
  use minwebgl::{self as gl, uniform::upload};
  use std::collections::HashMap;

  #[ derive( Default ) ]
  pub struct Mesh
  {
    pub program_list : Vec< Program >,
    pub program_map : HashMap< Box< str >, Program >
  }

  impl Mesh 
  {
    pub fn upload< D >( &self, gl : &gl::WebGl2RenderingContext, uniform_name : &str, data : &D ) -> Result< (), gl::WebglError >
    where 
      D : gl::UniformUpload + ?Sized
    {
      for p in self.program_list.iter()
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
      for p in self.program_list.iter()
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

    pub fn add_program( &mut self, name : &str, program : Program )
    {
      self.program_list.push( program.clone() );
      self.program_map.insert( name.into(), program );
    }

    pub fn draw( &self, gl : &gl::WebGl2RenderingContext )
    {
      for p in self.program_list.iter()
      {
        p.draw( gl );
      }
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