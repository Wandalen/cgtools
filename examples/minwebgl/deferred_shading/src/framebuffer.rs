use std::collections::HashMap;
use minwebgl as gl;
use gl::GL;
use web_sys::{ WebGlFramebuffer, WebGlRenderbuffer, WebGlTexture };

pub enum Attachment
{
  Texture( WebGlTexture ),
  Renderbuffer( WebGlRenderbuffer ),
}

impl Attachment
{
  pub fn unwrap_texture( &self ) -> &WebGlTexture
  {
    match self
    {
      Attachment::Texture( tex ) => tex,
      Attachment::Renderbuffer( _ ) => panic!( "Attachment is not a texture" ),
    }
  }

  pub fn unwrap_renderbuffer( &self ) -> &WebGlRenderbuffer
  {
    match self
    {
      Attachment::Texture( _ ) => panic!( "Attachment is not a renderbuffer" ),
      Attachment::Renderbuffer( rbuf ) => rbuf,
    }
  }
}

pub struct Framebuffer
{
  framebuffer : WebGlFramebuffer,
  attachments : HashMap< ColorAttachment, Attachment >,
  depthbuffer : Option< Attachment >,
}

impl Framebuffer
{
  pub fn bind()
  {

  }
}

pub struct FramebufferBuilder
{
  attachments : HashMap< ColorAttachment, Attachment >,
  depthbuffer : Option< ( DepthAttachment, Attachment ) >,
}

impl FramebufferBuilder
{
  pub fn new() -> Self
  {
    Self
    {
      attachments: Default::default(),
      depthbuffer: None,
    }
  }

  pub fn attach( mut self, index : ColorAttachment, attachment : Attachment ) -> Self
  {
    _ = self.attachments.insert( index, attachment );
    self
  }

  pub fn depthbuffer( mut self, r#type : DepthAttachment, attachment : Attachment ) -> Self
  {
    self.depthbuffer = Some( ( r#type, attachment ) );
    self
  }

  pub fn build( self, gl : &GL ) -> Result< Framebuffer, gl::WebglError >
  {
    let framebuffer = match gl.create_framebuffer()
    {
      Some( f ) => f,
      None => return Err( minwebgl::WebglError::FailedToAllocateResource( "Failed to create a framebuffer" ) ),
    };
    gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &framebuffer ) );

    for ( index, attachment ) in self.attachments.iter()
    {
      match attachment
      {
        Attachment::Texture( texture ) =>
        {
          gl.framebuffer_texture_2d
          (
            GL::FRAMEBUFFER,
            *index as u32,
            GL::TEXTURE_2D,
            Some( texture ),
            0
          );
        }
        Attachment::Renderbuffer( renderbuffer ) =>
        {
          gl.framebuffer_renderbuffer
          (
            GL::FRAMEBUFFER,
            *index as u32,
            GL::RENDERBUFFER,
            Some( renderbuffer ),
          );
        }
      }
    }

    if let Some( ( r#type, attachment ) ) = &self.depthbuffer
    {
      match attachment
      {
        Attachment::Texture( texture ) =>
        {
          gl.framebuffer_texture_2d
          (
            GL::FRAMEBUFFER,
            *r#type as u32,
            GL::TEXTURE_2D,
            Some( texture ),
            0
          );
        }
        Attachment::Renderbuffer( renderbuffer ) =>
        {
          gl.framebuffer_renderbuffer
          (
            GL::FRAMEBUFFER,
            *r#type as u32,
            GL::RENDERBUFFER,
            Some( renderbuffer ),
          );
        }
      }
    }

    let status = gl.check_framebuffer_status( GL::FRAMEBUFFER );
    if status != GL::FRAMEBUFFER_COMPLETE
    {
      return Err( minwebgl::WebglError::FailedToAllocateResource( "Framebuffer is incomplete" ) )
    }

    gl.bind_framebuffer( GL::FRAMEBUFFER, None );

    Ok
    (
      Framebuffer
      {
        framebuffer,
        attachments: self.attachments,
        depthbuffer: self.depthbuffer.map( | ( _, a ) | a ),
      }
    )
  }
}

#[ repr( u32 ) ]
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash ) ]
pub enum ColorAttachment
{
  N0 = GL::COLOR_ATTACHMENT0,
  N1,
  N2,
  N3,
  N4,
  N5,
  N6,
  N7,
  N8,
  N9,
  N10,
  N11,
  N12,
  N13,
  N14,
  N15,
}

#[ repr( u32 ) ]
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum DepthAttachment
{
  Depth = GL::DEPTH_ATTACHMENT,
  Stencil = GL::STENCIL_ATTACHMENT,
  DepthStencil = GL::DEPTH_STENCIL_ATTACHMENT,
}
