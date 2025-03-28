use std::collections::HashSet;

use minwebgl as gl;
use gl::GL;

const VERTEX_SHADER_SRC : &str = include_str!( "../shaders/shader.vert" );
const FRAGMENT_SHADER_SRC : &str = include_str!( "../shaders/shader.frag" );

#[ derive( Clone ) ]
pub struct GLMaterial
{
  pub program : gl::WebGlProgram,

  pub texture_names : Vec< Option< String > > 
}

impl GLMaterial 
{
  pub fn new_simple( gl : &GL ) -> Result< Self, gl::WebglError >
  {
    let vs_defines = String::from( "#version 300 es\n" );
    let fs_defines = String::from( "#version 300 es\n" );

    let vs = vs_defines + VERTEX_SHADER_SRC;
    let fs = fs_defines + FRAGMENT_SHADER_SRC;
    let program = gl::ProgramFromSources::new( &vs, &fs ).compile_and_link( gl )?;
    let mtl = None;
    let texture_names = Vec::new();

    Ok
    (
      GLMaterial
      {
        program,
        mtl,
        texture_names
      }
    )
  }

  pub fn from_gltf_material
  ( 
    gl : &GL, 
    material : &gltf::material::Material, 
  ) 
  -> Result< Self, gl::WebglError >
  {
    let vs_defines = String::from( "#version 300 es\n" );
    let mut fs_defines = String::from( "#version 300 es\n" ); 


    Ok
    (
      GLMaterial
      {
        program,
        mtl,
        texture_names
      }
    )
  }

  // Upload uniforms related to the shading that are not going to change
  pub fn init_uniforms( &self, gl : &GL )
  {
    gl.use_program( Some( &self.program ) );

    if self.mtl.is_none() { return; }
    let mtl = self.mtl.as_ref().unwrap();

    if let Some( ref ambient ) = mtl.ambient
    {
      gl::uniform::upload
      (
        &gl, 
        gl.get_uniform_location( &self.program, "ambient" ), 
        &ambient[ .. ]
      ).unwrap();
    }

    if let Some( ref diffuse ) = mtl.diffuse
    {
      gl::uniform::upload
      (
        &gl, 
        gl.get_uniform_location( &self.program, "diffuse" ), 
        &diffuse[ .. ]
      ).unwrap();
    }

    if let Some( ref specular ) = mtl.specular
    {
      gl::uniform::upload
      (
        &gl, 
        gl.get_uniform_location( &self.program, "specular" ), 
        &specular[ .. ]
      ).unwrap();
    }

    if let Some( ref shininess ) = mtl.shininess
    {
      gl::uniform::upload
      (
        &gl, 
        gl.get_uniform_location( &self.program, "shininess" ), 
        shininess
      ).unwrap();
    }

    if let Some( ref dissolve ) = mtl.dissolve
    {
      gl::uniform::upload
      (
        &gl, 
        gl.get_uniform_location( &self.program, "dissolve" ), 
        dissolve
      ).unwrap();
    }

    if let Some( ref optical_density ) = mtl.optical_density
    {
      gl::uniform::upload
      (
        &gl, 
        gl.get_uniform_location( &self.program, "optical_density" ), 
        optical_density
      ).unwrap();
    }
  }
}