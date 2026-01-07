use renderer::webgl::{ ShaderProgram, material::*, program::ProgramInfo, NodeContext };
use renderer::impl_locations;
use minwebgl as gl;
use gl::{ GL, F32x3, Former, WebGlProgram };
use rustc_hash::FxHashMap;
use uuid::Uuid;

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

/// The source code for the surface vertex shader.
const SURFACE_VERTEX_SHADER : &'static str = include_str!( "../shaders/surface.vert" );
/// The source code for the surface fragment shader.
const SURFACE_FRAGMENT_SHADER : &'static str = include_str!( "../shaders/surface.frag" );

/// Represents the visual properties of a surface.
#[ derive( Former, Debug ) ]
pub struct SurfaceMaterial
{
  /// A unique identifier for the material.
  pub id : Uuid,
  /// Shader program info
  program : SurfaceShader,
  /// Surface RGB color
  pub color : F32x3,
  /// Surface texture
  pub texture : Option< TextureInfo >,
  /// Signal for updating material uniforms
  pub needs_update : bool
}

impl SurfaceMaterial
{
  pub fn new( gl : &GL ) -> Self
  {
    // Compile and link a new WebGL program from the vertex and fragment shaders with the appropriate defines.
    let program = gl::ProgramFromSources::new
    (
      &format!( "#version 300 es\n{}", SURFACE_VERTEX_SHADER ),
      &format!( "#version 300 es\n{}", SURFACE_FRAGMENT_SHADER )
    ).compile_and_link( gl )
    .unwrap();

    Self
    {
      id : Uuid::new_v4(),
      program : SurfaceShader::new( gl, &program ),
      color : F32x3::from_array( [ 1.0, 1.0, 1.0 ] ),
      texture : None,
      needs_update : true
    }
  }
}

impl Material for SurfaceMaterial
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
    stringify!( SurfaceMaterial )
  }

  fn get_vertex_shader( &self ) -> String
  {
    SURFACE_VERTEX_SHADER.into()
  }

  fn get_fragment_shader( &self ) -> String
  {
    SURFACE_FRAGMENT_SHADER.into()
  }

  fn configure
  (
    &self,
    gl : &gl::WebGl2RenderingContext,
    _ibl_base_location : u32,
  )
  {
    self.program.bind( gl );
    let locations = self.program.locations();
    gl.uniform1i( locations.get( "surfaceTexture" ).unwrap().clone().as_ref(), 0 );
  }

  fn upload
  (
    &self,
    gl : &GL,
    _node_context : &NodeContext
  )
  -> Result< (), gl::WebglError >
  {
    self.program.bind( gl );
    let locations = self.program.locations();
    gl::uniform::upload( gl, locations.get( "surfaceColor" ).unwrap().clone(), self.color.0.as_slice() )?;
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
      // gl.enable( gl::CULL_FACE );
      // gl.cull_face( gl::BACK );
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
      needs_update : self.needs_update,
      program : self.program.clone()
    }
  }
}
