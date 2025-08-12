//! Isometric grid coordinate system implementation for pseudo-3D visualization.
//!
//! This module provides coordinate systems for isometric grids, commonly used in
//! strategy games, city builders, and RPGs for creating pseudo-3D visual effects
//! while maintaining 2D grid logic underneath.
//!
//! # Coordinate Systems
//!
//! The isometric coordinate system transforms a regular square grid into a 
//! diamond-shaped visual representation rotated 45 degrees. This creates the
//! illusion of depth while maintaining simple 2D coordinate mathematics.
//!
//! # Screen Coordinates vs World Coordinates
//!
//! - **World Coordinates**: Logical (x, y) grid positions in the underlying square grid
//! - **Screen Coordinates**: Visual pixel positions after isometric transformation
//!
//! The transformation involves:
//! - Rotating the grid 45 degrees
//! - Scaling the y-axis by ~0.5 to create the diamond shape
//! - Applying perspective adjustments for pseudo-3D effect
//!
//! # Examples
//!
//! ```rust
//! use tiles_tools::coordinates::isometric::{Coordinate, Diamond};
//! use tiles_tools::coordinates::{Distance, Neighbors};
//! use tiles_tools::coordinates::pixel::Pixel;
//! 
//! // Create an isometric coordinate
//! let coord = Coordinate::<Diamond>::new(3, 2);
//! 
//! // Get 4 orthogonal neighbors (inherits square grid logic)
//! let neighbors = coord.neighbors();
//! assert_eq!(neighbors.len(), 4);
//! 
//! // Convert to screen coordinates for rendering
//! let screen_pos: Pixel = coord.to_screen(32.0); // 32 pixels per tile
//! 
//! // Convert screen coordinates back to world coordinates
//! let world_coord = Coordinate::<Diamond>::from_screen(screen_pos, 32.0);
//! assert_eq!(world_coord, coord);
//! ```

use crate::coordinates::{ Distance, Neighbors };
use crate::coordinates::pixel::Pixel;
use serde::{ Deserialize, Serialize };
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

/// Marker struct representing the diamond-shaped isometric grid system.
/// 
/// Diamond isometric projection creates a diamond/rhombus visual appearance
/// from an underlying square grid, commonly used in strategy and simulation games.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Diamond;

/// Represents a coordinate in an isometric grid system.
///
/// The isometric coordinate system maintains logical square grid coordinates
/// internally while providing transformation methods to convert between
/// world coordinates and screen coordinates for rendering.
///
/// # Coordinate Transformation
///
/// The isometric transformation applies these operations:
/// 1. Rotate the square grid 45 degrees
/// 2. Scale the y-axis to create the diamond shape  
/// 3. Apply tile size scaling for pixel positioning
///
/// # Examples
///
/// ```rust
/// use tiles_tools::coordinates::isometric::{Coordinate, Diamond};
/// 
/// let coord = Coordinate::<Diamond>::new(2, 3);
/// assert_eq!(coord.x, 2);
/// assert_eq!(coord.y, 3);
/// 
/// // Check if coordinate is valid
/// assert!(coord.is_valid());
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Coordinate<Projection> {
  /// The x-coordinate in the logical grid
  pub x: i32,
  /// The y-coordinate in the logical grid
  pub y: i32,
  /// Phantom marker for the projection type
  #[serde(skip)]
  pub _marker: PhantomData<Projection>,
}

impl<Projection> Coordinate<Projection> {
  /// Creates a new isometric coordinate.
  ///
  /// # Arguments
  /// * `x` - The x-coordinate in the logical grid
  /// * `y` - The y-coordinate in the logical grid
  ///
  /// # Examples
  /// ```rust
  /// use tiles_tools::coordinates::isometric::{Coordinate, Diamond};
  /// 
  /// let coord = Coordinate::<Diamond>::new(5, 3);
  /// assert_eq!(coord.x, 5);
  /// assert_eq!(coord.y, 3);
  /// ```
  pub const fn new(x: i32, y: i32) -> Self {
    Self {
      x,
      y,
      _marker: PhantomData,
    }
  }

  /// Returns true if this coordinate represents a valid grid position.
  /// 
  /// All finite integer coordinates are considered valid in isometric grids.
  ///
  /// # Examples
  /// ```rust
  /// use tiles_tools::coordinates::isometric::{Coordinate, Diamond};
  /// 
  /// let coord = Coordinate::<Diamond>::new(-10, 15);
  /// assert!(coord.is_valid());
  /// ```
  pub fn is_valid(&self) -> bool {
    // All finite integer coordinates are valid in isometric grids
    true
  }
}

impl Coordinate<Diamond> {
  /// Converts this world coordinate to screen pixel coordinates.
  ///
  /// Applies the isometric diamond transformation to convert logical
  /// grid coordinates to visual screen positions.
  ///
  /// # Arguments
  /// * `tile_size` - The width/height of each tile in pixels
  ///
  /// # Returns
  /// A `Pixel` representing the screen position for rendering
  ///
  /// # Examples
  /// ```rust
  /// use tiles_tools::coordinates::isometric::{Coordinate, Diamond};
  /// 
  /// let coord = Coordinate::<Diamond>::new(2, 1);
  /// let screen_pos = coord.to_screen(32.0);
  /// 
  /// // Isometric transformation: x_screen = (x - y) * tile_size/2
  /// //                          y_screen = (x + y) * tile_size/4
  /// ```
  pub fn to_screen(&self, tile_size: f32) -> Pixel {
    // Standard isometric diamond transformation
    let x_screen = (self.x - self.y) as f32 * (tile_size / 2.0);
    let y_screen = (self.x + self.y) as f32 * (tile_size / 4.0);
    
    Pixel::new(x_screen, y_screen)
  }

  /// Converts screen pixel coordinates back to world coordinates.
  ///
  /// Applies the inverse isometric transformation to convert screen
  /// positions back to logical grid coordinates.
  ///
  /// # Arguments
  /// * `screen_pos` - The pixel position on screen
  /// * `tile_size` - The width/height of each tile in pixels
  ///
  /// # Returns
  /// The world coordinate corresponding to the screen position
  ///
  /// # Examples
  /// ```rust
  /// use tiles_tools::coordinates::isometric::{Coordinate, Diamond};
  /// use tiles_tools::coordinates::pixel::Pixel;
  /// 
  /// let screen_pos = Pixel::new(32.0, 24.0);
  /// let coord = Coordinate::<Diamond>::from_screen(screen_pos, 32.0);
  /// ```
  pub fn from_screen(screen_pos: Pixel, tile_size: f32) -> Self {
    // Inverse isometric transformation
    let x_norm = screen_pos.x() / (tile_size / 2.0);
    let y_norm = screen_pos.y() / (tile_size / 4.0);
    
    // Solve the system of equations:
    // x_norm = x - y  =>  x = (x_norm + y_norm) / 2
    // y_norm = x + y  =>  y = (y_norm - x_norm) / 2
    let x = ((x_norm + y_norm) / 2.0).round() as i32;
    let y = ((y_norm - x_norm) / 2.0).round() as i32;
    
    Self::new(x, y)
  }

  /// Returns the four corner points of this tile in screen coordinates.
  ///
  /// Useful for rendering tile boundaries or collision detection.
  ///
  /// # Arguments
  /// * `tile_size` - The width/height of each tile in pixels
  ///
  /// # Returns
  /// Array of 4 `Pixel` coordinates representing the diamond corners
  /// in clockwise order: top, right, bottom, left
  pub fn tile_corners(&self, tile_size: f32) -> [Pixel; 4] {
    let center = self.to_screen(tile_size);
    let half_width = tile_size / 2.0;
    let half_height = tile_size / 4.0;
    
    [
      Pixel::new(center.x(), center.y() - half_height), // Top
      Pixel::new(center.x() + half_width, center.y()),  // Right  
      Pixel::new(center.x(), center.y() + half_height), // Bottom
      Pixel::new(center.x() - half_width, center.y()),  // Left
    ]
  }
}

impl From<(i32, i32)> for Coordinate<Diamond> {
  /// Creates an isometric coordinate from a tuple (x, y).
  fn from((x, y): (i32, i32)) -> Self {
    Self::new(x, y)
  }
}

impl From<[i32; 2]> for Coordinate<Diamond> {
  /// Creates an isometric coordinate from an array [x, y].
  fn from([x, y]: [i32; 2]) -> Self {
    Self::new(x, y)
  }
}

impl Into<(i32, i32)> for Coordinate<Diamond> {
  /// Converts the isometric coordinate into a tuple (x, y).
  fn into(self) -> (i32, i32) {
    (self.x, self.y)
  }
}

impl Into<[i32; 2]> for Coordinate<Diamond> {
  /// Converts the isometric coordinate into an array [x, y].
  fn into(self) -> [i32; 2] {
    [self.x, self.y]
  }
}

/// Distance calculation for isometric coordinates.
///
/// Isometric grids use the same distance calculation as square grids
/// since they're built on top of square grid logic. The visual
/// transformation doesn't affect logical distances.
impl Distance for Coordinate<Diamond> {
  /// Calculates Manhattan distance between two isometric coordinates.
  ///
  /// Since isometric grids are based on square grids, we use Manhattan
  /// distance for the underlying logical coordinates.
  ///
  /// # Examples
  /// ```rust
  /// use tiles_tools::coordinates::isometric::{Coordinate, Diamond};
  /// use tiles_tools::coordinates::Distance;
  /// 
  /// let coord1 = Coordinate::<Diamond>::new(0, 0);
  /// let coord2 = Coordinate::<Diamond>::new(3, 4);
  /// let distance = coord1.distance(&coord2);
  /// assert_eq!(distance, 7); // |3-0| + |4-0| = 7
  /// ```
  fn distance(&self, other: &Self) -> u32 {
    // Manhattan distance for underlying square grid
    ((self.x - other.x).abs() + (self.y - other.y).abs()) as u32
  }
}

/// Four-neighbor connectivity for isometric coordinates.
///
/// Isometric grids maintain the same neighbor relationships as square grids
/// in the logical coordinate space, even though they appear diamond-shaped
/// when rendered.
impl Neighbors for Coordinate<Diamond> {
  /// Returns the 4 orthogonal neighbors of this isometric coordinate.
  ///
  /// The neighbors correspond to the logical square grid neighbors:
  /// right, left, up, down in world coordinates.
  ///
  /// # Examples
  /// ```rust
  /// use tiles_tools::coordinates::isometric::{Coordinate, Diamond};
  /// use tiles_tools::coordinates::Neighbors;
  /// 
  /// let coord = Coordinate::<Diamond>::new(2, 3);
  /// let neighbors = coord.neighbors();
  /// assert_eq!(neighbors.len(), 4);
  /// 
  /// // Neighbors are: (3,3), (1,3), (2,4), (2,2)
  /// ```
  fn neighbors(&self) -> Vec<Self> {
    vec![
      Self::new(self.x + 1, self.y),     // Right
      Self::new(self.x - 1, self.y),     // Left
      Self::new(self.x, self.y + 1),     // Up (in world coordinates)
      Self::new(self.x, self.y - 1),     // Down (in world coordinates)
    ]
  }
}

/// Convenience type alias for diamond isometric coordinates.
///
/// Equivalent to `Coordinate<Diamond>` with Manhattan distance calculation
/// and orthogonal neighbor finding, but with isometric screen transformations.
pub type IsometricCoord = Coordinate<Diamond>;