use minwebgl as gl;
use gl::JsCast as _;
use web_sys::{ wasm_bindgen::prelude::Closure, EventTarget };

pub fn prevent_rightclick( target : EventTarget )
{
  let prevent_default = | e : web_sys::Event | e.prevent_default();
  let prevent_default = Closure::< dyn Fn( _ ) >::new( prevent_default );
  target.add_event_listener_with_callback
  (
    "contextmenu",
    prevent_default.as_ref().unchecked_ref()
  ).unwrap();
  prevent_default.forget();
}
