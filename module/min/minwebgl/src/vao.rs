/// Internal namespace.
mod private
{
  use crate::*;
  pub use web_sys::WebGlVertexArrayObject;

  /// Creates a new WebGL Vertex Array Object (VAO).
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
