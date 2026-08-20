use minwebgl::wasm_bindgen;
use wasm_bindgen::prelude::*;
use web_sys::Blob;

#[ wasm_bindgen( module = "/bg_removal.js" ) ]
extern "C"
{
  #[ wasm_bindgen( js_name = "removeBg" ) ]
  pub async fn bg_wrapper_remove( blob : Blob ) -> JsValue;
}

pub async fn image_process( blob : Blob ) -> Option< Blob >
{
  let blob = bg_wrapper_remove( blob ).await;

  if blob.is_null()
  {
    return None;
  }

  blob.dyn_into::< Blob >().ok()
}
