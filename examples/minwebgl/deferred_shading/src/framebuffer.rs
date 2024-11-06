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
use std::collections::HashMap;

#[ repr( u32 ) ]
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord ) ]
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
  textures : [ Option< WebGlTexture >; 16 ],
  renderbuffers : [ Option< WebGlRenderbuffer >; 16 ],
  attachments : HashMap< ColorAttachment, Attachment >,
  depthbuffer : Option< ( DepthAttachment, Attachment ) >,
}

impl Framebuffer
{
  pub fn t_attachment( &self, index : ColorAttachment ) -> Option< &WebGlTexture >
  {
    let index = index as usize - ColorAttachment::N0 as usize;
    self.textures[ index ].as_ref()
  }

  pub fn r_attachment( &self, index : ColorAttachment ) -> Option< &WebGlRenderbuffer >
  {
    let index = index as usize - ColorAttachment::N0 as usize;
    self.renderbuffers[ index ].as_ref()
  }

  pub fn attachment( &self, index : ColorAttachment ) -> Option< &Attachment >
  {
    self.attachments.get( &index )
  }

  pub fn depthbuffer( &self ) -> Option< &Attachment >
  {
    self.depthbuffer.as_ref().map( | ( _, a ) | a )
  }

  /// binds all of attachments for drawing
  pub fn bind_draw( &self, gl : &GL )
  {
    // forms an array of attachments in their respective positions
    // for example if framebuffer has attachment0 and attachment2
    // then the array would be [ GL::ColorAttachment0, GL::NONE, GL::ColorAttachment2 ]
    // this is how WebGl wants it to be
    let mut attachments = self.attachments.keys().map( | a | *a as u32 ).collect::< Box< _ > >();
    attachments.sort();

    let mut drawbuffers = vec![];
    let mut start = GL::COLOR_ATTACHMENT0;
    for attachment in attachments
    {
      drawbuffers.extend( ( start..attachment ).map( | _ | GL::NONE ) );
      drawbuffers.push( attachment );
      start = attachment + 1;
    }
    gl::info!( "{drawbuffers:?}" );
    let iter = drawbuffers.into_iter().map( | v | JsValue::from_f64( v as f64 ) );

    let iter = self.textures.iter()
    .zip( self.renderbuffers.iter() )
    .enumerate()
    .map( | ( i, ( t, r ) ) | if t.is_some() || r.is_some() { GL::COLOR_ATTACHMENT0 + i as u32 } else { GL::NONE } )
    .map( | v | JsValue::from_f64( v as f64 ) );

    gl.bind_framebuffer( GL::DRAW_FRAMEBUFFER, Some( &self.framebuffer ) );
    gl.draw_buffers( &js_sys::Array::from_iter( iter ).into() );
  }

  /// binds specific attachments for drawing
  pub fn bind_draw_nth( &self, index : ColorAttachment, gl : &GL ) -> Result< (), String >
  {
    if !self.attachments.contains_key( &index )
    {
      return Err( format!( "Framebuffer does not has {:?}", index as u32 ) );
    }

    let iter = ( 0..( index as u32 ) )
    .map( | _ | GL::NONE )
    .chain( [ index as u32 ].into_iter() )
    .map( | v | JsValue::from_f64( v as f64 ) );

    gl.bind_framebuffer( GL::DRAW_FRAMEBUFFER, Some( &self.framebuffer ) );
    gl.draw_buffers( &js_sys::Array::from_iter( iter ).into() );
    Ok( () )
  }

  /// binds specific attachment for reading
  pub fn bind_read( &self, index : ColorAttachment, gl : &GL ) -> Result< (), String >
  {
    if !self.attachments.contains_key( &index )
    {
      return Err( format!( "Framebuffer does not has {:?}", index as u32 ) );
    }

    gl.bind_framebuffer( GL::READ_FRAMEBUFFER, Some( &self.framebuffer ) );
    gl.read_buffer( index as u32 );
    Ok( () )
  }

  /// binds depth buffer for reading
  pub fn bind_read_depth( &self, gl : &GL ) -> Result< (), String >
  {
    if self.depthbuffer.is_none()
    {
      return Err( format!( "Framebuffer does not has depthbuffer" ) );
    }
    gl.bind_framebuffer( GL::READ_FRAMEBUFFER, Some( &self.framebuffer ) );
    gl.read_buffer( self.depthbuffer.as_ref().unwrap().0 as u32 );
    Ok( () )
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

  pub fn attachment( mut self, index : ColorAttachment, attachment : Attachment ) -> Self
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

    let mut textures = [ const { None }; 16 ];
    let mut renderbuffers = [ const { None }; 16 ];

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
          let index = *index as usize - ColorAttachment::N0 as usize;
          textures[ index ] = Some( texture.clone() );
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
          let index = *index as usize - ColorAttachment::N0 as usize;
          renderbuffers[ index ] = Some( renderbuffer.clone() );
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
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );

    if status != GL::FRAMEBUFFER_COMPLETE
    {
      return Err( minwebgl::WebglError::FailedToAllocateResource( "Framebuffer is incomplete" ) )
    }

    Ok
    (
      Framebuffer
      {
        textures,
        renderbuffers,
        framebuffer,
        attachments: self.attachments,
        depthbuffer: self.depthbuffer,
      }
    )
  }
}
