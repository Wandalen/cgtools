/// Internal namespace.
mod private
{
  #[ allow( clippy::wildcard_imports, reason = "crate-root prelude from mod_interface!; enumerating would break on every layer change" ) ]
  use crate::*;
  pub use web_sys::WebGlVertexArrayObject;

  /// Creates a new WebGL Vertex Array Object (VAO).
  ///
  /// # Errors
  /// Returns `WebglError::FailedToAllocateResource` if the WebGL context fails to allocate the VAO.
  pub fn create( gl : &GL ) -> Result< WebGlVertexArrayObject, WebglError >
  {
    gl.create_vertex_array().ok_or( WebglError::FailedToAllocateResource( "VAO" ) )
  }

}

crate::mod_interface!
{

  orphan use WebGlVertexArrayObject;
  own use create;

}
