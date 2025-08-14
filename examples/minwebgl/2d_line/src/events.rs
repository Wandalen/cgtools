use std::rc::Rc;
use std::cell::RefCell;
use gl::wasm_bindgen::prelude::*;
use minwebgl as gl;


pub fn add_point_on_click
( 
  line : Rc< RefCell< line_tools::d2::Line > >,
  canvas : &gl::web_sys::HtmlCanvasElement
)
{
  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let callback : Closure< dyn Fn( _ ) > = Closure::new
  (
    {
      move | event : gl::web_sys::PointerEvent |
      {
        let mut x = event.screen_x() as f32;
        let mut y = height - event.screen_y() as f32;

        x = x * 2.0 - width;
        y = y * 2.0 - height;

        x /= 2.0;
        y /= 2.0;

        gl::info!( "{}|{}", x, y );
        line.borrow_mut().add_point( gl::F32x2::new( x, y ) );
      }
    }
  );

  let on_context_menu : Closure< dyn Fn( _ ) > = Closure::new
  (
    {
      move | e : gl::web_sys::PointerEvent |
      {
        e.prevent_default();
      }
    }
  );

  canvas.set_oncontextmenu( Some( on_context_menu.as_ref().unchecked_ref() ) );
  on_context_menu.forget();
  let _ = canvas.add_event_listener_with_callback( "click", callback.as_ref().unchecked_ref() ).unwrap();
  callback.forget();
}