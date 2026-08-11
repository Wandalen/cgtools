use minwebgl as gl;
use gl::wasm_bindgen::{ self, prelude::* };

#[ allow( unsafe_code, reason = "wasm_bindgen emits unsafe extern imports for these JS bindings — as safe as any generated binding; expect would be unfulfilled on non-wasm targets where the macro expands the block differently" ) ]
#[ wasm_bindgen( module = "/gui.js" ) ]
extern "C"
{
  #[ wasm_bindgen( js_name = "newGui" ) ]
  pub fn new_gui() -> JsValue;

  #[ wasm_bindgen( js_name = "addFolder" ) ]
  pub fn add_folder( gui : &JsValue, name : &str ) -> JsValue;

  #[ wasm_bindgen( js_name = "addSliderController" ) ]
  pub fn add_slider( gui : &JsValue, object : &JsValue, property : &str, min : f64, max : f64, step : f64 ) -> JsValue;

  #[ wasm_bindgen( js_name = "addDropdownController" ) ]
  pub fn add_dropdown( gui : &JsValue, object : &JsValue, property : &str, options : &JsValue ) -> JsValue;

  #[ wasm_bindgen( js_name = "onFinishChange" ) ]
  pub fn on_finish_change( gui : &JsValue, callback : &Closure< dyn FnMut( JsValue ) > ) -> JsValue;

  #[ wasm_bindgen( js_name = "onChange" ) ]
  pub fn on_change( gui : &JsValue, callback : &Closure< dyn FnMut( f32 ) > ) -> JsValue;

  #[ wasm_bindgen( js_name = "onChange" ) ]
  pub fn on_change_string( gui : &JsValue, callback : &Closure< dyn FnMut( String ) > ) -> JsValue;

  #[ wasm_bindgen( js_name = "setName" ) ]
  pub fn set_name( gui : &JsValue, value : &str ) -> JsValue;


  #[ wasm_bindgen( js_name = "hide" ) ]
  pub fn hide( gui : &JsValue ) -> JsValue;

  #[ wasm_bindgen( js_name = "show" ) ]
  pub fn show( gui : &JsValue ) -> JsValue;
}
