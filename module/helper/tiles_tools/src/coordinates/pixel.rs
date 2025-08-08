//! This module defines the `Pixel` struct, representing a 2D coordinate in a Cartesian space.
//! It provides conversions from and to hexagonal coordinate systems and other numeric types,
//! as well as basic vector arithmetic.

use super::hexagonal::*;

/// Represents a 2D pixel coordinate, typically used for rendering.
/// It is assumed that the Y-axis points downwards.
#[ derive( Debug, Clone, Copy, PartialEq ) ]
pub struct Pixel
{
  /// The raw [x, y] floating-point coordinates.
  pub data : [ f32; 2 ],
}

impl Pixel
{
  /// Creates a new `Pixel` from x and y components.
  pub fn new( x : f32, y : f32 ) -> Self
  {
    Self { data : [ x.into(), y.into() ] }
  }

  /// Returns the x component of the pixel coordinate.
  pub fn x( &self ) -> f32
  {
    self[ 0 ]
  }

  /// Returns the y component of the pixel coordinate.
  pub fn y( &self ) -> f32
  {
    self[ 1 ]
  }
}

impl< F > From< ( F, F ) > for Pixel
where
  F : Into< f32 >
{
  /// Creates a `Pixel` from a tuple of two convertible numeric types.
  fn from( ( x, y ) : ( F, F ) ) -> Self
  {
    Self { data : [ x.into(), y.into() ] }
  }
}

impl< F > From< [ F; 2 ] > for Pixel
where
  F : Into< f32 >
{
  /// Creates a `Pixel` from an array of two convertible numeric types.
  fn from( [ x, y ] : [ F; 2 ] ) -> Self
  {

    Self { data : [ x.into(), y.into() ] }
  }
}

impl From< Coordinate< Axial, Pointy > > for Pixel
{
  /// Converts a pointy-topped axial hex coordinate to its pixel-space center.
  fn from( value : Coordinate< Axial, Pointy > ) -> Self
  {
    let q = value.q as f32;
    let r = value.r as f32;
    let x = 3.0f32.sqrt() * q + 3.0f32.sqrt() / 2.0 * r;
    let y =                               3.0 / 2.0 * r;
    ( x, y ).into()
  }
}

impl From< Coordinate< Axial, Flat > > for Pixel
{
  /// Converts a flat-topped axial hex coordinate to its pixel-space center.
  fn from( value : Coordinate< Axial, Flat > ) -> Self
  {
    let q = value.q as f32;
    let r = value.r as f32;
    let x =           3.0 / 2.0 * q                    ;
    let y = 3.0f32.sqrt() / 2.0 * q + 3.0f32.sqrt() * r;
    ( x, y ).into()
  }
}

impl< E > From< ndarray_cg::Vector< E, 2 > > for Pixel
where
  E : ndarray_cg::MatEl + Into< f32 >
{
  /// Converts an `ndarray_cg` 2D vector into a `Pixel`.
  fn from( value : ndarray_cg::Vector< E, 2 >) -> Self
  {
    Self
    {
      data : [ value[ 0 ].into(), value[ 1 ].into() ]
    }
  }
}

impl Into< ndarray_cg::F32x2 > for Pixel
{
  /// Converts a `Pixel` into an `ndarray_cg::F32x2` vector.
  fn into( self ) -> ndarray_cg::F32x2
  {
    self.data.into()
  }
}

impl ndarray_cg::Collection for Pixel
{
  type Scalar = f32;
}

impl ndarray_cg::Add for Pixel
{
  type Output = Self;

  /// Performs vector addition on two `Pixel` coordinates.
  fn add( self, rhs : Self ) -> Self::Output
  {
    Self
    {
      data :
      [
        self.data[ 0 ] + rhs.data[ 0 ],
        self.data[ 1 ] + rhs.data[ 1 ]
      ]
    }
  }
}

impl ndarray_cg::Sub for Pixel
{
  type Output = Self;

  /// Performs vector subtraction on two `Pixel` coordinates.
  fn sub( self, rhs : Self ) -> Self::Output
  {
    Self
    {
      data :
      [
        self.data[ 0 ] - rhs.data[ 0 ],
        self.data[ 1 ] - rhs.data[ 1 ]
      ]
    }
  }
}

impl std::ops::Index< usize > for Pixel
{
  type Output = f32;

  /// Allows indexing into the `Pixel`'s data array (`[x, y]`).
  fn index( &self, index : usize ) -> &Self::Output
  {
    &self.data[ index ]
  }
}

impl std::ops::IndexMut< usize > for Pixel
{
  /// Allows mutable indexing into the `Pixel`'s data array (`[x, y]`).
  fn index_mut( &mut self, index : usize ) -> &mut Self::Output
  {
    &mut self.data[ index ]
  }
}
