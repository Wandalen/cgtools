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
  #[ cfg( feature = "webFuture"  ) ]
  layer future;

  /// File processing.
  #[ cfg( all( feature = "webFuture", feature = "webFile" ) ) ]
  layer file;

  /// Web utilities related to different models 
  #[ cfg( all( feature = "webFuture", feature = "webFile" ) ) ]
  layer model;

  #[ cfg( feature = "webLog"  ) ]
  layer log;
}