use minwebgl as gl;
use gl::wasm_bindgen::{ self, prelude::* };

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
  #[ wasm_bindgen( js_name = "addBoolean" ) ]
  pub fn add_boolean( gui : &JsValue,  object : &JsValue, property : &str ) -> JsValue;

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
  pub fn on_finish_change( gui : &JsValue, callback : &Closure< dyn FnMut( JsValue ) > ) -> JsValue;

  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "onChange" ) ]
  pub fn on_change( gui : &JsValue, callback : &Closure< dyn FnMut( f32 ) > ) -> JsValue;

  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "onChange" ) ]
  pub fn on_change_string( gui : &JsValue, callback : &Closure< dyn FnMut( String ) > ) -> JsValue;

  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "onChange" ) ]
  pub fn on_change_bool( gui : &JsValue, callback : &Closure< dyn FnMut( bool ) > ) -> JsValue;

  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "setName" ) ]
  pub fn set_name( gui : &JsValue, value : &str ) -> JsValue;


  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "hide" ) ]
  pub fn hide( gui : &JsValue ) -> JsValue;

  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "show" ) ]
  pub fn show( gui : &JsValue ) -> JsValue;
}
