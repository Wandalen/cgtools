mod private
{
  use crate as gl;
  use gl::GL;
  use wasm_bindgen::JsValue;

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

  /// This is just a wrapper over `gl.draw_buffers`. Provide attachments
  /// you want to draw into and it will the rest
  pub fn drawbuffers( gl : &GL, attachments : &[ ColorAttachment ] )
  {
    let mut buffers = [ 0; MAX_COLOR_ATTACHMENTS ];
    for attachment in attachments
    {
      let index = *attachment as usize - ColorAttachment::N0 as usize;
      buffers[ index ] =  *attachment as u32;
    }
    let last = buffers.iter().rposition( | item | *item != GL::NONE ).unwrap_or( 0 );
    let array = js_sys::Array::from_iter
    (
      buffers[ ..= last ].iter().map( | item | JsValue::from_f64( *item as f64 ) )
    );
    gl.draw_buffers( &array );
  }
}

crate::mod_interface!
{
  own use drawbuffers;
  own use ColorAttachment;
}
