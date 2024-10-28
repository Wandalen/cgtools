/// Internal namespace.
mod private
{
  use crate::*;
  pub use web_sys::WebGlBuffer;
  // use data_type::Const;

  /// Uploads data to a WebGL ELEMENT_ARRAY_BUFFER.
  ///
  /// # Arguments
  ///
  /// * `gl` - A reference to the WebGL context.
  /// * `buffer` - A reference to the WebGL buffer to upload data to.
  /// * `data` - A slice of data to upload.
  /// * `hint` - A usage hint for the buffer (e.g., `GL::STATIC_DRAW`).
  ///
  /// # Example
  ///
  /// ```rust, ignore
  /// minwebgl::index::upload( &gl, &buffer, &data, GL::STATIC_DRAW );
  /// ```
  pub fn upload< Data >( gl : &GL, buffer : &WebGlBuffer, data : &Data, data_usage : u32 )
  where
    Data : mem::AsBytes + ?Sized,
  {
    gl.bind_buffer( GL::ELEMENT_ARRAY_BUFFER, Some( buffer ) );
    gl.buffer_data_with_u8_array( GL::ELEMENT_ARRAY_BUFFER, data.as_bytes(), data_usage );
  }

}

crate::mod_interface!
{

  orphan use
  {
    upload,
    WebGlBuffer,
  };

}
