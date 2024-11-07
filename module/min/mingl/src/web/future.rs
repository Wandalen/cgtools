/// Internal namespace.
mod private
{
  // use crate::*;

}

crate::mod_interface!
{

  orphan use ::wasm_bindgen_futures::
  {
    JsFuture,
    future_to_promise,
    spawn_local,
  };

}
