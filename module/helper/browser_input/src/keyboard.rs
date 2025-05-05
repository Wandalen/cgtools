use std::str::FromStr;
use strum::EnumCount;

/// KeyboardCode represents all possible values for the KeyboardEvent.code property
/// as defined in the UI Events KeyboardEvent code Values specification.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount ) ]
pub enum KeyboardKey
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

impl KeyboardKey
{
  /// Convert a string code value to a KeyboardCode enum variant
  pub fn from_code( code : &str ) -> Self
  {
    KeyboardKey::from_str( code ).unwrap_or( KeyboardKey::Unidentified )
  }

  /// Get the string representation of this KeyboardCode
  pub fn as_str( &self ) -> &'static str
  {
    match self
    {
      // Modifier keys
      KeyboardKey::AltLeft => "AltLeft",
      KeyboardKey::AltRight => "AltRight",
      KeyboardKey::ControlLeft => "ControlLeft",
      KeyboardKey::ControlRight => "ControlRight",
      KeyboardKey::MetaLeft => "MetaLeft",
      KeyboardKey::MetaRight => "MetaRight",
      KeyboardKey::ShiftLeft => "ShiftLeft",
      KeyboardKey::ShiftRight => "ShiftRight",

      // Whitespace keys
      KeyboardKey::Enter => "Enter",
      KeyboardKey::Tab => "Tab",
      KeyboardKey::Space => "Space",

      // Navigation keys
      KeyboardKey::ArrowDown => "ArrowDown",
      KeyboardKey::ArrowLeft => "ArrowLeft",
      KeyboardKey::ArrowRight => "ArrowRight",
      KeyboardKey::ArrowUp => "ArrowUp",
      KeyboardKey::End => "End",
      KeyboardKey::Home => "Home",
      KeyboardKey::PageDown => "PageDown",
      KeyboardKey::PageUp => "PageUp",

      // UI keys
      KeyboardKey::Escape => "Escape",
      KeyboardKey::CapsLock => "CapsLock",
      KeyboardKey::ScrollLock => "ScrollLock",
      KeyboardKey::NumLock => "NumLock",
      KeyboardKey::PrintScreen => "PrintScreen",
      KeyboardKey::Pause => "Pause",
      KeyboardKey::ContextMenu => "ContextMenu",

      // Function keys
      KeyboardKey::F1 => "F1",
      KeyboardKey::F2 => "F2",
      KeyboardKey::F3 => "F3",
      KeyboardKey::F4 => "F4",
      KeyboardKey::F5 => "F5",
      KeyboardKey::F6 => "F6",
      KeyboardKey::F7 => "F7",
      KeyboardKey::F8 => "F8",
      KeyboardKey::F9 => "F9",
      KeyboardKey::F10 => "F10",
      KeyboardKey::F11 => "F11",
      KeyboardKey::F12 => "F12",
      KeyboardKey::F13 => "F13",
      KeyboardKey::F14 => "F14",
      KeyboardKey::F15 => "F15",
      KeyboardKey::F16 => "F16",
      KeyboardKey::F17 => "F17",
      KeyboardKey::F18 => "F18",
      KeyboardKey::F19 => "F19",
      KeyboardKey::F20 => "F20",
      KeyboardKey::F21 => "F21",
      KeyboardKey::F22 => "F22",
      KeyboardKey::F23 => "F23",
      KeyboardKey::F24 => "F24",

      // Editing keys
      KeyboardKey::Backspace => "Backspace",
      KeyboardKey::Clear => "Clear",
      KeyboardKey::Delete => "Delete",
      KeyboardKey::Insert => "Insert",

      // Alphanumeric keys
      KeyboardKey::Digit0 => "Digit0",
      KeyboardKey::Digit1 => "Digit1",
      KeyboardKey::Digit2 => "Digit2",
      KeyboardKey::Digit3 => "Digit3",
      KeyboardKey::Digit4 => "Digit4",
      KeyboardKey::Digit5 => "Digit5",
      KeyboardKey::Digit6 => "Digit6",
      KeyboardKey::Digit7 => "Digit7",
      KeyboardKey::Digit8 => "Digit8",
      KeyboardKey::Digit9 => "Digit9",
      KeyboardKey::KeyA => "KeyA",
      KeyboardKey::KeyB => "KeyB",
      KeyboardKey::KeyC => "KeyC",
      KeyboardKey::KeyD => "KeyD",
      KeyboardKey::KeyE => "KeyE",
      KeyboardKey::KeyF => "KeyF",
      KeyboardKey::KeyG => "KeyG",
      KeyboardKey::KeyH => "KeyH",
      KeyboardKey::KeyI => "KeyI",
      KeyboardKey::KeyJ => "KeyJ",
      KeyboardKey::KeyK => "KeyK",
      KeyboardKey::KeyL => "KeyL",
      KeyboardKey::KeyM => "KeyM",
      KeyboardKey::KeyN => "KeyN",
      KeyboardKey::KeyO => "KeyO",
      KeyboardKey::KeyP => "KeyP",
      KeyboardKey::KeyQ => "KeyQ",
      KeyboardKey::KeyR => "KeyR",
      KeyboardKey::KeyS => "KeyS",
      KeyboardKey::KeyT => "KeyT",
      KeyboardKey::KeyU => "KeyU",
      KeyboardKey::KeyV => "KeyV",
      KeyboardKey::KeyW => "KeyW",
      KeyboardKey::KeyX => "KeyX",
      KeyboardKey::KeyY => "KeyY",
      KeyboardKey::KeyZ => "KeyZ",

      // Numpad keys
      KeyboardKey::Numpad0 => "Numpad0",
      KeyboardKey::Numpad1 => "Numpad1",
      KeyboardKey::Numpad2 => "Numpad2",
      KeyboardKey::Numpad3 => "Numpad3",
      KeyboardKey::Numpad4 => "Numpad4",
      KeyboardKey::Numpad5 => "Numpad5",
      KeyboardKey::Numpad6 => "Numpad6",
      KeyboardKey::Numpad7 => "Numpad7",
      KeyboardKey::Numpad8 => "Numpad8",
      KeyboardKey::Numpad9 => "Numpad9",
      KeyboardKey::NumpadAdd => "NumpadAdd",
      KeyboardKey::NumpadSubtract => "NumpadSubtract",
      KeyboardKey::NumpadMultiply => "NumpadMultiply",
      KeyboardKey::NumpadDivide => "NumpadDivide",
      KeyboardKey::NumpadEnter => "NumpadEnter",
      KeyboardKey::NumpadDecimal => "NumpadDecimal",
      KeyboardKey::NumpadEqual => "NumpadEqual",
      KeyboardKey::NumpadComma => "NumpadComma",

      // Symbol keys
      KeyboardKey::Backquote => "Backquote",
      KeyboardKey::BracketLeft => "BracketLeft",
      KeyboardKey::BracketRight => "BracketRight",
      KeyboardKey::Comma => "Comma",
      KeyboardKey::Period => "Period",
      KeyboardKey::Semicolon => "Semicolon",
      KeyboardKey::Quote => "Quote",
      KeyboardKey::Backslash => "Backslash",
      KeyboardKey::Slash => "Slash",
      KeyboardKey::Minus => "Minus",
      KeyboardKey::Equal => "Equal",
      KeyboardKey::IntlBackslash => "IntlBackslash",
      KeyboardKey::IntlRo => "IntlRo",
      KeyboardKey::IntlYen => "IntlYen",

      // Media keys
      KeyboardKey::AudioVolumeDown => "AudioVolumeDown",
      KeyboardKey::AudioVolumeMute => "AudioVolumeMute",
      KeyboardKey::AudioVolumeUp => "AudioVolumeUp",
      KeyboardKey::BrowserBack => "BrowserBack",
      KeyboardKey::BrowserFavorites => "BrowserFavorites",
      KeyboardKey::BrowserForward => "BrowserForward",
      KeyboardKey::BrowserHome => "BrowserHome",
      KeyboardKey::BrowserRefresh => "BrowserRefresh",
      KeyboardKey::BrowserSearch => "BrowserSearch",
      KeyboardKey::BrowserStop => "BrowserStop",
      KeyboardKey::Eject => "Eject",
      KeyboardKey::LaunchApp1 => "LaunchApp1",
      KeyboardKey::LaunchApp2 => "LaunchApp2",
      KeyboardKey::LaunchMail => "LaunchMail",
      KeyboardKey::MediaPlayPause => "MediaPlayPause",
      KeyboardKey::MediaStop => "MediaStop",
      KeyboardKey::MediaTrackNext => "MediaTrackNext",
      KeyboardKey::MediaTrackPrevious => "MediaTrackPrevious",
      KeyboardKey::Power => "Power",
      KeyboardKey::Sleep => "Sleep",
      KeyboardKey::WakeUp => "WakeUp",

      // International keys
      KeyboardKey::Lang1 => "Lang1",
      KeyboardKey::Lang2 => "Lang2",
      KeyboardKey::Lang3 => "Lang3",
      KeyboardKey::Lang4 => "Lang4",
      KeyboardKey::Lang5 => "Lang5",
      KeyboardKey::Convert => "Convert",
      KeyboardKey::NonConvert => "NonConvert",
      KeyboardKey::KanaMode => "KanaMode",

      // Fallback
      KeyboardKey::Unidentified => "Unidentified",
    }
  }

  /// Check if this is a navigation key
  pub fn is_navigation( &self ) -> bool
  {
    matches!
    (
      self,
      KeyboardKey::ArrowDown |
      KeyboardKey::ArrowLeft |
      KeyboardKey::ArrowRight |
      KeyboardKey::ArrowUp |
      KeyboardKey::Home |
      KeyboardKey::End |
      KeyboardKey::PageUp |
      KeyboardKey::PageDown
    )
  }

  /// Check if this is a modifier key
  pub fn is_modifier( &self ) -> bool
  {
    matches!
    (
      self,
      KeyboardKey::AltLeft |
      KeyboardKey::AltRight |
      KeyboardKey::ControlLeft |
      KeyboardKey::ControlRight |
      KeyboardKey::ShiftLeft |
      KeyboardKey::ShiftRight |
      KeyboardKey::MetaLeft |
      KeyboardKey::MetaRight
    )
  }

  /// Check if this is a function key
  pub fn is_function_key( &self ) -> bool
  {
    matches!
    (
      self,
      KeyboardKey::F1 | KeyboardKey::F2 | KeyboardKey::F3 | KeyboardKey::F4 |
      KeyboardKey::F5 | KeyboardKey::F6 | KeyboardKey::F7 | KeyboardKey::F8 |
      KeyboardKey::F9 | KeyboardKey::F10 | KeyboardKey::F11 | KeyboardKey::F12 |
      KeyboardKey::F13 | KeyboardKey::F14 | KeyboardKey::F15 | KeyboardKey::F16 |
      KeyboardKey::F17 | KeyboardKey::F18 | KeyboardKey::F19 | KeyboardKey::F20 |
      KeyboardKey::F21 | KeyboardKey::F22 | KeyboardKey::F23 | KeyboardKey::F24
    )
  }

  /// Check if this is a numpad key
  pub fn is_numpad( &self ) -> bool
  {
    matches!
    (
      self,
      KeyboardKey::Numpad0 | KeyboardKey::Numpad1 | KeyboardKey::Numpad2 |
      KeyboardKey::Numpad3 | KeyboardKey::Numpad4 | KeyboardKey::Numpad5 |
      KeyboardKey::Numpad6 | KeyboardKey::Numpad7 | KeyboardKey::Numpad8 |
      KeyboardKey::Numpad9 | KeyboardKey::NumpadAdd | KeyboardKey::NumpadSubtract |
      KeyboardKey::NumpadMultiply | KeyboardKey::NumpadDivide | KeyboardKey::NumpadEnter |
      KeyboardKey::NumpadDecimal | KeyboardKey::NumpadEqual | KeyboardKey::NumpadComma
    )
  }
}

impl FromStr for KeyboardKey
{
  type Err = ();

  fn from_str( s : &str ) -> Result< Self, Self::Err >
  {
    match s
    {
      // Modifier keys
      "AltLeft" => Ok( KeyboardKey::AltLeft ),
      "AltRight" => Ok( KeyboardKey::AltRight ),
      "ControlLeft" => Ok( KeyboardKey::ControlLeft ),
      "ControlRight" => Ok( KeyboardKey::ControlRight ),
      "MetaLeft" => Ok( KeyboardKey::MetaLeft ),
      "MetaRight" => Ok( KeyboardKey::MetaRight ),
      "ShiftLeft" => Ok( KeyboardKey::ShiftLeft ),
      "ShiftRight" => Ok( KeyboardKey::ShiftRight ),

      // Whitespace keys
      "Enter" => Ok( KeyboardKey::Enter ),
      "Tab" => Ok( KeyboardKey::Tab ),
      "Space" => Ok( KeyboardKey::Space ),

      // Navigation keys
      "ArrowDown" => Ok( KeyboardKey::ArrowDown ),
      "ArrowLeft" => Ok( KeyboardKey::ArrowLeft ),
      "ArrowRight" => Ok( KeyboardKey::ArrowRight ),
      "ArrowUp" => Ok( KeyboardKey::ArrowUp ),
      "End" => Ok( KeyboardKey::End ),
      "Home" => Ok( KeyboardKey::Home ),
      "PageDown" => Ok( KeyboardKey::PageDown ),
      "PageUp" => Ok( KeyboardKey::PageUp ),

      // UI keys
      "Escape" => Ok( KeyboardKey::Escape ),
      "CapsLock" => Ok( KeyboardKey::CapsLock ),
      "ScrollLock" => Ok( KeyboardKey::ScrollLock ),
      "NumLock" => Ok( KeyboardKey::NumLock ),
      "PrintScreen" => Ok( KeyboardKey::PrintScreen ),
      "Pause" => Ok( KeyboardKey::Pause ),
      "ContextMenu" => Ok( KeyboardKey::ContextMenu ),

      // Function keys
      "F1" => Ok( KeyboardKey::F1 ),
      "F2" => Ok( KeyboardKey::F2 ),
      "F3" => Ok( KeyboardKey::F3 ),
      "F4" => Ok( KeyboardKey::F4 ),
      "F5" => Ok( KeyboardKey::F5 ),
      "F6" => Ok( KeyboardKey::F6 ),
      "F7" => Ok( KeyboardKey::F7 ),
      "F8" => Ok( KeyboardKey::F8 ),
      "F9" => Ok( KeyboardKey::F9 ),
      "F10" => Ok( KeyboardKey::F10 ),
      "F11" => Ok( KeyboardKey::F11 ),
      "F12" => Ok( KeyboardKey::F12 ),
      "F13" => Ok( KeyboardKey::F13 ),
      "F14" => Ok( KeyboardKey::F14 ),
      "F15" => Ok( KeyboardKey::F15 ),
      "F16" => Ok( KeyboardKey::F16 ),
      "F17" => Ok( KeyboardKey::F17 ),
      "F18" => Ok( KeyboardKey::F18 ),
      "F19" => Ok( KeyboardKey::F19 ),
      "F20" => Ok( KeyboardKey::F20 ),
      "F21" => Ok( KeyboardKey::F21 ),
      "F22" => Ok( KeyboardKey::F22 ),
      "F23" => Ok( KeyboardKey::F23 ),
      "F24" => Ok( KeyboardKey::F24 ),

      // Editing keys
      "Backspace" => Ok( KeyboardKey::Backspace ),
      "Clear" => Ok( KeyboardKey::Clear ),
      "Delete" => Ok( KeyboardKey::Delete ),
      "Insert" => Ok( KeyboardKey::Insert ),

      // Alphanumeric keys
      "Digit0" => Ok( KeyboardKey::Digit0 ),
      "Digit1" => Ok( KeyboardKey::Digit1 ),
      "Digit2" => Ok( KeyboardKey::Digit2 ),
      "Digit3" => Ok( KeyboardKey::Digit3 ),
      "Digit4" => Ok( KeyboardKey::Digit4 ),
      "Digit5" => Ok( KeyboardKey::Digit5 ),
      "Digit6" => Ok( KeyboardKey::Digit6 ),
      "Digit7" => Ok( KeyboardKey::Digit7 ),
      "Digit8" => Ok( KeyboardKey::Digit8 ),
      "Digit9" => Ok( KeyboardKey::Digit9 ),
      "KeyA" => Ok( KeyboardKey::KeyA ),
      "KeyB" => Ok( KeyboardKey::KeyB ),
      "KeyC" => Ok( KeyboardKey::KeyC ),
      "KeyD" => Ok( KeyboardKey::KeyD ),
      "KeyE" => Ok( KeyboardKey::KeyE ),
      "KeyF" => Ok( KeyboardKey::KeyF ),
      "KeyG" => Ok( KeyboardKey::KeyG ),
      "KeyH" => Ok( KeyboardKey::KeyH ),
      "KeyI" => Ok( KeyboardKey::KeyI ),
      "KeyJ" => Ok( KeyboardKey::KeyJ ),
      "KeyK" => Ok( KeyboardKey::KeyK ),
      "KeyL" => Ok( KeyboardKey::KeyL ),
      "KeyM" => Ok( KeyboardKey::KeyM ),
      "KeyN" => Ok( KeyboardKey::KeyN ),
      "KeyO" => Ok( KeyboardKey::KeyO ),
      "KeyP" => Ok( KeyboardKey::KeyP ),
      "KeyQ" => Ok( KeyboardKey::KeyQ ),
      "KeyR" => Ok( KeyboardKey::KeyR ),
      "KeyS" => Ok( KeyboardKey::KeyS ),
      "KeyT" => Ok( KeyboardKey::KeyT ),
      "KeyU" => Ok( KeyboardKey::KeyU ),
      "KeyV" => Ok( KeyboardKey::KeyV ),
      "KeyW" => Ok( KeyboardKey::KeyW ),
      "KeyX" => Ok( KeyboardKey::KeyX ),
      "KeyY" => Ok( KeyboardKey::KeyY ),
      "KeyZ" => Ok( KeyboardKey::KeyZ ),

      // Numpad keys
      "Numpad0" => Ok( KeyboardKey::Numpad0 ),
      "Numpad1" => Ok( KeyboardKey::Numpad1 ),
      "Numpad2" => Ok( KeyboardKey::Numpad2 ),
      "Numpad3" => Ok( KeyboardKey::Numpad3 ),
      "Numpad4" => Ok( KeyboardKey::Numpad4 ),
      "Numpad5" => Ok( KeyboardKey::Numpad5 ),
      "Numpad6" => Ok( KeyboardKey::Numpad6 ),
      "Numpad7" => Ok( KeyboardKey::Numpad7 ),
      "Numpad8" => Ok( KeyboardKey::Numpad8 ),
      "Numpad9" => Ok( KeyboardKey::Numpad9 ),
      "NumpadAdd" => Ok( KeyboardKey::NumpadAdd ),
      "NumpadSubtract" => Ok( KeyboardKey::NumpadSubtract ),
      "NumpadMultiply" => Ok( KeyboardKey::NumpadMultiply ),
      "NumpadDivide" => Ok( KeyboardKey::NumpadDivide ),
      "NumpadEnter" => Ok( KeyboardKey::NumpadEnter ),
      "NumpadDecimal" => Ok( KeyboardKey::NumpadDecimal ),
      "NumpadEqual" => Ok( KeyboardKey::NumpadEqual ),
      "NumpadComma" => Ok( KeyboardKey::NumpadComma ),

      // Symbol keys
      "Backquote" => Ok( KeyboardKey::Backquote ),
      "BracketLeft" => Ok( KeyboardKey::BracketLeft ),
      "BracketRight" => Ok( KeyboardKey::BracketRight ),
      "Comma" => Ok( KeyboardKey::Comma ),
      "Period" => Ok( KeyboardKey::Period ),
      "Semicolon" => Ok( KeyboardKey::Semicolon ),
      "Quote" => Ok( KeyboardKey::Quote ),
      "Backslash" => Ok( KeyboardKey::Backslash ),
      "Slash" => Ok( KeyboardKey::Slash ),
      "Minus" => Ok( KeyboardKey::Minus ),
      "Equal" => Ok( KeyboardKey::Equal ),
      "IntlBackslash" => Ok( KeyboardKey::IntlBackslash ),
      "IntlRo" => Ok( KeyboardKey::IntlRo ),
      "IntlYen" => Ok( KeyboardKey::IntlYen ),

      // Media keys
      "AudioVolumeDown" => Ok( KeyboardKey::AudioVolumeDown ),
      "AudioVolumeMute" => Ok( KeyboardKey::AudioVolumeMute ),
      "AudioVolumeUp" => Ok( KeyboardKey::AudioVolumeUp ),
      "BrowserBack" => Ok( KeyboardKey::BrowserBack ),
      "BrowserFavorites" => Ok( KeyboardKey::BrowserFavorites ),
      "BrowserForward" => Ok( KeyboardKey::BrowserForward ),
      "BrowserHome" => Ok( KeyboardKey::BrowserHome ),
      "BrowserRefresh" => Ok( KeyboardKey::BrowserRefresh ),
      "BrowserSearch" => Ok( KeyboardKey::BrowserSearch ),
      "BrowserStop" => Ok( KeyboardKey::BrowserStop ),
      "Eject" => Ok( KeyboardKey::Eject ),
      "LaunchApp1" => Ok( KeyboardKey::LaunchApp1 ),
      "LaunchApp2" => Ok( KeyboardKey::LaunchApp2 ),
      "LaunchMail" => Ok( KeyboardKey::LaunchMail ),
      "MediaPlayPause" => Ok( KeyboardKey::MediaPlayPause ),
      "MediaStop" => Ok( KeyboardKey::MediaStop ),
      "MediaTrackNext" => Ok( KeyboardKey::MediaTrackNext ),
      "MediaTrackPrevious" => Ok( KeyboardKey::MediaTrackPrevious ),
      "Power" => Ok( KeyboardKey::Power ),
      "Sleep" => Ok( KeyboardKey::Sleep ),
      "WakeUp" => Ok( KeyboardKey::WakeUp ),

      // International keys
      "Lang1" => Ok( KeyboardKey::Lang1 ),
      "Lang2" => Ok( KeyboardKey::Lang2 ),
      "Lang3" => Ok( KeyboardKey::Lang3 ),
      "Lang4" => Ok( KeyboardKey::Lang4 ),
      "Lang5" => Ok( KeyboardKey::Lang5 ),
      "Convert" => Ok( KeyboardKey::Convert ),
      "NonConvert" => Ok( KeyboardKey::NonConvert ),
      "KanaMode" => Ok( KeyboardKey::KanaMode ),

      // Unknown key
      _ => Ok( KeyboardKey::Unidentified ),
    }
  }
}
