use renderer::webgl::{ ShaderProgram, material::*, program::ProgramInfo, Node };
use renderer::impl_locations;
use minwebgl as gl;
use gl::{ GL, Former, WebGlProgram };
use rustc_hash::FxHashMap;
use uuid::Uuid;
use std:: { cell::RefCell, rc::Rc };

/// Gem shader
#[ derive( Debug ) ]
pub struct GemShader( ProgramInfo );

impl_locations!
(
  GemShader,
  "worldMatrix",
  "inverseWorldMatrix",
  "viewMatrix",
  "projectionMatrix",
  "normalMatrix",
  "offsetMatrix",
  "inverseOffsetMatrix",

  "envMap",
  "cubeNormalMap",

  "rayBounces",
  "diamondColor",

  "envMapIntensity",
  "radius",
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
  /// Shader program info
  program : GemShader,
  /// Ray bounces inside gem count
  pub ray_bounces : i32,
  /// Gem color
  pub color : gl::F32x3,
  /// Defines how fluent envMap on reflected light
  pub env_map_intensity : f32,
  /// Participates in calculation of point on the surface
  pub radius : f32,
  /// Equirectangular environment texture
  pub environment_texture : Option< TextureInfo >,
  /// Cube normal map texture
  pub cube_normal_map_texture : Option< TextureInfo >,
  /// Signal for updating material uniforms
  pub needs_update : bool
}

impl GemMaterial
{
  pub fn new( gl : &GL ) -> Self
  {
    // Compile and link a new WebGL program from the vertex and fragment shaders with the appropriate defines.
    let program = gl::ProgramFromSources::new
    (
      &format!( "#version 300 es\n{}", GEM_VERTEX_SHADER ),
      &format!( "#version 300 es\n{}", GEM_FRAGMENT_SHADER )
    ).compile_and_link( gl )
    .unwrap();

    Self
    {
      id : Uuid::new_v4(),
      program : GemShader::new( gl, &program ),
      ray_bounces : 5,
      color : gl::F32x3::from_array( [ 0.98, 0.95, 0.9 ] ),
      env_map_intensity : 1.0,
      radius : 1000.0,
      environment_texture : None,
      cube_normal_map_texture : None,
      needs_update : true
    }
  }
}

impl Material for GemMaterial
{
  fn get_id( &self ) -> Uuid
  {
    self.id
  }

  fn needs_update( &self ) -> bool
  {
    self.needs_update
  }

  fn shader( &self ) -> &dyn ShaderProgram
  {
    &self.program
  }

  fn shader_mut( &mut self ) -> &mut dyn ShaderProgram
  {
    &mut self.program
  }

  fn type_name( &self ) -> &'static str
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
    node : Rc< RefCell< Node > >,
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

    let bb = node.borrow().bounding_box();
    let c = bb.center();
    let max_distance = ( bb.max - c ).mag().max( ( bb.min - c ).mag() );

    upload( "envMapIntensity", self.env_map_intensity )?;
    upload( "radius", max_distance )?;

    upload_array( "diamondColor", self.color.0.as_slice() )?;

    let offset_mat = gl::math::mat3x3h::translation( -node.borrow().bounding_box().center() );

    gl::uniform::matrix_upload( gl, locations.get( "offsetMatrix" ).unwrap().clone(), offset_mat.raw_slice(), true )?;
    gl::uniform::matrix_upload( gl, locations.get( "inverseOffsetMatrix" ).unwrap().clone(), offset_mat.inverse().unwrap().raw_slice(), true )?;

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
      env_map_intensity : self.env_map_intensity,
      radius : self.radius,
      environment_texture : self.environment_texture.clone(),
      cube_normal_map_texture : self.cube_normal_map_texture.clone(),
      needs_update : self.needs_update,
      program : self.program.clone()
    }
  }
}
