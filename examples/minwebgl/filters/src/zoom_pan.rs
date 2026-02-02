#![ allow( clippy::expect_fun_call ) ]

use web_sys::wasm_bindgen;
use wasm_bindgen::prelude::*;

#[ wasm_bindgen( module = "/zoom_pan.js" ) ]
extern "C"
{
  #[ wasm_bindgen( js_name = setupZoomPan ) ]
  pub fn setup_zoom_pan();
}
