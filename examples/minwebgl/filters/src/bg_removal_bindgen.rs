use minwebgl::wasm_bindgen;
use wasm_bindgen::prelude::*;
use web_sys::Blob;

#[ wasm_bindgen( module = "/bg_removal.js" ) ]
extern "C"
{
  #[ wasm_bindgen( js_name = "removeBg" ) ]
  pub async fn remove_bg_wrapper( blob : Blob ) -> JsValue;
}

pub async fn process_image( blob : Blob ) -> Option< Blob >
{
  let blob = remove_bg_wrapper( blob ).await;

  if blob.is_null()
  {
    return None;
  }

  blob.dyn_into::< Blob >().ok()
}
