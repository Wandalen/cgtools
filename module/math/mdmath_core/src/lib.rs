//! Core multidimensional mathematics library.
#![ doc( html_root_url = "https://docs.rs/mdmath_core/latest/mdmath_core/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Core multidimensional mathematics library" ) ]

#[ cfg( feature = "enabled" ) ]
use ::mod_interface::mod_interface;

#[ cfg( feature = "enabled" ) ]
mod private
{
}

#[ cfg( feature = "enabled" ) ]
crate::mod_interface!
{

  /// Approximate equality for floating-point types can be determined using either relative difference
  /// or comparisons based on units in the last place (ULPs).
  #[ cfg( feature = "approx" ) ]
  layer approx;

  /// Multidimensional indices.
  #[ cfg( feature = "index" ) ]
  layer index;

  /// Describe general floats and operations on them.
  #[ cfg( feature = "float" ) ]
  layer float;

  /// General math traits.
  #[ cfg( feature = "general" ) ]
  layer general;

  /// Reusing `nd_array`.
  #[ cfg( feature = "nd" ) ]
  layer nd;

  /// Strides for plain multidemnsional space.
  layer plain;

  /// General traits, not necessarily special for math.
  layer traits;

  /// Univeral vector.
  layer vector;

}
