#![ allow( missing_docs ) ]

/// Internal namespace.
mod private
{
  use crate::*;
  pub use web_sys::{ WebGlShader, WebGlProgram };
  use std::cell::RefCell;
  use
  {
    uniform::{ UniformUpload, WebGlUniformLocation },
  };

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

  /// Utilities for working with shader types.
  pub mod typ
  {
    use super::*;

    /// Convert shader type constant to human-readable string.
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

  /// Information about a shader necessary to compile it.
  #[ derive( Former, Debug, Default ) ]
  pub struct ShaderSource< 'a >
  {
    /// The type of the shader (e.g., `GL::VERTEX_SHADER`).
    shader_type : u32,
    /// The source code of the shader.
    source : &'a str,
    /// An optional name for the shader.
    shader_name : Option< &'a str >,
  }

  /// Implementation for the `Former` pattern for `ShaderSource`.
  impl< 'a > ShaderSourceFormer< 'a >
  {

    /// Compiles the formed `ShaderSource` into a `WebGlShader`.
    pub fn compile( self, gl : &GL ) -> Result< WebGlShader, Error >
    {
      self.form().compile( gl )
    }

  }

  /// Implementation for `ShaderSource`.
  impl< 'a > ShaderSource< 'a >
  {

    /// Deduce the shader's name. Returns an empty string if no name is provided.
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

    /// Compiles the shader source code and returns a `WebGlShader`.
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

  #[ derive( New ) ]
  /// Compile shaders and link them into a program, give readable diagnostic information if fail.
  pub struct ProgramFromSources< 'a >
  {
    /// The source code for the vertex shader.
    vertex_shader : &'a str,
    /// The source code for the fragment shader.
    fragment_shader : &'a str,
  }

  // /// Implementation for `ProgramFromSources`.
  // impl< 'a > ProgramFromSources< 'a >
  // {
  //   /// Create a new ProgramFromSources with vertex and fragment shader source code.
  //   pub fn new( vertex_shader : &'a str, fragment_shader : &'a str ) -> Self
  //   {
  //     Self { vertex_shader, fragment_shader }
  //   }
  // }

  impl< 'a > ProgramFromSources< 'a >
  {
    /// Compiles and links the shaders into a program.
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

      let shaders_for_program = program::ProgramShaders::new( &vertex_shader, &fragment_shader );
      shaders_for_program.link( &gl )
    }

  }

  /// Set of shaders necessary to compile a GPU program.
  pub struct ProgramShaders< 'a >
  {
    /// A reference to the compiled vertex shader.
    vertex_shader : &'a WebGlShader,
    /// A reference to the compiled fragment shader.
    fragment_shader : &'a WebGlShader,
  }

  /// Implementation for `ProgramShaders`.
  impl< 'a > ProgramShaders< 'a >
  {
    /// Create a new ProgramShaders with compiled vertex and fragment shaders.
    pub fn new( vertex_shader : &'a WebGlShader, fragment_shader : &'a WebGlShader ) -> Self
    {
      Self { vertex_shader, fragment_shader }
    }
  }

  impl< 'a > ProgramShaders< 'a >
  {

    /// Link the vertex and fragment shaders into a WebGL program.
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

  /// An interface for shader programs.
  ///
  /// This trait declares the required methods for working with shader programs
  /// in a WebGL context, including compilation, setting uniforms (including matrix uniforms),
  /// and drawing.
  pub trait ProgramInterface
  {
    /// Compiles and links shader source code and updates the program.
    fn compile_and_link( &self, vertex_src : &str, fragment_src : &str ) -> Result< (), String >;
    /// Sets a uniform value in the shader.
    fn uniform_upload< D >( &self, name : &str, value : &D )
    where
      D : UniformUpload + std::fmt::Debug + ?Sized;
    /// Sets a matrix uniform value in the shader.
    fn uniform_matrix_upload< D >( &self, name : &str, data : &D, column_major : bool )
    where
      D : uniform::UniformMatrixUpload + ?Sized;

    // xxx : clean
    // /// Draws the active shader program.
    // fn draw( &self, mode : u32, count : i32 );

  }

  /// A shader program for rendering with WebGL.
  /// This structure encapsulates a WebGL program and provides methods to compile,
  /// link, set uniforms (including matrix uniforms), and draw with the program.
  #[ derive( Debug ) ]
  pub struct Program
  {
    /// The graphical context.
    gl : GL,
    /// Reference to `WebGlProgram`
    program : RefCell< Option< WebGlProgram > >,
  }

  /// Implementation for `Program`.
  impl Program
  {
    /// Creates a new `Program` from vertex and fragment shader source code.
    ///
    /// # Parameters
    /// - `gl`: The WebGL context.
    /// - `vertex_src`: The source code for the vertex shader.
    /// - `fragment_src`: The source code for the fragment shader.
    ///
    /// # Returns
    /// A `Result` which is:
    /// - `Ok(Program)` if the shaders compile and link successfully.
    /// - `Err(WebglError)` if there is an error during shader compilation or linking.
    pub fn new( gl : GL, vertex_src : &str, fragment_src : &str ) -> Result< Self, WebglError >
    {
      let program = Self::compile_and_link( &gl, vertex_src, fragment_src )?;
      Ok( Program { gl, program : RefCell::new( Some( program )) })
    }

    /// Compiles and links the vertex and fragment shaders into a WebGL program
    /// using the structures provided by `program`.
    ///
    /// # Parameters
    /// - `gl`: The WebGL context.
    /// - `vertex_src`: The source code for the vertex shader.
    /// - `fragment_src`: The source code for the fragment shader.
    ///
    /// # Returns
    /// A `Result` which is:
    /// - `Ok(WebGlProgram)` if the shaders compile and link successfully.
    /// - `Err(WebglError)` if there is an error during shader compilation or linking.
    pub fn compile_and_link( gl : &GL, vertex_src : &str, fragment_src : &str ) -> Result< WebGlProgram, WebglError >
    {
      // Use the ProgramFromSources structure from program to compile and link shaders.
      ProgramFromSources::new( vertex_src, fragment_src )
      .compile_and_link( gl )
      .map_err( |e| e.into() )
    }

    /// Sets the current WebGL program as the active program in the WebGL context.
    ///
    /// The function retrieves the program from the internal RefCell and calls
    /// use_program on the WebGL context with an Option, which is Some if a
    /// program has been compiled and linked, or None otherwise.
    pub fn activate( &self )
    {
      self.gl.use_program( self.program.borrow().as_ref() );
    }

    /// Sets a uniform value in the shader using a type that implements `UniformUpload`.
    ///
    /// # Parameters
    /// - `name`: The name of the uniform variable.
    /// - `value`: A reference to the value to upload, which must implement `UniformUpload`.
    pub fn uniform_upload< D >( &self, name : &str, value : &D )
    where
      D : UniformUpload + std::fmt::Debug + ?Sized,
    {
      if let Some( ref prog ) = *self.program.borrow()
      {
        let location : Option< WebGlUniformLocation > = self.gl.get_uniform_location( prog, name );
        // log::info!( "location : {:?}", location );
        // log::info!( "value : {:?}", value );
        uniform::upload( &self.gl, location, value ).unwrap();
      }
    }

    /// Sets a matrix uniform value in the shader using a type that implements `UniformMatrixUpload`.
    ///
    /// # Parameters
    /// - `name`: The name of the uniform variable.
    /// - `data`: A reference to the matrix data to upload, which must implement `UniformMatrixUpload`.
    /// - `column_major`: A boolean indicating whether the matrix data is in column-major order.
    pub fn uniform_matrix_upload< D >( &self, name : &str, data : &D, column_major : bool )
    where
      D : uniform::UniformMatrixUpload + ?Sized,
    {
      if let Some( ref prog ) = *self.program.borrow()
      {
        let location : Option< WebGlUniformLocation > = self.gl.get_uniform_location( prog, name );
        uniform::matrix_upload( &self.gl, location, data, column_major ).unwrap();
      }
    }

    // xxx : clean
    // /// Draws the active shader program using the specified mode and vertex count.
    // ///
    // /// # Parameters
    // /// - `mode`: The primitive type to render (e.g., `GL::TRIANGLES`).
    // /// - `count`: The number of vertices to render.
    // pub fn draw( &self, mode : u32, count : i32 )
    // {
    //   // Assumes the program is already in use.
    //   self.gl.draw_arrays( mode, 0, count );
    // }

  }

  impl ProgramInterface for Program
  {
    /// Compiles and link new shader sources and updates the program.
    ///
    /// # Parameters
    /// - `vertex_src`: The new vertex shader source.
    /// - `fragment_src`: The new fragment shader source.
    ///
    /// # Returns
    /// A `Result` which is:
    /// - `Ok(())` if the shaders compile and link successfully.
    /// - `Err(String)` containing an error message if compilation or linking failed.
    fn compile_and_link( &self, vertex_src : &str, fragment_src : &str ) -> Result< (), String >
    {
      let program = Self::compile_and_link( &self.gl, vertex_src, fragment_src )
        .map_err( |e| e.to_string() )?;
      *self.program.borrow_mut() = Some( program );
      Ok(())
    }

    /// Sets a uniform value in the shader using a type that implements `UniformUpload`.
    ///
    /// # Parameters
    /// - `name`: The name of the uniform variable.
    /// - `value`: A reference to the value to upload, which must implement `UniformUpload`.
    fn uniform_upload< D >( &self, name : &str, value : &D )
    where
      D : UniformUpload + std::fmt::Debug + ?Sized,
    {
      Program::uniform_upload( self, name, value );
    }

    /// Sets a matrix uniform value in the shader using a type that implements `UniformMatrixUpload`.
    ///
    /// # Parameters
    /// - `name`: The name of the uniform variable.
    /// - `data`: A reference to the matrix data to upload, which must implement `UniformMatrixUpload`.
    /// - `column_major`: A boolean indicating whether the matrix data is in column-major order.
    fn uniform_matrix_upload< D >( &self, name : &str, data : &D, column_major : bool )
    where
      D : uniform::UniformMatrixUpload + ?Sized,
    {
      Program::uniform_matrix_upload( self, name, data, column_major );
    }

    // xxx : clean
    // /// Draws the active shader program using the specified mode and vertex count.
    // ///
    // /// # Parameters
    // /// - `mode`: The primitive type to render (e.g., `GL::TRIANGLES`).
    // /// - `count`: The number of vertices to render.
    // fn draw( &self, mode : u32, count : i32 )
    // {
    //   Program::draw( self, mode, count );
    // }

  }

}

crate::mod_interface!
{

  orphan use
  {

    WebGlShader,
    WebGlProgram,

    ProgramFromSources,
    ProgramShaders,
    ShaderSource,

    ProgramInterface,
    Program,

  };

  own use
  {
    Error,
    typ,
  };

}
