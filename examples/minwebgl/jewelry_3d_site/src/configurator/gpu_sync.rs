use minwebgl as gl;
use gl::GL;
use web_sys::{ WebGlFramebuffer, WebGlTexture };

#[ non_exhaustive ]
pub struct GpuSync
{
  pub fbo: Option< WebGlFramebuffer >,
  pub tex : Option< WebGlTexture >,
  gl : GL,
}

impl GpuSync
{
  /// Creates a new [`GpuSync`] instance
  ///
  /// # Errors
  /// Returns an error if framebuffer creation fails or framebuffer is not complete
  #[ inline ]
  pub fn new( gl : &GL ) -> Result< Self, gl::WebglError >
  {
    let tex = gl.create_texture()
    .ok_or( gl::WebglError::FailedToAllocateResource( "GpuSync texture" ) )?;
    gl.bind_texture( gl::TEXTURE_2D, Some( &tex ) );
    gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::RGBA8, 1, 1 );
    gl::texture::d2::filter_nearest( &gl );

    let fbo = gl.create_framebuffer()
    .ok_or( gl::WebglError::FailedToAllocateResource( "GpuSync framebuffer" ) )?;
    gl.bind_framebuffer( gl::FRAMEBUFFER, Some( &fbo ) );
    gl.framebuffer_texture_2d
    (
      gl::FRAMEBUFFER,
      gl::COLOR_ATTACHMENT0,
      gl::TEXTURE_2D,
      Some( &tex ),
      0,
    );

    let status = gl.check_framebuffer_status( gl::FRAMEBUFFER );
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );

    if status != gl::FRAMEBUFFER_COMPLETE
    {
      return Err( gl::WebglError::Other( "GpuSync framebuffer is not complete" ) );
    }

    Ok( Self { fbo : Some( fbo ), tex : Some( tex ), gl : gl.clone(), } )
  }

  #[ inline ]
  pub fn sync( &self )
  {
    let mut pixel = [ 1u8; 4 ];

    self.gl.bind_framebuffer( GL::READ_FRAMEBUFFER, self.fbo.as_ref() );
    if let Err( e ) = self.gl.read_pixels_with_opt_u8_array
    (
      0,
      0,
      1,
      1,
      gl::RGBA,
      gl::UNSIGNED_BYTE,
      Some( &mut pixel ),
    )
    {
      gl::log::error!( "GPU sync failed to read pixels: {:?}", e );
    }
  }
}

impl Drop for GpuSync
{
  #[ inline ]
  fn drop( &mut self )
  {
    self.gl.delete_framebuffer( self.fbo.as_ref() );
    self.gl.delete_texture( self.tex.as_ref() );
  }
}
