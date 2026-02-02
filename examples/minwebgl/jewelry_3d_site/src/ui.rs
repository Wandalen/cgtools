use minwebgl as gl;
use gl::wasm_bindgen::{ self, prelude::* };
use serde::{ Serialize, Deserialize };

/// Shared between JS and Rust ui state of configurator.
/// Used to change gem, metal, ring type etc
#[ derive( Debug, Clone, Default, Serialize, Deserialize ) ]
pub struct UiState
{
  /// Current used gem type
  pub gem : String,
  /// Current used metal type
  pub metal : String,
  /// Current used ring index
  pub ring : u32,
  /// Configurator work state (configurator/hero)
  pub state : String,
  /// Camera eye
  #[ serde( rename = "position" ) ]
  pub eye : [ f32; 3 ],
  /// Transition animation in process
  #[ serde( rename = "transitionAnimationEnabled" ) ]
  pub transition_animation_enabled : bool,
  /// Camera center
  #[ serde( rename = "target" ) ]
  pub center : [ f32; 3 ],
  /// Debug mode custom gem color
  #[ serde( rename = "gemCustomColor" ) ]
  pub gem_custom_color : Vec< f32 >,
  /// Debug mode custom gem multiplier for making color brighter/darker
  #[ serde( rename = "gemMultiplier" ) ]
  pub gem_multiplier : f32,
  /// Debug mode custom metal color
  #[ serde( rename = "metalCustomColor" ) ]
  pub metal_custom_color : Vec< f32 >,
  /// Debug mode custom metal multiplier for making color brighter/darker
  #[ serde( rename = "metalMultiplier" ) ]
  pub metal_multiplier : f32,
  /// List of changed [`UiState`] related elements
  pub changed : Vec< String >
}

/// Retrieves UiState from JS
pub fn get_ui_state() -> Option< UiState >
{
  serde_wasm_bindgen::from_value::< UiState >( _get_ui_state() ).ok()
}

/// Check if ui state is changed
pub fn is_changed() -> bool
{
  _is_changed().as_bool().unwrap()
}

/// Check if debug mode is enabled ( on JS side )
#[ wasm_bindgen ]
pub fn is_debug_mode() -> bool
{
  cfg!( debug_assertions )
}

#[ wasm_bindgen( module = "/index.js" ) ]
extern "C"
{
  /// Signal that renderer is loaded
  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "setRendererLoaded" ) ]
  pub fn set_renderer_loaded();

  /// Retrieves UiState from JS
  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "getUiState" ) ]
  fn _get_ui_state() -> JsValue;

  /// Check if ui state is changed
  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "isChanged" ) ]
  fn _is_changed() -> JsValue;

  /// Clear list of changed elements
  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "clearChanged" ) ]
  pub fn clear_changed();

  /// Enables debug mode on Rust side ( used for custom gem and metal colors )
  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "enableDebugControls" ) ]
  fn _enable_debug_controls();

  /// Updates UI selection highlight for gem or metal
  #[ allow( unsafe_code ) ]
  #[ wasm_bindgen( js_name = "updateSelectionHighlight" ) ]
  pub fn update_selection_highlight( selection_type : &str, value : &str );
}

/// Checks if debug is enabled if true enables debug on JS side
pub fn enable_debug_controls_if_needed()
{
  #[ cfg( debug_assertions ) ]
  {
    gl::debug!( "Debug mode detected, enabling color controls from Rust" );
    _enable_debug_controls();
  }
}
