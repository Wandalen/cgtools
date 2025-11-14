use renderer::webgl::{ ShaderProgram, material::*, program::ProgramInfo };
use renderer::impl_locations;
use minwebgl as gl;
use gl::{ GL, F32x3, Former };
use rustc_hash::FxHashMap;
use uuid::Uuid;

/// Gem shader
pub struct GemShader;

impl_locations!
(
  GemShader,
  "worldMatrix",
  "inverseWorldMatrix",
  "viewMatrix",
  "projectionMatrix",

  "envMap",
  "cubeNormalMap",

  "rayBounces",
  "color",
  "boostFactors",

  "envMapIntensity",
  "rainbowDelta",
  "squashFactor",
  "radius",
  "geometryFactor",
  "absorptionFactor",
  "colorAbsorption",
  "cameraPosition"
);

/// The source code for the gem vertex shader.
const GEM_VERTEX_SHADER : &'static str = include_str!( "../shaders/gem.vert" );
/// The source code for the gem fragment shader.
const GEM_FRAGMENT_SHADER : &'static str = include_str!( "../shaders/gem.frag" );

/// Represents the visual properties of a gem surface.
#[ derive( Former, Debug ) ]
pub struct GemMaterial
{
  /// A unique identifier for the material.
  pub id : Uuid,
  /// Ray bounces inside gem count
  pub ray_bounces : i32,
  /// Gem color
  pub color : gl::F32x4,
  ///
  pub boost_factors : F32x3,
  ///
  pub env_map_intensity : f32,
  ///
  pub rainbow_delta : f32,
  ///
  pub squash_factor : f32,
  ///
  pub radius : f32,
  ///
  pub geometry_factor : f32,
  ///
  pub absorption_factor : f32,
  ///
  pub color_absorption : F32x3,
  /// Equirectangular environment texture
  pub environment_texture : Option< TextureInfo >,
  /// Cube normal map texture
  pub cube_normal_map_texture : Option< TextureInfo >,
}

impl Material for GemMaterial
{
  fn get_id( &self ) -> Uuid
  {
    self.id
  }

  /// Returns [`ProgramInfo`] with shader locations and used [`ShaderProgram`]
  fn get_program_info( &self, gl : &GL, program : &gl::WebGlProgram ) -> ProgramInfo
  {
    ProgramInfo::new( gl, program, GemShader.dyn_clone() )
  }

  fn get_type_name( &self ) -> &'static str
  {
    "GemMaterial"
  }

  fn get_vertex_shader( &self ) -> String
  {
    GEM_VERTEX_SHADER.into()
  }

  fn get_fragment_shader( &self ) -> String
  {
    GEM_FRAGMENT_SHADER.into()
  }

  fn configure
  (
    &self,
    gl : &gl::WebGl2RenderingContext,
    locations : &FxHashMap< String, Option< gl::WebGlUniformLocation > >,
    _ibl_base_location : u32,
  )
  {
    gl.uniform1i( locations.get( "envMap" ).unwrap().clone().as_ref() , 0 );
    gl.uniform1i( locations.get( "cubeNormalMap" ).unwrap().clone().as_ref() , 1 );
  }

  fn upload
  (
    &self,
    gl : &GL,
    locations : &FxHashMap< String, Option< gl::WebGlUniformLocation > >
  )
  -> Result< (), gl::WebglError >
  {
    let upload = | loc, value : f32 | -> Result< (), gl::WebglError >
    {
      gl::uniform::upload( gl, locations.get( loc ).unwrap().clone(), &value )?;
      Ok( () )
    };

    let upload_array = | loc, value : &[ f32 ] | -> Result< (), gl::WebglError >
    {
      gl::uniform::upload( gl, locations.get( loc ).unwrap().clone(), value )?;
      Ok( () )
    };

    gl::uniform::upload( gl, locations.get( "rayBounces" ).unwrap().clone(), &self.ray_bounces )?;

    upload( "envMapIntensity", self.env_map_intensity )?;
    upload( "rainbowDelta", self.rainbow_delta )?;
    upload( "squashFactor", self.squash_factor )?;
    upload( "radius", self.radius )?;
    upload( "geometryFactor", self.geometry_factor )?;
    upload( "absorptionFactor", self.absorption_factor )?;

    upload_array( "color", self.color.0.as_slice() )?;
    upload_array( "boostFactors", self.boost_factors.0.as_slice() )?;
    upload_array( "colorAbsorption", self.color_absorption.0.as_slice() )?;

    self.upload_textures( gl );

    Ok( () )
  }

  fn upload_textures( &self, gl : &GL )
  {
    if let Some( ref t ) = self.environment_texture { t.upload( gl ); }
    if let Some( ref t ) = self.cube_normal_map_texture { t.upload( gl ); }
  }

  fn bind( &self, gl : &GL )
  {
    let bind = | texture : &Option< TextureInfo >, i |
    {
      if let Some( ref t ) = texture
      {
        gl.active_texture( gl::TEXTURE0 + i );
        t.bind( gl );
      }
    };

    bind( &self.environment_texture, 0 );
    bind( &self.cube_normal_map_texture, 1 );
  }

  fn dyn_clone( &self ) -> Box< dyn Material >
  {
    Box::new( self.clone() )
  }
}

impl Clone for GemMaterial
{
  fn clone( &self ) -> Self
  {
    GemMaterial
    {
      id : Uuid::new_v4(),
      ray_bounces : self.ray_bounces,
      color : self.color.clone(),
      boost_factors : self.boost_factors.clone(),
      env_map_intensity : self.env_map_intensity,
      rainbow_delta : self.rainbow_delta,
      squash_factor : self.squash_factor,
      radius : self.radius,
      geometry_factor : self.geometry_factor,
      absorption_factor : self.absorption_factor,
      color_absorption : self.color_absorption,
      environment_texture : self.environment_texture.clone(),
      cube_normal_map_texture : self.cube_normal_map_texture.clone(),
    }
  }
}

impl Default for GemMaterial
{
  fn default() -> Self
  {
    return Self
    {
      id : Uuid::new_v4(),
      ray_bounces : 7,
      color : gl::F32x4::from_array( [ 0.98, 0.95, 0.9, 1.0 ] ),
      boost_factors : F32x3::from_array( [ 0.8920, 0.8920, 0.9860 ] ),
      env_map_intensity : 0.7,
      rainbow_delta : 0.012,
      squash_factor : 0.98,
      radius : 1000.0,
      geometry_factor : 0.5,
      absorption_factor : 0.7,
      color_absorption : F32x3::from_array( [ 0.9911, 0.9911, 0.9911 ] ),
      environment_texture : None,
      cube_normal_map_texture : None
    };
  }
}
