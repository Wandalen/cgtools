use minwebgl as gl;
use gl::wasm_bindgen::{ self, prelude::* };

#[ allow( unsafe_code, reason = "wasm_bindgen emits unsafe extern imports for these JS bindings — as safe as any generated binding; expect would be unfulfilled on non-wasm targets where the macro expands the block differently" ) ]
#[ wasm_bindgen( module = "/gui.js" ) ]
extern "C"
{
  #[ wasm_bindgen( js_name = "newGui" ) ]
  pub fn new_gui() -> JsValue;

  #[ wasm_bindgen( js_name = "addFolder" ) ]
  pub fn folder_add( gui : &JsValue, name : &str ) -> JsValue;

  #[ wasm_bindgen( js_name = "addSliderController" ) ]
  pub fn slider_add( gui : &JsValue, object : &JsValue, property : &str, min : f64, max : f64, step : f64 ) -> JsValue;

  #[ wasm_bindgen( js_name = "addDropdownController" ) ]
  pub fn dropdown_add( gui : &JsValue, object : &JsValue, property : &str, options : &JsValue ) -> JsValue;

  #[ wasm_bindgen( js_name = "onFinishChange" ) ]
  pub fn on_finish_change( gui : &JsValue, callback : &Closure< dyn FnMut( JsValue ) > ) -> JsValue;

  #[ wasm_bindgen( js_name = "onChange" ) ]
  pub fn on_change( gui : &JsValue, callback : &Closure< dyn FnMut( f32 ) > ) -> JsValue;

  // Fix(BUG-339): the js_name attribute below pointed at a title-getter export that only exists
  // in sibling crates' older gui.js copies this file was copy-pasted from — this crate's own
  // gui.js exports no such function, only a matching two-argument title-setter instead.
  // Root cause: bindings file copy-pasted from a sibling crate without re-checking this crate's
  // own gui.js exports.
  // Pitfall: a wasm_bindgen extern binding compiles fine even when its js_name has no real JS
  // export — the mismatch only surfaces as a runtime error if the binding is ever called.
  #[ wasm_bindgen( js_name = "set_name" ) ]
  pub fn name_set( gui : &JsValue, value : &str ) -> JsValue;

  #[ wasm_bindgen( js_name = "onChange" ) ]
  pub fn on_change_string( gui : &JsValue, callback : &Closure< dyn FnMut( String ) > ) -> JsValue;

  #[ wasm_bindgen( js_name = "hide" ) ]
  pub fn hide( gui : &JsValue ) -> JsValue;

  #[ wasm_bindgen( js_name = "show" ) ]
  pub fn show( gui : &JsValue ) -> JsValue;
}
