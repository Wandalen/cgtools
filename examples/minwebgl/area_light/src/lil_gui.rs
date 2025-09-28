use minwebgl as gl;
use gl::wasm_bindgen::{ self, prelude::* };
use web_sys::js_sys::Array;

#[ wasm_bindgen( module = "/gui.js" ) ]
extern "C"
{
  // The bindings below are safe as any other normal WASM bindings
  // produced by `wasm_bindgen` crate,
  // but the linter consider them unsafe for any reason,
  // so in order to not be distracted by the linter,
  // these bindings are attributed as `allow( unsafe_code )`

  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "newGui" ) ]
  pub fn new_gui() -> JsValue;

  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "addFolder" ) ]
  pub fn add_folder( gui : &JsValue, name : &str ) -> JsValue;

  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "addSliderController" ) ]
  pub fn add_slider( gui : &JsValue, object : &JsValue, property : &str, min : f64, max : f64, step : f64 ) -> JsValue;

  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "addDropdownController" ) ]
  pub fn add_dropdown( gui : &JsValue, object : &JsValue, property : &str, options : &JsValue ) -> JsValue;

  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "onFinishChange" ) ]
  pub fn on_finish_change( gui : &JsValue, callback : &web_sys::js_sys::Function );

  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "getTitle" ) ]
  pub fn get_title( gui : &JsValue ) -> String;

  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "getFolders" ) ]
  pub fn get_folders( gui : &JsValue ) -> Array;

  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "hide" ) ]
  pub fn hide( gui : &JsValue ) -> JsValue;

  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "show" ) ]
  pub fn show( gui : &JsValue ) -> JsValue;
}
