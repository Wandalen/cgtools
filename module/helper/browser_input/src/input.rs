//! This module provides a comprehensive input handler for web applications,
//! capturing mouse, keyboard, and wheel events. It maintains an internal state
//! and an event queue for structured input processing in an application loop.

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

/// Represents the state of a button or key press.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum Action
{
  /// Indicates that a button or key has been pressed down.
  Press,
  /// Indicates that a button or key has been released.
  Release,
}

/// Enumerates the different types of input events that can be captured.
#[ non_exhaustive ]
#[ derive( Debug, Clone, Copy, PartialEq ) ]
pub enum EventType
{
  /// A keyboard key event, specifying the key and its action (press or release).
  KeyboardKey( KeyboardKey, Action ),
  /// A mouse button event, specifying the button and its action.
  MouseButton( MouseButton, Action ),
  /// A mouse movement event, containing the new pointer position.
  MouseMovement( I32x2 ),
  /// A mouse wheel scroll event, containing the scroll delta on each axis.
  Wheel( F64x3 ),
}

/// Represents a single, complete input event, including its type and any active modifier keys.
#[ derive( Debug, Clone, Copy, PartialEq ) ]
pub struct Event
{
  /// The specific type of event that occurred.
  pub event_type : EventType,
  /// True if the 'Alt' key was held down during the event.
  pub alt : bool,
  /// True if the 'Ctrl' key was held down during the event.
  pub ctrl : bool,
  /// True if the 'Shift' key was held down during the event.
  pub shift : bool,
}

/// Internal struct to hold the current state of all tracked inputs.
#[ derive( Debug ) ]
struct State
{
  /// The current pressed/released state of all keyboard keys.
  keyboard_keys : [ bool; KeyboardKey::COUNT ],
  /// The current pressed/released state of all mouse buttons.
  mouse_buttons : [ bool; MouseButton::COUNT ],
  /// The last known position of the mouse pointer.
  pointer_position : I32x2,
  /// The accumulated scroll value.
  scroll : F64x3,
}

impl State
{
  /// Creates a new `State` instance with default values.
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

/// A function to get pointer coordinates relative to the client area (the viewport).
pub static CLIENT : fn( &PointerEvent ) -> I32x2 = | event |
{
  I32x2::from_array( [ event.client_x(), event.client_y() ] )
};

/// A function to get pointer coordinates relative to the entire page, including scrolled-out areas.
pub static PAGE : fn( &PointerEvent ) -> I32x2 = | event |
{
  I32x2::from_array( [ event.page_x(), event.page_y() ] )
};

/// A function to get pointer coordinates relative to the user's screen.
pub static SCREEN : fn( &PointerEvent ) -> I32x2 = | event |
{
  I32x2::from_array( [ event.screen_x(), event.screen_y() ] )
};

/// The main input handler struct, responsible for setting up and managing browser event listeners.
pub struct Input
{
  /// A queue of events that have occurred since the last update.
  event_queue : Rc< RefCell< Vec< Event > > >,
  /// The closure handling pointer button down and up events.
  pointerbutton_closure : Closure< dyn Fn( PointerEvent ) >,
  /// The closure handling pointer movement events.
  pointermove_closure : Closure< dyn Fn( PointerEvent ) >,
  /// The closure handling keyboard down and up events.
  keyboard_closure : Closure< dyn Fn( KeyboardEvent ) >,
  /// The closure handling mouse wheel events.
  wheel_closure : Closure< dyn Fn( WheelEvent ) >,
  /// The specific DOM element to which pointer events are attached.
  pointer_event_target : Option< EventTarget >,
  /// The current state of inputs (e.g., which keys are down).
  state : State,
}

impl Input
{
  /// Creates a new `Input` handler and attaches event listeners to the document and an optional target.
  ///
  /// # Arguments
  /// * `pointer_event_target` - An optional `EventTarget` for pointer events. If `None`, the document is used.
  /// * `get_coords` - A function that specifies how to extract coordinates from a `PointerEvent`.
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

  /// Returns an immutable reference to the event queue.
  pub fn event_queue( &self ) -> Ref< '_, Vec< Event > >
  {
    self.event_queue.borrow()
  }

  /// Checks if a specific mouse button is currently held down.
  pub fn is_button_down( &self, button : MouseButton ) -> bool
  {
    self.state.mouse_buttons[ button as usize ]
  }

  /// Checks if a specific keyboard key is currently held down.
  pub fn is_key_down( &self, key : KeyboardKey ) -> bool
  {
    self.state.keyboard_keys[ key as usize ]
  }

  /// Returns the last recorded pointer position.
  pub fn pointer_position( &self ) -> I32x2
  {
    self.state.pointer_position
  }

  /// Returns a reference to the accumulated scroll delta.
  pub fn scroll( &self ) -> &F64x3
  {
    &self.state.scroll
  }

  /// Processes all pending events in the queue and updates the internal input state.
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

  /// Clears all events from the event queue.
  pub fn clear_events( &mut self )
  {
    self.event_queue.borrow_mut().clear();
  }
}

impl Drop for Input
{
  /// Cleans up by removing all attached event listeners from the DOM when the `Input` handler is dropped.
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
