mod private
{
  use crate as gl;
  use gl::GL;
  use wasm_bindgen::JsValue;

  // Maximum amount of ColorAttachments supported by WebGl2
  const MAX_COLOR_ATTACHMENTS : usize = 16;

  /// This is just a wrapper over `gl.draw_buffers`. Provide attachments
  /// you want to draw into and it will do the rest
  pub fn drawbuffers( gl : &GL, attachments : &[ u32 ] )
  {
    let mut buffers = [ GL::NONE; MAX_COLOR_ATTACHMENTS ];
    for attachment in attachments
    {
      let index = *attachment as usize;
      let attachment = attachment
      .checked_add( GL::COLOR_ATTACHMENT0 )
      .expect( &format!( "Invalid color attachment {}", *attachment ) );
      buffers[ index ] = attachment;
    }
    let last = buffers.iter().rposition( | item | *item != GL::NONE ).map_or( 0, | i | i + 1 );
    let array = js_sys::Array::from_iter
    (
      buffers[ .. last ].iter().map( | item | JsValue::from_f64( *item as f64 ) )
    );

    gl.draw_buffers( &array );
  }
}

crate::mod_interface!
{
  own use drawbuffers;
}
