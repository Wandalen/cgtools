//! This module provides a comprehensive input handler for web applications,
//! capturing mouse, keyboard, and wheel events. It maintains an internal state
//! and an event queue for structured input processing in an application loop.

use ndarray_cg::{ I32x2, F64x3 };
use web_sys::
{
  wasm_bindgen::{ JsCast as _, prelude::Closure },
  Event as DomEvent,
  EventTarget,
  KeyboardEvent,
  PointerEvent,
  WheelEvent,
};
use std::cell::{ Cell, Ref, RefCell };
use alloc::{ rc::Rc, fmt };
use strum::EnumCount as _;
use crate::keyboard::KeyboardKey;
use crate::mouse::MouseButton;

/// Error type for browser input initialization failures.
#[ non_exhaustive ]
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
  #[ inline ]
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
    match self
    {
      Self::WindowNotAvailable => write!( f, "Browser window object not available" ),
      Self::DocumentNotAvailable => write!( f, "Document object not available" ),
      Self::DocumentCastFailed => write!( f, "Failed to cast document to EventTarget" ),
      Self::AddEventListenerFailed( event ) => write!( f, "Failed to add event listener for '{event}'" ),
    }
  }
}

impl std::error::Error for BrowserInputError {}

/// Maximum number of simultaneous active pointers to prevent unbounded memory growth.
/// 32 pointers is far more than any realistic multi-touch scenario (typically 10 fingers max).
const MAX_ACTIVE_POINTERS : usize = 32;

/// Represents the state of a button or key press.
#[ non_exhaustive ]
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum Action
{
  /// Indicates that a button or key has been pressed down.
  Press,
  /// Indicates that a button or key has been released.
  Release,
}

/// The kind of physical input device that produced a pointer event.
///
/// Mirrors the `pointerType` field of the DOM `PointerEvent` interface. Useful
/// for branching UI behaviour on devices where touch and mouse coexist — e.g.
/// hiding a cursor-follow preview when no finger is on the screen.
#[ non_exhaustive ]
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Default ) ]
pub enum PointerType
{
  /// A mouse, trackpad, or other indirect pointing device.
  Mouse,
  /// A finger on a touchscreen.
  Touch,
  /// A stylus or active pen.
  Pen,
  /// No pointer events have been seen yet, or the device reported an
  /// unrecognised `pointerType`. These two cases are intentionally
  /// indistinguishable from the caller's perspective.
  #[ default ]
  Unknown,
}

impl From< &str > for PointerType
{
  /// Convert from the DOM `PointerEvent.pointerType` string.
  #[ inline ]
  fn from( s : &str ) -> Self
  {
    match s
    {
      "mouse" => Self::Mouse,
      "touch" => Self::Touch,
      "pen"   => Self::Pen,
      _       => Self::Unknown, // includes "" — spec-defined for when device type cannot be determined
    }
  }
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
  /// The window lost focus (`blur`) or the page became hidden (`visibilitychange`) --
  /// e.g. the user alt-tabbed away, switched browser tabs, or minimized the window.
  /// No further `keyup`/`pointerup` is guaranteed to ever arrive for whatever was
  /// physically held at that moment, so every tracked key/button/pointer is force-released.
  FocusLost,
}

/// Represents a single, complete input event, including its type and any active modifier keys.
#[ non_exhaustive ]
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

impl Event
{
  /// Creates a new `Event` from its type and the modifier keys held during it.
  #[ inline ]
  #[ must_use ]
  pub fn new( event_type : EventType, alt : bool, ctrl : bool, shift : bool ) -> Self
  {
    Self { event_type, alt, ctrl, shift }
  }
}

/// Internal struct to hold the current state of all tracked inputs.
#[ non_exhaustive ]
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
  /// Internal bookkeeping: which buttons each currently-active pointer id holds,
  /// as a bitmask (bit `n` set means the `MouseButton` whose `as usize` is `n`
  /// is held by that pointer). `mouse_buttons` and `active_pointers` are the
  /// public derived view, unioned/gated from this per-pointer source of truth --
  /// not exposed directly since it is a bookkeeping detail, not a queryable input.
  /// Insertion of a new (not-yet-tracked) pointer id is capped at
  /// `MAX_ACTIVE_POINTERS`, mirroring `active_pointers`' own cap (Fix(BUG-212)).
  held_buttons : std::collections::HashMap< i32, u32 >,
  /// Internal bookkeeping: per-pointer count of currently-held *distinct real*
  /// mouse buttons that all fell back to `MouseButton::Unknown` (any DOM
  /// `button` value outside 0-4 aliases to this one variant). A single bit in
  /// `held_buttons` cannot distinguish "one Unknown button held" from "two
  /// different Unknown buttons held" -- this count lets `MouseButton::Unknown`'s
  /// bit stay set until every aliased button this pointer holds has released
  /// (Fix(BUG-213)). Same insertion cap as `held_buttons`.
  unknown_button_counts : std::collections::HashMap< i32, u32 >,
  /// Internal bookkeeping: count of currently-held keys that fell back to
  /// `KeyboardKey::Unidentified` (any `code` string not mapped to a known
  /// variant aliases to this one). Same rationale as `unknown_button_counts`,
  /// but global rather than per-pointer since keyboard events carry no pointer
  /// id (Fix(BUG-213)).
  unidentified_key_hold_count : u32,
}

impl State
{
  /// Creates a new `State` with all inputs in their default unpressed/zero state.
  #[ inline ]
  #[ must_use ]
  pub fn new() -> Self
  {
    Self
    {
      keyboard_keys : [ false; KeyboardKey::COUNT ],
      mouse_buttons : [ false; MouseButton::COUNT ],
      pointer_position : I32x2::default(),
      scroll : F64x3::default(),
      active_pointers : Vec::new(),
      held_buttons : std::collections::HashMap::new(),
      unknown_button_counts : std::collections::HashMap::new(),
      unidentified_key_hold_count : 0,
    }
  }
}

impl Default for State
{
  #[ inline ]
  fn default() -> Self
  {
    Self::new()
  }
}

/// A function to get pointer coordinates relative to the client area (the viewport).
// Browser pointer coordinates are conceptually integer pixel values; truncation is not expected in practice.
// Fix(BUG-053): `PointerEvent` derefs to `MouseEvent`, whose `client_x`/`client_y` return `i32`
// or `f64` depending on `web_sys_unstable_apis` (see minwebgl/src/texture/d2.rs); `as i32` is a
// real truncating cast in the `f64` case and a same-type identity cast clippy calls
// "unnecessary" in the `i32` case — both are the same source line.
#[ allow( clippy::unnecessary_cast, reason = "cfg-dependent per the Fix(BUG-053) note above — the cast is real under the web_sys_unstable_apis f64 signature, so expect would be unfulfilled there" ) ]
pub static CLIENT : fn( &PointerEvent ) -> I32x2 = | event |
{
  I32x2::from_array( [ event.client_x() as i32, event.client_y() as i32 ] )
};

/// A function to get pointer coordinates relative to the entire page, including scrolled-out areas.
// Browser pointer coordinates are conceptually integer pixel values; truncation is not expected in practice.
// Fix(BUG-053): `PointerEvent` derefs to `MouseEvent`, whose `page_x`/`page_y` return `i32` or
// `f64` depending on `web_sys_unstable_apis` (see minwebgl/src/texture/d2.rs); `as i32` is a
// real truncating cast in the `f64` case and a same-type identity cast clippy calls
// "unnecessary" in the `i32` case — both are the same source line.
#[ allow( clippy::unnecessary_cast, reason = "cfg-dependent per the Fix(BUG-053) note above — the cast is real under the web_sys_unstable_apis f64 signature, so expect would be unfulfilled there" ) ]
pub static PAGE : fn( &PointerEvent ) -> I32x2 = | event |
{
  I32x2::from_array( [ event.page_x() as i32, event.page_y() as i32 ] )
};

/// A function to get pointer coordinates relative to the user's screen.
// Browser pointer coordinates are conceptually integer pixel values; truncation is not expected in practice.
// Fix(BUG-053): `PointerEvent` derefs to `MouseEvent`, whose `screen_x`/`screen_y` return `i32`
// or `f64` depending on `web_sys_unstable_apis` (see minwebgl/src/texture/d2.rs); `as i32` is a
// real truncating cast in the `f64` case and a same-type identity cast clippy calls
// "unnecessary" in the `i32` case — both are the same source line.
#[ allow( clippy::unnecessary_cast, reason = "cfg-dependent per the Fix(BUG-053) note above — the cast is real under the web_sys_unstable_apis f64 signature, so expect would be unfulfilled there" ) ]
pub static SCREEN : fn( &PointerEvent ) -> I32x2 = | event |
{
  I32x2::from_array( [ event.screen_x() as i32, event.screen_y() as i32 ] )
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
  /// The closure handling focus-loss events (`blur` on the window, `visibilitychange` on the
  /// document) -- see [`EventType::FocusLost`] and its `Fix(BUG-214)` doc comment.
  focus_lost_closure : Closure< dyn Fn( DomEvent ) >,
  /// The specific DOM element to which pointer events are attached.
  pointer_event_target : Option< EventTarget >,
  /// The current state of inputs (e.g., which keys are down).
  state : State,
  /// Type of the most recently observed pointer event. Shared with the pointer
  /// callbacks via [`Rc<Cell>`] so they can write it without going through the
  /// event queue — keeps the enum `EventType` API stable.
  last_pointer_type : Rc< Cell< PointerType > >,
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
  #[ inline ]
  #[ expect( clippy::too_many_lines, reason = "sets up 6 independent event closures sharing captured state ( event_queue, get_coords, last_pointer_type ) via Rc::clone; splitting each into its own function would thread that shared state through extra parameters for no behavioral change" ) ]
  pub fn new< F >
  (
    pointer_event_target : Option< EventTarget >,
    get_coords : F,
  ) -> Result< Self, BrowserInputError >
  where
    F : Fn( &PointerEvent ) -> I32x2 + 'static
  {
    let event_queue = Rc::new( RefCell::new( Vec::< Event >::new() ) );
    let last_pointer_type = Rc::new( Cell::new( PointerType::default() ) );

    // Wrap in Rc<dyn Fn> so both the button and move closures can share the same extractor.
    let get_coords : Rc< dyn Fn( &PointerEvent ) -> I32x2 > = Rc::new( get_coords );

    let pointerbutton_callback =
    {
      let event_queue = event_queue.clone();
      let get_coords = get_coords.clone();
      let last_pointer_type = last_pointer_type.clone();
      move | event : PointerEvent |
      {
        let pointer_id = event.pointer_id();
        let pos = ( *get_coords )( &event );
        let button = MouseButton::from_button( event.button() );
        let action = if event.type_() == "pointerdown" { Action::Press } else { Action::Release };
        last_pointer_type.set( PointerType::from( event.pointer_type().as_str() ) );

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
      let last_pointer_type = last_pointer_type.clone();
      move | event : PointerEvent |
      {
        // The Pointer Events spec does not guarantee valid coordinates or button data
        // for pointercancel.
        let pointer_id = event.pointer_id();
        last_pointer_type.set( PointerType::from( event.pointer_type().as_str() ) );
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
      let last_pointer_type = last_pointer_type.clone();
      move | event : PointerEvent |
      {
        let pointer_id = event.pointer_id();
        let position = ( *get_coords )( &event );
        last_pointer_type.set( PointerType::from( event.pointer_type().as_str() ) );
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
        // Fix(BUG-213): OS-level auto-repeat re-fires `keydown` for an already-held key
        // without a matching `keyup` in between. Left unfiltered, this would corrupt the
        // Unidentified-key hold-count fix in `events_apply_to_state` (each repeat would
        // increment the count again, requiring that many releases to actually clear it).
        // No legitimate signal is lost: every mapped key already tracks "held" as a level,
        // not an edge, so repeat events carry no information beyond the initial press.
        if event.repeat()
        {
          return;
        }
        let code = KeyboardKey::from( event.code().as_str() );
        let action = if event.type_() == "keydown" { Action::Press } else { Action::Release };
        let event_type = EventType::KeyboardKey( code, action );
        let alt = event.alt_key();
        let ctrl = event.ctrl_key();
        let shift = event.shift_key();
        event_queue.borrow_mut().push( Event { event_type, alt, ctrl, shift } );
      }
    };

    let focus_lost_callback =
    {
      let event_queue = event_queue.clone();
      move | _event : DomEvent |
      {
        // Fix(BUG-214): fires on `blur` (window loses OS focus) and `visibilitychange`
        // (tab hidden) alike -- both share the same "we will not receive matching release
        // events for whatever is currently held" consequence, so both push the same
        // FocusLost event. `visibilitychange` fires for both directions (hidden and
        // visible); only the queued event's effect (a full state reset) matters, and
        // resetting an already-empty state on the "became visible" case is a harmless no-op.
        let event_type = EventType::FocusLost;
        event_queue.borrow_mut().push( Event { event_type, alt : false, ctrl : false, shift : false } );
      }
    };

    let pointerbutton_closure = Closure::< dyn Fn( _ ) >::new( pointerbutton_callback );
    let pointercancel_closure = Closure::< dyn Fn( _ ) >::new( pointercancel_callback );
    let pointermove_closure = Closure::< dyn Fn( _ ) >::new( pointermove_callback );
    let wheel_closure = Closure::< dyn Fn( _ ) >::new( wheel_callback );
    let keyboard_closure = Closure::< dyn Fn( _ ) >::new( keyboard_callback );
    let focus_lost_closure = Closure::< dyn Fn( _ ) >::new( focus_lost_callback );

    let input = Self
    {
      event_queue,
      pointerbutton_closure,
      pointercancel_closure,
      pointermove_closure,
      keyboard_closure,
      wheel_closure,
      focus_lost_closure,
      pointer_event_target,
      state : State::new(),
      last_pointer_type,
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
    window.add_event_listener_with_callback
    (
      "blur",
      input.focus_lost_closure.as_ref().unchecked_ref()
    ).map_err( | _ | BrowserInputError::AddEventListenerFailed( "blur".to_string() ) )?;
    document.add_event_listener_with_callback
    (
      "visibilitychange",
      input.focus_lost_closure.as_ref().unchecked_ref()
    ).map_err( | _ | BrowserInputError::AddEventListenerFailed( "visibilitychange".to_string() ) )?;

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
  #[ inline ]
  #[ must_use ]
  pub fn event_queue( &self ) -> Ref< '_, Vec< Event > >
  {
    self.event_queue.borrow()
  }

  /// Checks if a specific mouse button is currently held down.
  #[ inline ]
  #[ must_use ]
  pub fn is_button_down( &self, button : MouseButton ) -> bool
  {
    self.state.mouse_buttons[ button as usize ]
  }

  /// Checks if a specific keyboard key is currently held down.
  #[ inline ]
  #[ must_use ]
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
  #[ inline ]
  #[ must_use ]
  pub fn pointer_position( &self ) -> I32x2
  {
    self.state.pointer_position
  }

  /// Returns a reference to the accumulated scroll delta.
  #[ inline ]
  #[ must_use ]
  pub fn scroll( &self ) -> &F64x3
  {
    &self.state.scroll
  }

  /// Returns the [`PointerType`] of the most recently observed pointer event.
  ///
  /// Returns [`PointerType::Unknown`] before the first pointer event fires, or
  /// when the browser reports an unrecognised `pointerType` string.
  /// Useful for adapting UI to the active input modality on hybrid devices —
  /// e.g. switching cursor-follow behaviour once the user switches from mouse
  /// to touch.
  ///
  /// Note: this value does not reset when pointers are released; after a finger
  /// lifts, it persists as `Touch` until the next pointer event. To check
  /// whether any pointer is currently active, use [`Input::active_pointers`].
  ///
  /// # Test coverage
  /// The string-to-variant mapping is covered by the `From< &str >` pins in
  /// `tests/pointer_type_test.rs`.
  /// End-to-end wiring through DOM callbacks requires a `wasm-bindgen-test` environment
  /// and is not covered on the native target.
  #[ inline ]
  #[ must_use ]
  pub fn last_pointer_type( &self ) -> PointerType
  {
    self.last_pointer_type.get()
  }

  /// Returns all currently active pointer contacts as a slice of `(pointer_id, position)` pairs.
  ///
  /// On desktop this typically contains at most one entry (the mouse while a button is held).
  /// On touch screens it contains one entry per finger currently in contact with the screen.
  /// Use this to implement multi-touch gestures such as pinch-to-zoom or two-finger pan.
  #[ inline ]
  #[ must_use ]
  pub fn active_pointers( &self ) -> &[ ( i32, I32x2 ) ]
  {
    &self.state.active_pointers
  }

  /// Processes all pending events in the queue and updates the internal input state.
  #[ inline ]
  pub fn state_update( &mut self )
  {
    events_apply_to_state( &mut self.state, &self.event_queue.borrow() );
  }

  /// Clears all events from the event queue.
  #[ inline ]
  pub fn events_clear( &mut self )
  {
    self.event_queue.borrow_mut().clear();
    self.state.scroll = F64x3::default();
  }
}

/// Applies a slice of events to the given state, updating it accordingly.
#[ inline ]
#[ expect( clippy::too_many_lines, reason = "one match dispatching per-EventType state updates across 6 variants ( 3 of which carry independently-necessary alias/cap bookkeeping for BUG-212/BUG-213/BUG-214 ); splitting arms into separate functions would fragment one conceptual state machine over &mut State for no behavioral change" ) ]
pub fn events_apply_to_state( state : &mut State, events : &[ Event ] )
{
  for Event { event_type, .. } in events
  {
    match event_type
    {
      EventType::KeyboardKey( keyboard_key, action ) =>
      {
        // Fix(BUG-213)
        // Root cause: any `code` string not mapped to a known variant aliases to
        // `KeyboardKey::Unidentified` -- a flat last-writer-wins bool cannot tell
        // two DIFFERENT physical keys sharing that one fallback apart, so
        // releasing one falsely cleared the other's still-held state.
        // Pitfall: only reachable with two simultaneously-held exotic/unmapped
        // keys -- invisible for every one of the individually-mapped keys, which
        // have no aliasing and need no counting.
        if *keyboard_key == KeyboardKey::Unidentified
        {
          match action
          {
            Action::Press => state.unidentified_key_hold_count += 1,
            Action::Release =>
              state.unidentified_key_hold_count = state.unidentified_key_hold_count.saturating_sub( 1 ),
          }
          state.keyboard_keys[ *keyboard_key as usize ] = state.unidentified_key_hold_count > 0;
        }
        else
        {
          state.keyboard_keys[ *keyboard_key as usize ] = *action == Action::Press;
        }
      }
      EventType::PointerButton( pointer_id, pos, mouse_button, action ) =>
      {
        let bit = 1u32 << ( *mouse_button as u32 );
        // Fix(BUG-130)
        // Root cause: `mouse_buttons` was a flat last-writer-wins toggle keyed only
        // by button, and `active_pointers` evicted a pointer id on ANY release --
        // both assume exactly one button is ever in play per pointer at a time.
        // That is true for a single touch contact (whose `button` is always
        // `Main` per the Pointer Events spec) but false for two simultaneous
        // pointers sharing a button value, or one physical mouse holding two
        // buttons under one shared `pointer_id`. `held_buttons` now tracks each
        // pointer's own held-button bitmask so both derived views only change
        // once that pointer's actual contribution changes.
        // Pitfall: global input state that is "set" per event instead of
        // "derived from all current sources" silently breaks the instant two
        // sources can overlap -- verify against the *simultaneous* case, not
        // just sequential press/release pairs.
        match action
        {
          Action::Press =>
          {
            // Fix(BUG-212)
            // Root cause: `held_buttons` inserted a new pointer id unconditionally,
            // while `active_pointers` already gated new insertions behind
            // `MAX_ACTIVE_POINTERS` -- a source sending Press events under
            // ever-new synthetic pointer ids (with no matching Release) grew
            // `held_buttons` without bound even though `active_pointers` stayed
            // capped. Gate `held_buttons` (and the Fix(BUG-213) counter below)
            // behind the identical "already tracked or under the cap" check.
            // Pitfall: two collections meant to track the same conceptual set
            // (which pointer ids are currently active) drifted because only one
            // of them enforced the shared invariant -- a cap added to one
            // sibling collection is not automatically inherited by another.
            let already_tracked = state.held_buttons.contains_key( pointer_id );
            if already_tracked || state.held_buttons.len() < MAX_ACTIVE_POINTERS
            {
              // Fix(BUG-213): see the KeyboardKey arm above for the mirrored
              // keyboard-side fix and full rationale -- `MouseButton::Unknown`
              // is the mouse equivalent of `KeyboardKey::Unidentified`.
              if *mouse_button == MouseButton::Unknown
              {
                *state.unknown_button_counts.entry( *pointer_id ).or_insert( 0 ) += 1;
              }
              *state.held_buttons.entry( *pointer_id ).or_insert( 0 ) |= bit;
            }
            if !state.active_pointers.iter().any( | ( id, _ ) | *id == *pointer_id )
              && state.active_pointers.len() < MAX_ACTIVE_POINTERS
            {
              state.active_pointers.push( ( *pointer_id, *pos ) );
            }
          }
          Action::Release =>
          {
            // Fix(BUG-213): an Unknown-button release only actually clears the
            // bit once every aliased real button this pointer holds has
            // released -- tracked via `unknown_button_counts` (Press arm above).
            let still_aliased = if *mouse_button == MouseButton::Unknown
            {
              let new_count = state.unknown_button_counts.get( pointer_id )
                .copied().unwrap_or( 0 ).saturating_sub( 1 );
              if new_count > 0
              {
                state.unknown_button_counts.insert( *pointer_id, new_count );
                true
              }
              else
              {
                state.unknown_button_counts.remove( pointer_id );
                false
              }
            }
            else
            {
              false
            };

            if !still_aliased
            {
              if let Some( mask ) = state.held_buttons.get_mut( pointer_id )
              {
                *mask &= !bit;
                if *mask == 0
                {
                  state.held_buttons.remove( pointer_id );
                  state.active_pointers.retain( | ( id, _ ) | *id != *pointer_id );
                }
              }
              else
              {
                // No tracked press for this id (e.g. it arrived before state was
                // reset) -- still don't leave a stale active_pointers entry.
                state.active_pointers.retain( | ( id, _ ) | *id != *pointer_id );
              }
            }
          }
        }
        state.mouse_buttons[ *mouse_button as usize ] =
          state.held_buttons.values().any( | mask | mask & bit != 0 );
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
        // Fix(BUG-130)
        // Root cause: this cleared ALL buttons whenever `active_pointers` happened
        // to become empty, instead of only the cancelled pointer's own buttons --
        // wrong whenever a different pointer (e.g. a mouse button held alongside a
        // cancelled touch) is still legitimately active. Now that `held_buttons`
        // tracks per-pointer state, only the cancelled pointer's own bits are
        // removed, and only the buttons it actually held are re-derived.
        // Pitfall: "if the aggregate is empty, reset everything" is only correct
        // when the aggregate and the thing being reset are updated by the exact
        // same events -- here `active_pointers` (per-pointer) and `mouse_buttons`
        // (per-button) diverge as soon as more than one pointer can be active.
        state.active_pointers.retain( | ( id, _ ) | *id != *pointer_id );
        // Fix(BUG-213): drop this pointer's own alias-hold count too, otherwise
        // it leaks (a cancelled pointer id is never pressed or released again).
        state.unknown_button_counts.remove( pointer_id );
        if let Some( mask ) = state.held_buttons.remove( pointer_id )
        {
          for i in 0 .. MouseButton::COUNT
          {
            if mask & ( 1u32 << i ) != 0
            {
              state.mouse_buttons[ i ] =
                state.held_buttons.values().any( | m | m & ( 1u32 << i ) != 0 );
            }
          }
        }
      }
      EventType::FocusLost =>
      {
        // Fix(BUG-214)
        // Root cause: no `blur`/`visibilitychange` listener existed at all, so a
        // key/button held at the moment the user alt-tabbed or switched tabs
        // never received its matching `keyup`/`pointerup` (the OS delivers the
        // physical release to whichever window/app now has focus, not this
        // page) -- the held flag stayed stuck `true` until an unrelated later
        // event happened to touch that same slot.
        // Pitfall: invisible in every normal press/release sequence -- only
        // manifests once focus actually leaves the page mid-hold, which no
        // sequential keydown/keyup or pointerdown/pointerup test can exercise.
        state.keyboard_keys = [ false; KeyboardKey::COUNT ];
        state.mouse_buttons = [ false; MouseButton::COUNT ];
        state.active_pointers.clear();
        state.held_buttons.clear();
        state.unknown_button_counts.clear();
        state.unidentified_key_hold_count = 0;
        // `pointer_position` and `scroll` are deliberately left untouched --
        // they are last-known-value/accumulator state, not "currently held"
        // state, and remain meaningful after focus returns. `last_pointer_type`
        // is also untouched, but for a different reason: it lives on `Input`
        // itself (shared with the DOM callbacks via `Rc<Cell>`), not on `State`,
        // so this function has no access to it at all.
      }
    }
  }
}

impl Drop for Input
{
  /// Cleans up by removing all attached event listeners from the DOM when the `Input` handler is dropped.
  #[ inline ]
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
    _ = window.remove_event_listener_with_callback
    (
      "blur",
      self.focus_lost_closure.as_ref().unchecked_ref()
    );
    _ = document.remove_event_listener_with_callback
    (
      "visibilitychange",
      self.focus_lost_closure.as_ref().unchecked_ref()
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
