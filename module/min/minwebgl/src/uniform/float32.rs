use crate::*;
use core::any::type_name_of_val;

impl UniformUpload for f32
{
  fn upload( &self, gl : &GL, uniform_location : Option< WebGlUniformLocation > ) -> Result< (), WebglError >
  {
    Ok( gl.uniform1f( uniform_location.as_ref(), *self ) )
  }
}

impl UniformUpload for [ f32 ]
{
  fn upload( &self, gl : &GL, uniform_location : Option< WebGlUniformLocation > ) -> Result< (), WebglError >
  {
    match self.len()
    {
      1 => Ok( gl.uniform1fv_with_f32_array( uniform_location.as_ref(), self ) ),
      2 => Ok( gl.uniform2fv_with_f32_array( uniform_location.as_ref(), self ) ),
      3 => Ok( gl.uniform3fv_with_f32_array( uniform_location.as_ref(), self ) ),
      4 => Ok( gl.uniform4fv_with_f32_array( uniform_location.as_ref(), self ) ),
      _ => Err
      (
        WebglError::CanUploadUniform
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
      4 => Ok( gl.uniform_matrix2fv_with_f32_array( uniform_location.as_ref(), !column_major, self ) ),
      9 => Ok( gl.uniform_matrix3fv_with_f32_array( uniform_location.as_ref(), !column_major, self ) ),
      16 => Ok( gl.uniform_matrix4fv_with_f32_array( uniform_location.as_ref(), !column_major, self ) ),
      _ => Err
      (
        WebglError::CanUploadUniform
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
      1 => Ok( gl.uniform1fv_with_f32_array( uniform_location.as_ref(), self ) ),
      2 => Ok( gl.uniform2fv_with_f32_array( uniform_location.as_ref(), self ) ),
      3 => Ok( gl.uniform3fv_with_f32_array( uniform_location.as_ref(), self ) ),
      4 => Ok( gl.uniform4fv_with_f32_array( uniform_location.as_ref(), self ) ),
      _ => Err
      (
        WebglError::CanUploadUniform
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
      4 => Ok( gl.uniform_matrix2fv_with_f32_array( uniform_location.as_ref(), !column_major, self ) ),
      9 => Ok( gl.uniform_matrix3fv_with_f32_array( uniform_location.as_ref(), !column_major, self ) ),
      16 => Ok( gl.uniform_matrix4fv_with_f32_array( uniform_location.as_ref(), !column_major, self ) ),
      _ => Err
      (
        WebglError::CanUploadUniform
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
