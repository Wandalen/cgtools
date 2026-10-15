use minwebgl as gl;
use gl::GL;
use web_sys::
{
  WebGlFramebuffer,
  WebGlTexture,
};

pub struct Framebuffer
{
  gl : GL,
  handle : WebGlFramebuffer,
  color_attachment : WebGlTexture,
  width : i32,
  height : i32,
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

    Some( Self { gl : gl.clone(), handle : framebuffer, color_attachment : texture, width, height } )
  }

  pub fn width( &self ) -> i32
  {
    self.width
  }

  pub fn height( &self ) -> i32
  {
    self.height
  }

  pub fn color_attachment( &self ) -> &WebGlTexture
  {
    &self.color_attachment
  }

  pub fn framebuffer( &self ) -> &WebGlFramebuffer
  {
    &self.handle
  }
}

// Fix(BUG-463): free the GPU-side framebuffer + color-attachment texture when a
// `Framebuffer` is dropped, instead of only dropping the Rust-side JS handles.
// Root cause: `Renderer::framebuffer_size_update` replaces `self.framebuffer`
// with a brand-new `Framebuffer` on every resize ( including the resize-sync
// check that runs after every filter apply ), and this type never called
// `gl.delete_framebuffer`/`gl.delete_texture` on the value being replaced --
// dropping a `WebGlFramebuffer`/`WebGlTexture` handle only releases the Rust/JS
// reference wrapper, it does not free the underlying GPU resource.
// Pitfall: a WebGL object handle needs an explicit `gl.delete_*` call to free
// its GPU resource -- letting Rust's own `Drop` glue run on the handle wrapper
// alone is not enough, so any type that owns one should implement `Drop` itself
// ( matching this workspace's own `renderer::webgl::shadow::ShadowMap` pattern ).
impl Drop for Framebuffer
{
  fn drop( &mut self )
  {
    self.gl.delete_framebuffer( Some( &self.handle ) );
    self.gl.delete_texture( Some( &self.color_attachment ) );
  }
}
