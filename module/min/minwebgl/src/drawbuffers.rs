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
  /// Panics, with `color_attachment_index_validate`'s own descriptive message, if an
  /// attachment index is `>= MAX_COLOR_ATTACHMENTS` ( 16 ), or overflows a valid color
  /// attachment constant.
  //
  // Fix(UX-007): propagated `color_attachment_index_validate`'s `Result` through a bare
  // `.expect( "Invalid color attachment" )` instead of surfacing its own richer message
  // ( which names the offending index and the bound it exceeded ).
  // Root cause: `color_attachment_index_validate` was extracted ( Fix(BUG-159) ) specifically
  // to return a descriptive `Result`, but `drawbuffers` itself kept its pre-existing `()`
  // signature and just re-panicked with a generic literal instead of adopting the new message.
  // This function's own public signature stays `()` ( not `Result` ) rather than propagating
  // further: `drawbuffers` is called directly ( no `?` / no `.unwrap()` handling ) from ~20
  // call sites across `module/helper/renderer`, `module/helper/canvas_renderer`, and multiple
  // `examples/` crates, all outside this fix's edit scope. This workspace builds with
  // `RUSTFLAGS="-D warnings"`, so switching to a `#[must_use] Result` here would turn every one
  // of those call sites' now-unused return value into a hard compile error with no way to fix
  // it in this change. Surfacing the validator's own message via the panic ( instead of a
  // generic literal ) is the safe, scope-respecting improvement available without touching
  // those call sites.
  pub fn drawbuffers( gl : &GL, attachments : &[ u32 ] )
  {
    let mut buffers = [ gl::NONE; MAX_COLOR_ATTACHMENTS ];
    for attachment in attachments
    {
      let index = color_attachment_index_validate( *attachment as usize ).unwrap_or_else( | e | panic!( "{e}" ) );
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
