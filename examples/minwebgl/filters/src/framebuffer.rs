use minwebgl as gl;
use gl::GL;
use web_sys::
{
  WebGlFramebuffer,
  WebGlTexture,
};

pub struct Framebuffer
{
  framebuffer : WebGlFramebuffer,
  color_attachment : WebGlTexture,
}

impl Framebuffer
{
  pub fn new( gl : &GL, width : i32, height : i32 ) -> Option< Self >
  {
    let texture = gl.create_texture()?;
    gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) );
    gl.tex_storage_2d( GL::TEXTURE_2D, 1, gl::RGBA8, width, height );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );

    let framebuffer = gl.create_framebuffer()?;
    gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &framebuffer ) );
    gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some( &texture ), 0 );
    gl.bind_framebuffer( gl::FRAMEBUFFER, None );

    Some( Self { framebuffer, color_attachment : texture } )
  }

  pub fn color_attachment( &self ) -> &WebGlTexture
  {
    &self.color_attachment
  }

  pub fn framebuffer( &self ) -> &WebGlFramebuffer
  {
    &self.framebuffer
  }
}
