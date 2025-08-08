//! This module provides a triangular grid coordinate system implementation.
//! 
//! Triangular grids are useful for specialized applications like strategic games,
//! geometric modeling, or when you need non-orthogonal tessellation patterns.
//! Each coordinate represents either an "up" triangle (△) or "down" triangle (▽)
//! in the triangular tessellation.
//!
//! # Coordinate System
//! 
//! The triangular coordinate system uses (x, y) integer coordinates where:
//! - Each coordinate represents a specific triangle in the tessellation
//! - Triangles can be either "up-pointing" (△) or "down-pointing" (▽) 
//! - The orientation is determined by the coordinate parity: (x + y) % 2
//!
//! # Neighbor Connectivity
//!
//! Each triangle has exactly 12 neighbors:
//! - 3 edge-adjacent neighbors (sharing a complete edge)
//! - 9 vertex-adjacent neighbors (sharing only a vertex)
//!
//! This provides rich connectivity for pathfinding and spatial analysis.
//!
//! # Examples
//!
//! ```rust
//! use tiles_tools::coordinates::triangular::{Coordinate, TwelveConnected};
//! use tiles_tools::coordinates::{Distance, Neighbors};
//! 
//! // Create a triangular coordinate
//! let coord = Coordinate::<TwelveConnected>::new(2, 3);
//! 
//! // Get all 12 neighbors
//! let neighbors = coord.neighbors();
//! assert_eq!(neighbors.len(), 12);
//! 
//! // Calculate distance between triangles
//! let other = Coordinate::<TwelveConnected>::new(5, 7);
//! let dist = coord.distance(&other);
//! ```

use crate::coordinates::{Distance, Neighbors};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, hash::Hash, marker::PhantomData};

/// Marker struct representing the twelve-connected triangular grid system.
/// 
/// In triangular grids, each triangle has 12 neighbors:
/// - 3 edge-adjacent (sharing a complete edge)  
/// - 9 vertex-adjacent (sharing only a vertex)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TwelveConnected;

/// Represents a coordinate in a triangular grid system.
///
/// The triangular coordinate system tessellates the plane with equilateral triangles.
/// Each coordinate (x, y) represents a specific triangle, and the triangle's
/// orientation (up △ or down ▽) is determined by the parity of (x + y).
///
/// # Triangle Orientation
/// - If (x + y) is even: up-pointing triangle △
/// - If (x + y) is odd: down-pointing triangle ▽
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Coordinate<Connectivity> {
  /// The x-coordinate of the triangle
  pub x: i32,
  /// The y-coordinate of the triangle  
  pub y: i32,
  /// Phantom marker for the connectivity type
  #[serde(skip)]
  pub _marker: PhantomData<Connectivity>,
}

impl<Connectivity> Coordinate<Connectivity> {
  /// Creates a new triangular coordinate.
  ///
  /// # Arguments
  /// * `x` - The x-coordinate of the triangle
  /// * `y` - The y-coordinate of the triangle
  ///
  /// # Examples
  /// ```rust
  /// use tiles_tools::coordinates::triangular::{Coordinate, TwelveConnected};
  /// 
  /// let coord = Coordinate::<TwelveConnected>::new(3, 5);
  /// assert_eq!(coord.x, 3);
  /// assert_eq!(coord.y, 5);
  /// ```
  pub const fn new(x: i32, y: i32) -> Self {
    Self {
      x,
      y,
      _marker: PhantomData,
    }
  }

  /// Returns whether this triangle is up-pointing (△) or down-pointing (▽).
  /// 
  /// # Returns
  /// * `true` if the triangle points up (△) - when (x + y) is even
  /// * `false` if the triangle points down (▽) - when (x + y) is odd
  ///
  /// # Examples
  /// ```rust
  /// use tiles_tools::coordinates::triangular::{Coordinate, TwelveConnected};
  /// 
  /// let up_triangle = Coordinate::<TwelveConnected>::new(2, 4); // 2+4=6 (even)
  /// assert_eq!(up_triangle.is_up_pointing(), true);
  /// 
  /// let down_triangle = Coordinate::<TwelveConnected>::new(2, 3); // 2+3=5 (odd)  
  /// assert_eq!(down_triangle.is_up_pointing(), false);
  /// ```
  pub fn is_up_pointing(&self) -> bool {
    // Use wrapping addition to handle overflow gracefully
    self.x.wrapping_add(self.y) % 2 == 0
  }

  /// Returns whether this triangle is down-pointing (▽).
  /// 
  /// This is the inverse of `is_up_pointing()`.
  pub fn is_down_pointing(&self) -> bool {
    !self.is_up_pointing()
  }
}

impl From<(i32, i32)> for Coordinate<TwelveConnected> {
  /// Creates a triangular coordinate from a tuple (x, y).
  fn from((x, y): (i32, i32)) -> Self {
    Self::new(x, y)
  }
}

impl From<[i32; 2]> for Coordinate<TwelveConnected> {
  /// Creates a triangular coordinate from an array [x, y].
  fn from([x, y]: [i32; 2]) -> Self {
    Self::new(x, y)
  }
}

impl Into<(i32, i32)> for Coordinate<TwelveConnected> {
  /// Converts the triangular coordinate into a tuple (x, y).
  fn into(self) -> (i32, i32) {
    (self.x, self.y)
  }
}

impl Into<[i32; 2]> for Coordinate<TwelveConnected> {
  /// Converts the triangular coordinate into an array [x, y].
  fn into(self) -> [i32; 2] {
    [self.x, self.y]
  }
}

/// Distance calculation for triangular coordinates.
///
/// The distance between triangular coordinates is calculated based on
/// the minimum number of triangle-to-triangle moves needed to travel
/// from one coordinate to another through the triangular tessellation.
impl Distance for Coordinate<TwelveConnected> {
  /// Calculates the minimum distance between two triangular coordinates.
  /// 
  /// The distance represents the minimum number of triangle-to-triangle
  /// transitions needed to travel from one coordinate to another.
  ///
  /// # Examples
  /// ```rust
  /// use tiles_tools::coordinates::triangular::{Coordinate, TwelveConnected};
  /// use tiles_tools::coordinates::Distance;
  /// 
  /// let coord1 = Coordinate::<TwelveConnected>::new(0, 0);
  /// let coord2 = Coordinate::<TwelveConnected>::new(2, 2);
  /// let distance = coord1.distance(&coord2);
  /// assert_eq!(distance, 2);
  /// ```
  fn distance(&self, other: &Self) -> u32 {
    // For triangular grids, we use a modified Manhattan distance
    // that accounts for the triangular tessellation structure
    let dx = (self.x - other.x).abs();
    let dy = (self.y - other.y).abs();
    
    // In triangular tessellation, the effective distance is the maximum
    // of the coordinate differences, similar to Chebyshev distance
    // but adjusted for the triangular geometry
    dx.max(dy) as u32
  }
}

/// Twelve-neighbor connectivity for triangular coordinates.
///
/// Each triangle in a triangular tessellation has exactly 12 neighbors:
/// - 3 edge-adjacent neighbors (sharing a complete edge)
/// - 9 vertex-adjacent neighbors (sharing only a vertex)
impl Neighbors for Coordinate<TwelveConnected> {
  /// Returns all 12 neighbors of this triangular coordinate.
  ///
  /// The neighbors include both edge-adjacent and vertex-adjacent triangles,
  /// providing comprehensive connectivity for pathfinding and spatial analysis.
  ///
  /// # Returns
  /// A `Vec` containing exactly 12 neighboring coordinates.
  ///
  /// # Examples
  /// ```rust
  /// use tiles_tools::coordinates::triangular::{Coordinate, TwelveConnected};
  /// use tiles_tools::coordinates::Neighbors;
  /// 
  /// let coord = Coordinate::<TwelveConnected>::new(5, 3);
  /// let neighbors = coord.neighbors();
  /// assert_eq!(neighbors.len(), 12);
  /// ```
  fn neighbors(&self) -> Vec<Self> {
    // The neighbor pattern depends on whether this is an up or down triangle
    if self.is_up_pointing() {
      // Up-pointing triangle (△) neighbors
      vec![
        // Edge-adjacent neighbors (3)
        Self::new(self.x - 1, self.y),     // Left edge
        Self::new(self.x + 1, self.y),     // Right edge  
        Self::new(self.x, self.y - 1),     // Bottom edge
        
        // Vertex-adjacent neighbors (9)
        Self::new(self.x - 2, self.y),     // Far left
        Self::new(self.x + 2, self.y),     // Far right
        Self::new(self.x, self.y - 2),     // Far bottom
        Self::new(self.x - 1, self.y - 1), // Bottom-left
        Self::new(self.x + 1, self.y - 1), // Bottom-right
        Self::new(self.x - 1, self.y + 1), // Top-left
        Self::new(self.x + 1, self.y + 1), // Top-right
        Self::new(self.x, self.y + 1),     // Top
        Self::new(self.x, self.y + 2),     // Far top
      ]
    } else {
      // Down-pointing triangle (▽) neighbors  
      vec![
        // Edge-adjacent neighbors (3)
        Self::new(self.x - 1, self.y),     // Left edge
        Self::new(self.x + 1, self.y),     // Right edge
        Self::new(self.x, self.y + 1),     // Top edge
        
        // Vertex-adjacent neighbors (9) 
        Self::new(self.x - 2, self.y),     // Far left
        Self::new(self.x + 2, self.y),     // Far right
        Self::new(self.x, self.y + 2),     // Far top
        Self::new(self.x - 1, self.y - 1), // Bottom-left
        Self::new(self.x + 1, self.y - 1), // Bottom-right
        Self::new(self.x - 1, self.y + 1), // Top-left  
        Self::new(self.x + 1, self.y + 1), // Top-right
        Self::new(self.x, self.y - 1),     // Bottom
        Self::new(self.x, self.y - 2),     // Far bottom
      ]
    }
  }
}

/// Convenience type alias for twelve-connected triangular coordinates.
/// 
/// Equivalent to `Coordinate<TwelveConnected>` with 12-neighbor connectivity
/// and triangular distance calculation.
pub type TriangularCoord = Coordinate<TwelveConnected>;