#![ allow( clippy::expect_fun_call ) ]

use web_sys::wasm_bindgen;
use wasm_bindgen::prelude::*;

#[ wasm_bindgen( module = "/sidebar_toggle.js" ) ]
extern "C"
{
  #[ wasm_bindgen( js_name = setupSidebarToggle ) ]
  pub fn setup_sidebar_toggle();
}
