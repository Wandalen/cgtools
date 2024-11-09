use minwebgl as gl;
use gl::GL;
use web_sys::
{
  WebGlFramebuffer,
  WebGlRenderbuffer,
  WebGlTexture,
  wasm_bindgen::prelude::*,
  js_sys,
};
use std::ops::Index;

// Maximum amount of ColorAttachments supported by WebGl2
// Actual amount may be reduced by hardware implementation
const MAX_COLOR_ATTACHMENTS : usize = 16;

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
  color_attachments : [ Option< Attachment >; MAX_COLOR_ATTACHMENTS ],
  depth_attachment : Option< ( DepthAttachment, Attachment ) >,
}

impl Framebuffer
{
  pub fn attachment( &self, index : ColorAttachment ) -> Option< &Attachment >
  {
    let index = index as usize - ColorAttachment::N0 as usize;
    match self.color_attachments.get( index ).as_ref()
    {
      Some( attachment ) => attachment.as_ref(),
      None => None,
    }
  }

  pub fn depth_attachment( &self ) -> Option< &Attachment >
  {
    self.depth_attachment.as_ref().map( | ( _, a ) | a )
  }

  /// Binds all of color attachments and framebuffer for drawing
  pub fn bind_draw( &self, gl : &GL )
  {
    // forms an array of attachments in their respective positions
    // for example if framebuffer has attachment0 and attachment2
    // then the array would be [ GL::ColorAttachment0, GL::NONE, GL::ColorAttachment2 ]
    // this is how WebGl wants it to be

    // find index of last non-None value
    let last = self.color_attachments.iter().rposition( | v | v.is_some() ).unwrap_or( 0 );
    // create iterator with actual values
    let iter = self.color_attachments[ ..=last ]
    .iter()
    .enumerate()
    .map
    (
      | ( i, a ) |
      if a.is_some()
      {
        GL::COLOR_ATTACHMENT0 + i as u32
      }
      else
      {
        GL::NONE
      }
    )
    .map( | v | JsValue::from_f64( v as f64 ) );

    gl.bind_framebuffer( GL::DRAW_FRAMEBUFFER, Some( &self.framebuffer ) );
    gl.draw_buffers( &js_sys::Array::from_iter( iter ) );
  }

  /// Binds specific color attachments and famebuffer for drawing.
  /// Returns `Error` if some of indices are not actualy attached to framebuffer
  pub fn bind_draw_nths( &self, indices : &[ ColorAttachment ], gl : &GL ) -> Result< (), String >
  {
    let mut drawbuffers = [ GL::NONE; MAX_COLOR_ATTACHMENTS ];
    for index in indices
    {
      let idx = *index as usize - ColorAttachment::N0 as usize;
      if self.color_attachments[ idx ].is_none()
      {
        return Err( format!( "Framebuffer does not has ColorAttachment{:?}", idx as u32 ) );
      }
      drawbuffers[ idx ] = *index as u32;
    }
    let last = drawbuffers.iter().rposition( | v | *v != GL::NONE ).unwrap_or( 0 );
    let iter = drawbuffers[ ..=last ].iter().map( | v | JsValue::from_f64( *v as f64 ) );

    gl.bind_framebuffer( GL::DRAW_FRAMEBUFFER, Some( &self.framebuffer ) );
    gl.draw_buffers( &js_sys::Array::from_iter( iter ) );
    Ok( () )
  }

  /// Binds specific color attachment and framebuffer for reading.
  /// Returns `Error` if index is not actualy attached to framebuffer
  pub fn bind_read( &self, index : ColorAttachment, gl : &GL ) -> Result< (), String >
  {
    let idx = index as usize - ColorAttachment::N0 as usize;
    if !self.color_attachments[ idx ].is_none()
    {
      return Err( format!( "Framebuffer does not has ColorAttachment{:?}", idx as u32 ) );
    }

    gl.bind_framebuffer( GL::READ_FRAMEBUFFER, Some( &self.framebuffer ) );
    gl.read_buffer( index as u32 );
    Ok( () )
  }

  /// Binds depth attachment and framebuffer for reading.
  /// Returns `Error` if there is no depth attachment
  pub fn bind_read_depth_attachment( &self, gl : &GL ) -> Result< (), String >
  {
    if self.depth_attachment.is_none()
    {
      return Err( format!( "Framebuffer does not has depthbuffer" ) );
    }

    gl.bind_framebuffer( GL::READ_FRAMEBUFFER, Some( &self.framebuffer ) );
    gl.read_buffer( self.depth_attachment.as_ref().unwrap().0 as u32 );
    Ok( () )
  }
}

impl Index< ColorAttachment > for Framebuffer
{
  type Output = Attachment;

  fn index( &self, index: ColorAttachment ) -> &Self::Output
  {
    let index = index as usize - ColorAttachment::N0 as usize;
    self.color_attachments[ index ]
    .as_ref()
    .expect( &format!( "Framebuffer does not has ColorAttachment{:?}", index as u32 ) )
  }
}

pub struct FramebufferBuilder
{
  attachments : [ Option< Attachment >; MAX_COLOR_ATTACHMENTS ],
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

  pub fn color_attachment( mut self, index : ColorAttachment, attachment : Attachment ) -> Self
  {
    let index = index as usize - ColorAttachment::N0 as usize;
    self.attachments[ index ] = Some( attachment );
    self
  }

  pub fn depth_attachment( mut self, r#type : DepthAttachment, attachment : Attachment ) -> Self
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

    for ( i, attachment ) in self.attachments.iter().enumerate()
    {
      let index = GL::COLOR_ATTACHMENT0 + i as u32;
      if let Some( attachment ) = attachment
      {
        match attachment
        {
          Attachment::Texture( texture ) =>
          {
            gl.framebuffer_texture_2d
            (
              GL::FRAMEBUFFER,
              index,
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
              index,
              GL::RENDERBUFFER,
              Some( renderbuffer ),
            );
          }
        }
      }
    }

    if let Some( ( r#type, attachment ) ) = &self.depthbuffer
    {
      match &attachment
      {
        Attachment::Texture( texture ) =>
        {
          gl.framebuffer_texture_2d
          (
            GL::FRAMEBUFFER,
            *r#type as u32,
            GL::TEXTURE_2D,
            Some( &texture ),
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
            Some( &renderbuffer ),
          );
        }
      }
    }

    let status = gl.check_framebuffer_status( GL::FRAMEBUFFER );
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );

    if status != GL::FRAMEBUFFER_COMPLETE
    {
      return Err( minwebgl::WebglError::FailedToAllocateResource( "Framebuffer is incomplete" ) )
    }

    Ok
    (
      Framebuffer
      {
        framebuffer,
        color_attachments : self.attachments,
        depth_attachment: self.depthbuffer,
      }
    )
  }
}
