/// Internal namespace.
mod private
{

  /// Creates a blob from u8 slice sequence and options
  ///
  /// # Errors
  /// Returns `Err` if the browser fails to construct the `Blob` from the given data and
  /// options, or to create an object URL for it.
  pub fn blob_create< T : Into< web_sys::js_sys::Array > >( data : T, mime_type : &str )
  -> Result< String, crate::JsValue >
  {
    let blob_props = web_sys::BlobPropertyBag::new();
    blob_props.set_type( mime_type );

    let blob = web_sys::Blob::new_with_u8_slice_sequence_and_options( &( data.into() ), &blob_props )?;
    web_sys::Url::create_object_url_with_blob( &blob )
  }

}

crate::mod_interface!
{
  own use blob_create;
}
