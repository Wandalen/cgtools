use minwebgl as min;
use min::{ JsCast as _, I32x2, F64x3 };
use web_sys::
{
  wasm_bindgen::prelude::Closure,
  EventTarget,
  KeyboardEvent,
  PointerEvent,
  WheelEvent,
};
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
  Wheel( F64x3 ),
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
  scroll : F64x3,
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

pub static CLIENT : fn( &PointerEvent ) -> I32x2 = | event |
{
  I32x2::from_array( [ event.client_x(), event.client_y() ] )
};

pub static PAGE : fn( &PointerEvent ) -> I32x2 = | event |
{
  I32x2::from_array( [ event.page_x(), event.page_y() ] )
};

pub static SCREEN : fn( &PointerEvent ) -> I32x2 = | event |
{
  I32x2::from_array( [ event.screen_x(), event.screen_y() ] )
};

pub struct Input
{
  event_queue : Rc< RefCell< Vec< Event > > >,
  pointerbutton_closure : Closure< dyn Fn( PointerEvent ) >,
  pointermove_closure : Closure< dyn Fn( PointerEvent ) >,
  keyboard_closure : Closure< dyn Fn( KeyboardEvent ) >,
  wheel_closure : Closure< dyn Fn( WheelEvent ) >,
  pointer_event_target : Option< EventTarget >,
  state : State,
}

impl Input
{
  pub fn new< F >
  (
    pointer_event_target : Option< EventTarget >,
    get_coords : F,
  ) -> Self
  where
    F : Fn( &PointerEvent ) -> I32x2 + 'static
  {
    let event_queue = Rc::new( RefCell::new( Vec::< Event >::new() ) );

    let pointerbutton_callback =
    {
      let event_queue = event_queue.clone();
      move | event : PointerEvent |
      {
        let button = MouseButton::from_button( event.button() );

        let action = if event.type_() == "pointerdown" { Action::Press } else { Action::Release };

        let event_type = EventType::MouseButton( button, action );
        let alt = event.alt_key();
        let ctrl = event.ctrl_key();
        let shift = event.shift_key();

        let event = Event { event_type, alt, ctrl, shift };
        event_queue.borrow_mut().push( event );
      }
    };

    let pointermove_callback =
    {
      let event_queue = event_queue.clone();
      move | event : PointerEvent |
      {
        let position = get_coords( &event );

        let event_type = EventType::MouseMovement( position );
        let alt = event.alt_key();
        let ctrl = event.ctrl_key();
        let shift = event.shift_key();

        let event = Event { event_type, alt, ctrl, shift };
        event_queue.borrow_mut().push( event );
      }
    };

    let wheel_callback =
    {
      let event_queue = event_queue.clone();
      move | event : WheelEvent |
      {
        let delta_x = event.delta_x();
        let delta_y = event.delta_y();
        let delta_z = event.delta_z();
        let event_type = EventType::Wheel( F64x3::new( delta_x, delta_y, delta_z ) );
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

    let pointerbutton_closure = Closure::< dyn Fn( _ ) >::new( pointerbutton_callback );
    let pointermove_closure = Closure::< dyn Fn( _ ) >::new( pointermove_callback );
    let wheel_closure = Closure::< dyn Fn( _ ) >::new( wheel_callback );
    let keyboard_closure = Closure::< dyn Fn( _ ) >::new( keyboard_callback );

    let input = Self
    {
      event_queue,
      pointerbutton_closure,
      pointermove_closure,
      keyboard_closure,
      wheel_closure,
      pointer_event_target,
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
    let pointer_event_target = input.pointer_event_target.as_ref().unwrap_or( &document );
    pointer_event_target.add_event_listener_with_callback
    (
      "pointerdown",
      input.pointerbutton_closure.as_ref().unchecked_ref()
    ).unwrap();
    pointer_event_target.add_event_listener_with_callback
    (
      "pointerup",
      input.pointerbutton_closure.as_ref().unchecked_ref()
    ).unwrap();
    pointer_event_target.add_event_listener_with_callback
    (
      "pointermove",
      input.pointermove_closure.as_ref().unchecked_ref()
    ).unwrap();
    pointer_event_target.add_event_listener_with_callback
    (
      "wheel",
      input.wheel_closure.as_ref().unchecked_ref()
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

  pub fn scroll( &self ) -> &F64x3
  {
    &self.state.scroll
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
        EventType::Wheel( delta ) => self.state.scroll += *delta,
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
  // Unsubscribe all event listeners
  fn drop( &mut self )
  {
    let document = web_sys::window().unwrap().document().unwrap();
    _ = document.remove_event_listener_with_callback
    (
      "keydown",
      self.keyboard_closure.as_ref().unchecked_ref()
    );
    _ = document.remove_event_listener_with_callback
    (
      "keyup",
      self.keyboard_closure.as_ref().unchecked_ref()
    );

    let document = document.dyn_into().unwrap();
    let pointer_event_target = self.pointer_event_target.as_ref().unwrap_or( &document );
    _ = pointer_event_target.remove_event_listener_with_callback
    (
      "pointerdown",
      self.pointerbutton_closure.as_ref().unchecked_ref()
    );
    _ = pointer_event_target.remove_event_listener_with_callback
    (
      "pointerup",
      self.pointerbutton_closure.as_ref().unchecked_ref()
    );
    _ = pointer_event_target.remove_event_listener_with_callback
    (
      "pointermove",
      self.pointermove_closure.as_ref().unchecked_ref()
    );
    _ = pointer_event_target.remove_event_listener_with_callback
    (
      "wheel",
      self.wheel_closure.as_ref().unchecked_ref()
    );
  }
}
