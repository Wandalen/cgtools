mod private
{
  use crate::*;
  use crate as gl;
  use std::collections::HashMap;
  use web_sys::
  {
    WebGlFramebuffer,
    WebGlRenderbuffer,
    WebGlTexture
  };

  // Maximum amount of ColorAttachments supported by WebGl2
  const MAX_COLOR_ATTACHMENTS : usize = 16;

  #[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord ) ]
  #[ repr( u32 ) ]
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

  #[ derive( Default, Clone, Copy ) ]
  pub struct Attachments
  {
    attachments : [ u32; MAX_COLOR_ATTACHMENTS ],
  }

  impl Attachments
  {
    pub fn new() -> Self
    {
      Self { attachments: [ GL::NONE; MAX_COLOR_ATTACHMENTS ] }
    }

    pub fn from_slice( attachments : &[ ColorAttachment ] ) -> Self
    {
      let mut this = Self::new();
      for attachment in attachments
      {
        let index = *attachment as usize - ColorAttachment::N0 as usize;
        this.attachments[ index ] =  *attachment as u32;
      }
    }

    pub fn insert( &mut self, attachment : ColorAttachment )
    {
      let index = attachment as usize - ColorAttachment::N0 as usize;
      self.attachments[ index ] = attachment as u32;
    }

    pub fn remove( &mut self, attachment : ColorAttachment )
    {
      let index = attachment as usize - ColorAttachment::N0 as usize;
      self.attachments[ index ] = GL::NONE;
    }

    pub fn clear( &mut self )
    {
      self.attachments = [ GL::NONE; MAX_COLOR_ATTACHMENTS ];
    }

    /// Converts to an array intended to be passed into gl.draw_buffers
    pub fn as_drawbuffers( &self ) -> js_sys::Array
    {
      let last = self.attachments.iter().rposition( | item | *item != GL::NONE ).map_or( 0, | pos | pos + 1 );
      js_sys::Array::from_iter( self.attachments[ .. last ].iter().map( | item | JsValue::from_f64( *item as f64 ) ) )
    }
  }
}

crate::mod_interface!
{
  own use ColorAttachment;
  own use Attachments;
}
