#![ doc = include_str!( "../readme.md" ) ]

#[ cfg( feature = "enabled" ) ]
mod private {}

#[ cfg( feature = "enabled" ) ]
::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;
  own use ::visioncortex;
  own use ::palette;
  own use ::fastrand;

  /// Module for converting to svg
  layer svg;
  /// Vectorization functionality
  layer actions;
  /// Module for dealing with CLI commands
  layer commands;
}