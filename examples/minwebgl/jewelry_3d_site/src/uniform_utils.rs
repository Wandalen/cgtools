/// Utility functions for working with WebGL uniforms
use std::collections::HashMap;
use minwebgl as gl;

/// Helper function to get uniform location from a HashMap with error handling.
///
/// # Arguments
/// * `locations` - HashMap of uniform names to locations (wrapped in Option)
/// * `name` - Name of the uniform to retrieve
///
/// # Returns
/// * `Ok(WebGlUniformLocation)` if found
///
/// # Errors
/// * `Err(WebglError)` if the uniform is not found in the HashMap
#[ allow( clippy::ptr_arg ) ]
#[ inline ]
pub fn get_uniform_location< S : ::std::hash::BuildHasher >
(
  locations : &HashMap< String, Option< web_sys::WebGlUniformLocation >, S >,
  name : &str
)
-> Result< web_sys::WebGlUniformLocation, gl::WebglError >
{
  locations.get( name )
  .and_then( std::clone::Clone::clone )
  .ok_or( gl::WebglError::Other( "Shader uniform not found" ) )
}
