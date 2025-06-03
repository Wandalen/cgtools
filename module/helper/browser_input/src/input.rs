use minwebgl as min;
use min::{ JsCast as _, I32x2 };
use web_sys::{ wasm_bindgen::prelude::Closure, EventTarget, KeyboardEvent, MouseEvent };
use std::{ cell::{ Ref, RefCell }, rc::Rc };
use strum::EnumCount as _;
use crate::keyboard::KeyboardKey;
use crate::mouse::MouseButton;

#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum Action
{
  Press,
  Release,
}

#[ non_exhaustive ]
#[ derive( Debug, Clone, Copy, PartialEq ) ]
pub enum EventType
{
  KeyboardKey( KeyboardKey, Action ),
  MouseButton( MouseButton, Action ),
  MouseMovement( I32x2 ),
  // Wheel, // not supported yet
}

#[ derive( Debug, Clone, Copy, PartialEq ) ]
pub struct Event
{
  pub event_type : EventType,
  pub alt : bool,
  pub ctrl : bool,
  pub shift : bool,
}

#[ derive( Debug ) ]
struct State
{
  keyboard_keys : [ bool; KeyboardKey::COUNT ],
  mouse_buttons : [ bool; MouseButton::COUNT ],
  pointer_position : I32x2,
  scroll : i32,
}

impl State
{
  pub fn new() -> Self
  {
    Self
    {
      keyboard_keys : [ false; KeyboardKey::COUNT ],
      mouse_buttons : [ false; MouseButton::COUNT ],
      pointer_position : Default::default(),
      scroll : Default::default()
    }
  }
}

pub struct Input
{
  event_queue : Rc< RefCell< Vec< Event > > >,
  mousebutton_closure : Closure< dyn Fn( MouseEvent ) >,
  mousemove_closure : Closure< dyn Fn( MouseEvent ) >,
  keyboard_closure : Closure< dyn Fn( KeyboardEvent ) >,
  mouse_event_target : Option< EventTarget >,
  state : State,
}

impl Input
{
  pub fn new( mouse_event_target : Option< EventTarget > ) -> Self
  {
    // TODO:
    // pointer events instead of mouse events
    // customization of pointer coordinates eg client, screen, page, etc
    let event_queue = Rc::new( RefCell::new( Vec::< Event >::new() ) );

    let mousebutton_callback =
    {
      let event_queue = event_queue.clone();
      move | event : MouseEvent |
      {
        let button = MouseButton::from_button( event.button() );

        let action = if event.type_() == "mousedown" { Action::Press } else { Action::Release };

        let event_type = EventType::MouseButton( button, action );
        let alt = event.alt_key();
        let ctrl = event.ctrl_key();
        let shift = event.shift_key();

        let event = Event { event_type, alt, ctrl, shift };
        event_queue.borrow_mut().push( event );
      }
    };

    let mousemove_callback =
    {
      let event_queue = event_queue.clone();
      move | event : MouseEvent |
      {
        let position = I32x2::from_array( [ event.client_x(), event.client_y() ] );

        let event_type = EventType::MouseMovement( position );
        let alt = event.alt_key();
        let ctrl = event.ctrl_key();
        let shift = event.shift_key();

        let event = Event { event_type, alt, ctrl, shift };
        event_queue.borrow_mut().push( event );
      }
    };

    let keyboard_callback =
    {
      let event_queue = event_queue.clone();
      move | event : KeyboardEvent |
      {
        let code = KeyboardKey::from_code( &event.code() );
        let action = if event.type_() == "keydown" { Action::Press } else { Action::Release };

        let event_type = EventType::KeyboardKey( code, action );
        let alt = event.alt_key();
        let ctrl = event.ctrl_key();
        let shift = event.shift_key();

        let event = Event { event_type, alt, ctrl, shift };
        event_queue.borrow_mut().push( event );
      }
    };

    let mousebutton_closure = Closure::< dyn Fn( _ ) >::new( mousebutton_callback );
    let mousemove_closure = Closure::< dyn Fn( _ ) >::new( mousemove_callback );
    let keyboard_closure = Closure::< dyn Fn( _ ) >::new( keyboard_callback );

    let input = Self
    {
      event_queue,
      mousebutton_closure,
      mousemove_closure,
      keyboard_closure,
      mouse_event_target,
      state : State::new(),
    };

    let document = web_sys::window().unwrap().document().unwrap();

    document.add_event_listener_with_callback
    (
      "keydown",
      input.keyboard_closure.as_ref().unchecked_ref()
    ).unwrap();
    document.add_event_listener_with_callback
    (
      "keyup",
      input.keyboard_closure.as_ref().unchecked_ref()
    ).unwrap();

    let document = document.dyn_into().unwrap();
    let mouse_event_target = input.mouse_event_target.as_ref().unwrap_or( &document );
    mouse_event_target.add_event_listener_with_callback
    (
      "mousedown",
      input.mousebutton_closure.as_ref().unchecked_ref()
    ).unwrap();
    mouse_event_target.add_event_listener_with_callback
    (
      "mouseup",
      input.mousebutton_closure.as_ref().unchecked_ref()
    ).unwrap();
    mouse_event_target.add_event_listener_with_callback
    (
      "mousemove",
      input.mousemove_closure.as_ref().unchecked_ref()
    ).unwrap();

    input
  }

  pub fn event_queue( &self ) -> Ref< '_, Vec< Event > >
  {
    self.event_queue.borrow()
  }

  pub fn is_button_down( &self, button : MouseButton ) -> bool
  {
    self.state.mouse_buttons[ button as usize ]
  }

  pub fn is_key_down( &self, key : KeyboardKey ) -> bool
  {
    self.state.keyboard_keys[ key as usize ]
  }

  /// Returns pointer position on the page.
  pub fn pointer_position( &self ) -> I32x2
  {
    self.state.pointer_position
  }

  pub fn scroll( &self ) -> i32
  {
    self.state.scroll
  }

  /// Updates inner state considering event that are currently in the queue.
  pub fn update_state( &mut self )
  {
    for Event { event_type, .. } in self.event_queue.borrow().as_slice()
    {
      match event_type
      {
        EventType::KeyboardKey( keyboard_key, action ) =>
        {
          self.state.keyboard_keys[ *keyboard_key as usize ] = *action == Action::Press
        }
        EventType::MouseButton( mouse_button, action ) =>
        {
          self.state.mouse_buttons[ *mouse_button as usize ] = *action == Action::Press
        }
        EventType::MouseMovement( position ) => self.state.pointer_position = *position,
        // EventType::Wheel => todo!(),
      }
    }
  }

  /// Clear the event queue after you have processed events.
  pub fn clear_events( &mut self )
  {
    self.event_queue.borrow_mut().clear();
  }
}

impl Drop for Input
{
  fn drop( &mut self )
  {
    let document = web_sys::window().unwrap().document().unwrap();
    _ = document.remove_event_listener_with_callback( "keydown", self.keyboard_closure.as_ref().unchecked_ref() );
    _ = document.remove_event_listener_with_callback( "keyup", self.keyboard_closure.as_ref().unchecked_ref() );

    let document = document.dyn_into().unwrap();
    let mouse_event_target = self.mouse_event_target.as_ref().unwrap_or( &document );
    _ = mouse_event_target.remove_event_listener_with_callback( "mousedown", self.mousebutton_closure.as_ref().unchecked_ref() );
    _ = mouse_event_target.remove_event_listener_with_callback( "mouseup", self.mousebutton_closure.as_ref().unchecked_ref() );
    _ = mouse_event_target.remove_event_listener_with_callback( "mousemove", self.mousemove_closure.as_ref().unchecked_ref() );
    // _ = window.remove_event_listener_with_callback( "mousemove", ( *self.mousemove_closure ).as_ref().unchecked_ref() );
  }
}
