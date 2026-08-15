//! FFI bindings into `controls.js`'s minimal slider panel and live source
//! editor -- `addSlider` creates one labeled range input, `onChange`
//! registers the single callback fired (with every slider's current value,
//! as a JS object keyed by `property`) whenever any slider moves;
//! `initEditor`/`onEdit` are the same shape for the Shadertoy-style WGSL
//! textarea, and `setDiagnostics`/`clearDiagnostics` show or hide the last
//! compile/pipeline error underneath it.

use web_sys::wasm_bindgen::{ self, prelude::* };

#[ wasm_bindgen( module = "/controls.js" ) ]
extern "C"
{
  #[ wasm_bindgen( js_name = addSlider ) ]
  pub fn slider_add( label : &str, property : &str, value : f64, min : f64, max : f64, step : f64 );

  #[ wasm_bindgen( js_name = onChange ) ]
  pub fn on_change( callback : &JsValue );

  #[ wasm_bindgen( js_name = initEditor ) ]
  pub fn editor_init( initial_source : &str );

  #[ wasm_bindgen( js_name = onEdit ) ]
  pub fn on_edit( callback : &JsValue );

  #[ wasm_bindgen( js_name = setDiagnostics ) ]
  pub fn diagnostics_set( text : &str );

  #[ wasm_bindgen( js_name = clearDiagnostics ) ]
  pub fn diagnostics_clear();
}
