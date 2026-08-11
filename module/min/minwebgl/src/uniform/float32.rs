#[ allow( clippy::wildcard_imports, reason = "crate-root prelude from mod_interface!; enumerating would break on every layer change" ) ]
use crate::*;
use core::any::type_name_of_val;

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
