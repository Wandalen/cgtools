#[ allow( clippy::wildcard_imports, reason = "crate-root prelude from mod_interface!; enumerating would break on every layer change" ) ]
use crate::*;
use core::any::type_name_of_val;

impl UniformUpload for i32
{
  fn upload( &self, gl : &GL, uniform_location : Option< WebGlUniformLocation > ) -> Result< (), WebglError >
  {
    gl.uniform1i( uniform_location.as_ref(), *self );
    Ok( () )
  }
}

impl UniformUpload for [ i32 ]
{
  fn upload( &self, gl : &GL, uniform_location : Option< WebGlUniformLocation > ) -> Result< (), WebglError >
  {
    match self.len()
    {
      1 => { gl.uniform1iv_with_i32_array( uniform_location.as_ref(), self ); Ok( () ) },
      2 => { gl.uniform2iv_with_i32_array( uniform_location.as_ref(), self ); Ok( () ) },
      3 => { gl.uniform3iv_with_i32_array( uniform_location.as_ref(), self ); Ok( () ) },
      4 => { gl.uniform4iv_with_i32_array( uniform_location.as_ref(), self ); Ok( () ) },
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
      1 => { gl.uniform1iv_with_i32_array( uniform_location.as_ref(), self ); Ok( () ) },
      2 => { gl.uniform2iv_with_i32_array( uniform_location.as_ref(), self ); Ok( () ) },
      3 => { gl.uniform3iv_with_i32_array( uniform_location.as_ref(), self ); Ok( () ) },
      4 => { gl.uniform4iv_with_i32_array( uniform_location.as_ref(), self ); Ok( () ) },
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
      1 => { gl.uniform1iv_with_i32_array( uniform_location.as_ref(), self.as_flattened() ); Ok( () ) },
      2 => { gl.uniform2iv_with_i32_array( uniform_location.as_ref(), self.as_flattened() ); Ok( () ) },
      3 => { gl.uniform3iv_with_i32_array( uniform_location.as_ref(), self.as_flattened() ); Ok( () ) },
      4 => { gl.uniform4iv_with_i32_array( uniform_location.as_ref(), self.as_flattened() ); Ok( () ) },
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
