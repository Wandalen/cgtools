//! FFI bindings into `controls.js`'s minimal slider panel -- `addSlider`
//! creates one labeled range input, `onChange` registers the single
//! callback fired (with every slider's current value, as a JS object keyed
//! by `property`) whenever any slider moves.

use web_sys::wasm_bindgen::{ self, prelude::* };

#[ wasm_bindgen( module = "/controls.js" ) ]
extern "C"
{
  #[ wasm_bindgen( js_name = addSlider ) ]
  pub fn slider_add( label : &str, property : &str, value : f64, min : f64, max : f64, step : f64 );

  #[ wasm_bindgen( js_name = onChange ) ]
  pub fn on_change( callback : &JsValue );
}
