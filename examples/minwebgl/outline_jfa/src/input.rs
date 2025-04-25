use js_sys::wasm_bindgen::{
  JsCast,
  JsValue,
};
use web_sys::{
  KeyboardEvent,
  MouseEvent,
};

#[ wasm_bindgen ]
#[ derive( Default ) ]
pub struct InputState
{
  pub timestamp : Option< f64 >,
  pub viewport : Option< ( u32, u32 ) >,
  pub keyboard_events : Vec< KeyboardEvent >,
  pub mouse_event : Vec< MouseEvent >,
}

#[ wasm_bindgen ]
impl InputState
{
  #[ wasm_bindgen( constructor ) ]
  fn new() -> Self
  {
    Self::default()
  }

  #[ wasm_bindgen ]
  fn reset( &mut self )
  {
    *self = Self {
      ..Default::default()
    };
  }

  #[ wasm_bindgen ]
  fn set_timestamp(
    &mut self,
    timestamp : f64,
  )
  {
    self.timestamp = Some( timestamp );
  }

  #[ wasm_bindgen ]
  fn set_viewport(
    &mut self,
    width : u32,
    height : u32,
  )
  {
    self.viewport = Some( ( width, height ) );
  }

  #[ wasm_bindgen ]
  fn add_keyboard_event(
    &mut self,
    keyboard_event : JsValue,
  )
  {
    let Ok( keyboard_event ) = keyboard_event.dyn_into::< KeyboardEvent >()
    else
    {
      return;
    };
    self.keyboard_events.push( keyboard_event );
  }

  #[ wasm_bindgen ]
  fn add_mouse_event(
    &mut self,
    mouse_event : JsValue,
  )
  {
    let Ok( mouse_event ) = keyboard_event.dyn_into::< MouseEvent >()
    else
    {
      return;
    };
    self.mouse_events.push( mouse_event );
  }
}
