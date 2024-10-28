//! # Uniform Manipulation Module
//!
//! This module provides traits and functions for uploading uniform data to WebGL shaders. It supports various data types, including floats, integers, and matrices, and handles both single values and arrays.

mod private
{

  use crate::*;
  use core::any::type_name_of_val;
  pub use web_sys::WebGlUniformLocation;

  /// Trait for uploading uniform data to a WebGL shader.
  ///
  /// Implement this trait for types that can be uploaded as uniforms.
  pub trait UniformUpload
  {
    /// Uploads the uniform data to the specified location in the WebGL context.
    ///
    /// # Arguments
    ///
    /// * `gl` - The WebGL context.
    /// * `uniform_location` - The location of the uniform variable in the shader.
    ///
    /// # Returns
    ///
    /// * `Result<(), WebglError>` - Result indicating success or failure.
    fn upload
    (
      &self,
      gl : &GL,
      uniform_location : Option< WebGlUniformLocation >
    )
    -> Result< (), WebglError >;
  }

  /// Trait for uploading matrix uniform data to a WebGL shader.
  ///
  /// Implement this trait for matrix types that can be uploaded as uniforms.
  pub trait UniformMatrixUpload
  {
    /// Uploads the matrix uniform data to the specified location in the WebGL context.
    ///
    /// # Arguments
    ///
    /// * `gl` - The WebGL context.
    /// * `uniform_location` - The location of the uniform variable in the shader.
    /// * `column_major` - Whether the matrix is in column-major order.
    ///
    /// # Returns
    ///
    /// * `Result<(), WebglError>` - Result indicating success or failure.
    fn matrix_upload
    (
      &self,
      _gl : &GL,
      _uniform_location : Option< WebGlUniformLocation >,
      _column_major : bool
    )
    -> Result< (), WebglError >
    {
      Err( WebglError::NotSupportedForType( type_name_of_val( self ) ) )
    }
  }

  /// Uploads uniform data to a WebGL shader.
  ///
  /// # Arguments
  ///
  /// * `gl` - The WebGL context.
  /// * `uniform_location` - The location of the uniform variable in the shader.
  /// * `data` - The data to upload.
  ///
  /// # Returns
  ///
  /// * `Result<(), WebglError>` - Result indicating success or failure.
  pub fn upload< D >
  (
    gl : &GL,
    uniform_location : Option< WebGlUniformLocation >,
    data : &D
  )
  -> Result< (), WebglError >
  where
    D : UniformUpload + ?Sized,
  {
    data.upload( gl, uniform_location )
  }

  /// Uploads matrix uniform data to a WebGL shader.
  ///
  /// # Arguments
  ///
  /// * `gl` - The WebGL context.
  /// * `uniform_location` - The location of the uniform variable in the shader.
  /// * `data` - The matrix data to upload.
  /// * `column_major` - Whether the matrix is in column-major order.
  ///
  /// # Returns
  ///
  /// * `Result<(), WebglError>` - Result indicating success or failure.
  pub fn matrix_upload< D >
  (
    gl : &GL,
    uniform_location : Option< WebGlUniformLocation >,
    data : &D,
    column_major : bool
  )
  -> Result< (), WebglError >
  where
    D : UniformMatrixUpload + ?Sized,
  {
    data.matrix_upload( gl, uniform_location, column_major )
  }

}

mod float32;
mod int32;
mod unsigned32;

crate::mod_interface!
{
  prelude use UniformUpload;
  prelude use UniformMatrixUpload;
  orphan use WebGlUniformLocation;
  own use { upload, matrix_upload };

}