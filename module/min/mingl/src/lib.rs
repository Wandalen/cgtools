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
  #[ cfg( all( feature = "math", feature = "camera_orbit_controls" ) ) ]
  layer camera_orbit_controls;

  #[ cfg( all( feature = "math", feature = "diagnostics" ) ) ]
  layer diagnostics;

  /// Utilities related to different models
  #[ cfg( feature = "math" ) ]
  layer model;

  /// Multi-dimensional math.
  #[ cfg( feature = "math" ) ]
  layer math;

  /// Geometry math.
  #[ cfg( feature = "math" ) ]
  layer geometry;

  /// Web related stuff
  #[ cfg( feature = "web" ) ]
  layer web;

}
