use crate::*;
use core::any::type_name_of_val;

impl UniformUpload for u32
{
  fn upload( &self, gl : &GL, uniform_location : Option< WebGlUniformLocation > ) -> Result< (), WebglError >
  {
    Ok( gl.uniform1ui( uniform_location.as_ref(), *self ) )
  }
}

impl UniformUpload for [ u32 ]
{
  fn upload( &self, gl : &GL, uniform_location : Option< WebGlUniformLocation > ) -> Result< (), WebglError >
  {
    match self.len()
    {
      1 => Ok( gl.uniform1uiv_with_u32_array( uniform_location.as_ref(), self ) ),
      2 => Ok( gl.uniform2uiv_with_u32_array( uniform_location.as_ref(), self ) ),
      3 => Ok( gl.uniform3uiv_with_u32_array( uniform_location.as_ref(), self ) ),
      4 => Ok( gl.uniform4uiv_with_u32_array( uniform_location.as_ref(), self ) ),
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

impl< const N : usize > UniformUpload for [ u32 ; N ]
{
  fn upload( &self, gl : &GL, uniform_location : Option< WebGlUniformLocation > ) -> Result< (), WebglError >
  {
    match self.len()
    {
      1 => Ok( gl.uniform1uiv_with_u32_array( uniform_location.as_ref(), self ) ),
      2 => Ok( gl.uniform2uiv_with_u32_array( uniform_location.as_ref(), self ) ),
      3 => Ok( gl.uniform3uiv_with_u32_array( uniform_location.as_ref(), self ) ),
      4 => Ok( gl.uniform4uiv_with_u32_array( uniform_location.as_ref(), self ) ),
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
