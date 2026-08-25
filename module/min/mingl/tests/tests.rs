//!
//! This file serves as the entry point for the test suite of the `mingl` crate.
//! It aggregates various test modules to ensure the crate's functionality is correct.
//!

use mingl as the_module;

/// The primary module containing all tests for the `mingl` crate.
mod tests
{
  use super::*;

  /// Contains tests specifically related to `ndarray` functionalities.
  mod nd_test;

  /// Verifies VectorDataType descriptor invariants across primitives.
  mod data_type_test;

  /// Tests for bounding box calculations
  mod bounding_box;

  /// Tests for bounding sphere calculations
  mod bounding_sphere;

  /// Verifies `model::obj::num_faces_compute`'s face-count derivation from `tobj` mesh data.
  #[ cfg( feature = "model_obj" ) ]
  mod model_obj_test;

  /// Tests for camera orbit controls
  #[ cfg( feature = "camera_orbit_controls" ) ]
  mod camera_orbit_controls;

  /// Tests for character controls
  #[ cfg( all( feature = "character_controls", feature = "web" ) ) ]
  mod character_controls;

  /// Verifies `web::file` URL resolution and data-URL payload helpers.
  #[ cfg( feature = "web" ) ]
  mod web_file_test;
}
