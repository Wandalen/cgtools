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
use std::{ cell::{ Ref, RefCell }, rc::Rc, fmt };
use strum::EnumCount as _;
use crate::keyboard::KeyboardKey;
use crate::mouse::MouseButton;

/// Error type for browser input initialization failures.
#[ derive( Debug ) ]
pub enum BrowserInputError
{
  /// Failed to access the browser's window object.
  WindowNotAvailable,
  /// Failed to access the document object.
  DocumentNotAvailable,
  /// Failed to cast document to EventTarget.
  DocumentCastFailed,
  /// Failed to add an event listener.
  AddEventListenerFailed( String ),
}

impl fmt::Display for BrowserInputError
{
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
    match self
    {
      Self::WindowNotAvailable => write!( f, "Browser window object not available" ),
      Self::DocumentNotAvailable => write!( f, "Document object not available" ),
      Self::DocumentCastFailed => write!( f, "Failed to cast document to EventTarget" ),
      Self::AddEventListenerFailed( event ) => write!( f, "Failed to add event listener for '{}'", event ),
    }
  }
}

impl std::error::Error for BrowserInputError {}

/// Maximum number of simultaneous active pointers to prevent unbounded memory growth.
/// 32 pointers is far more than any realistic multi-touch scenario (typically 10 fingers max).
const MAX_ACTIVE_POINTERS : usize = 32;

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
pub struct State
{
  /// The current pressed/released state of all keyboard keys.
  pub keyboard_keys : [ bool; KeyboardKey::COUNT ],
  /// The current pressed/released state of all mouse buttons.
  pub mouse_buttons : [ bool; MouseButton::COUNT ],
  /// The last known position of the most recently moved pointer.
  pub pointer_position : I32x2,
  /// The accumulated scroll value.
  pub scroll : F64x3,
  /// All currently active pointer contacts as `(pointer_id, position)` pairs.
  /// Updated on press, move, and release. Useful for multi-touch (e.g., pinch-to-zoom).
  /// On desktop this usually has at most one entry; on touch screens one per finger.
  pub active_pointers : Vec< ( i32, I32x2 ) >,
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
  ///
  /// # Errors
  /// Returns `BrowserInputError` if browser APIs are unavailable or event listener registration fails.
  pub fn new< F >
  (
    pointer_event_target : Option< EventTarget >,
    get_coords : F,
  ) -> Result< Self, BrowserInputError >
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

    let window = web_sys::window().ok_or( BrowserInputError::WindowNotAvailable )?;
    let document = window.document().ok_or( BrowserInputError::DocumentNotAvailable )?;

    document.add_event_listener_with_callback
    (
      "keydown",
      input.keyboard_closure.as_ref().unchecked_ref()
    ).map_err( | _ | BrowserInputError::AddEventListenerFailed( "keydown".to_string() ) )?;
    document.add_event_listener_with_callback
    (
      "keyup",
      input.keyboard_closure.as_ref().unchecked_ref()
    ).map_err( | _ | BrowserInputError::AddEventListenerFailed( "keyup".to_string() ) )?;

    let document = document.dyn_into().map_err( | _ | BrowserInputError::DocumentCastFailed )?;
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
    ).map_err( | _ | BrowserInputError::AddEventListenerFailed( "pointerdown".to_string() ) )?;
    pointer_event_target.add_event_listener_with_callback
    (
      "pointerup",
      input.pointerbutton_closure.as_ref().unchecked_ref()
    ).map_err( | _ | BrowserInputError::AddEventListenerFailed( "pointerup".to_string() ) )?;
    pointer_event_target.add_event_listener_with_callback
    (
      "pointercancel",
      input.pointercancel_closure.as_ref().unchecked_ref()
    ).map_err( | _ | BrowserInputError::AddEventListenerFailed( "pointercancel".to_string() ) )?;
    pointer_event_target.add_event_listener_with_callback
    (
      "pointermove",
      input.pointermove_closure.as_ref().unchecked_ref()
    ).map_err( | _ | BrowserInputError::AddEventListenerFailed( "pointermove".to_string() ) )?;
    pointer_event_target.add_event_listener_with_callback
    (
      "wheel",
      input.wheel_closure.as_ref().unchecked_ref()
    ).map_err( | _ | BrowserInputError::AddEventListenerFailed( "wheel".to_string() ) )?;

    Ok( input )
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
    self.state.scroll = Default::default();
  }
}

pub fn apply_events_to_state( state : &mut State, events : &[ Event ] )
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
              if state.active_pointers.len() < MAX_ACTIVE_POINTERS
              {
                state.active_pointers.push( ( *pointer_id, *pos ) );
              }
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
    let Some( window ) = web_sys::window() else { return };
    let Some( document ) = window.document() else { return };
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

    let Ok( document ) = document.dyn_into() else { return };
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
