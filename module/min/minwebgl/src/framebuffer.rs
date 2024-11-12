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

  /// Container for objects attachable to a framebuffer
  pub enum Attachment
  {
    Texture( WebGlTexture ),
    Renderbuffer( WebGlRenderbuffer ),
  }

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

  #[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord ) ]
  #[ repr( u32 ) ]
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

  /// # Framebuffer stucture
  /// This structure aims to automate WebGl's
  /// annoying syntax when you want to have multiple render
  /// outputs. It keeps track of color attachments and provide
  /// API to bind all or several of attachments for drawing
  /// in a convenient way.
  pub struct Framebuffer
  {
    // bit mask for keeping track of color attachments
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

    /// Attaches 2D texture to the framebuffer.
    /// Default framebuffer will be bound after this call.
    /// If you want to remove attachment provide `None`
    pub fn texture_2d( &mut self, attachment : AttachmentType, texture : Option< &WebGlTexture >, level : i32, gl : &GL )
    {
      gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &self.framebuffer ) );
      gl.framebuffer_texture_2d( GL::FRAMEBUFFER, attachment.as_u32(), GL::TEXTURE_2D, texture, level );
      gl.bind_framebuffer( GL::FRAMEBUFFER, None );

      self.update_bitmask( attachment, texture.is_some() );
    }

    /// Attaches a layer of 3D texture or texture array to the framebuffer.
    /// Default framebuffer will be bound after this call.
    /// If you want to remove attachment provide `None`
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

    /// Attaches renderbuffer to the framebuffer.
    /// Default framebuffer will be bound after this call.
    /// If you want to remove attachment provide `None`
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

    /// Binds framebuffer for drawing along with
    /// all the color attachments as draw buffers
    pub fn bind_all_drawbuffers( &self, gl : &GL )
    {
      // forms an array of attachments in their respective positions
      // for binding them as draw buffers
      // for example if framebuffer has attachment0 and attachment2
      // then the array should be [ GL::ColorAttachment0, GL::NONE, GL::ColorAttachment2 ]
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

    /// Binds framebuffer for drawing along with the
    /// color attachments specified in `attachments` parameter.
    /// You can provide empty slice if you want no color attachments as draw buffers.
    /// ## Returns:
    /// `Err` If framebuffer does not have a specified attachment
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

    /// Binds framebuffer for reading
    pub fn bind_read( &self, gl : &GL )
    {
      gl.bind_framebuffer( GL::READ_FRAMEBUFFER, Some( &self.framebuffer ) );
    }
  }

  /// This structure provides exactly the same API
  /// as `Framebuffer`, but it also stores attachments in it.
  /// So if you want to have a framebuffer and its attachments in one place -
  /// this is what you want to use.
  pub struct PreserveFramebuffer
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

    /// Returns texture at `attachment` if available
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

    /// Returns renderbuffer at `attachment` if available
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

    /// Attaches 2D texture to the framebuffer.
    /// Default framebuffer will be bound after this call.
    /// If you want to remove attachment provide `None`
    pub fn texture_2d( &mut self, attachment: AttachmentType, texture : Option< WebGlTexture >, level : i32, gl : &GL )
    {
      self.framebuffer.texture_2d( attachment, texture.as_ref(), level, gl );
      match texture
      {
        Some( tex ) => _ = self.attachmets.insert( attachment, Attachment::Texture( tex ) ),
        None => _ = self.attachmets.remove( &attachment ),
      }
    }

    /// Attaches a layer of 3D texture or texture array to the framebuffer.
    /// Default framebuffer will be bound after this call.
    /// If you want to remove attachment provide `None`
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

    /// Attaches renderbuffer to the framebuffer.
    /// Default framebuffer will be bound after this call.
    /// If you want to remove attachment provide `None`
    pub fn renderbuffer( &mut self, attachment: AttachmentType, renderbuffer : Option< WebGlRenderbuffer >, gl : &GL )
    {
      self.framebuffer.renderbuffer( attachment, renderbuffer.as_ref(), gl );
      match renderbuffer
      {
        Some( renderbuffer ) => _ = self.attachmets.insert( attachment, Attachment::Renderbuffer( renderbuffer ) ),
        None => _ = self.attachmets.remove( &attachment ),
      }
    }

    /// Binds framebuffer for drawing along with
    /// all the color attachments as draw buffers
    pub fn bind_all_drawbuffers( &self, gl : &GL )
    {
      self.framebuffer.bind_all_drawbuffers( gl );
    }

    /// Binds framebuffer for drawing along with the
    /// color attachments specified in `attachments` parameter.
    /// You can provide empty slice if you want no color attachments as draw buffers.
    /// ## Returns:
    /// `Err` If framebuffer does not have a specified attachment
    pub fn bind_drawbuffers( &self, attachments : &[ Color ], gl : &GL ) -> Result< (), String >
    {
      self.framebuffer.bind_drawbuffers( attachments, gl )
    }

    /// Binds framebuffer for reading
    pub fn bind_read( &self, gl : &GL )
    {
      self.framebuffer.bind_read( gl );
    }
  }
}

crate::mod_interface!
{
  own use Attachment;
  own use AttachmentType;
  own use Color;
  own use Framebuffer;
  own use PreserveFramebuffer;
}
