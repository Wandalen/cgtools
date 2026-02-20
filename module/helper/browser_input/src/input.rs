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
  /// A pointer button event: pointer id, position at the moment of press/release,
  /// the button, and the action. Covers both mouse clicks and touch contacts.
  PointerButton( i32, I32x2, MouseButton, Action ),
  /// A pointer movement event: pointer id and new position.
  /// Covers mouse movement and touch drag from any active finger.
  PointerMove( i32, I32x2 ),
  /// A mouse wheel scroll event, containing the scroll delta on each axis.
  Wheel( F64x3 ),
  /// A pointer contact was cancelled by the browser (e.g. interrupted by a system gesture or
  /// the pointer leaving the screen). Only the pointer id is reliable; position and button
  /// data from the underlying event are not guaranteed to be valid.
  PointerCancel( i32 ),
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
  /// The last known position of the most recently moved pointer.
  pointer_position : I32x2,
  /// The accumulated scroll value.
  scroll : F64x3,
  /// All currently active pointer contacts as `(pointer_id, position)` pairs.
  /// Updated on press, move, and release. Useful for multi-touch (e.g., pinch-to-zoom).
  /// On desktop this usually has at most one entry; on touch screens one per finger.
  active_pointers : Vec< ( i32, I32x2 ) >,
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
      scroll : Default::default(),
      active_pointers : Vec::new(),
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
  /// The closure handling pointer cancel events (browser cancels an active touch contact).
  pointercancel_closure : Closure< dyn Fn( PointerEvent ) >,
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
  /// Sets `touch-action: none` on the pointer event target so the browser does not intercept
  /// touch gestures (scroll, pinch-zoom) before they reach the application.
  /// Calls `setPointerCapture` on every `pointerdown` so drag events keep firing
  /// even when the pointer moves outside the target element.
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

    // Wrap in Rc<dyn Fn> so both the button and move closures can share the same extractor.
    let get_coords : Rc< dyn Fn( &PointerEvent ) -> I32x2 > = Rc::new( get_coords );

    let pointerbutton_callback =
    {
      let event_queue = event_queue.clone();
      let get_coords = get_coords.clone();
      move | event : PointerEvent |
      {
        let pointer_id = event.pointer_id();
        let pos = ( *get_coords )( &event );
        let button = MouseButton::from_button( event.button() );
        let action = if event.type_() == "pointerdown" { Action::Press } else { Action::Release };

        // On press, capture the pointer so drag events keep arriving even when the
        // finger or cursor moves outside the target element's bounding box.
        if action == Action::Press
        {
          if let Some( target ) = event.target()
          {
            if let Ok( element ) = target.dyn_into::< web_sys::Element >()
            {
              let _ = element.set_pointer_capture( pointer_id );
            }
          }
        }

        let event_type = EventType::PointerButton( pointer_id, pos, button, action );
        let alt = event.alt_key();
        let ctrl = event.ctrl_key();
        let shift = event.shift_key();
        event_queue.borrow_mut().push( Event { event_type, alt, ctrl, shift } );
      }
    };

    let pointercancel_callback =
    {
      let event_queue = event_queue.clone();
      move | event : PointerEvent |
      {
        // The Pointer Events spec does not guarantee valid coordinates or button data
        // for pointercancel; only the pointer_id is reliable.
        let pointer_id = event.pointer_id();
        let event_type = EventType::PointerCancel( pointer_id );
        let alt = event.alt_key();
        let ctrl = event.ctrl_key();
        let shift = event.shift_key();
        event_queue.borrow_mut().push( Event { event_type, alt, ctrl, shift } );
      }
    };

    let pointermove_callback =
    {
      let event_queue = event_queue.clone();
      move | event : PointerEvent |
      {
        let pointer_id = event.pointer_id();
        let position = ( *get_coords )( &event );
        let event_type = EventType::PointerMove( pointer_id, position );
        let alt = event.alt_key();
        let ctrl = event.ctrl_key();
        let shift = event.shift_key();
        event_queue.borrow_mut().push( Event { event_type, alt, ctrl, shift } );
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
        event_queue.borrow_mut().push( Event { event_type, alt, ctrl, shift } );
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
        event_queue.borrow_mut().push( Event { event_type, alt, ctrl, shift } );
      }
    };

    let pointerbutton_closure = Closure::< dyn Fn( _ ) >::new( pointerbutton_callback );
    let pointercancel_closure = Closure::< dyn Fn( _ ) >::new( pointercancel_callback );
    let pointermove_closure = Closure::< dyn Fn( _ ) >::new( pointermove_callback );
    let wheel_closure = Closure::< dyn Fn( _ ) >::new( wheel_callback );
    let keyboard_closure = Closure::< dyn Fn( _ ) >::new( keyboard_callback );

    let input = Self
    {
      event_queue,
      pointerbutton_closure,
      pointercancel_closure,
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

    // Prevent the browser from consuming touch gestures (scroll, pinch-zoom) on the target
    // so all pointer events reach the application uninterrupted.
    if let Some( target ) = input.pointer_event_target.as_ref()
    {
      if let Some( html_element ) = target.dyn_ref::< web_sys::HtmlElement >()
      {
        let _ = html_element.style().set_property( "touch-action", "none" );
      }
    }

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
      "pointercancel",
      input.pointercancel_closure.as_ref().unchecked_ref()
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

  /// Returns the last recorded pointer position (position of the most recently moved pointer).
  ///
  /// # Note
  /// On touch screens with multiple simultaneous contacts this value is non-deterministic —
  /// it reflects whichever finger sent the last `PointerMove` event. For multi-touch
  /// tracking use [`Input::active_pointers`] instead.
  pub fn pointer_position( &self ) -> I32x2
  {
    self.state.pointer_position
  }

  /// Returns a reference to the accumulated scroll delta.
  pub fn scroll( &self ) -> &F64x3
  {
    &self.state.scroll
  }

  /// Returns all currently active pointer contacts as a slice of `(pointer_id, position)` pairs.
  ///
  /// On desktop this typically contains at most one entry (the mouse while a button is held).
  /// On touch screens it contains one entry per finger currently in contact with the screen.
  /// Use this to implement multi-touch gestures such as pinch-to-zoom or two-finger pan.
  pub fn active_pointers( &self ) -> &[ ( i32, I32x2 ) ]
  {
    &self.state.active_pointers
  }

  /// Processes all pending events in the queue and updates the internal input state.
  pub fn update_state( &mut self )
  {
    apply_events_to_state( &mut self.state, &self.event_queue.borrow() );
  }

  /// Clears all events from the event queue.
  pub fn clear_events( &mut self )
  {
    self.event_queue.borrow_mut().clear();
  }
}

fn apply_events_to_state( state : &mut State, events : &[ Event ] )
{
  for Event { event_type, .. } in events
  {
    match event_type
    {
      EventType::KeyboardKey( keyboard_key, action ) =>
      {
        state.keyboard_keys[ *keyboard_key as usize ] = *action == Action::Press
      }
      EventType::PointerButton( pointer_id, pos, mouse_button, action ) =>
      {
        state.mouse_buttons[ *mouse_button as usize ] = *action == Action::Press;
        match action
        {
          Action::Press =>
          {
            if !state.active_pointers.iter().any( | ( id, _ ) | *id == *pointer_id )
            {
              state.active_pointers.push( ( *pointer_id, *pos ) );
            }
          }
          Action::Release =>
          {
            state.active_pointers.retain( | ( id, _ ) | *id != *pointer_id );
          }
        }
      }
      EventType::PointerMove( pointer_id, pos ) =>
      {
        state.pointer_position = *pos;
        if let Some( entry ) = state.active_pointers.iter_mut().find( | ( id, _ ) | *id == *pointer_id )
        {
          entry.1 = *pos;
        }
      }
      EventType::Wheel( delta ) => state.scroll += *delta,
      EventType::PointerCancel( pointer_id ) =>
      {
        state.active_pointers.retain( | ( id, _ ) | *id != *pointer_id );
        if state.active_pointers.is_empty()
        {
          state.mouse_buttons.fill( false );
        }
      }
    }
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
      "pointercancel",
      self.pointercancel_closure.as_ref().unchecked_ref()
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

#[ cfg( test ) ]
mod tests
{
  use super::*;

  fn ev( event_type : EventType ) -> Event
  {
    Event { event_type, alt : false, ctrl : false, shift : false }
  }

  fn p( x : i32, y : i32 ) -> I32x2
  {
    I32x2::from_array( [ x, y ] )
  }

  fn press( id : i32, x : i32, y : i32 ) -> Event
  {
    ev( EventType::PointerButton( id, p( x, y ), MouseButton::Main, Action::Press ) )
  }

  fn release( id : i32, x : i32, y : i32 ) -> Event
  {
    ev( EventType::PointerButton( id, p( x, y ), MouseButton::Main, Action::Release ) )
  }

  fn move_to( id : i32, x : i32, y : i32 ) -> Event
  {
    ev( EventType::PointerMove( id, p( x, y ) ) )
  }

  fn cancel( id : i32 ) -> Event
  {
    ev( EventType::PointerCancel( id ) )
  }

  #[ test ]
  fn press_adds_one_entry()
  {
    let mut state = State::new();
    apply_events_to_state( &mut state, &[ press( 1, 10, 20 ) ] );
    assert_eq!( state.active_pointers, [ ( 1, p( 10, 20 ) ) ] );
  }

  #[ test ]
  fn two_presses_add_two_entries()
  {
    let mut state = State::new();
    apply_events_to_state( &mut state, &[ press( 1, 10, 20 ), press( 2, 30, 40 ) ] );
    assert_eq!( state.active_pointers.len(), 2 );
    assert!( state.active_pointers.contains( &( 1, p( 10, 20 ) ) ) );
    assert!( state.active_pointers.contains( &( 2, p( 30, 40 ) ) ) );
  }

  #[ test ]
  fn move_updates_position()
  {
    let mut state = State::new();
    apply_events_to_state( &mut state, &[ press( 1, 10, 20 ), move_to( 1, 50, 60 ) ] );
    assert_eq!( state.active_pointers, [ ( 1, p( 50, 60 ) ) ] );
  }

  #[ test ]
  fn release_removes_entry()
  {
    let mut state = State::new();
    apply_events_to_state( &mut state, &[ press( 1, 10, 20 ), press( 2, 30, 40 ), release( 1, 10, 20 ) ] );
    assert_eq!( state.active_pointers, [ ( 2, p( 30, 40 ) ) ] );
  }

  #[ test ]
  fn cancel_removes_entry()
  {
    let mut state = State::new();
    apply_events_to_state( &mut state, &[ press( 1, 10, 20 ), press( 2, 30, 40 ), cancel( 2 ) ] );
    assert_eq!( state.active_pointers, [ ( 1, p( 10, 20 ) ) ] );
  }

  #[ test ]
  fn duplicate_press_is_idempotent()
  {
    let mut state = State::new();
    apply_events_to_state( &mut state, &[ press( 1, 10, 20 ), press( 1, 15, 25 ) ] );
    // Guard fires: second press for the same id does not add a duplicate entry.
    assert_eq!( state.active_pointers, [ ( 1, p( 10, 20 ) ) ] );
  }

  #[ test ]
  fn full_sequence_ends_empty()
  {
    let mut state = State::new();
    apply_events_to_state
    (
      &mut state,
      &
      [
        press( 1, 10, 20 ),
        press( 2, 30, 40 ),
        release( 1, 10, 20 ),
        cancel( 2 ),
      ]
    );
    assert!( state.active_pointers.is_empty() );
  }
}
