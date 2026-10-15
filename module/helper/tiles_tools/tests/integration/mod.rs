//! Integration tests for tiles_tools
//!
//! All integration tests are feature-gated with the "integration" feature
//! to allow for selective test execution.

#![cfg(feature = "integration")]

mod conversion_tests;
mod coordinates_tests;
mod ecs_tests;
mod field_of_view_tests;
mod flowfield_tests;
mod geometry_tests;
mod isometric_coords_tests;
mod square_coords_tests;
mod triangular_coords_tests;
