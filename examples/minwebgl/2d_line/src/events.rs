#![ allow( clippy::std_instead_of_alloc ) ]
#![ allow( clippy::needless_pass_by_value ) ]

use std::rc::Rc;
use std::cell::RefCell;
use minwebgl as gl;


pub fn update
(
  line : Rc< RefCell< line_tools::d2::Line > >,
  canvas : &gl::web_sys::HtmlCanvasElement,
  input : &mut browser_input::Input
)
{
  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  input.update_state();

  for browser_input::Event { event_type, .. } in input.event_queue().iter()
  {
    if let browser_input::EventType::PointerButton
    (
      _,
      pos,
      browser_input::mouse::MouseButton::Main,
      browser_input::Action::Press
    ) = *event_type
    {
      let mut x = pos.0[ 0 ] as f32;
      let mut y = height - pos.0[ 1 ] as f32;

      x = x * 2.0 - width;
      y = y * 2.0 - height;

      x /= 2.0;
      y /= 2.0;

      line.borrow_mut().add_point( gl::F32x2::new( x, y ) );
    }
  }

  input.clear_events();
}
