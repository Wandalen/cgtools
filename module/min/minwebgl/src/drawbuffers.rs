mod private
{
  use crate as gl;
  use gl::{ GL, WebglError };
  use wasm_bindgen::JsValue;

  // Maximum amount of ColorAttachments supported by WebGl2
  const MAX_COLOR_ATTACHMENTS : usize = 16;

  /// Validates a color attachment index against `MAX_COLOR_ATTACHMENTS`, returning
  /// `WebglError::IdOutOfRange` instead of allowing an out-of-bounds array index through.
  ///
  /// # Errors
  /// Returns [`WebglError::IdOutOfRange`] when `index >= MAX_COLOR_ATTACHMENTS` ( 16 ).
  // Fix(BUG-159)
  // Root cause: `drawbuffers` indexed a fixed MAX_COLOR_ATTACHMENTS-element array with the
  // caller's raw attachment index and no bounds check; the only guard present ( checked_add
  // against u32::MAX ) never rejects an ordinary out-of-range index like 16, so it panicked via
  // a raw, undocumented "index out of bounds" instead of this function's own documented message.
  // Pitfall: MAX_COLOR_ATTACHMENTS bounds the ARRAY INDEX, not the attachment id after adding
  // COLOR_ATTACHMENT0 -- validate the index itself, before the add, not the sum.
  pub fn color_attachment_index_validate( index : usize ) -> Result< usize, WebglError >
  {
    if index < MAX_COLOR_ATTACHMENTS
    {
      Ok( index )
    }
    else
    {
      Err( WebglError::IdOutOfRange( format!( "Invalid color attachment index {index}: must be < {MAX_COLOR_ATTACHMENTS}" ) ) )
    }
  }

  /// Wrapper over `gl.draw_buffers`. Provide attachments
  /// you want to draw into and it will do the rest.
  ///
  /// # Example
  ///
  /// binds for drawing GL::ATTACHMENT0, GL::ATTACHMENT1, GL::ATTACHMENT3
  ///
  /// `drawbuffers( &gl, &[ 0, 1, 3 ] );`
  ///
  /// # Panics
  /// Panics if an attachment index is `>= MAX_COLOR_ATTACHMENTS` ( 16 ), or overflows a valid
  /// color attachment constant.
  pub fn drawbuffers( gl : &GL, attachments : &[ u32 ] )
  {
    let mut buffers = [ gl::NONE; MAX_COLOR_ATTACHMENTS ];
    for attachment in attachments
    {
      let index = color_attachment_index_validate( *attachment as usize ).expect( "Invalid color attachment" );
      let attachment = attachment
      .checked_add( gl::COLOR_ATTACHMENT0 )
      .unwrap_or_else( || panic!( "Invalid color attachment {}", *attachment ) );
      buffers[ index ] = attachment;
    }

    let last = buffers.iter().rposition( | item | *item != gl::NONE ).map_or( 1, | i | i + 1 );
    let array : js_sys::Array = buffers[ .. last ].iter().map( | item | JsValue::from_f64( f64::from( *item ) ) ).collect();

    gl.draw_buffers( &array );
  }
}

crate::mod_interface!
{
  own use
  {
    color_attachment_index_validate,
    drawbuffers
  };
}
