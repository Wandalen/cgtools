#![allow(unsafe_code)]
//! Custom controls module for filter parameters

use minwebgl::wasm_bindgen;
use minwebgl::JsValue;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/controls.js")]
extern "C"
{
  #[wasm_bindgen(js_name = clearControls)]
  pub fn clear_controls();

  #[wasm_bindgen(js_name = addSlider)]
  pub fn add_slider(label: &str, property: &str, value: f64, min: f64, max: f64, step: f64);

  #[wasm_bindgen(js_name = addDropdown)]
  pub fn add_dropdown(label: &str, property: &str, value: &str, options: &JsValue);

  #[wasm_bindgen(js_name = onChange)]
  pub fn on_change(callback: &JsValue);

  #[wasm_bindgen(js_name = getValues)]
  pub fn get_values() -> JsValue;

  #[wasm_bindgen(js_name = show)]
  pub fn show();

  #[wasm_bindgen(js_name = hide)]
  pub fn hide();
}
