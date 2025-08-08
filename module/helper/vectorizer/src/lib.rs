//! Vector graphics processing and conversion utilities.
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

#[ cfg( feature = "enabled" ) ]
mod private {}

#[ cfg( feature = "enabled" ) ]
::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;
  own use ::visioncortex;
  own use ::palette;
  own use ::fastrand;

  layer svg;
  layer actions;
  layer commands;
}