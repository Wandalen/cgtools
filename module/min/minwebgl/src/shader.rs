/// Internal namespace.
mod private
{
  use crate::*;
  pub use web_sys::{ WebGlShader };

  /// Represents errors related to shaders compilations.
  #[ derive( Debug, error::typed::Error ) ]
  pub enum Error
  {
    /// Error compiling shader.
    #[ error( "Error compiling shader\n\n  ShaderSource Type : {0}\n  ShaderSource Name : {1}\n\n = Explanation\n\n{2}\n= Source\n\n{3}" ) ]
    ShaderCompilationError( &'static str, String, String, String ),
    /// Error linking shaders into a program.
    #[ error( "Error linking shaders into a program\n\n = Explanation\n\n{0}\n" ) ]
    LinkingError( String ),
  }

  pub mod typ
  {
    use super::*;

    pub fn to_str( shader_type : u32 ) -> &'static str
    {
      match shader_type
      {
        GL::VERTEX_SHADER => "vertex shader",
        GL::FRAGMENT_SHADER => "fragment shader",
        _ => "unknown shader",
      }
    }

  }

  /// Information about shader necessary to compile it.
  #[ derive( Former, Debug, Default ) ]
  pub struct ShaderSource< 'a >
  {
    shader_type : u32,
    source : &'a str,
    shader_name : Option< &'a str >,
  }

  impl< 'a > ShaderSourceFormer< 'a >
  {

    pub fn compile( self, gl : &GL ) -> Result< WebGlShader, Error >
    {
      self.form().compile( gl )
    }

  }

  impl< 'a > ShaderSource< 'a >
  {

    /// Deduce shader name.
    pub fn name( &self ) -> &str
    {
      if let Some( name ) = self.shader_name
      {
        name
      }
      else
      {
        ""
      }
    }

    /// Utility function to compile a shader
    pub fn compile
    (
      &self,
      gl : &GL,
    ) -> Result< WebGlShader, Error >
    {

      let shader = gl.create_shader( self.shader_type )
      .ok_or_else( || String::from( "Unable to create shader object" ) )
      .map_err
      (
        | exp | Error::ShaderCompilationError
        (
          typ::to_str( self.shader_type ),
          self.name().to_string(),
          exp,
          self.source.to_string()
        )
      )?;
      gl.shader_source( &shader, self.source );
      gl.compile_shader( &shader );

      if gl.get_shader_parameter( &shader, GL::COMPILE_STATUS ).as_bool().unwrap_or( false )
      {
        Ok( shader )
      }
      else
      {
        Err
        (
          gl
          .get_shader_info_log( &shader )
          .unwrap_or_else( || String::from( "Unknown shader compilation error" ) )
        )
        .map_err
        (
          | exp | Error::ShaderCompilationError
          (
            typ::to_str( self.shader_type ),
            self.name().to_string(),
            exp,
            self.source.to_string()
          )
        )
      }
    }

  }

}

crate::mod_interface!
{

  orphan use
  {
    WebGlShader,
    ShaderSource,
  };

  own use
  {
    Error,
    typ,
  };

}
