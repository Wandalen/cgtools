pub fn create_blob( data : web_sys::js_sys::Array, mime_type : &str ) -> Result< String, minwebgl::JsValue >
{
  let blob_props = web_sys::BlobPropertyBag::new();
  blob_props.set_type( mime_type );
  let blob = web_sys::Blob::new_with_str_sequence_and_options( &data, &blob_props ).unwrap();
  web_sys::Url::create_object_url_with_blob( &blob )
}
