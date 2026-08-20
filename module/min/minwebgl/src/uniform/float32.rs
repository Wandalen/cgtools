#[ allow( clippy::wildcard_imports, reason = "crate-root prelude from mod_interface!; enumerating would break on every layer change" ) ]
use crate::*;
use core::any::type_name_of_val;
// `own use` in `uniform.rs`'s `mod_interface!` exposes this at the `uniform::` path but does not
// bubble it into the crate-root wildcard glob above ( unlike `prelude use`-marked items ), so it
// needs its own explicit import.
use crate::uniform::f32_matrix_length_error;

impl UniformUpload for f32
{
  fn upload( &self, gl : &GL, uniform_location : Option< WebGlUniformLocation > ) -> Result< (), WebglError >
  {
    gl.uniform1f( uniform_location.as_ref(), *self );
    Ok( () )
  }
}

impl UniformUpload for [ f32 ]
{
  fn upload( &self, gl : &GL, uniform_location : Option< WebGlUniformLocation > ) -> Result< (), WebglError >
  {
    match self.len()
    {
      1 => { gl.uniform1fv_with_f32_array( uniform_location.as_ref(), self ); Ok( () ) },
      2 => { gl.uniform2fv_with_f32_array( uniform_location.as_ref(), self ); Ok( () ) },
      3 => { gl.uniform3fv_with_f32_array( uniform_location.as_ref(), self ); Ok( () ) },
      4 => { gl.uniform4fv_with_f32_array( uniform_location.as_ref(), self ); Ok( () ) },
      _ => Err
      (
        WebglError::CantUploadUniform
        (
          "vector",
          type_name_of_val( self ),
          self.len(),
          "1, 2, 3, 4",
        ),
      )
    }
  }
}

impl UniformMatrixUpload for [ f32 ]
{
  fn matrix_upload( &self, gl : &GL, uniform_location : Option< WebGlUniformLocation >, column_major : bool ) -> Result< (), WebglError >
  {
    match self.len()
    {
      4 => { gl.uniform_matrix2fv_with_f32_array( uniform_location.as_ref(), !column_major, self ); Ok( () ) },
      9 => { gl.uniform_matrix3fv_with_f32_array( uniform_location.as_ref(), !column_major, self ); Ok( () ) },
      16 => { gl.uniform_matrix4fv_with_f32_array( uniform_location.as_ref(), !column_major, self ); Ok( () ) },
      // Fix(BUG-277): report this arm's error via the matrix-specific `f32_matrix_length_error`
      // builder instead of a copy-pasted vector-upload error literally reading item kind
      // "vector" and known lengths "1, 2, 3, 4" -- both wrong for a matrix upload.
      // Root cause: this match arm's error branch was copy-pasted from `UniformUpload::upload`'s
      // vector-length error above, and the literal "vector"/"1, 2, 3, 4" strings were never
      // updated for the matrix context ( valid flat lengths are 4, 9, 16 -- 2x2/3x3/4x4 ).
      // Pitfall: `WebglError::CantUploadUniform`'s constant string arguments have no compiler
      // link to the match arms around them -- a copy-pasted error branch silently carries a
      // stale, misleading message that only a reader ( or a test on message content ) catches.
      _ => Err( f32_matrix_length_error( type_name_of_val( self ), self.len() ) ),
    }
  }
}

impl< const N : usize > UniformUpload for [ f32 ; N ]
{
  fn upload( &self, gl : &GL, uniform_location : Option< WebGlUniformLocation > ) -> Result< (), WebglError >
  {
    match self.len()
    {
      1 => { gl.uniform1fv_with_f32_array( uniform_location.as_ref(), self ); Ok( () ) },
      2 => { gl.uniform2fv_with_f32_array( uniform_location.as_ref(), self ); Ok( () ) },
      3 => { gl.uniform3fv_with_f32_array( uniform_location.as_ref(), self ); Ok( () ) },
      4 => { gl.uniform4fv_with_f32_array( uniform_location.as_ref(), self ); Ok( () ) },
      _ => Err
      (
        WebglError::CantUploadUniform
        (
          "vector",
          type_name_of_val( self ),
          self.len(),
          "1, 2, 3, 4",
        ),
      )
    }
  }
}

impl< const N : usize > UniformUpload for [ [ f32 ; N ] ]
{
  fn upload( &self, gl : &GL, uniform_location : Option< WebGlUniformLocation > ) -> Result< (), WebglError >
  {
    match N
    {
      1 => { gl.uniform1fv_with_f32_array( uniform_location.as_ref(), self.as_flattened() ); Ok( () ) },
      2 => { gl.uniform2fv_with_f32_array( uniform_location.as_ref(), self.as_flattened() ); Ok( () ) },
      3 => { gl.uniform3fv_with_f32_array( uniform_location.as_ref(), self.as_flattened() ); Ok( () ) },
      4 => { gl.uniform4fv_with_f32_array( uniform_location.as_ref(), self.as_flattened() ); Ok( () ) },
      _ => Err
      (
        WebglError::CantUploadUniform
        (
          "vector",
          type_name_of_val( self ),
          self.len(),
          "1, 2, 3, 4",
        ),
      )
    }
  }
}

impl< const N : usize > UniformMatrixUpload for [ f32 ; N ]
{
  fn matrix_upload( &self, gl : &GL, uniform_location : Option< WebGlUniformLocation >, column_major : bool ) -> Result< (), WebglError >
  {
    match self.len()
    {
      4 => { gl.uniform_matrix2fv_with_f32_array( uniform_location.as_ref(), !column_major, self ); Ok( () ) },
      9 => { gl.uniform_matrix3fv_with_f32_array( uniform_location.as_ref(), !column_major, self ); Ok( () ) },
      16 => { gl.uniform_matrix4fv_with_f32_array( uniform_location.as_ref(), !column_major, self ); Ok( () ) },
      // Fix(BUG-277): same copy-pasted-error fix as the `[ f32 ]` impl above -- see its comment.
      _ => Err( f32_matrix_length_error( type_name_of_val( self ), self.len() ) ),
    }
  }
}
