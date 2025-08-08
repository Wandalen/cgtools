//! Line drawing and manipulation utilities for 2D and 3D graphics.
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

mod private
{

}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  layer d2;
  layer d3;

  layer joins;
  layer caps;

  layer mesh;
  layer program;

  layer helpers;
}