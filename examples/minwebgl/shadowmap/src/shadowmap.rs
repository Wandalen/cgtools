use minwebgl as gl;
use gl::{ GL, Program };
use web_sys::{ WebGlFramebuffer, WebGlTexture };

pub struct Shadowmap
{
  framebuffer : Option< WebGlFramebuffer >,
  depth_texture : Option< WebGlTexture >,
  program : Program,
}

impl Shadowmap
{
  pub fn new( gl : &GL, resolution : u32 ) -> Result< Self, gl::WebglError >
  {
    let depth_texture = gl.create_texture();
    gl.bind_texture( gl::TEXTURE_2D, depth_texture.as_ref() );
    gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::DEPTH_COMPONENT24, resolution as i32, resolution as i32 );
    gl::texture::d2::wrap_clamp( gl );

    // Depth textures in WebGL 2.0 only support NEAREST filtering
    // Softness comes from PCSS multiple samples, not texture filtering
    gl::texture::d2::filter_nearest( gl );

    let framebuffer = gl.create_framebuffer();
    gl.bind_framebuffer( gl::FRAMEBUFFER, framebuffer.as_ref() );
    gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, depth_texture.as_ref(), 0 );
    gl.bind_framebuffer( gl::FRAMEBUFFER, None );

    let vertex = include_str!( "shaders/shadowmap.vert" );
    let fragment = include_str!( "shaders/shadowmap.frag" );
    let program = gl::Program::new( gl.clone(), vertex, fragment )?;

    Ok
    (
      Self
      {
        framebuffer,
        depth_texture,
        program,
      }
    )
  }

  pub fn bind( &self, gl : &GL )
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
    self.program.activate();
  }

  pub fn upload_mvp( &self, mvp : gl::F32x4x4 )
  {
    self.program.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
  }

  pub fn depth_texture( &self ) -> Option< &WebGlTexture >
  {
    self.depth_texture.as_ref()
  }
}

pub struct LightSource
{
  position : gl::F32x3,
  orientation : gl::QuatF32,
  projection : gl::F32x4x4,
  mvp : Option< gl::F32x4x4 >,
}

impl LightSource
{
  pub fn new
  (
    position : gl::F32x3,
    orientation : gl::QuatF32,
    projection : gl::F32x4x4,
  ) -> Self
  {
    Self { position, orientation, projection, mvp : None }
  }

  pub fn position( &self ) -> gl::F32x3
  {
    self.position
  }

  pub fn orientation( &self ) -> gl::QuatF32
  {
    self.orientation
  }

  pub fn projection( &self ) -> gl::F32x4x4
  {
    self.projection
  }

  pub fn set_position( &mut self, position : gl::F32x3 )
  {
    self.position = position;
    self.mvp = None;
  }

  pub fn set_orientation( &mut self, orientation : gl::QuatF32 )
  {
    self.orientation = orientation;
    self.mvp = None;
  }

  pub fn set_projection( &mut self, projection : gl::F32x4x4 )
  {
    self.projection = projection;
    self.mvp = None;
  }

  pub fn view_projection( &mut self ) -> gl::F32x4x4
  {
    if let Some( mvp ) = self.mvp
    {
      mvp
    }
    else
    {
      let view = self.orientation.invert().to_matrix().to_homogenous() * gl::math::mat3x3h::translation( -self.position );
      let view_projection = self.projection * view;
      self.mvp = Some( view_projection );

      view_projection
    }
  }
}
