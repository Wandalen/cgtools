//! This module provides structures and traits for working with different coordinate systems,
//! primarily focused on hexagonal grids and their conversion to and from pixel space.

/// Defines coordinate systems and operations for hexagonal grids.
pub mod hexagonal;
/// Defines a coordinate system for pixel-based (2D Cartesian) space.
pub mod pixel;

/// A trait for calculating the distance between two points in a grid.
pub trait Distance
{
  /// Calculates the grid-specific distance between this point and another.
  fn distance( &self, other: &Self ) -> u32;
}

/// A trait for finding all adjacent coordinates to a point in a grid.
pub trait Neigbors : Sized
{
  /// Returns a `Vec` containing all direct neighbors of the current coordinate.
  fn neighbors( &self ) -> Vec< Self >;
}
