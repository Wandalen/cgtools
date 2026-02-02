//! Framebuffer creation and management

use minwebgl as gl;
use gl::GL;
use web_sys::WebGlTexture;
use crate::types::{ Framebuffers, GBuffer };

/// Create framebuffers for deferred rendering
pub fn create_framebuffers( gl : &GL, width : i32, height : i32 ) -> Framebuffers
{
  let gbuffer = create_gbuffer( gl, width, height );

  let offscreen_color = tex_storage_2d( gl, GL::RGBA8, width, height );
  let offscreen_buffer = gl.create_framebuffer();
  gl.bind_framebuffer( GL::FRAMEBUFFER, offscreen_buffer.as_ref() );
  gl.framebuffer_texture_2d
  (
    GL::FRAMEBUFFER,
    GL::COLOR_ATTACHMENT0,
    GL::TEXTURE_2D,
    offscreen_color.as_ref(),
    0
  );

  Framebuffers
  {
    gbuffer,
    offscreen_buffer,
    offscreen_color,
  }
}

/// Creates and configures the G-buffer framebuffer and its associated textures
/// (position, normal, color) and depth renderbuffer.
pub fn create_gbuffer( gl : &GL, width : i32, height : i32 ) -> GBuffer
{
  // Create textures for position, normal, and color
  // RGBA16F for position and normal to store floating-point data
  let position = tex_storage_2d( gl, GL::RGBA16F, width, height );
  let normal = tex_storage_2d( gl, GL::RGBA16F, width, height );
  // RGBA8 for color (standard 8-bit per channel)
  let color = tex_storage_2d( gl, GL::RGBA8, width, height );

  // Create a renderbuffer for depth
  let depth = gl.create_renderbuffer();
  gl.bind_renderbuffer( GL::RENDERBUFFER, depth.as_ref() );
  gl.renderbuffer_storage( GL::RENDERBUFFER, GL::DEPTH_COMPONENT24, width, height );

  // Create the framebuffer
  let framebuffer = gl.create_framebuffer();
  gl.bind_framebuffer( GL::FRAMEBUFFER, framebuffer.as_ref() );

  // Attach the textures and depth buffer to the framebuffer's attachment points
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, position.as_ref(), 0 );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT1, GL::TEXTURE_2D, normal.as_ref(), 0 );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT2, GL::TEXTURE_2D, color.as_ref(), 0 );
  gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::RENDERBUFFER, depth.as_ref() );

  GBuffer { framebuffer, position, normal, color, depth }
}

/// Helper function to create a 2D texture with specified format, width, and height,
/// and set its filtering to nearest.
pub fn tex_storage_2d( gl : &GL, format : u32, width : i32, height : i32 ) -> Option< WebGlTexture >
{
  let tex = gl.create_texture()?;
  gl.bind_texture( GL::TEXTURE_2D, Some( &tex ) );
  // Allocate texture storage
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, format, width, height );
  // Set texture filtering to nearest (important for G-buffer sampling)
  gl::texture::d2::filter_nearest( gl );
  Some( tex )
}
