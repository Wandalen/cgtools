use std::rc::Rc;
use std::cell::RefCell;
use minwebgl as gl;


pub fn update
( 
  canvas : &gl::web_sys::HtmlCanvasElement,
  input : &mut browser_input::Input
)
{
  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  input.update_state();
  let mouse_pos = input.pointer_position();

  for browser_input::Event { event_type, .. } in input.event_queue().iter()
  {
    if let browser_input::EventType::MouseButton
    ( 
      browser_input::mouse::MouseButton::Main, 
      browser_input::Action::Press 
    ) = *event_type
    {
      let mut x = mouse_pos.0[ 0 ] as f32;
      let mut y = height - mouse_pos.0[ 1 ] as f32;

      x = x * 2.0 - width;
      y = y * 2.0 - height;

      x /= 2.0;
      y /= 2.0;

    }
  }

  input.clear_events();
}