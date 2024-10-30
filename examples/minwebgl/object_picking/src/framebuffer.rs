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

pub struct Framebuffer< const COLOR_ATTACHMENTS : usize >
{
  framebuffer : Option< WebGlFramebuffer >,
  color_attachments : [ Option< WebGlTexture >; COLOR_ATTACHMENTS ],
  renderbuffer : Option< WebGlRenderbuffer >
}

impl< const COLOR_ATTACHMENTS : usize > Framebuffer< COLOR_ATTACHMENTS >
{
  pub fn new( gl : &GL ) -> Option< Self >
  {
    let max = gl.get_parameter( GL::MAX_DRAW_BUFFERS ).unwrap().as_f64().unwrap() as usize;
    if COLOR_ATTACHMENTS > max
    {
      return None;
    }
    let framebuffer = gl.create_framebuffer();

    Some
    (
      Self
      {
        framebuffer,
        color_attachments: [ const { None }; COLOR_ATTACHMENTS ],
        renderbuffer: None,
      }
    )
  }

  pub fn get_color_attachment( &self, index : usize ) -> Option< &WebGlTexture >
  {
    self.color_attachments[ index ].as_ref()
  }

  pub fn attach_texture( &mut self, index : usize, tex : Option< WebGlTexture >, gl : &GL )
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
    gl.framebuffer_texture_2d
    (
      GL::FRAMEBUFFER,
      GL::COLOR_ATTACHMENT0 + index as u32,
      GL::TEXTURE_2D,
      tex.as_ref(),
      0
    );
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );
    self.color_attachments[ index ] = tex;
  }

  /// `attachment` is either `GL::DEPTH_ATTACHMENT`, `GL::STENCIL_ATTACHMENT` or `GL::DEPTH_STENCIL_ATTACHMENT`
  pub fn attach_renderbuffer( &mut self, renderbuffer : Option< WebGlRenderbuffer >, attachment : u32, gl : &GL )
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
    gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, attachment, GL::RENDERBUFFER, renderbuffer.as_ref() );
    self.renderbuffer = renderbuffer;
  }

  pub fn bind( &self, gl : &GL )
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
    let mut draw_buffers = [ gl::NONE; COLOR_ATTACHMENTS ];
    for ( i, attachment ) in self.color_attachments.iter().enumerate()
    {
      if attachment.is_some()
      {
        draw_buffers[ i ] = GL::COLOR_ATTACHMENT0 + i as u32;
      }
    }
    let iter = draw_buffers.into_iter().map( | item | JsValue::from_f64( item as f64 ) );
    gl.draw_buffers( &js_sys::Array::from_iter( iter ).into() );
  }

  pub fn bind_nth( &self, n : usize, gl : &GL )
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
    let mut draw_buffers = [ gl::NONE; COLOR_ATTACHMENTS ];
    draw_buffers[ n ] = self.color_attachments[ n ].as_ref().map_or( GL::NONE, | _ | GL::COLOR_ATTACHMENT0 + n as u32, );
    let iter = draw_buffers.into_iter().map( | item | JsValue::from_f64( item as f64 ) );
    gl.draw_buffers( &js_sys::Array::from_iter( iter ).into() );
  }
}

pub fn texture2d( gl : &GL, internal_format : u32, width : i32, height : i32 ) -> Option< WebGlTexture >
{
  let texture = gl.create_texture();
  gl.bind_texture( GL::TEXTURE_2D, texture.as_ref() );
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, internal_format, width, height );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32 );
  texture
}

pub fn depth_stencil_buffer( gl : &GL, width : i32, height : i32 ) -> Option< WebGlRenderbuffer >
{
  let renderbuffer = gl.create_renderbuffer();
  gl.bind_renderbuffer( GL::RENDERBUFFER, renderbuffer.as_ref() );
  gl.renderbuffer_storage( GL::RENDERBUFFER, GL::DEPTH24_STENCIL8, width, height );
  renderbuffer
}
