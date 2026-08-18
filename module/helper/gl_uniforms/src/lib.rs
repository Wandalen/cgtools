//! Program-scoped WebGL uniform upload wrapper.
//!
//! Collapses the `gl.get_uniform_location( program, name )` +
//! `gl::uniform::(matrix_)upload( ... )` + `.expect( "uniform upload should not fail" )`
//! boilerplate repeated at every uniform call site across a WebGL2 renderer's shaders
//! into a single `.upload( name, &value )` / `.matrix_upload( name, &value, column_major )`
//! call.

use minwebgl as gl;

/// Binds a `GL` context and a linked `WebGlProgram` together so uniform uploads only need a
/// name and a value -- the location lookup and panic-on-failure `.expect` happen once, here,
/// instead of being repeated at every call site.
pub struct ProgramUniforms< 'a >
{
  gl : &'a gl::GL,
  program : &'a gl::WebGlProgram,
}

impl< 'a > ProgramUniforms< 'a >
{
  /// Wraps `gl`/`program` for uniform uploads. Callers must have already called
  /// `gl.use_program( Some( program ) )` -- this wrapper does not do so itself, since some
  /// call sites issue other GL calls (e.g. `draw_arrays`) between binding the program and
  /// uploading uniforms.
  #[ must_use ]
  pub fn new( gl : &'a gl::GL, program : &'a gl::WebGlProgram ) -> Self
  {
    Self { gl, program }
  }

  /// Looks up `name` in the bound program and uploads `data` as a non-matrix uniform.
  ///
  /// # Panics
  /// Panics if the upload fails -- matches every existing call site's own
  /// `.expect( "uniform upload should not fail" )`.
  pub fn upload< D >( &self, name : &str, data : &D )
  where
    D : gl::UniformUpload + ?Sized,
  {
    gl::uniform::upload( self.gl, self.gl.get_uniform_location( self.program, name ), data )
    .expect( "uniform upload should not fail" );
  }

  /// Looks up `name` in the bound program and uploads `data` as a matrix uniform.
  ///
  /// # Panics
  /// Panics if the upload fails -- matches every existing call site's own
  /// `.expect( "uniform upload should not fail" )`.
  pub fn matrix_upload< D >( &self, name : &str, data : &D, column_major : bool )
  where
    D : gl::UniformMatrixUpload + ?Sized,
  {
    gl::uniform::matrix_upload( self.gl, self.gl.get_uniform_location( self.program, name ), data, column_major )
    .expect( "uniform upload should not fail" );
  }
}

impl std::fmt::Debug for ProgramUniforms< '_ >
{
  fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
  {
    f.debug_struct( "ProgramUniforms" ).finish_non_exhaustive()
  }
}
