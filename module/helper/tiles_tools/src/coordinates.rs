//! This module provides structures and traits for working with different coordinate systems,
//! supporting hexagonal, square, triangular, isometric, and pixel coordinate spaces with comprehensive conversion utilities.

/// Universal coordinate system conversion utilities.
pub mod conversion;
/// Defines coordinate systems and operations for hexagonal grids.
pub mod hexagonal;
/// Defines coordinate systems for isometric grids with pseudo-3D visualization.
pub mod isometric;
/// Defines a coordinate system for pixel-based (2D Cartesian) space.
pub mod pixel;
/// Defines coordinate systems for square/rectangular grids.
pub mod square;
/// Defines coordinate systems for triangular grids.
pub mod triangular;

/// A trait for calculating the distance between two points in a grid.
pub trait Distance
{
  /// Calculates the grid-specific distance between this point and another.
  fn distance( &self, other : &Self ) -> u32;
}

/// A trait for finding all adjacent coordinates to a point in a grid.
pub trait Neighbors : Sized
{
  /// Returns a `Vec` containing all direct neighbors of the current coordinate.
  fn neighbors( &self ) -> Vec< Self >;
}
