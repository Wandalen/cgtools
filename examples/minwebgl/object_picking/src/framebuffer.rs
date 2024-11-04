use minwebgl as gl;
use gl::{ GL, WebglError };
use std::collections::HashMap;
use web_sys::
{
  js_sys,
  wasm_bindgen::JsValue,
  WebGlFramebuffer,
  WebGlRenderbuffer,
  WebGlTexture
};

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

#[ repr( u32 ) ]
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum DepthAttachment
{
  Depth = GL::DEPTH_ATTACHMENT,
  Stencil = GL::STENCIL_ATTACHMENT,
  DepthStencil = GL::DEPTH_STENCIL_ATTACHMENT,
}

pub struct Framebuffer
{
  framebuffer : WebGlFramebuffer,
  attachments : HashMap< u32, Attachment >,
  depthbuffer : Option< Attachment >,
}

impl Framebuffer
{
  pub fn new( gl : &GL ) -> Result< Self, WebglError >
  {
    let framebuffer = match gl.create_framebuffer()
    {
        Some( f ) => f,
        None => return Err( WebglError::FailedToAllocateResource( "Can't create framebuffer" ) ),
    };
    Ok( Self { framebuffer, attachments: Default::default(), depthbuffer: None } )
  }

  /// use `GL::COLOR_ATTACHMENT#` as index
  pub fn get_attachment( &self, index : u32 ) -> &Attachment
  {
    &self.attachments[ &index ]
  }

  pub fn get_depthbuffer( &self ) -> Option< &Attachment >
  {
    self.depthbuffer.as_ref()
  }

  pub fn bind( &self, gl : &GL )
  {
    let mut attachments = self.attachments.keys().collect::< Box< _ > >();
    attachments.sort();

    let mut drawbuffers = vec![];
    let mut start = GL::COLOR_ATTACHMENT0;
    for attachment in attachments
    {
      drawbuffers.extend( ( start..*attachment ).map( | _ | GL::NONE ) );
      drawbuffers.push( *attachment );
      start = *attachment + 1;
    }
    let iter = drawbuffers.iter().map( | item | JsValue::from_f64( *item as f64 ) );

    gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &self.framebuffer ) );
    gl.draw_buffers( &js_sys::Array::from_iter( iter ).into() );
  }

  /// use `GL::COLOR_ATTACHMENT#` as index
  pub fn attach( &mut self, attachment : Attachment, index : u32, gl : &GL )
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &self.framebuffer ) );

    match &attachment
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

    gl.bind_framebuffer( GL::FRAMEBUFFER, None );
    _ = self.attachments.insert( index, attachment );
  }

  pub fn set_depthbuffer( &mut self, attachment : Attachment, r#type : DepthAttachment, gl : &GL )
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &self.framebuffer ) );

    match &attachment
    {
      Attachment::Texture( texture ) =>
      {
        gl.framebuffer_texture_2d
        (
          GL::FRAMEBUFFER,
          r#type as u32,
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
          r#type as u32,
          GL::RENDERBUFFER,
          Some( renderbuffer ),
        );
      }
    }

    gl.bind_framebuffer( GL::FRAMEBUFFER, None );
    self.depthbuffer = Some( attachment );
  }
}
