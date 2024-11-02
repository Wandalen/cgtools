use minwebgl as gl;
use gl::GL;
use web_sys::
{
  js_sys,
  wasm_bindgen::JsValue,
  WebGlFramebuffer,
  WebGlRenderbuffer,
  WebGlTexture
};

pub struct Framebuffer
{
  framebuffer : Option< WebGlFramebuffer >,
  attachments : Vec< Option< Attachment > >,
  renderbuffer : Option< WebGlRenderbuffer >
}

impl Framebuffer
{
  pub fn new( gl : &GL ) -> Option< Self >
  {
    let framebuffer = gl.create_framebuffer();

    Some
    (
      Self
      {
        framebuffer,
        attachments: vec![],
        renderbuffer: None,
      }
    )
  }

  pub fn get_attachment( &self, index : usize ) -> Option< &Attachment >
  {
    self.attachments[ index ].as_ref()
  }

  pub fn get_renderbuffer( &self ) -> Option< &WebGlRenderbuffer >
  {
    self.renderbuffer.as_ref()
  }

  pub fn attach_texture( &mut self, tex : Option< WebGlTexture >, gl : &GL )
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
    gl.framebuffer_texture_2d
    (
      GL::FRAMEBUFFER,
      GL::COLOR_ATTACHMENT0 + self.attachments.len() as u32,
      GL::TEXTURE_2D,
      tex.as_ref(),
      0
    );
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );
    self.attachments.push( tex.map_or( None, | t | Some( Attachment::Texture( t ) ) ) );
  }

  pub fn attach_renderbuffer( &mut self, rbuf : Option< WebGlRenderbuffer >, gl : &GL )
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
    gl.framebuffer_renderbuffer
    (
      GL::FRAMEBUFFER,
      GL::COLOR_ATTACHMENT0 + self.attachments.len() as u32,
      GL::RENDERBUFFER,
      rbuf.as_ref(),
    );
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );
    self.attachments.push( rbuf.map_or( None, | r | Some( Attachment::Renderbuffer( r ) ) ) );
  }

  pub fn renderbuffer( &mut self, renderbuffer : Option< WebGlRenderbuffer >, attachment : RenderbufferAttachment, gl : &GL )
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
    gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, attachment as u32, GL::RENDERBUFFER, renderbuffer.as_ref() );
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );
    self.renderbuffer = renderbuffer;
  }

  pub fn bind( &self, gl : &GL )
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
  }

  /// binds framebuffer with all the draw buffers
  pub fn bind_drawbuffers( &self, gl : &GL )
  {
    let iter = self.attachments
    .iter()
    .enumerate()
    .map( | ( i, item ) | if item.is_some() { GL::COLOR_ATTACHMENT0 + i as u32 } else { GL::NONE } )
    .map( | item | JsValue::from_f64( item as f64 ) );

    gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
    gl.draw_buffers( &js_sys::Array::from_iter( iter ).into() );
  }

  // binds framebuffer with a specific draw buffer
  pub fn bind_drawbuffer( &self, n : usize, gl : &GL )
  {
    let mut draw_buffers = vec![ gl::NONE; self.attachments.len() ];
    draw_buffers[ n ] = self.attachments[ n ].as_ref().map_or( GL::NONE, | _ | GL::COLOR_ATTACHMENT0 + n as u32 );
    let iter = draw_buffers.into_iter().map( | item | JsValue::from_f64( item as f64 ) );

    gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
    gl.draw_buffers( &js_sys::Array::from_iter( iter ).into() );
  }
}

pub enum Attachment
{
  Texture( WebGlTexture ),
  Renderbuffer( WebGlRenderbuffer ),
}

#[ repr( u32 ) ]
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum RenderbufferAttachment
{
  Depth = GL::DEPTH_ATTACHMENT,
  Stencil = GL::STENCIL_ATTACHMENT,
  DepthStencil = GL::DEPTH_STENCIL_ATTACHMENT,
}
