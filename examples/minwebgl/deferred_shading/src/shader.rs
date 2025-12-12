//! Shader program loading

use minwebgl as gl;
use gl::GL;
use crate::types::Shaders;

/// Load all shader programs
pub fn load_shaders( gl : &GL ) -> Result< Shaders, gl::WebglError >
{
  let vert = include_str!( "../shaders/light_volume.vert" );
  let frag = include_str!( "../shaders/deferred.frag" );
  let light = gl::shader::Program::new( gl.clone(), vert, frag )?;

  let vert = include_str!( "../shaders/main.vert" );
  let frag = include_str!( "../shaders/gbuffer.frag" );
  let object = gl::shader::Program::new( gl.clone(), vert, frag )?;

  let vert = include_str!( "../shaders/big_triangle.vert" );
  let frag = include_str!( "../shaders/screen_texture.frag" );
  let screen = gl::shader::Program::new( gl.clone(), vert, frag )?;

  let vert = include_str!( "../shaders/light_sphere.vert" );
  let frag = include_str!( "../shaders/light_sphere.frag" );
  let light_sphere = gl::shader::Program::new( gl.clone(), vert, frag )?;

  Ok( Shaders { light, object, screen, light_sphere } )
}
