//! Square/rectangular grid coordinate system implementation
//!
//! This module provides coordinate systems for square/rectangular grids with support
//! for both 4-connected (orthogonal only) and 8-connected (orthogonal + diagonal)
//! neighbor patterns.
//!
//! # Examples
//!
//! ```rust
//! use tiles_tools::coordinates::square::{Coordinate, FourConnected, EightConnected};
//! use tiles_tools::coordinates::{Distance, Neighbors};
//!
//! // Create a 4-connected square coordinate
//! let coord = Coordinate::<FourConnected>::new(2, 3);
//! let neighbors = coord.neighbors();
//! assert_eq!(neighbors.len(), 4);
//!
//! // Calculate Manhattan distance
//! let other = Coordinate::<FourConnected>::new(5, 7);
//! let distance = coord.distance(&other);
//! assert_eq!(distance, 7); // |5-2| + |7-3| = 3 + 4 = 7
//!
//! // Create an 8-connected square coordinate  
//! let coord8 = Coordinate::<EightConnected>::new(2, 3);
//! let neighbors8 = coord8.neighbors();
//! assert_eq!(neighbors8.len(), 8);
//!
//! // Calculate Chebyshev distance (max of x,y differences)
//! let other8 = Coordinate::<EightConnected>::new(5, 7);
//! let distance8 = coord8.distance(&other8);
//! assert_eq!(distance8, 4); // max(|5-2|, |7-3|) = max(3, 4) = 4
//! ```

use crate::coordinates::{ Distance, Neighbors };
use serde::{ Deserialize, Serialize };
use std::{ fmt::Debug, hash::Hash, marker::PhantomData };

/// Square grid system marker
#[derive(Debug)]
pub struct Square;

/// Four-connected neighbors (orthogonal only: up, down, left, right)
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Default ) ]
pub struct FourConnected;

/// Eight-connected neighbors (orthogonal + diagonal: all 8 surrounding cells)
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Default ) ]
pub struct EightConnected;

/// Square coordinate with connectivity type parameter
///
/// This coordinate system uses standard Cartesian coordinates (x, y) where:
/// - x increases rightward
/// - y increases upward (mathematical convention)
///
/// The connectivity type parameter determines neighbor patterns and distance calculations:
/// - `FourConnected`: Only orthogonal neighbors, Manhattan distance
/// - `EightConnected`: All 8 neighbors, Chebyshev distance
#[ derive( Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default ) ]
pub struct Coordinate< Connectivity >
{
  /// The x-coordinate (horizontal position)
  pub x : i32,
  /// The y-coordinate (vertical position)
  pub y : i32,
  /// Phantom data to hold the connectivity type
  #[ serde( skip ) ]
  pub _marker : PhantomData< Connectivity >,
}

impl< Connectivity > Coordinate< Connectivity >
{
  /// Creates a new square coordinate
  ///
  /// # Examples
  /// ```
  /// # use tiles_tools::coordinates::square::{ Coordinate, FourConnected };
  /// let coord = Coordinate::< FourConnected >::new( 3, 4 );
  /// assert_eq!( coord.x, 3 );
  /// assert_eq!( coord.y, 4 );
  /// ```
  pub const fn new( x : i32, y : i32 ) -> Self
  {
    Self
    {
      x,
      y,
      _marker : PhantomData,
    }
  }
}

impl< Connectivity > From< ( i32, i32 ) > for Coordinate< Connectivity >
{
  /// Creates a coordinate from a tuple (x, y)
  fn from( ( x, y ) : ( i32, i32 ) ) -> Self
  {
    Self::new( x, y )
  }
}

impl< Connectivity > From< [ i32; 2 ] > for Coordinate< Connectivity >
{
  /// Creates a coordinate from an array [x, y]
  fn from( [ x, y ] : [ i32; 2 ] ) -> Self
  {
    Self::new( x, y )
  }
}

impl< Connectivity > Into< ( i32, i32 ) > for Coordinate< Connectivity >
{
  /// Converts the coordinate into a tuple (x, y)
  fn into( self ) -> ( i32, i32 )
  {
    ( self.x, self.y )
  }
}

impl< Connectivity > std::ops::Add for Coordinate< Connectivity >
{
  type Output = Self;

  /// Adds two coordinates (vector addition)
  fn add( self, rhs : Self ) -> Self::Output
  {
    Self::new( self.x + rhs.x, self.y + rhs.y )
  }
}

impl< Connectivity > std::ops::Sub for Coordinate< Connectivity >
{
  type Output = Self;

  /// Subtracts two coordinates (vector subtraction)
  fn sub( self, rhs : Self ) -> Self::Output
  {
    Self::new( self.x - rhs.x, self.y - rhs.y )
  }
}

impl< Connectivity > std::ops::Mul< i32 > for Coordinate< Connectivity >
{
  type Output = Self;

  /// Scales a coordinate by an integer factor
  fn mul( self, rhs : i32 ) -> Self::Output
  {
    Self::new( self.x * rhs, self.y * rhs )
  }
}

impl< Connectivity > std::ops::Div< i32 > for Coordinate< Connectivity >
{
  type Output = Self;

  /// Divides a coordinate by an integer factor
  fn div( self, rhs : i32 ) -> Self::Output
  {
    Self::new( self.x / rhs, self.y / rhs )
  }
}

// Distance implementation for 4-connected (Manhattan distance)
impl Distance for Coordinate< FourConnected >
{
  /// Calculates Manhattan distance between two 4-connected coordinates
  ///
  /// Manhattan distance is the sum of absolute differences of coordinates,
  /// representing the minimum number of orthogonal moves needed.
  ///
  /// # Examples
  /// ```
  /// # use tiles_tools::coordinates::square::{ Coordinate, FourConnected };
  /// # use tiles_tools::coordinates::Distance;
  /// let coord1 = Coordinate::< FourConnected >::new( 0, 0 );
  /// let coord2 = Coordinate::< FourConnected >::new( 3, 4 );
  /// assert_eq!( coord1.distance( &coord2 ), 7 ); // |3-0| + |4-0| = 7
  /// ```
  fn distance( &self, other : &Self ) -> u32
  {
    ( ( self.x - other.x ).abs() + ( self.y - other.y ).abs() ) as u32
  }
}

// Distance implementation for 8-connected (Chebyshev distance)
impl Distance for Coordinate< EightConnected >
{
  /// Calculates Chebyshev distance between two 8-connected coordinates
  ///
  /// Chebyshev distance is the maximum of absolute differences of coordinates,
  /// representing the minimum number of moves when diagonal movement is allowed.
  ///
  /// # Examples
  /// ```
  /// # use tiles_tools::coordinates::square::{ Coordinate, EightConnected };
  /// # use tiles_tools::coordinates::Distance;
  /// let coord1 = Coordinate::< EightConnected >::new( 0, 0 );
  /// let coord2 = Coordinate::< EightConnected >::new( 3, 4 );
  /// assert_eq!( coord1.distance( &coord2 ), 4 ); // max(|3-0|, |4-0|) = max(3, 4) = 4
  /// ```
  fn distance( &self, other : &Self ) -> u32
  {
    ( ( self.x - other.x ).abs().max( ( self.y - other.y ).abs() ) ) as u32
  }
}

// Neighbors implementation for 4-connected
impl Neighbors for Coordinate< FourConnected >
{
  /// Returns the 4 orthogonal neighbors of this coordinate
  ///
  /// Returns neighbors in the order: right, left, up, down
  ///
  /// # Examples
  /// ```
  /// # use tiles_tools::coordinates::square::{ Coordinate, FourConnected };
  /// # use tiles_tools::coordinates::Neighbors;
  /// let coord = Coordinate::< FourConnected >::new( 2, 3 );
  /// let neighbors = coord.neighbors();
  /// assert_eq!( neighbors.len(), 4 );
  /// assert!( neighbors.contains( &Coordinate::new( 3, 3 ) ) ); // right
  /// assert!( neighbors.contains( &Coordinate::new( 1, 3 ) ) ); // left  
  /// assert!( neighbors.contains( &Coordinate::new( 2, 4 ) ) ); // up
  /// assert!( neighbors.contains( &Coordinate::new( 2, 2 ) ) ); // down
  /// ```
  fn neighbors( &self ) -> Vec< Self >
  {
    vec!
    [
      Self::new( self.x + 1, self.y ), // Right
      Self::new( self.x - 1, self.y ), // Left
      Self::new( self.x, self.y + 1 ), // Up
      Self::new( self.x, self.y - 1 ), // Down
    ]
  }
}

// Neighbors implementation for 8-connected
impl Neighbors for Coordinate< EightConnected >
{
  /// Returns the 8 surrounding neighbors of this coordinate
  ///
  /// Returns all orthogonal and diagonal neighbors
  ///
  /// # Examples
  /// ```
  /// # use tiles_tools::coordinates::square::{ Coordinate, EightConnected };
  /// # use tiles_tools::coordinates::Neighbors;
  /// let coord = Coordinate::< EightConnected >::new( 2, 3 );
  /// let neighbors = coord.neighbors();
  /// assert_eq!( neighbors.len(), 8 );
  /// // All 8 surrounding positions should be included
  /// assert!( neighbors.contains( &Coordinate::new( 3, 3 ) ) ); // right
  /// assert!( neighbors.contains( &Coordinate::new( 3, 4 ) ) ); // up-right (diagonal)
  /// ```
  fn neighbors( &self ) -> Vec< Self >
  {
    vec!
    [
      // Orthogonal neighbors
      Self::new( self.x + 1, self.y ), // Right
      Self::new( self.x - 1, self.y ), // Left
      Self::new( self.x, self.y + 1 ), // Up
      Self::new( self.x, self.y - 1 ), // Down
      // Diagonal neighbors
      Self::new( self.x + 1, self.y + 1 ), // Up-Right
      Self::new( self.x - 1, self.y + 1 ), // Up-Left
      Self::new( self.x + 1, self.y - 1 ), // Down-Right
      Self::new( self.x - 1, self.y - 1 ), // Down-Left
    ]
  }
}

/// Convenience type alias for 4-connected square coordinates
///
/// Equivalent to `Coordinate<FourConnected>` with Manhattan distance calculation
/// and orthogonal-only neighbor finding.
pub type SquareCoord4 = Coordinate< FourConnected >;

/// Convenience type alias for 8-connected square coordinates
///
/// Equivalent to `Coordinate<EightConnected>` with Chebyshev distance calculation
/// and 8-directional (including diagonal) neighbor finding.
pub type SquareCoord8 = Coordinate< EightConnected >;
