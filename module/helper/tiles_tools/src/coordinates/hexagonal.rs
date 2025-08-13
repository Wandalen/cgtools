//! This module provides a generic `Coordinate` struct for representing positions in a hexagonal grid.
//! It supports different coordinate systems (Axial and Offset) and orientations (Pointy-topped and Flat-topped)
//! through a flexible, type-safe generic implementation. It also includes conversions between systems
//! and from pixel coordinates, as well as utility functions for grid operations like distance and neighbor finding.

use crate::coordinates::{ Distance, Neighbors };
use crate::coordinates::pixel::Pixel;
use ndarray_cg::I32x2;
use serde::{ Deserialize, Serialize };
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

/// A marker struct representing the Axial coordinate system for hexagonal grids.
#[ derive( Debug ) ]
pub struct Axial;

/// A marker struct representing the Offset coordinate system, parameterized by `Parity` (Odd or Even).
#[ derive( Debug ) ]
pub struct Offset< Parity >( PhantomData< Parity > );

/// A marker struct for pointy-topped hexagon orientation.
#[ derive( Debug ) ]
pub struct Pointy;

/// A marker struct for flat-topped hexagon orientation.
#[ derive( Debug ) ]
pub struct Flat;

/// A marker struct for "odd" parity in an Offset coordinate system.
#[ derive( Debug ) ]
pub struct Odd;

/// A marker struct for "even" parity in an Offset coordinate system.
#[ derive( Debug ) ]
pub struct Even;

/// Represents a coordinate in a hexagonal grid with a specific system and orientation.
#[ derive( Serialize, Deserialize ) ]
pub struct Coordinate< System, Orientation >
{
    /// The column index of the coordinate (often 'q' or 'col').
  pub q : i32,
    /// The row index of the coordinate (often 'r' or 'row').
  pub r : i32,
    /// A marker to hold the generic types for the coordinate system and orientation.
  #[ serde( skip ) ]
  pub _marker : PhantomData< ( System, Orientation ) >,
}

impl< System, Orientation > Debug for Coordinate< System, Orientation >
{
    /// Formats the coordinate for debugging, including its system type.
  fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
  {
    f.debug_struct( "Coordinate" )
      .field( "q", &self.q )
      .field( "r", &self.r )
      .field( "system", &self._marker )
      .finish()
    }
}

impl< System, Orientation > Clone for Coordinate< System, Orientation >
{
    /// Clones the coordinate.
  fn clone( &self ) -> Self
  {
    Self::new_uncheked( self.q, self.r )
    }
}

impl< System, Orientation > Copy for Coordinate< System, Orientation > {}

impl< System, Orientation > Eq for Coordinate< System, Orientation > {}

impl< System, Orientation > PartialEq for Coordinate< System, Orientation >
{
    /// Checks for equality between two coordinates based on their `q` and `r` values.
  fn eq( &self, other : &Self ) -> bool
  {
    self.q == other.q && self.r == other.r
    }
}

impl< System, Orientation > Hash for Coordinate< System, Orientation >
{
    /// Hashes the coordinate based on its `q`, `r`, and type markers.
  fn hash< H : std::hash::Hasher >( &self, state : &mut H )
  {
    self.q.hash( state );
    self.r.hash( state );
    self._marker.hash( state );
    }
}

impl< System, Orientation > Default for Coordinate< System, Orientation >
{
    /// Returns a default coordinate at (0, 0).
  fn default() -> Self
  {
    Self
    {
      q : Default::default(),
      r : Default::default(),
      _marker : Default::default(),
        }
    }
}

impl< System, Orientation > Into< I32x2 > for Coordinate< System, Orientation >
{
    /// Converts the coordinate into an `I32x2` vector.
  fn into( self ) -> I32x2
  {
    I32x2::from_array( [ self.q, self.r ] )
    }
}

impl< System, Orientation > Into< ( i32, i32 ) > for Coordinate< System, Orientation >
{
    /// Converts the coordinate into a tuple `(q, r)`.
  fn into( self ) -> ( i32, i32 )
  {
    ( self.q, self.r )
    }
}

impl< System, Orientation > Coordinate< System, Orientation >
{
    /// Creates a new coordinate without any checks. For internal use.
  pub( crate ) const fn new_uncheked( q : i32, r : i32 ) -> Self
  {
    Self
    {
      q,
      r,
      _marker : PhantomData,
    }
  }

  /// Creates a new `Offset` coordinate.
  pub const fn new( q : i32, r : i32 ) -> Self
  {
    Self::new_uncheked( q, r )
  }
}

// impl< Orientation, Parity > Coordinate< Offset< Parity >, Orientation >
// {
// }

impl< Orientation > Coordinate< Axial, Orientation >
{
  /// Calculates the grid distance between two `Axial` coordinates.
  pub fn distance( &self, Self { q, r, .. } : Self ) -> i32
  {
    let s = -self.q - self.r;
    let other_s = -q - r;
    let q = self.q - q;
    let r = self.r - r;
    let s = s - other_s;
    ( q.abs() + r.abs() + s.abs() ) / 2
  }
}

impl From< Coordinate< Axial, Pointy > > for Coordinate< Offset< Odd >, Pointy >
{
  /// Converts from Axial (Pointy) to Offset (Odd, Pointy) coordinates.
  fn from( value : Coordinate< Axial, Pointy > ) -> Self
  {
    let col = value.q + ( value.r - ( value.r & 1 ) ) / 2;
    let row = value.r;
    Self::new( col, row )
  }
}

impl From< Coordinate< Axial, Pointy > > for Coordinate< Offset< Even >, Pointy >
{
  /// Converts from Axial (Pointy) to Offset (Even, Pointy) coordinates.
  fn from( value : Coordinate< Axial, Pointy > ) -> Self
  {
    let col = value.q + ( value.r + ( value.r & 1 ) ) / 2;
    let row = value.r;
    Self::new( col, row )
  }
}

impl From< Coordinate< Axial, Flat > > for Coordinate< Offset< Odd >, Flat >
{
  /// Converts from Axial (Flat) to Offset (Odd, Flat) coordinates.
  fn from( value : Coordinate< Axial, Flat > ) -> Self
  {
    let col = value.q;
    let row = value.r + ( value.q - ( value.q & 1 ) ) / 2;
    Self::new( col, row )
  }
}

impl From< Coordinate< Axial, Flat > > for Coordinate< Offset< Even >, Flat >
{
  /// Converts from Axial (Flat) to Offset (Even, Flat) coordinates.
  fn from( value : Coordinate< Axial, Flat > ) -> Self
  {
    let col = value.q;
    let row = value.r + ( value.q + ( value.q & 1 ) ) / 2;
    Self::new( col, row )
  }
}

impl From< Coordinate< Offset< Odd >, Pointy > > for Coordinate< Axial, Pointy >
{
  /// Converts from Offset (Odd, Pointy) to Axial (Pointy) coordinates.
  fn from( value : Coordinate< Offset< Odd >, Pointy > ) -> Self
  {
    let q = value.q - ( value.r - ( value.r & 1 ) ) / 2;
    let r = value.r;
    Self::new( q, r )
  }
}

impl From< Coordinate< Offset< Even >, Pointy > > for Coordinate< Axial, Pointy >
{
  /// Converts from Offset (Even, Pointy) to Axial (Pointy) coordinates.
  fn from( value : Coordinate< Offset< Even >, Pointy > ) -> Self
  {
    let q = value.q - ( value.r + ( value.r & 1 ) ) / 2;
    let r = value.r;
    Self::new( q, r )
  }
}

impl From< Coordinate< Offset< Odd >, Flat > > for Coordinate< Axial, Flat >
{
  /// Converts from Offset (Odd, Flat) to Axial (Flat) coordinates.
  fn from( value : Coordinate< Offset< Odd >, Flat > ) -> Self
  {
    let q = value.q;
    let r = value.r - ( value.q - ( value.q & 1 ) ) / 2;
    Self::new( q, r )
  }
}

impl From< Coordinate< Offset< Even >, Flat > > for Coordinate< Axial, Flat >
{
  /// Converts from Offset (Even, Flat) to Axial (Flat) coordinates.
  fn from( value : Coordinate< Offset< Even >, Flat > ) -> Self
  {
    let q = value.q;
    let r = value.r - ( value.q + ( value.q & 1 ) ) / 2;
    Self::new( q, r )
  }
}

impl< System, Orientation > From< ( i32, i32 ) > for Coordinate< System, Orientation >
{
  /// Creates a coordinate from a tuple `(q, r)`.
  fn from( ( q, r ) : ( i32, i32 ) ) -> Self
  {
    Self::new_uncheked( q, r )
  }
}

impl< System, Orientation > From< [ i32; 2 ] > for Coordinate< System, Orientation >
{
  /// Creates a coordinate from an array `[q, r]`.
  fn from( [ q, r ] : [ i32; 2 ] ) -> Self
  {
    Self::new_uncheked( q, r )
  }
}

impl< System, Orientation > From< I32x2 > for Coordinate< System, Orientation >
{
  /// Creates a coordinate from an `I32x2` vector.
  fn from( ndarray_cg::Vector( [ q, r ] ) : I32x2 ) -> Self
  {
    Self::new_uncheked( q, r )
  }
}

impl< Orientation > std::ops::Add for Coordinate< Axial, Orientation >
{
  type Output = Self;

  /// Adds two `Axial` coordinates (vector addition).
  fn add( self, rhs : Self ) -> Self::Output
  {
    Self::new( self.q + rhs.q, self.r + rhs.r )
  }
}

impl< Orientation > std::ops::Sub for Coordinate< Axial, Orientation >
{
  type Output = Self;

  /// Subtracts two `Axial` coordinates (vector subtraction).
  fn sub( self, rhs : Self ) -> Self::Output
  {
    Self::new( self.q - rhs.q, self.r - rhs.r )
  }
}

impl< Orientation > std::ops::Mul< i32 > for Coordinate< Axial, Orientation >
{
  type Output = Self;

  /// Scales an `Axial` coordinate by an integer factor.
  fn mul( self, rhs : i32 ) -> Self::Output
  {
    Self::new( self.q * rhs, self.r * rhs )
  }
}

impl< Orientation > std::ops::Div< i32 > for Coordinate< Axial, Orientation >
{
  type Output = Self;

  /// Divides an `Axial` coordinate by an integer factor.
  fn div( self, rhs : i32 ) -> Self::Output
  {
    Self::new( self.q / rhs, self.r / rhs )
  }
}

impl From< Pixel > for Coordinate< Axial, Pointy >
{
  /// Converts pixel coordinates to the nearest `Axial` coordinate with pointy-topped orientation.
  fn from( Pixel { data : [ x, y ] } : Pixel ) -> Self
  {
    // implementation is taken from https://www.redblobgames.com/grids/hexagons/#pixel-to-hex
    let q = 3.0f32.sqrt() / 3.0 * x - 1.0 / 3.0 * y;
    let r = 2.0 / 3.0 * y;
    let ( q, r ) = axial_round( q, r );
    Self::new( q, r )
  }
}

impl From< Pixel > for Coordinate< Axial, Flat >
{
  /// Converts pixel coordinates to the nearest `Axial` coordinate with flat-topped orientation.
  fn from( Pixel { data : [ x, y ] } : Pixel ) -> Self
  {
    // implementation is taken from https://www.redblobgames.com/grids/hexagons/#pixel-to-hex
    let q = 2.0 / 3.0 * x;
    let r = -1.0 / 3.0 * x + 3.0f32.sqrt() / 3.0 * y;
    let ( q, r ) = axial_round( q, r );
    Self::new( q, r )
  }
}

/// Rounds floating-point axial coordinates to the nearest integer axial coordinates.
///
/// This function converts fractional `(q, r)` axial coordinates (which can arise from
/// pixel-to-hex conversions) into the nearest valid integer `(q, r)` hex coordinate by
/// using a third "s" coordinate (`s = -q - r`) and rounding each to the nearest integer,
/// then resolving any inconsistencies.
///
/// # Arguments
/// * `q` - The floating-point q-coordinate.
/// * `r` - The floating-point r-coordinate.
///
/// # Returns
/// A tuple `(i32, i32)` containing the rounded integer q and r coordinates.
fn axial_round( q : f32, r : f32 ) -> ( i32, i32 )
{
  // implementation is taken from https://www.redblobgames.com/grids/hexagons/#rounding
  let s = -q - r;

  let mut rq = q.round();
  let mut rr = r.round();
  let rs = s.round();

  let q_diff = ( rq - q ).abs();
  let r_diff = ( rr - r ).abs();
  let s_diff = ( rs - s ).abs();

  if q_diff > r_diff && q_diff > s_diff
  {
    rq = -rr - rs;
  }
  else if r_diff > s_diff
  {
    rr = -rq - rs;
  }

  ( rq as i32, rr as i32 )
}

impl< Orientation > Distance for Coordinate< Axial, Orientation >
{
  /// Calculates the grid distance between two `Axial` coordinates.
  fn distance( &self, Self { q, r, .. } : &Self ) -> u32
  {
    let s = -self.q as i64 - self.r as i64;
    let other_s = -q as i64 - *r as i64;
    let q = self.q as i64 - *q as i64;
    let r = self.r as i64 - *r as i64;
    let s = s - other_s;
    ( q.abs() as u32 + r.abs() as u32 + s.abs() as u32 ) / 2
  }
}

impl< Orientation > Neighbors for Coordinate< Axial, Orientation >
{
  /// Returns a `Vec` containing all 6 direct neighbors of an `Axial` coordinate.
  fn neighbors( &self ) -> Vec< Self >
  {
    [
      *self + ( 1, 0 ).into(),
      *self + ( 1, -1 ).into(),
      *self + ( 0, -1 ).into(),
      *self + ( -1, 0 ).into(),
      *self + ( -1, 1 ).into(),
      *self + ( 0, 1 ).into(),
    ]
    .into()
  }
}

impl Coordinate< Axial, Flat >
{
  /// Returns the coordinate directly above in a flat-topped layout.
  pub fn up( &self ) -> Self
  {
    Self::new( self.q, self.r - 1 )
  }

  /// Returns the coordinate directly below in a flat-topped layout.
  pub fn down( &self ) -> Self
  {
    Self::new( self.q, self.r + 1 )
  }

  /// Returns the coordinate to the upper-left in a flat-topped layout.
  pub fn left_up( &self ) -> Self
  {
    Self::new( self.q - 1, self.r )
  }

  /// Returns the coordinate to the lower-left in a flat-topped layout.
  pub fn left_down( &self ) -> Self
  {
    Self::new( self.q - 1, self.r + 1 )
  }

  /// Returns the coordinate to the upper-right in a flat-topped layout.
  pub fn right_up( &self ) -> Self
  {
    Self::new( self.q + 1, self.r - 1 )
  }

  /// Returns the coordinate to the lower-right in a flat-topped layout.
  pub fn right_down( &self ) -> Self
  {
    Self::new( self.q + 1, self.r )
  }
}
