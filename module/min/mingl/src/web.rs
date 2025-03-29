mod private
{
}

crate::mod_interface!
{

  own use ::wasm_bindgen;
  own use ::web_sys;
  own use ::js_sys;
  own use ::wasm_bindgen::JsValue;

  /// Main loop.
  layer exec_loop;
  /// Operations on canvas.
  layer canvas;
  /// Operations on DOM elements.
  layer dom;

  /// Utils for handling rust's futures
  #[ cfg( feature = "web_future"  ) ]
  layer future;

  /// File processing.
  #[ cfg( all( feature = "web_future", feature = "web_file" ) ) ]
  layer file;

  /// Web utilities related to different models
  #[ cfg( all( feature = "math", feature = "web_future", feature = "web_file" ) ) ]
  layer model;

  #[ cfg( feature = "web_log"  ) ]
  layer log;

}
