//! Integration tests for tiles_tools
//!
//! All integration tests are feature-gated with the "integration" feature
//! to allow for selective test execution.

#![cfg(feature = "integration")]

mod conversion_tests;
mod coordinates_tests;
mod ecs_tests;
mod isometric_coords_tests;
mod square_coords_tests;
mod triangular_coords_tests;
// mod grid_tests;
// mod pathfinding_tests;
// mod generation_tests;
