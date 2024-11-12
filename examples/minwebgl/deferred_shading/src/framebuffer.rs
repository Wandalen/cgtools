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

// Maximum amount of ColorAttachments supported by WebGl2
// Actual amount may be reduced by hardware implementation
const MAX_COLOR_ATTACHMENTS : usize = 16;

pub enum Attachment
{
  Texture( WebGlTexture ),
  Renderbuffer( WebGlRenderbuffer ),
}

#[ repr( u32 ) ]
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord ) ]
pub enum AttachmentType
{
  Color( Color ),
  Depth,
  Stencil,
  DepthStencil,
}

impl AttachmentType
{
  pub fn as_u32( &self ) -> u32
  {
    match self
    {
      AttachmentType::Color( color ) => *color as u32,
      AttachmentType::Depth => GL::DEPTH_ATTACHMENT,
      AttachmentType::Stencil => GL::STENCIL_ATTACHMENT,
      AttachmentType::DepthStencil => GL::DEPTH_STENCIL_ATTACHMENT,
    }
  }
}

#[ repr( u32 ) ]
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord ) ]
pub enum Color
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

pub struct Framebuffer
{
  // bit mask of current color attachments
  attachments : u16,
  framebuffer : WebGlFramebuffer,
}

impl Framebuffer
{
  pub fn new( gl : &GL ) -> Result< Self, gl::WebglError >
  {
    let framebuffer = gl.create_framebuffer();
    match framebuffer
    {
      Some( framebuffer ) => Ok
      (
        Self
        {
          attachments : 0,
          framebuffer,
        }
      ),
      None => Err( gl::WebglError::FailedToAllocateResource( "Failed to allocate framebuffer" ) ),
    }
  }

  pub fn texture_2d( &mut self, attachment : AttachmentType, texture : Option< &WebGlTexture >, level : i32, gl : &GL )
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &self.framebuffer ) );
    gl.framebuffer_texture_2d( GL::FRAMEBUFFER, attachment.as_u32(), GL::TEXTURE_2D, texture, level );
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );

    self.update_bitmask( attachment, texture.is_some() );
  }

  pub fn texture_layer
  (
    &mut self,
    attachment : AttachmentType,
    texture : Option< &WebGlTexture >,
    level : i32,
    layer : i32,
    gl : &GL
  )
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &self.framebuffer ) );
    gl.framebuffer_texture_layer( GL::FRAMEBUFFER, attachment.as_u32(), texture, level, layer );
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );

    self.update_bitmask( attachment, texture.is_some() );
  }

  pub fn renderbuffer( &mut self, attachment : AttachmentType, renderbuffer : Option< &WebGlRenderbuffer >, gl : &GL )
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &self.framebuffer ) );
    gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, attachment.as_u32(), GL::RENDERBUFFER, renderbuffer );
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );

    self.update_bitmask( attachment, renderbuffer.is_some() );
  }

  fn update_bitmask( &mut self, attachment : AttachmentType, value : bool )
  {
    if let AttachmentType::Color( color ) = attachment
    {
      let shift = color as u32 - Color::N0 as u32;
      if value
      {
        self.attachments |= 1 << shift
      }
      else
      {
        self.attachments &= !( 1 << shift )
      }
    }
  }

  pub fn bind_all_drawbuffers( &self, gl : &GL )
  {
    // forms an array of attachments in their respective positions
    // for example if framebuffer has attachment0 and attachment2
    // then the array would be [ GL::ColorAttachment0, GL::NONE, GL::ColorAttachment2 ]
    // this is how WebGl wants it to be

    let mut drawbuffers = [ GL::NONE; MAX_COLOR_ATTACHMENTS ];
    for ( i, item ) in drawbuffers.iter_mut().enumerate()
    {
      let attachment = GL::COLOR_ATTACHMENT0 + i as u32;
      let bit = ( self.attachments & ( 1 << i ) ) as u32;
      if bit != 0
      {
        *item = attachment;
      }
    }

    let last = drawbuffers.iter().rposition( | v | *v != GL::NONE ).map_or( 0, | pos | pos + 1 );
    let iter = drawbuffers[ .. last ].iter().map( | v | JsValue::from_f64( *v as f64 ) );

    gl.bind_framebuffer( GL::DRAW_FRAMEBUFFER, Some( &self.framebuffer ) );
    gl.draw_buffers( &js_sys::Array::from_iter( iter ) );
  }

  /// Binds specific color attachments and famebuffer for drawing.
  /// Returns `Error` if some of indices are not actualy attached to framebuffer
  pub fn bind_drawbuffers( &self, attachments : &[ Color ], gl : &GL ) -> Result< (), String >
  {
    let mut drawbuffers = [ GL::NONE; MAX_COLOR_ATTACHMENTS ];
    for attachment in attachments
    {
      let index = *attachment as usize - Color::N0 as usize;
      let bit = ( self.attachments & ( 1 << index ) ) as u32;

      if bit == 0
      {
        return Err( format!( "Framebuffer does not has ColorAttachment{:?}", index as u32 ) );
      }

      drawbuffers[ index ] = *attachment as u32;
    }

    let last = drawbuffers.iter().rposition( | v | *v != GL::NONE ).map_or( 0, | pos | pos + 1 );
    let iter = drawbuffers[ .. last ].iter().map( | v | JsValue::from_f64( *v as f64 ) );
    gl::info!( "{iter:?}" );
    gl.bind_framebuffer( GL::DRAW_FRAMEBUFFER, Some( &self.framebuffer ) );
    gl.draw_buffers( &js_sys::Array::from_iter( iter ) );
    Ok( () )
  }

  pub fn bind_read( &self, gl : &GL )
  {
    gl.bind_framebuffer( GL::READ_FRAMEBUFFER, Some( &self.framebuffer ) );
  }
}

struct PreserveFramebuffer
{
  framebuffer : Framebuffer,
  attachmets : HashMap< AttachmentType, Attachment >,
}

impl PreserveFramebuffer
{
  pub fn new( gl : &GL ) -> Result< Self, gl::WebglError >
  {
    let framebuffer = Framebuffer::new( gl )?;
    Ok( Self { framebuffer, attachmets: Default::default() } )
  }

  pub fn get_texture( &self, attachment: AttachmentType ) -> Option< &WebGlTexture >
  {
    match self.attachmets.get( &attachment )
    {
      Some( attachment ) => match attachment
      {
        Attachment::Texture( tex ) => Some( tex ),
        Attachment::Renderbuffer( _ ) => None,
      }
      None => None,
    }
  }

  pub fn get_renderbuffer( &self, attachment: AttachmentType ) -> Option< &WebGlRenderbuffer >
  {
    match self.attachmets.get( &attachment )
    {
      Some( attachment ) => match attachment
      {
        Attachment::Texture( _ ) => None,
        Attachment::Renderbuffer( renderbuffer ) => Some( renderbuffer ),
      }
      None => None,
    }
  }

  pub fn texture_2d( &mut self, attachment: AttachmentType, texture : Option< WebGlTexture >, level : i32, gl : &GL )
  {
    self.framebuffer.texture_2d( attachment, texture.as_ref(), level, gl );
    match texture
    {
      Some( tex ) => _ = self.attachmets.insert( attachment, Attachment::Texture( tex ) ),
      None => _ = self.attachmets.remove( &attachment ),
    }
  }

  pub fn texture_layer
  (
    &mut self,
    attachment: AttachmentType,
    texture : Option< WebGlTexture >,
    level : i32,
    layer : i32,
    gl : &GL,
  )
  {
    self.framebuffer.texture_layer( attachment, texture.as_ref(), level, layer, gl );
    match texture
    {
      Some( tex ) => _ = self.attachmets.insert( attachment, Attachment::Texture( tex ) ),
      None => _ = self.attachmets.remove( &attachment ),
    }
  }

  pub fn renderbuffer( &mut self, attachment: AttachmentType, renderbuffer : Option< WebGlRenderbuffer >, gl : &GL )
  {
    self.framebuffer.renderbuffer( attachment, renderbuffer.as_ref(), gl );
    match renderbuffer
    {
      Some( renderbuffer ) => _ = self.attachmets.insert( attachment, Attachment::Renderbuffer( renderbuffer ) ),
      None => _ = self.attachmets.remove( &attachment ),
    }
  }

  pub fn bind_all_drawbuffers( &self, gl : &GL )
  {
    self.framebuffer.bind_all_drawbuffers( gl );
  }

  pub fn bind_drawbuffers( &self, attachments : &[ Color ], gl : &GL ) -> Result< (), String >
  {
    self.framebuffer.bind_drawbuffers( attachments, gl )
  }

  pub fn bind_read( &self, gl : &GL )
  {
    self.framebuffer.bind_read( gl );
  }
}
