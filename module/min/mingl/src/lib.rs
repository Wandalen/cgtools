#![ doc = include_str!( "../readme.md" ) ]

// use error::prelude::*;
// use former::Former;
// use derive_tools::{ New };

#[ cfg( feature = "enabled" ) ]
mod private {}

#[ cfg( feature = "enabled" ) ]
::mod_interface::mod_interface!
{

  // own use ::error;
  // own use ::log;
  own use ::mod_interface::mod_interface;

  /// Derives.
  layer derive;

  /// Errors handling.
  layer error;
  /// Buffer-related.
  layer buffer;
  /// Descriptors of primitive data types.
  layer data_type;
  /// Memory-related entities.
  layer mem;

  // Camera with controls
  #[ cfg( feature = "camera_orbit_controls" ) ]
  layer camera_orbit_controls;

  #[ cfg( feature = "diagnostics" ) ]
  layer diagnostics;

  #[ cfg( feature = "model" ) ]
  layer model;

  /// Multi-dimensional math.
  #[ cfg( feature = "ndarray" ) ]
  layer nd;

}
