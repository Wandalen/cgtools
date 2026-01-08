use minwebgl as gl;
use gl::wasm_bindgen::{ self, prelude::* };
use serde::{ Serialize, Deserialize };

#[ derive( Debug, Clone, Default, Serialize, Deserialize ) ]
pub struct UiState
{
  pub gem : String,
  pub metal : String,
  pub ring : u32,
  #[ serde( rename = "gemCustomColor" ) ]
  pub gem_custom_color : Vec< f32 >,
  #[ serde( rename = "gemMultiplier" ) ]
  pub gem_multiplier : f32,
  #[ serde( rename = "metalCustomColor" ) ]
  pub metal_custom_color : Vec< f32 >,
  #[ serde( rename = "metalMultiplier" ) ]
  pub metal_multiplier : f32,
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

#[ wasm_bindgen ]
pub fn is_debug_mode() -> bool
{
  cfg!( debug_assertions )
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

  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "enableDebugControls" ) ]
  fn _enable_debug_controls();
}

pub fn enable_debug_controls_if_needed()
{
  #[ cfg( debug_assertions ) ]
  {
    gl::debug!( "Debug mode detected, enabling color controls from Rust" );
    _enable_debug_controls();
  }
}
