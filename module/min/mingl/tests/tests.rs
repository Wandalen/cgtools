//!
//! This file serves as the entry point for the test suite of the `mingl` crate.
//! It aggregates various test modules to ensure the crate's functionality is correct.
//!

#[ allow( unused_imports ) ]
use test_tools::exposed::*;
#[ allow( unused_imports ) ]
use mingl as the_module;

/// The primary module containing all tests for the `mingl` crate.
mod tests
{
  #[ allow( unused_imports ) ]
  use super::*;

  /// Contains tests specifically related to `ndarray` functionalities.
  mod nd_test;

  /// Tests for bounding box calculations
  mod bounding_box;

  /// Tests for bounding sphere calculations
  mod bounding_sphere;

  /// Tests for camera orbit controls
  mod camera_orbit_controls;
}
