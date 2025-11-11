use minwebgl as gl;
use gl::wasm_bindgen::{ self, prelude::* };
use serde::{ Serialize, Deserialize };

#[ derive( Debug, Clone, Default, Serialize, Deserialize ) ]
pub struct UiState
{
  pub gem : String,
  pub metal : String,
  pub ring : u32,
  pub changed : Vec< String >
}

pub fn get_ui_state() -> Option< UiState >
{
  serde_wasm_bindgen::from_value::< UiState >( _get_ui_state() ).ok()
}

pub fn is_changed() -> bool
{
  _is_changed().as_bool().unwrap()
}

#[ wasm_bindgen( module = "/index.js" ) ]
extern "C"
{
  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "getUiState" ) ]
  fn _get_ui_state() -> JsValue;

  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "isChanged" ) ]
  fn _is_changed() -> JsValue;

  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "clearChanged" ) ]
  pub fn clear_changed();
}
