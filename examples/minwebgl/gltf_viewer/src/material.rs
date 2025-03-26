use std::collections::HashSet;

use minwebgl as gl;
use gl::GL;

const VERTEX_SHADER_SRC : &str = include_str!( "../shaders/shader.vert" );
const FRAGMENT_SHADER_SRC : &str = include_str!( "../shaders/shader.frag" );

// These are the types supported by the tobj crate
#[ derive( Clone, PartialEq, Eq, Hash ) ]
pub enum TextureType
{
  Ambient,
  Diffuse,
  Specular,
  Normal,
  Shininess,
  Dissolve
}

#[ derive( Clone ) ]
pub struct GLMaterial
{
  pub program : gl::WebGlProgram,
  pub mtl : Option < tobj::Material >,
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

  pub fn from_tobj_material
  ( 
    gl : &GL, 
    material : &tobj::Material, 
    textures : &mut HashSet< ( String, TextureType ) > 
  ) 
  -> Result< Self, gl::WebglError >
  {
    let vs_defines = String::from( "#version 300 es\n" );
    let mut fs_defines = String::from( "#version 300 es\n" ); 

    if material.ambient.is_some() { fs_defines.push_str( "#define AMBIENT_COLOR\n" ); }
    if material.diffuse.is_some() { fs_defines.push_str( "#define DIFFUSE_COLOR\n" ); }
    if material.specular.is_some() { fs_defines.push_str( "#define SPECULAR_COLOR\n" ); }
    if material.shininess.is_some() { fs_defines.push_str( "#define SHININESS\n" ); }
    if material.dissolve.is_some() { fs_defines.push_str( "#define DISSOLVE\n" ); }
    if material.optical_density.is_some() { fs_defines.push_str( "#define OPTICAL_DENSITY\n" ); }

    let mut texture_names = Vec::with_capacity( 6 );
    let mut add_texture = | name : Option< &String >, define_str : &str, t_type : TextureType |
    {
      if let Some( name ) = name
      {
        fs_defines.push_str( define_str );
        let name = name.split_whitespace().last().unwrap().to_string();
        textures.insert( ( name.clone(), t_type ) );
        texture_names.push( Some( name ) );
      }
      else 
      {
        texture_names.push( None );   
      }
    };

    add_texture( material.ambient_texture.as_ref(), "#define AMBIENT_TEXTURE\n", TextureType::Ambient );
    add_texture( material.diffuse_texture.as_ref(), "#define DIFFUSE_TEXTURE\n", TextureType::Diffuse );
    add_texture( material.specular_texture.as_ref(), "#define SPECULAR_TEXTURE\n", TextureType::Specular );
    add_texture( material.normal_texture.as_ref(), "#define NORMAL_TEXTURE\n", TextureType::Normal );
    add_texture( material.shininess_texture.as_ref(), "#define SHININESS_TEXTURE\n", TextureType::Shininess );
    add_texture( material.dissolve_texture.as_ref(), "#define DISSOLVE_TEXTURE\n", TextureType::Dissolve );
    
    let vs = vs_defines + VERTEX_SHADER_SRC;
    let fs = fs_defines + FRAGMENT_SHADER_SRC;
    let program = gl::ProgramFromSources::new( &vs, &fs ).compile_and_link( gl )?;
    let mtl = Some( material.clone() );
    gl.use_program( Some( &program ) );

    // Assign a texture unit for each type of texture
    gl.uniform1i( gl.get_uniform_location( &program, "ambient_texture" ).as_ref() , 0 );
    gl.uniform1i( gl.get_uniform_location( &program, "diffuse_texture" ).as_ref() , 1 );
    gl.uniform1i( gl.get_uniform_location( &program, "specular_texture" ).as_ref() , 2 );
    gl.uniform1i( gl.get_uniform_location( &program, "normal_texture" ).as_ref() , 3 );
    gl.uniform1i( gl.get_uniform_location( &program, "shininess_texture" ).as_ref() , 4 );
    gl.uniform1i( gl.get_uniform_location( &program, "dissolve_texture" ).as_ref() , 5 );

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