//! MouseButton for representing the different mouse buttons as defined by the MouseEvent.button property

use std::str::FromStr;
use strum::EnumCount;

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
  /// Left button (0)
  Main,
  /// Middle button (1)
  Auxiliary,
  /// Right button (2)
  Secondary,
  /// Back button (3)
  Back,
  /// Forward button (4)
  Forward,
  /// For any other values
  Unknown,
}

impl MouseButton
{
  /// Convert a numeric button value to the corresponding MouseButton enum variant
  pub const fn from_button( button : i16 ) -> Self
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
  pub const fn name( &self ) -> &'static str
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
  pub const fn is_main( &self ) -> bool
  {
    matches!( self, MouseButton::Main )
  }

  /// Check if this is the secondary (usually right) button
  pub const fn is_secondary( &self ) -> bool
  {
    matches!( self, MouseButton::Secondary )
  }

  /// Check if this is the auxiliary (usually middle/wheel) button
  pub const fn is_auxiliary( &self ) -> bool
  {
    matches!( self, MouseButton::Auxiliary )
  }

  /// Check if this is a navigation button (Back/Forward)
  pub const fn is_navigation( &self ) -> bool
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
