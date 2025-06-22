/// Internal namespace.
mod private
{

  /// Creates a blob from u8 slice sequence and options
  pub fn create_blob< T : Into< web_sys::js_sys::Array > >( data : T, mime_type : &str )
  -> Result< String, crate::JsValue >
  {
    let blob_props = web_sys::BlobPropertyBag::new();
    blob_props.set_type( mime_type );

    let blob = web_sys::Blob::new_with_u8_slice_sequence_and_options( &( data.into() ), &blob_props ).unwrap();
    web_sys::Url::create_object_url_with_blob( &blob )
  }

}

crate::mod_interface!
{
  own use create_blob;
}
