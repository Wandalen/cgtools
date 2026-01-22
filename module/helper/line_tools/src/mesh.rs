mod private
{
  use crate::*;
  use minwebgl as gl;
  use std::collections::HashMap;

  /// Represents a renderable mesh object, containing its associated WebGL resources.
  #[ derive( Default, Debug, Clone ) ]
  pub struct Mesh
  {
    /// A map from a program name to its corresponding `Program` object.
    pub program_map : HashMap< Box< str >, Program >,
    /// A map from a buffer name to its corresponding WebGL buffer handle.
    pub buffers : HashMap< Box< str >, gl::WebGlBuffer >
  }

  impl Mesh
  {
    /// Uploads a uniform value to all programs associated with the mesh.
    pub fn upload< D : Into< Uniform > + Copy >( &mut self, gl : &gl::WebGl2RenderingContext, uniform_name : &str, data : &D ) -> Result< (), gl::WebglError >
    {
      for p in self.program_map.values_mut()
      {
        p.upload( gl, uniform_name, data )?;
      }

      Ok( () )
    }

    /// Uploads a uniform value to a single, named program.
    pub fn upload_to< D : Into< Uniform > + Copy >( &mut self, gl : &gl::WebGl2RenderingContext, program_name : &str, uniform_name : &str, data : &D ) -> Result< (), gl::WebglError >
    {
      self.program_map.get_mut( program_name ).ok_or( gl::WebglError::Other( "Program with a specified name does not exist" ) )?
      .upload( gl, uniform_name, data )?;

      Ok( () )
    }

    /// Adds a new shader program to the mesh's collection.
    pub fn program_add< T : Into< Box< str > > >( &mut self, name : T, program : Program )
    {
      self.program_map.insert( name.into(), program );
    }

    /// Retrieves a reference to a program by its name.
    pub fn program_get( &self, name : &str ) -> &Program
    {
      self.program_map.get( name ).expect( "Program with the specified name does not exist" )
    }

    /// Retrieves a mutable reference to a program by its name.
    pub fn program_get_mut( &mut self, name : &str ) -> &mut Program
    {
      self.program_map.get_mut( name ).expect( "Program with the specified name does not exist" )
    }

    /// Draws the mesh using a single named program.
    pub fn draw( &self, gl : &gl::WebGl2RenderingContext, name : &str )
    {
      if let Some( p ) = self.program_map.get( name )
      {
        p.draw( gl );
      }
    }

    /// Draws the mesh using all programs in its collection.
    pub fn all_draw( &self, gl : &gl::WebGl2RenderingContext )
    {
      for p in self.program_map.values()
      {
        p.draw( gl );
      }
    }

    /// Retrieves a reference to a WebGL buffer by its name.
    pub fn buffer_get( &self, name : &str ) -> &gl::WebGlBuffer
    {
      self.buffers.get( name ).expect( "Buffer with the specified name does not exist" )
    }

    /// Adds a new WebGL buffer to the mesh's collection.
    pub fn buffer_add< T : Into< Box< str > > >( &mut self, name : T, buffer : gl::WebGlBuffer )
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
