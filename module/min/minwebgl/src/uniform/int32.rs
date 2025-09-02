use crate::*;
use core::any::type_name_of_val;

impl UniformUpload for i32
{
  fn upload( &self, gl : &GL, uniform_location : Option< WebGlUniformLocation > ) -> Result< (), WebglError >
  {
    Ok( gl.uniform1i( uniform_location.as_ref(), *self ) )
  }
}

impl UniformUpload for [ i32 ]
{
  fn upload( &self, gl : &GL, uniform_location : Option< WebGlUniformLocation > ) -> Result< (), WebglError >
  {
    match self.len()
    {
      1 => Ok( gl.uniform1iv_with_i32_array( uniform_location.as_ref(), self ) ),
      2 => Ok( gl.uniform2iv_with_i32_array( uniform_location.as_ref(), self ) ),
      3 => Ok( gl.uniform3iv_with_i32_array( uniform_location.as_ref(), self ) ),
      4 => Ok( gl.uniform4iv_with_i32_array( uniform_location.as_ref(), self ) ),
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

impl< const N : usize > UniformUpload for [ i32 ; N ]
{
  fn upload( &self, gl : &GL, uniform_location : Option< WebGlUniformLocation > ) -> Result< (), WebglError >
  {
    match self.len()
    {
      1 => Ok( gl.uniform1iv_with_i32_array( uniform_location.as_ref(), self ) ),
      2 => Ok( gl.uniform2iv_with_i32_array( uniform_location.as_ref(), self ) ),
      3 => Ok( gl.uniform3iv_with_i32_array( uniform_location.as_ref(), self ) ),
      4 => Ok( gl.uniform4iv_with_i32_array( uniform_location.as_ref(), self ) ),
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

impl< const N : usize > UniformUpload for [ [ i32 ; N ] ]
{
  fn upload( &self, gl : &GL, uniform_location : Option< WebGlUniformLocation > ) -> Result< (), WebglError >
  {
    match N
    {
      1 => Ok( gl.uniform1iv_with_i32_array( uniform_location.as_ref(), self.as_flattened() ) ),
      2 => Ok( gl.uniform2iv_with_i32_array( uniform_location.as_ref(), self.as_flattened() ) ),
      3 => Ok( gl.uniform3iv_with_i32_array( uniform_location.as_ref(), self.as_flattened() ) ),
      4 => Ok( gl.uniform4iv_with_i32_array( uniform_location.as_ref(), self.as_flattened() ) ),
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
