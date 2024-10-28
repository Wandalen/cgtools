/// Internal namespace.
mod private
{
  use crate::*;
  pub use shader::Error;
  pub use web_sys::WebGlProgram;

  /// Compile shaders and link them into a program, give readable diagnostic information if fail.
  #[ derive( New ) ]
  pub struct ProgramFromSources< 'a >
  {
    vertex_shader : &'a str,
    fragment_shader : &'a str,
  }

  impl< 'a > ProgramFromSources< 'a >
  {

    /// Compile shaders and link them into a program, give readable diagnostic information if fail.
    pub fn compile_and_link( &self, gl : &GL ) -> Result< WebGlProgram, Error >
    {

      let vertex_shader = ShaderSource::former()
      .shader_type( GL::VERTEX_SHADER )
      .source( self.vertex_shader )
      .compile( &gl )?;

      let fragment_shader = ShaderSource::former()
      .shader_type( GL::FRAGMENT_SHADER )
      .source( self.fragment_shader )
      .compile( &gl )?;

      let shaders_for_program = program::ShadersForProgram::new( &vertex_shader, &fragment_shader );
      shaders_for_program.link( &gl )
    }

  }

  /// Set of shaders necessary to compile a GPU program.
  #[ derive( New ) ]
  pub struct ShadersForProgram< 'a >
  {
    vertex_shader : &'a WebGlShader,
    fragment_shader : &'a WebGlShader,
  }

  impl< 'a > ShadersForProgram< 'a >
  {

    // Utility function to link a program
    pub fn link
    (
      &self,
      gl : &GL,
    ) -> Result< WebGlProgram, Error >
    {
      let program = gl.create_program().ok_or_else( ||
      {
        let reason = "Unable to create shader object".to_string();
        Error::LinkingError( reason )
      })?;
      gl.attach_shader( &program, self.vertex_shader );
      gl.attach_shader( &program, self.fragment_shader );
      gl.link_program( &program );

      if gl.get_program_parameter( &program, GL::LINK_STATUS ).as_bool().unwrap_or( false )
      {
        Ok( program )
      }
      else
      {
        let reason = gl.get_program_info_log( &program ).unwrap_or_else( || String::from( "Unknown error creating program object" ) );
        Err( Error::LinkingError( reason ) )
      }
    }

  }

}

crate::mod_interface!
{

  own use
  {
    Error,
  };

  orphan use
  {
    WebGlProgram,
    ProgramFromSources,
    ShadersForProgram,
  };

}
