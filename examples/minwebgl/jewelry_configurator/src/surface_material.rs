use renderer::webgl::{ ShaderProgram, material::{ Material, TextureInfo, CullMode }, program::ProgramInfo, MaterialUploadContext };
use renderer::impl_locations;
use minwebgl as gl;
use gl::{ GL, F32x3, WebGlProgram };
use rustc_hash::FxHashMap;
use uuid::Uuid;
use crate::helpers::get_uniform_location;

// Surface shader locations
impl_locations!
(
  SurfaceShader,
  "worldMatrix",
  "viewMatrix",
  "projectionMatrix",
  "normalMatrix",
  "surfaceColor",
  "surfaceTexture"
);

/// Represents the visual properties of a ground surface.
#[ derive( Debug ) ]
pub struct SurfaceMaterial
{
  /// A unique identifier for the material.
  pub id : Uuid,
  /// Surface RGB color
  pub color : F32x3,
  /// Surface texture (shadow map)
  pub texture : Option< TextureInfo >,
  /// Signal for updating material uniforms
  pub needs_update : bool
}

impl SurfaceMaterial
{
  pub fn new( _gl : &GL ) -> Self
  {
    Self
    {
      id : Uuid::new_v4(),
      color : F32x3::from_array( [ 1.0, 1.0, 1.0 ] ),
      texture : None,
      needs_update : true
    }
  }
}

impl Material for SurfaceMaterial
{
  fn get_cull_mode( &self ) -> Option< CullMode >
  {
    Some( CullMode::Back )
  }

  fn get_id( &self ) -> Uuid
  {
    self.id
  }

  fn needs_update( &self ) -> bool
  {
    self.needs_update
  }

  fn make_shader_program( &self, gl : &gl::WebGl2RenderingContext, program : &gl::WebGlProgram ) -> Box< dyn ShaderProgram >
  {
    SurfaceShader::new( gl, program ).dyn_clone()
  }

  fn type_name( &self ) -> &'static str
  {
    stringify!( SurfaceMaterial )
  }

  fn get_vertex_shader( &self ) -> String
  {
    include_str!( "../shaders/surface.vert" ).into()
  }

  fn get_fragment_shader( &self ) -> String
  {
    include_str!( "../shaders/surface.frag" ).into()
  }

  fn configure
  (
    &self,
    gl : &gl::WebGl2RenderingContext,
    ctx : &MaterialUploadContext< '_ >
  )
  {
    let locations = ctx.locations;
    let loc = match get_uniform_location( locations, "surfaceTexture" )
    {
      Ok( loc ) => loc,
      Err( e ) =>
      {
        gl::log::error!( "SurfaceMaterial::configure error: {e}" );
        return;
      }
    };
    gl.uniform1i( Some( &loc ), 0 );
  }

  fn upload_on_state_change
  (
    &self,
    gl : &GL,
    ctx : &MaterialUploadContext< '_ >
  )
  -> Result< (), gl::WebglError >
  {
    let locations = ctx.locations;
    let loc = get_uniform_location( locations, "surfaceColor" )?;
    gl::uniform::upload( gl, Some( loc ), self.color.0.as_slice() )?;
    self.upload_textures( gl );
    Ok( () )
  }

  fn upload_textures( &self, gl : &GL )
  {
    gl.active_texture( gl::TEXTURE0 );
    if let Some( ref t ) = self.texture
    {
      t.upload( gl );
    }
  }

  fn bind( &self, gl : &GL )
  {
    if let Some( ref t ) = self.texture
    {
      gl.active_texture( gl::TEXTURE0 );
      t.bind( gl );
    }
  }

  fn dyn_clone( &self ) -> Box< dyn Material >
  {
    Box::new( self.clone() )
  }
}

impl Clone for SurfaceMaterial
{
  fn clone( &self ) -> Self
  {
    SurfaceMaterial
    {
      id : Uuid::new_v4(),
      color : self.color,
      texture : self.texture.clone(),
      needs_update : self.needs_update
    }
  }
}
