use minwebgl as min;
use min::{ JsCast as _, I32x2 };
use web_sys::{ wasm_bindgen::prelude::Closure, EventTarget, KeyboardEvent, MouseEvent };
use std::{ cell::{ Ref, RefCell }, rc::Rc, str::FromStr };
use bitflags::{ bitflags, Flags as _ };
use strum::EnumCount;
// use min::derive::EnumCount;
// use derive_tools::dependency::strum;

bitflags!
{
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub struct EventFlags : u8
  {
    const KeyboardButton = 0b0001;
    const MouseButton =    0b0010;
    const MouseMovement =  0b0100;
    const Wheel =          0b1000;
  }
}

#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum Action
{
  Press,
  Release,
}

#[ derive( Debug, Clone, Copy, PartialEq ) ]
pub enum EventType
{
  KeyboardButton( KeyboardCode, Action ),
  MouseButton( MouseButton, Action ),
  MouseMovement( I32x2 ),
  Wheel,
}

#[ derive( Debug, Clone, Copy, PartialEq ) ]
pub struct Event
{
  pub event_type : EventType,
  pub alt : bool,
  pub ctrl : bool,
  pub shift : bool,
}

#[ derive( Debug, Clone ) ]
pub struct InputState
{
  events : Vec< Event >,
  keyboard_state : [ bool; KeyboardCode::COUNT ],
  mouse_state : [ bool; MouseButton::COUNT ],
  mouse_position : I32x2,
}

impl InputState
{
  pub fn events( &self ) -> &[ Event ]
  {
    &self.events
  }

  pub fn mouse_position( &self ) -> I32x2
  {
    self.mouse_position
  }

  pub fn is_button_down( &self, button : MouseButton ) -> bool
  {
    let index = button as usize;
    self.mouse_state[ index ]
  }

  pub fn is_key_down( &self, key : KeyboardCode ) -> bool
  {
    self.keyboard_state[ key as usize ]
  }
}

type Callback = Rc< RefCell< dyn FnMut( &InputState, Event ) > >;

type Callbacks = [ Vec< Callback >; EventFlags::FLAGS.len() ];

pub struct Input
{
  input_state : Rc< RefCell< InputState > >,
  callbacks : Rc< RefCell< Callbacks > >,
  mousebutton_closure : Closure< dyn Fn( MouseEvent ) >,
  mousemove_closure : Closure< dyn Fn( MouseEvent ) >,
  keyboardbutton_closure : Closure< dyn Fn( KeyboardEvent ) >,
  mouse_event_target : Option< EventTarget >,
}

impl Input
{
  pub fn new( collect_events : bool, mouse_event_target : Option< EventTarget > ) -> Self
  {
    // TODO:
    // - Support different cursor coordinates like page, screen, client, etc.
    // - Use pointer events instead of mouse events.
    // - Implement wheel.

    let document = web_sys::window().unwrap().document().unwrap();

    let input_state = InputState
    {
      events : Vec::new(),
      keyboard_state : [ false; KeyboardCode::COUNT ],
      mouse_state : [ false; MouseButton::COUNT ],
      mouse_position : I32x2::from_array( [ 0, 0 ] ),
    };

    let input_state = Rc::new( RefCell::new( input_state ) );
    let callbacks = Rc::new( RefCell::new( [ Vec::new(), Vec::new(), Vec::new(), Vec::new() ] ) );

    let mousebutton_closure : Box< dyn Fn( _ ) > =
    {
      let input_state = input_state.clone();
      let callbacks = callbacks.clone();
      if collect_events
      {
        Box::new
        (
          move | event : MouseEvent |
          {
            let event = Self::mousebutton_callback( &mut input_state.borrow_mut(), &callbacks.borrow(), event );
            input_state.borrow_mut().events.push( event );
          }
        )
      }
      else
      {
        Box::new
        (
          move | event : MouseEvent |
          {
            _ = Self::mousebutton_callback( &mut input_state.borrow_mut(), &callbacks.borrow(), event );
          }
        )
      }
    };

    let mousemove_closure : Box< dyn Fn( _ ) > =
    {
      let input_state = input_state.clone();
      let callbacks = callbacks.clone();
      if collect_events
      {
        Box::new
        (
          move | event : MouseEvent |
          {
            let event = Self::mousemove_callback( &mut input_state.borrow_mut(), &callbacks.borrow(), event );
            input_state.borrow_mut().events.push( event );
          }
        )
      }
      else
      {
        Box::new
        (
          move | event : MouseEvent |
          {
            _ = Self::mousemove_callback( &mut input_state.borrow_mut(), &callbacks.borrow(), event );
          }
        )
      }
    };

    let keyboardbutton_closure : Box< dyn Fn( _ ) > =
    {
      let input_state = input_state.clone();
      let callbacks = callbacks.clone();
      if collect_events
      {
        Box::new
        (
          move | event : KeyboardEvent |
          {
            let event = Self::keyboardbutton_callback( &mut input_state.borrow_mut(), &callbacks.borrow(), event );
            input_state.borrow_mut().events.push( event );
          }
        )
      }
      else
      {
        Box::new
        (
          move | event : KeyboardEvent |
          {
            _ = Self::keyboardbutton_callback( &mut input_state.borrow_mut(), &callbacks.borrow(), event );
          }
        )
      }
    };

    let mousebutton_closure = Closure::< dyn Fn( _ ) >::wrap( mousebutton_closure );
    let mousemove_closure = Closure::< dyn Fn( _ ) >::wrap( mousemove_closure );
    let keyboardbutton_closure = Closure::< dyn Fn( _ ) >::wrap( keyboardbutton_closure );

    let input = Input
    {
      input_state,
      callbacks,
      mousebutton_closure,
      mousemove_closure,
      keyboardbutton_closure,
      mouse_event_target,
    };

    document.add_event_listener_with_callback
    (
      "keydown",
      input.keyboardbutton_closure.as_ref().unchecked_ref()
    ).unwrap();
    document.add_event_listener_with_callback
    (
      "keyup",
      input.keyboardbutton_closure.as_ref().unchecked_ref()
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
    // window.add_event_listener_with_callback( "wheel", todo!() ).unwrap();

    input
  }

  pub fn input_state( &self ) -> Ref< InputState >
  {
    self.input_state.borrow()
  }

  pub fn clear_events( &mut self )
  {
    self.input_state.borrow_mut().events.clear();
  }

  pub fn add_callback< F >( &self, callback : F, flags : EventFlags )
  where
    F : FnMut( &InputState, Event ) + 'static
  {
    let callback = Rc::new( RefCell::new( callback ) );
    for flag in EventFlags::FLAGS
    {
      if flags.contains( *flag.value() )
      {
        let index = flag.value().bits().trailing_zeros() as usize;
        self.callbacks.borrow_mut()[ index ].push( callback.clone() );
      }
    }
  }

  pub fn clear_callbacks( &mut self )
  {
    for callbacks in &mut self.callbacks.borrow_mut()[ .. ]
    {
      callbacks.clear();
    }
  }

  fn mousebutton_callback
  (
    input_state : &mut InputState,
    callbacks : &Callbacks,
    event : MouseEvent
  )
  -> Event
  {
    let button = MouseButton::from_button( event.button() );
    let action = if event.type_() == "mousedown"
    {
      input_state.mouse_state[ button as usize ] = true;
      Action::Press
    }
    else
    {
      input_state.mouse_state[ button as usize ] = false;
      Action::Release
    };

    let event_type = EventType::MouseButton( button, action );
    let alt = event.alt_key();
    let ctrl = event.ctrl_key();
    let shift = event.shift_key();

    let event = Event { event_type, alt, ctrl, shift };

    let index = EventFlags::MouseButton.bits().trailing_zeros() as usize;
    for callback in &callbacks[ index ]
    {
      ( callback.borrow_mut() )( input_state, event );
    }

    event
  }

  fn mousemove_callback
  (
    input_state : &mut InputState,
    callbacks : &Callbacks,
    event : MouseEvent
  )
  -> Event
  {
    let position = I32x2::from_array( [ event.client_x(), event.client_y() ] );
    let delta =
    [
      position[ 0 ] - input_state.mouse_position[ 0 ],
      position[ 1 ] - input_state.mouse_position[ 1 ]
    ].into();
    input_state.mouse_position = position;

    let event_type = EventType::MouseMovement( delta );
    let alt = event.alt_key();
    let ctrl = event.ctrl_key();
    let shift = event.shift_key();

    let event = Event { event_type, alt, ctrl, shift };

    let index = EventFlags::MouseMovement.bits().trailing_zeros() as usize;
    for callback in &callbacks[ index ]
    {
      ( callback.borrow_mut() )( &input_state, event );
    }

    event
  }

  fn keyboardbutton_callback
  (
    input_state : &mut InputState,
    callbacks : &Callbacks,
    event : KeyboardEvent
  )
  -> Event
  {
    let code = KeyboardCode::from_code( &event.code() );
    let action = if event.type_() == "keydown"
    {
      input_state.keyboard_state[ code as usize ] = true;
      Action::Press
    }
    else
    {
      input_state.keyboard_state[ code as usize ] = false;
      Action::Release
    };

    let event_type = EventType::KeyboardButton( code, action );
    let alt = event.alt_key();
    let ctrl = event.ctrl_key();
    let shift = event.shift_key();

    let event = Event { event_type, alt, ctrl, shift };

    let index = EventFlags::KeyboardButton.bits().trailing_zeros() as usize;
    for callback in &callbacks[ index ]
    {
      ( callback.borrow_mut() )( &input_state, event );
    }

    event
  }
}

impl Drop for Input
{
  fn drop( &mut self )
  {
    let document = web_sys::window().unwrap().document().unwrap();
    _ = document.remove_event_listener_with_callback( "keydown", self.keyboardbutton_closure.as_ref().unchecked_ref() );
    _ = document.remove_event_listener_with_callback( "keyup", self.keyboardbutton_closure.as_ref().unchecked_ref() );

    let document = document.dyn_into().unwrap();
    let mouse_event_target = self.mouse_event_target.as_ref().unwrap_or( &document );
    _ = mouse_event_target.remove_event_listener_with_callback( "mousedown", self.mousebutton_closure.as_ref().unchecked_ref() );
    _ = mouse_event_target.remove_event_listener_with_callback( "mouseup", self.mousebutton_closure.as_ref().unchecked_ref() );
    _ = mouse_event_target.remove_event_listener_with_callback( "mousemove", self.mousemove_closure.as_ref().unchecked_ref() );
    // _ = window.remove_event_listener_with_callback( "mousemove", ( *self.mousemove_closure ).as_ref().unchecked_ref() );
  }
}

/// KeyboardCode represents all possible values for the KeyboardEvent.code property
/// as defined in the UI Events KeyboardEvent code Values specification.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount ) ]
pub enum KeyboardCode
{
  // Modifier keys
  AltLeft,
  AltRight,
  ControlLeft,
  ControlRight,
  MetaLeft,
  MetaRight,
  ShiftLeft,
  ShiftRight,

  // Whitespace keys
  Enter,
  Tab,
  Space,

  // Navigation keys
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  ArrowUp,
  End,
  Home,
  PageDown,
  PageUp,

  // UI keys
  Escape,
  CapsLock,
  ScrollLock,
  NumLock,
  PrintScreen,
  Pause,
  ContextMenu,

  // Common function keys
  F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
  F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24,

  // Editing keys
  Backspace,
  Clear,
  Delete,
  Insert,

  // Alphanumeric keys
  Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9,
  KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ, KeyK, KeyL, KeyM,
  KeyN, KeyO, KeyP, KeyQ, KeyR, KeyS, KeyT, KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ,

  // Numpad keys
  Numpad0, Numpad1, Numpad2, Numpad3, Numpad4, Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
  NumpadAdd, NumpadSubtract, NumpadMultiply, NumpadDivide, NumpadEnter, NumpadDecimal, NumpadEqual, NumpadComma,

  // Symbol keys
  Backquote,
  BracketLeft,
  BracketRight,
  Comma,
  Period,
  Semicolon,
  Quote,
  Backslash,
  Slash,
  Minus,
  Equal,
  IntlBackslash,
  IntlRo,
  IntlYen,

  // Mobile and special buttons
  AudioVolumeDown,
  AudioVolumeMute,
  AudioVolumeUp,
  BrowserBack,
  BrowserFavorites,
  BrowserForward,
  BrowserHome,
  BrowserRefresh,
  BrowserSearch,
  BrowserStop,
  Eject,
  LaunchApp1,
  LaunchApp2,
  LaunchMail,
  MediaPlayPause,
  MediaStop,
  MediaTrackNext,
  MediaTrackPrevious,
  Power,
  Sleep,
  WakeUp,

  // Extra keys for international keyboards
  Lang1, Lang2, Lang3, Lang4, Lang5,
  Convert,
  NonConvert,
  KanaMode,

  // Unidentified key
  Unidentified,
}

impl KeyboardCode
{
  /// Convert a string code value to a KeyboardCode enum variant
  pub fn from_code( code : &str ) -> Self
  {
    KeyboardCode::from_str( code ).unwrap_or( KeyboardCode::Unidentified )
  }

  /// Get the string representation of this KeyboardCode
  pub fn as_str( &self ) -> &'static str
  {
    match self
    {
      // Modifier keys
      KeyboardCode::AltLeft => "AltLeft",
      KeyboardCode::AltRight => "AltRight",
      KeyboardCode::ControlLeft => "ControlLeft",
      KeyboardCode::ControlRight => "ControlRight",
      KeyboardCode::MetaLeft => "MetaLeft",
      KeyboardCode::MetaRight => "MetaRight",
      KeyboardCode::ShiftLeft => "ShiftLeft",
      KeyboardCode::ShiftRight => "ShiftRight",

      // Whitespace keys
      KeyboardCode::Enter => "Enter",
      KeyboardCode::Tab => "Tab",
      KeyboardCode::Space => "Space",

      // Navigation keys
      KeyboardCode::ArrowDown => "ArrowDown",
      KeyboardCode::ArrowLeft => "ArrowLeft",
      KeyboardCode::ArrowRight => "ArrowRight",
      KeyboardCode::ArrowUp => "ArrowUp",
      KeyboardCode::End => "End",
      KeyboardCode::Home => "Home",
      KeyboardCode::PageDown => "PageDown",
      KeyboardCode::PageUp => "PageUp",

      // UI keys
      KeyboardCode::Escape => "Escape",
      KeyboardCode::CapsLock => "CapsLock",
      KeyboardCode::ScrollLock => "ScrollLock",
      KeyboardCode::NumLock => "NumLock",
      KeyboardCode::PrintScreen => "PrintScreen",
      KeyboardCode::Pause => "Pause",
      KeyboardCode::ContextMenu => "ContextMenu",

      // Function keys
      KeyboardCode::F1 => "F1",
      KeyboardCode::F2 => "F2",
      KeyboardCode::F3 => "F3",
      KeyboardCode::F4 => "F4",
      KeyboardCode::F5 => "F5",
      KeyboardCode::F6 => "F6",
      KeyboardCode::F7 => "F7",
      KeyboardCode::F8 => "F8",
      KeyboardCode::F9 => "F9",
      KeyboardCode::F10 => "F10",
      KeyboardCode::F11 => "F11",
      KeyboardCode::F12 => "F12",
      KeyboardCode::F13 => "F13",
      KeyboardCode::F14 => "F14",
      KeyboardCode::F15 => "F15",
      KeyboardCode::F16 => "F16",
      KeyboardCode::F17 => "F17",
      KeyboardCode::F18 => "F18",
      KeyboardCode::F19 => "F19",
      KeyboardCode::F20 => "F20",
      KeyboardCode::F21 => "F21",
      KeyboardCode::F22 => "F22",
      KeyboardCode::F23 => "F23",
      KeyboardCode::F24 => "F24",

      // Editing keys
      KeyboardCode::Backspace => "Backspace",
      KeyboardCode::Clear => "Clear",
      KeyboardCode::Delete => "Delete",
      KeyboardCode::Insert => "Insert",

      // Alphanumeric keys
      KeyboardCode::Digit0 => "Digit0",
      KeyboardCode::Digit1 => "Digit1",
      KeyboardCode::Digit2 => "Digit2",
      KeyboardCode::Digit3 => "Digit3",
      KeyboardCode::Digit4 => "Digit4",
      KeyboardCode::Digit5 => "Digit5",
      KeyboardCode::Digit6 => "Digit6",
      KeyboardCode::Digit7 => "Digit7",
      KeyboardCode::Digit8 => "Digit8",
      KeyboardCode::Digit9 => "Digit9",
      KeyboardCode::KeyA => "KeyA",
      KeyboardCode::KeyB => "KeyB",
      KeyboardCode::KeyC => "KeyC",
      KeyboardCode::KeyD => "KeyD",
      KeyboardCode::KeyE => "KeyE",
      KeyboardCode::KeyF => "KeyF",
      KeyboardCode::KeyG => "KeyG",
      KeyboardCode::KeyH => "KeyH",
      KeyboardCode::KeyI => "KeyI",
      KeyboardCode::KeyJ => "KeyJ",
      KeyboardCode::KeyK => "KeyK",
      KeyboardCode::KeyL => "KeyL",
      KeyboardCode::KeyM => "KeyM",
      KeyboardCode::KeyN => "KeyN",
      KeyboardCode::KeyO => "KeyO",
      KeyboardCode::KeyP => "KeyP",
      KeyboardCode::KeyQ => "KeyQ",
      KeyboardCode::KeyR => "KeyR",
      KeyboardCode::KeyS => "KeyS",
      KeyboardCode::KeyT => "KeyT",
      KeyboardCode::KeyU => "KeyU",
      KeyboardCode::KeyV => "KeyV",
      KeyboardCode::KeyW => "KeyW",
      KeyboardCode::KeyX => "KeyX",
      KeyboardCode::KeyY => "KeyY",
      KeyboardCode::KeyZ => "KeyZ",

      // Numpad keys
      KeyboardCode::Numpad0 => "Numpad0",
      KeyboardCode::Numpad1 => "Numpad1",
      KeyboardCode::Numpad2 => "Numpad2",
      KeyboardCode::Numpad3 => "Numpad3",
      KeyboardCode::Numpad4 => "Numpad4",
      KeyboardCode::Numpad5 => "Numpad5",
      KeyboardCode::Numpad6 => "Numpad6",
      KeyboardCode::Numpad7 => "Numpad7",
      KeyboardCode::Numpad8 => "Numpad8",
      KeyboardCode::Numpad9 => "Numpad9",
      KeyboardCode::NumpadAdd => "NumpadAdd",
      KeyboardCode::NumpadSubtract => "NumpadSubtract",
      KeyboardCode::NumpadMultiply => "NumpadMultiply",
      KeyboardCode::NumpadDivide => "NumpadDivide",
      KeyboardCode::NumpadEnter => "NumpadEnter",
      KeyboardCode::NumpadDecimal => "NumpadDecimal",
      KeyboardCode::NumpadEqual => "NumpadEqual",
      KeyboardCode::NumpadComma => "NumpadComma",

      // Symbol keys
      KeyboardCode::Backquote => "Backquote",
      KeyboardCode::BracketLeft => "BracketLeft",
      KeyboardCode::BracketRight => "BracketRight",
      KeyboardCode::Comma => "Comma",
      KeyboardCode::Period => "Period",
      KeyboardCode::Semicolon => "Semicolon",
      KeyboardCode::Quote => "Quote",
      KeyboardCode::Backslash => "Backslash",
      KeyboardCode::Slash => "Slash",
      KeyboardCode::Minus => "Minus",
      KeyboardCode::Equal => "Equal",
      KeyboardCode::IntlBackslash => "IntlBackslash",
      KeyboardCode::IntlRo => "IntlRo",
      KeyboardCode::IntlYen => "IntlYen",

      // Media keys
      KeyboardCode::AudioVolumeDown => "AudioVolumeDown",
      KeyboardCode::AudioVolumeMute => "AudioVolumeMute",
      KeyboardCode::AudioVolumeUp => "AudioVolumeUp",
      KeyboardCode::BrowserBack => "BrowserBack",
      KeyboardCode::BrowserFavorites => "BrowserFavorites",
      KeyboardCode::BrowserForward => "BrowserForward",
      KeyboardCode::BrowserHome => "BrowserHome",
      KeyboardCode::BrowserRefresh => "BrowserRefresh",
      KeyboardCode::BrowserSearch => "BrowserSearch",
      KeyboardCode::BrowserStop => "BrowserStop",
      KeyboardCode::Eject => "Eject",
      KeyboardCode::LaunchApp1 => "LaunchApp1",
      KeyboardCode::LaunchApp2 => "LaunchApp2",
      KeyboardCode::LaunchMail => "LaunchMail",
      KeyboardCode::MediaPlayPause => "MediaPlayPause",
      KeyboardCode::MediaStop => "MediaStop",
      KeyboardCode::MediaTrackNext => "MediaTrackNext",
      KeyboardCode::MediaTrackPrevious => "MediaTrackPrevious",
      KeyboardCode::Power => "Power",
      KeyboardCode::Sleep => "Sleep",
      KeyboardCode::WakeUp => "WakeUp",

      // International keys
      KeyboardCode::Lang1 => "Lang1",
      KeyboardCode::Lang2 => "Lang2",
      KeyboardCode::Lang3 => "Lang3",
      KeyboardCode::Lang4 => "Lang4",
      KeyboardCode::Lang5 => "Lang5",
      KeyboardCode::Convert => "Convert",
      KeyboardCode::NonConvert => "NonConvert",
      KeyboardCode::KanaMode => "KanaMode",

      // Fallback
      KeyboardCode::Unidentified => "Unidentified",
    }
  }

  /// Check if this is a navigation key
  pub fn is_navigation( &self ) -> bool
  {
    matches!
    (
      self,
      KeyboardCode::ArrowDown |
      KeyboardCode::ArrowLeft |
      KeyboardCode::ArrowRight |
      KeyboardCode::ArrowUp |
      KeyboardCode::Home |
      KeyboardCode::End |
      KeyboardCode::PageUp |
      KeyboardCode::PageDown
    )
  }

  /// Check if this is a modifier key
  pub fn is_modifier( &self ) -> bool
  {
    matches!
    (
      self,
      KeyboardCode::AltLeft |
      KeyboardCode::AltRight |
      KeyboardCode::ControlLeft |
      KeyboardCode::ControlRight |
      KeyboardCode::ShiftLeft |
      KeyboardCode::ShiftRight |
      KeyboardCode::MetaLeft |
      KeyboardCode::MetaRight
    )
  }

  /// Check if this is a function key
  pub fn is_function_key( &self ) -> bool
  {
    matches!
    (
      self,
      KeyboardCode::F1 | KeyboardCode::F2 | KeyboardCode::F3 | KeyboardCode::F4 |
      KeyboardCode::F5 | KeyboardCode::F6 | KeyboardCode::F7 | KeyboardCode::F8 |
      KeyboardCode::F9 | KeyboardCode::F10 | KeyboardCode::F11 | KeyboardCode::F12 |
      KeyboardCode::F13 | KeyboardCode::F14 | KeyboardCode::F15 | KeyboardCode::F16 |
      KeyboardCode::F17 | KeyboardCode::F18 | KeyboardCode::F19 | KeyboardCode::F20 |
      KeyboardCode::F21 | KeyboardCode::F22 | KeyboardCode::F23 | KeyboardCode::F24
    )
  }

  /// Check if this is a numpad key
  pub fn is_numpad( &self ) -> bool
  {
    matches!
    (
      self,
      KeyboardCode::Numpad0 | KeyboardCode::Numpad1 | KeyboardCode::Numpad2 |
      KeyboardCode::Numpad3 | KeyboardCode::Numpad4 | KeyboardCode::Numpad5 |
      KeyboardCode::Numpad6 | KeyboardCode::Numpad7 | KeyboardCode::Numpad8 |
      KeyboardCode::Numpad9 | KeyboardCode::NumpadAdd | KeyboardCode::NumpadSubtract |
      KeyboardCode::NumpadMultiply | KeyboardCode::NumpadDivide | KeyboardCode::NumpadEnter |
      KeyboardCode::NumpadDecimal | KeyboardCode::NumpadEqual | KeyboardCode::NumpadComma
    )
  }
}

impl FromStr for KeyboardCode
{
  type Err = ();

  fn from_str( s : &str ) -> Result< Self, Self::Err >
  {
    match s
    {
      // Modifier keys
      "AltLeft" => Ok( KeyboardCode::AltLeft ),
      "AltRight" => Ok( KeyboardCode::AltRight ),
      "ControlLeft" => Ok( KeyboardCode::ControlLeft ),
      "ControlRight" => Ok( KeyboardCode::ControlRight ),
      "MetaLeft" => Ok( KeyboardCode::MetaLeft ),
      "MetaRight" => Ok( KeyboardCode::MetaRight ),
      "ShiftLeft" => Ok( KeyboardCode::ShiftLeft ),
      "ShiftRight" => Ok( KeyboardCode::ShiftRight ),

      // Whitespace keys
      "Enter" => Ok( KeyboardCode::Enter ),
      "Tab" => Ok( KeyboardCode::Tab ),
      "Space" => Ok( KeyboardCode::Space ),

      // Navigation keys
      "ArrowDown" => Ok( KeyboardCode::ArrowDown ),
      "ArrowLeft" => Ok( KeyboardCode::ArrowLeft ),
      "ArrowRight" => Ok( KeyboardCode::ArrowRight ),
      "ArrowUp" => Ok( KeyboardCode::ArrowUp ),
      "End" => Ok( KeyboardCode::End ),
      "Home" => Ok( KeyboardCode::Home ),
      "PageDown" => Ok( KeyboardCode::PageDown ),
      "PageUp" => Ok( KeyboardCode::PageUp ),

      // UI keys
      "Escape" => Ok( KeyboardCode::Escape ),
      "CapsLock" => Ok( KeyboardCode::CapsLock ),
      "ScrollLock" => Ok( KeyboardCode::ScrollLock ),
      "NumLock" => Ok( KeyboardCode::NumLock ),
      "PrintScreen" => Ok( KeyboardCode::PrintScreen ),
      "Pause" => Ok( KeyboardCode::Pause ),
      "ContextMenu" => Ok( KeyboardCode::ContextMenu ),

      // Function keys
      "F1" => Ok( KeyboardCode::F1 ),
      "F2" => Ok( KeyboardCode::F2 ),
      "F3" => Ok( KeyboardCode::F3 ),
      "F4" => Ok( KeyboardCode::F4 ),
      "F5" => Ok( KeyboardCode::F5 ),
      "F6" => Ok( KeyboardCode::F6 ),
      "F7" => Ok( KeyboardCode::F7 ),
      "F8" => Ok( KeyboardCode::F8 ),
      "F9" => Ok( KeyboardCode::F9 ),
      "F10" => Ok( KeyboardCode::F10 ),
      "F11" => Ok( KeyboardCode::F11 ),
      "F12" => Ok( KeyboardCode::F12 ),
      "F13" => Ok( KeyboardCode::F13 ),
      "F14" => Ok( KeyboardCode::F14 ),
      "F15" => Ok( KeyboardCode::F15 ),
      "F16" => Ok( KeyboardCode::F16 ),
      "F17" => Ok( KeyboardCode::F17 ),
      "F18" => Ok( KeyboardCode::F18 ),
      "F19" => Ok( KeyboardCode::F19 ),
      "F20" => Ok( KeyboardCode::F20 ),
      "F21" => Ok( KeyboardCode::F21 ),
      "F22" => Ok( KeyboardCode::F22 ),
      "F23" => Ok( KeyboardCode::F23 ),
      "F24" => Ok( KeyboardCode::F24 ),

      // Editing keys
      "Backspace" => Ok( KeyboardCode::Backspace ),
      "Clear" => Ok( KeyboardCode::Clear ),
      "Delete" => Ok( KeyboardCode::Delete ),
      "Insert" => Ok( KeyboardCode::Insert ),

      // Alphanumeric keys
      "Digit0" => Ok( KeyboardCode::Digit0 ),
      "Digit1" => Ok( KeyboardCode::Digit1 ),
      "Digit2" => Ok( KeyboardCode::Digit2 ),
      "Digit3" => Ok( KeyboardCode::Digit3 ),
      "Digit4" => Ok( KeyboardCode::Digit4 ),
      "Digit5" => Ok( KeyboardCode::Digit5 ),
      "Digit6" => Ok( KeyboardCode::Digit6 ),
      "Digit7" => Ok( KeyboardCode::Digit7 ),
      "Digit8" => Ok( KeyboardCode::Digit8 ),
      "Digit9" => Ok( KeyboardCode::Digit9 ),
      "KeyA" => Ok( KeyboardCode::KeyA ),
      "KeyB" => Ok( KeyboardCode::KeyB ),
      "KeyC" => Ok( KeyboardCode::KeyC ),
      "KeyD" => Ok( KeyboardCode::KeyD ),
      "KeyE" => Ok( KeyboardCode::KeyE ),
      "KeyF" => Ok( KeyboardCode::KeyF ),
      "KeyG" => Ok( KeyboardCode::KeyG ),
      "KeyH" => Ok( KeyboardCode::KeyH ),
      "KeyI" => Ok( KeyboardCode::KeyI ),
      "KeyJ" => Ok( KeyboardCode::KeyJ ),
      "KeyK" => Ok( KeyboardCode::KeyK ),
      "KeyL" => Ok( KeyboardCode::KeyL ),
      "KeyM" => Ok( KeyboardCode::KeyM ),
      "KeyN" => Ok( KeyboardCode::KeyN ),
      "KeyO" => Ok( KeyboardCode::KeyO ),
      "KeyP" => Ok( KeyboardCode::KeyP ),
      "KeyQ" => Ok( KeyboardCode::KeyQ ),
      "KeyR" => Ok( KeyboardCode::KeyR ),
      "KeyS" => Ok( KeyboardCode::KeyS ),
      "KeyT" => Ok( KeyboardCode::KeyT ),
      "KeyU" => Ok( KeyboardCode::KeyU ),
      "KeyV" => Ok( KeyboardCode::KeyV ),
      "KeyW" => Ok( KeyboardCode::KeyW ),
      "KeyX" => Ok( KeyboardCode::KeyX ),
      "KeyY" => Ok( KeyboardCode::KeyY ),
      "KeyZ" => Ok( KeyboardCode::KeyZ ),

      // Numpad keys
      "Numpad0" => Ok( KeyboardCode::Numpad0 ),
      "Numpad1" => Ok( KeyboardCode::Numpad1 ),
      "Numpad2" => Ok( KeyboardCode::Numpad2 ),
      "Numpad3" => Ok( KeyboardCode::Numpad3 ),
      "Numpad4" => Ok( KeyboardCode::Numpad4 ),
      "Numpad5" => Ok( KeyboardCode::Numpad5 ),
      "Numpad6" => Ok( KeyboardCode::Numpad6 ),
      "Numpad7" => Ok( KeyboardCode::Numpad7 ),
      "Numpad8" => Ok( KeyboardCode::Numpad8 ),
      "Numpad9" => Ok( KeyboardCode::Numpad9 ),
      "NumpadAdd" => Ok( KeyboardCode::NumpadAdd ),
      "NumpadSubtract" => Ok( KeyboardCode::NumpadSubtract ),
      "NumpadMultiply" => Ok( KeyboardCode::NumpadMultiply ),
      "NumpadDivide" => Ok( KeyboardCode::NumpadDivide ),
      "NumpadEnter" => Ok( KeyboardCode::NumpadEnter ),
      "NumpadDecimal" => Ok( KeyboardCode::NumpadDecimal ),
      "NumpadEqual" => Ok( KeyboardCode::NumpadEqual ),
      "NumpadComma" => Ok( KeyboardCode::NumpadComma ),

      // Symbol keys
      "Backquote" => Ok( KeyboardCode::Backquote ),
      "BracketLeft" => Ok( KeyboardCode::BracketLeft ),
      "BracketRight" => Ok( KeyboardCode::BracketRight ),
      "Comma" => Ok( KeyboardCode::Comma ),
      "Period" => Ok( KeyboardCode::Period ),
      "Semicolon" => Ok( KeyboardCode::Semicolon ),
      "Quote" => Ok( KeyboardCode::Quote ),
      "Backslash" => Ok( KeyboardCode::Backslash ),
      "Slash" => Ok( KeyboardCode::Slash ),
      "Minus" => Ok( KeyboardCode::Minus ),
      "Equal" => Ok( KeyboardCode::Equal ),
      "IntlBackslash" => Ok( KeyboardCode::IntlBackslash ),
      "IntlRo" => Ok( KeyboardCode::IntlRo ),
      "IntlYen" => Ok( KeyboardCode::IntlYen ),

      // Media keys
      "AudioVolumeDown" => Ok( KeyboardCode::AudioVolumeDown ),
      "AudioVolumeMute" => Ok( KeyboardCode::AudioVolumeMute ),
      "AudioVolumeUp" => Ok( KeyboardCode::AudioVolumeUp ),
      "BrowserBack" => Ok( KeyboardCode::BrowserBack ),
      "BrowserFavorites" => Ok( KeyboardCode::BrowserFavorites ),
      "BrowserForward" => Ok( KeyboardCode::BrowserForward ),
      "BrowserHome" => Ok( KeyboardCode::BrowserHome ),
      "BrowserRefresh" => Ok( KeyboardCode::BrowserRefresh ),
      "BrowserSearch" => Ok( KeyboardCode::BrowserSearch ),
      "BrowserStop" => Ok( KeyboardCode::BrowserStop ),
      "Eject" => Ok( KeyboardCode::Eject ),
      "LaunchApp1" => Ok( KeyboardCode::LaunchApp1 ),
      "LaunchApp2" => Ok( KeyboardCode::LaunchApp2 ),
      "LaunchMail" => Ok( KeyboardCode::LaunchMail ),
      "MediaPlayPause" => Ok( KeyboardCode::MediaPlayPause ),
      "MediaStop" => Ok( KeyboardCode::MediaStop ),
      "MediaTrackNext" => Ok( KeyboardCode::MediaTrackNext ),
      "MediaTrackPrevious" => Ok( KeyboardCode::MediaTrackPrevious ),
      "Power" => Ok( KeyboardCode::Power ),
      "Sleep" => Ok( KeyboardCode::Sleep ),
      "WakeUp" => Ok( KeyboardCode::WakeUp ),

      // International keys
      "Lang1" => Ok( KeyboardCode::Lang1 ),
      "Lang2" => Ok( KeyboardCode::Lang2 ),
      "Lang3" => Ok( KeyboardCode::Lang3 ),
      "Lang4" => Ok( KeyboardCode::Lang4 ),
      "Lang5" => Ok( KeyboardCode::Lang5 ),
      "Convert" => Ok( KeyboardCode::Convert ),
      "NonConvert" => Ok( KeyboardCode::NonConvert ),
      "KanaMode" => Ok( KeyboardCode::KanaMode ),

      // Unknown key
      _ => Ok( KeyboardCode::Unidentified ),
    }
  }
}

/// MouseButton represents the different mouse buttons as defined by the
/// MouseEvent.button property in the DOM specification.
///
/// Values correspond to:
/// - 0: Main button (usually left)
/// - 1: Auxiliary button (usually middle/wheel)
/// - 2: Secondary button (usually right)
/// - 3: Fourth button (usually "Browser Back")
/// - 4: Fifth button (usually "Browser Forward")
///
/// See: https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount ) ]
pub enum MouseButton
{
  Main,        // Left button (0)
  Auxiliary,   // Middle button (1)
  Secondary,   // Right button (2)
  Back,        // Back button (3)
  Forward,     // Forward button (4)
  Unknown, // For any other values
}

impl MouseButton
{
  /// Convert a numeric button value to the corresponding MouseButton enum variant
  pub fn from_button( button : i16 ) -> Self
  {
    match button
    {
      0 => MouseButton::Main,
      1 => MouseButton::Auxiliary,
      2 => MouseButton::Secondary,
      3 => MouseButton::Back,
      4 => MouseButton::Forward,
      _ => MouseButton::Unknown,
    }
  }

  /// Convert a string representation to the corresponding MouseButton enum variant
  pub fn from_name( name : &str ) -> Self
  {
    MouseButton::from_str( name ).unwrap_or( MouseButton::Unknown )
  }

  /// Get the numeric button value for this MouseButton
  pub fn button_value( &self ) -> i16
  {
    match self
    {
      MouseButton::Main => 0,
      MouseButton::Auxiliary => 1,
      MouseButton::Secondary => 2,
      MouseButton::Back => 3,
      MouseButton::Forward => 4,
      MouseButton::Unknown => 5,
    }
  }

  /// Get a user-friendly name for this MouseButton
  pub fn name( &self ) -> &'static str
  {
    match self
    {
      MouseButton::Main => "Left",
      MouseButton::Auxiliary => "Middle",
      MouseButton::Secondary => "Right",
      MouseButton::Back => "Back",
      MouseButton::Forward => "Forward",
      MouseButton::Unknown => "Unknown",
    }
  }

  /// Get the technical name for this MouseButton
  pub fn technical_name( &self ) -> &'static str
  {
    match self
    {
      MouseButton::Main => "Main",
      MouseButton::Auxiliary => "Auxiliary",
      MouseButton::Secondary => "Secondary",
      MouseButton::Back => "Back",
      MouseButton::Forward => "Forward",
      MouseButton::Unknown => "Unknown",
    }
  }

  /// Check if this is the main (usually left) button
  pub fn is_main( &self ) -> bool
  {
    matches!( self, MouseButton::Main )
  }

  /// Check if this is the secondary (usually right) button
  pub fn is_secondary( &self ) -> bool
  {
    matches!( self, MouseButton::Secondary )
  }

  /// Check if this is the auxiliary (usually middle/wheel) button
  pub fn is_auxiliary( &self ) -> bool
  {
    matches!( self, MouseButton::Auxiliary )
  }

  /// Check if this is a navigation button (Back/Forward)
  pub fn is_navigation( &self ) -> bool
  {
    matches!( self, MouseButton::Back | MouseButton::Forward )
  }
}

impl FromStr for MouseButton
{
  type Err = ();

  fn from_str( s : &str ) -> Result< Self, Self::Err >
  {
    match s.to_lowercase().as_str()
    {
      "main" | "left" | "primary" => Ok( MouseButton::Main ),
      "auxiliary" | "middle" | "wheel" => Ok( MouseButton::Auxiliary ),
      "secondary" | "right" | "context" => Ok( MouseButton::Secondary ),
      "back" => Ok( MouseButton::Back ),
      "forward" => Ok( MouseButton::Forward ),
      _ => Err( () ),
    }
  }
}

impl From< i16 > for MouseButton
{
  fn from( value : i16 ) -> Self
  {
    MouseButton::from_button( value )
  }
}
