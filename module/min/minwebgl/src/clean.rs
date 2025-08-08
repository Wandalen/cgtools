/// Internal namespace.
mod private
{
  use crate::*;
  type GL = WebGl2RenderingContext;

  /// Unbind the currently bound 2D texture.
  pub fn texture_2d( gl : &GL )
  {
    gl.bind_texture( GL::TEXTURE_2D, None );
  }

  /// Unbind the 2D texture from a specific texture unit.
  pub fn texture_2d_active( gl : &GL, active : u32 )
  {
    gl.active_texture( GL::TEXTURE0 + active );
    texture_2d( gl );
  }

  /// Unbind 2D textures from multiple texture units.
  pub fn texture_2d_array< T, E >( gl : &GL, active : T )
  where 
    T : IntoIterator,
    E : std::fmt::Debug,
    T::Item : TryInto< u32, Error = E >
  {
    for i in active
    {
      texture_2d_active( gl, i.try_into().expect( "Active id is out of range" ) );
    }
  }

  /// Unbind the currently bound framebuffer.
  pub fn framebuffer( gl : &GL )
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );
  }

  /// Detach a 2D texture from a specific framebuffer attachment.
  pub fn framebuffer_texture_2d_attachment( gl : &GL, attachment : u32 )
  {
    gl.framebuffer_texture_2d
    (
      GL::FRAMEBUFFER, 
      GL::COLOR_ATTACHMENT0 + attachment, 
      GL::TEXTURE_2D, 
      None, 
      0
    );
  } 

  /// Detach the 2D texture from framebuffer attachment 0.
  pub fn framebuffer_texture_2d( gl : &GL )
  {
    framebuffer_texture_2d_attachment( gl, 0 );
  } 

  /// Detach 2D textures from multiple framebuffer attachments.
  pub fn framebuffer_texture_2d_array< T, E >( gl : &GL, attachments : T )
  where 
    T : IntoIterator,
    E : std::fmt::Debug,
    T::Item : TryInto< u32, Error = E >
  {
    for i in attachments
    {
      framebuffer_texture_2d_attachment( gl, i.try_into().expect( "Attachment id is out of range" ) );
    }
  } 

  /// Detach a renderbuffer from a specific framebuffer attachment.
  pub fn framebuffer_renderbuffer_attachment( gl : &GL, attachment : u32 )
  {
    gl.framebuffer_texture_2d
    (
      GL::FRAMEBUFFER, 
      GL::COLOR_ATTACHMENT0 + attachment, 
      GL::RENDERBUFFER, 
      None, 
      0
    );
  } 

  /// Detach the renderbuffer from framebuffer attachment 0.
  pub fn framebuffer_renderbuffer( gl : &GL )
  {
    framebuffer_renderbuffer_attachment( gl, 0 );
  } 

  /// Detach renderbuffers from multiple framebuffer attachments.
  pub fn framebuffer_renderbuffer_array< T, E >( gl : &GL, attachments : T )
  where 
    T : IntoIterator,
    E : std::fmt::Debug,
    T::Item : TryInto< u32, Error = E >
  {
    for i in attachments
    {
      framebuffer_renderbuffer_attachment( gl, i.try_into().expect( "Attachment id is out of range" ) );
    }
  }

}

crate::mod_interface!
{
  own use
  {
    framebuffer,
    framebuffer_renderbuffer,
    framebuffer_renderbuffer_array,
    framebuffer_renderbuffer_attachment,
    framebuffer_texture_2d,
    framebuffer_texture_2d_array,
    framebuffer_texture_2d_attachment,
    texture_2d,
    texture_2d_array,
    texture_2d_active
  };
}
