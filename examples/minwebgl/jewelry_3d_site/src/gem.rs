#![ allow( clippy::exhaustive_structs ) ]

use renderer::webgl::{ ShaderProgram, material::*, program::ProgramInfo, MaterialUploadContext };
use renderer::impl_locations;
use minwebgl as gl;
use gl::{ GL, Former, WebGlProgram };
use rustc_hash::FxHashMap;
use uuid::Uuid;
use crate::cube_normal_map_generator::CubeNormalData;
use crate::{gem_frag, gem_vert};
use crate::uniform_utils::get_uniform_location;

// Gem shader
impl_locations!
(
  GemShader,
  "worldMatrix",
  "inverseWorldMatrix",
  "viewMatrix",
  "projectionMatrix",
  "normalMatrix",
  "d",
  "l",

  "v",
  "m",

  "n",
  "r",

  "s",
  "w",
  "cameraPosition",

  "a",
  "c",
  "e"
);

/// Represents the visual properties of a gem surface.
#[ allow( clippy::exhaustive_structs ) ]
#[ derive( Former, Debug ) ]
#[ non_exhaustive ]
pub struct GemMaterial
{
  /// A unique identifier for the material.
  pub id : Uuid,
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
  pub cube_normal_map_texture : CubeNormalData,
  /// Signal for updating material uniforms
  pub needs_update : bool,
  /// Refraction index of the diamond
  pub n2 : f32,
  /// Refractive index delta difference for red and blue color relative to n2
  /// r = n2 + rainbow_delta
  /// b = n2 - rainbow_delta
  pub rainbow_delta : f32,
  /// How fast light is absorbed inside of the medium
  pub distance_attenuation_speed : f32
}

impl GemMaterial
{
  #[ must_use ]
  #[ inline ]
  pub fn new( _gl : &GL ) -> Self
  {
    Self
    {
      id : Uuid::new_v4(),
      ray_bounces : 5,
      color : gl::F32x3::from_array( [ 0.98, 0.95, 0.9 ] ),
      env_map_intensity : 1.0,
      radius : 1000.0,
      environment_texture : None,
      cube_normal_map_texture : CubeNormalData::default(),
      needs_update : true,
      n2 : 2.62,
      rainbow_delta : 0.02,
      distance_attenuation_speed : 0.1
    }
  }
}

impl Material for GemMaterial
{
  #[ inline ]
  fn get_id( &self ) -> Uuid
  {
    self.id
  }

  #[ inline ]
  fn needs_update( &self ) -> bool
  {
    self.needs_update
  }

  #[ inline ]
  fn make_shader_program( &self, gl : &gl::WebGl2RenderingContext, program : &gl::WebGlProgram ) -> Box< dyn ShaderProgram >
  {
    GemShader::new( gl, program ).dyn_clone()
  }

  #[ inline ]
  fn type_name( &self ) -> &'static str
  {
    stringify!( GemMaterial )
  }

  #[ inline ]
  fn get_vertex_shader( &self ) -> String
  {
    String::from_utf8_lossy( gem_vert::INPUT ).into()
  }

  #[ inline ]
  fn get_fragment_shader( &self ) -> String
  {
    String::from_utf8_lossy( gem_frag::INPUT ).into()
  }

  #[ inline ]
  fn configure
  (
    &self,
    gl : &gl::WebGl2RenderingContext,
    ctx : &MaterialUploadContext< '_ >
  )
  {
    let locations = ctx.locations;

    // Get uniform locations with error handling
    let v_loc = match get_uniform_location( locations, "v" ) 
    {
      Ok( loc ) => loc,
      Err( e ) => 
      {
        gl::log::error!( "GemMaterial::configure error: {}", e );
        return;
      }
    };

    let m_loc = match get_uniform_location( locations, "m" ) 
    {
      Ok( loc ) => loc,
      Err( e ) => 
      {
        gl::log::error!( "GemMaterial::configure error: {}", e );
        return;
      }
    };

    gl.uniform1i( Some( &v_loc ), 0 );
    gl.uniform1i( Some( &m_loc ), 1 );
  }

  #[ inline ]
  fn upload_on_state_change
  (
    &self,
    gl : &GL,
    ctx : &MaterialUploadContext< '_ >
  )
  -> Result< (), gl::WebglError >
  {
    let locations = ctx.locations;
    let upload = | loc, value : f32 | -> Result< (), gl::WebglError >
    {
      let uniform_loc = get_uniform_location( locations, loc )?;
      gl::uniform::upload( gl, Some( uniform_loc ), &value )?;
      Ok( () )
    };

    let upload_array = | loc, value : &[ f32 ] | -> Result< (), gl::WebglError >
    {
      let uniform_loc = get_uniform_location( locations, loc )?;
      gl::uniform::upload( gl, Some( uniform_loc ), value )?;
      Ok( () )
    };

    gl::uniform::upload( gl, Some( get_uniform_location( locations, "n" )? ), &self.ray_bounces )?;

    // Handle singular matrices (e.g., degenerate transforms with zero scale) with identity fallback
    let inv_world = ctx.node.get_world_matrix().inverse().unwrap_or_else( gl::math::mat4x4::identity );

    let mut bb = ctx.node.bounding_box();

    bb.apply_transform_mut( inv_world );
    let c = bb.center();
    
    upload( "s", self.env_map_intensity )?;
    upload( "w", self.cube_normal_map_texture.max_distance )?;
    upload( "a", self.n2 )?;
    upload( "c", self.rainbow_delta )?;
    upload( "e", self.distance_attenuation_speed )?;
    upload_array( "r", self.color.0.as_slice() )?;

    let rest_mat = gl::math::mat3x3h::translation( -c ) * inv_world;

    gl::uniform::matrix_upload( gl, Some( get_uniform_location(locations, "d" )? ), rest_mat.raw_slice(), true )?;
    // Handle singular matrix with identity fallback to prevent panic on degenerate transforms
    let rest_mat_inv = rest_mat.inverse().unwrap_or_else( gl::math::mat4x4::identity );
    gl::uniform::matrix_upload( gl, Some( get_uniform_location(locations, "l" )? ), rest_mat_inv.raw_slice(), true )?;

    self.upload_textures( gl );

    Ok( () )
  }

  #[ inline ]
  fn upload_textures( &self, gl : &GL )
  {
    if let Some( ref t ) = self.environment_texture { t.upload( gl ); }
    if let Some( ref t ) = self.cube_normal_map_texture.texture { t.upload( gl ); }
  }

  #[ inline ]
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
    bind( &self.cube_normal_map_texture.texture, 1 );
  }

  #[ inline ]
  fn dyn_clone( &self ) -> Box< dyn Material >
  {
    Box::new( self.clone() )
  }
}

impl Clone for GemMaterial
{
  #[ inline ]
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
      n2 : self.n2,
      rainbow_delta : self.rainbow_delta,
      distance_attenuation_speed : self.distance_attenuation_speed
    }
  }
}
