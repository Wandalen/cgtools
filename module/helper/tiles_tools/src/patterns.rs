use crate::*;
use ndarray_cg::ArrayRef;
use layout::{ HexLayout, Orientation };
use coordinates::{ Coordinate, Axial };
// use crate::layout::*;
// use crate::coordinates::*;

/// An enum that represents the type of shift in a shifted rectangle.
/// The shift can be either odd or even and determines which column or row will be shifted.
#[ derive( Debug, Copy, Clone, PartialEq, Eq ) ]
pub enum Parity
{
  Odd = 0,
  Even = 1,
}

/// A struct that holds the data needed to iterate over a shifted rectangle.
#[ derive( Debug ) ]
struct ShiftedRectangleIterData
{
  rows : i32,
  columns : i32,
  current_row : i32,
  current_column : i32,
  offset : i32,
  shift_type : Parity,
}

impl ShiftedRectangleIterData
{
  fn new( rows : i32, columns : i32, shift_type : Parity ) -> Self
  {
    Self
    {
      rows,
      columns,
      current_row : 0,
      current_column : 0,
      offset : 0,
      shift_type,
    }
  }
}

/// An iterator that generates axial coordinates in a shifted rectangle pattern.
/// Shifted rectangle is a rectangle where every other row or column is shifted.
#[ derive( Debug ) ]
pub struct ShiftedRectangleIter
{
  layout : HexLayout,
  data : ShiftedRectangleIterData,
}

impl ShiftedRectangleIter
{
  /// Creates a new `ShiftedRectangleIter`.
  ///
  /// # Parameters
  /// - `size`: The number of rows in the rectangle and yhe number of columns in the rectangle.
  /// - `shift_type`: The type of shift in the rectangle.
  /// - `layout`: The layout of the hexagons.
  ///
  /// # Returns
  /// A new `ShiftedRectangleIter`.
  pub fn new< V2 : ArrayRef< i32, 2 > >( size : V2, shift_type : Parity, layout : HexLayout ) -> Self
  {
    Self
    {
      layout,
      data : ShiftedRectangleIterData::new( size.array_ref()[ 0 ], size.array_ref()[ 1 ], shift_type ),
    }
  }

  fn next_pointy( data : &mut ShiftedRectangleIterData ) -> Option< Coordinate< Axial, PointyTopped, OddParity > >
  {
    if data.current_row >= data.rows
    {
      return None;
    }

    let coord = Coordinate::new( data.current_column - data.offset, data.current_row );

    data.current_column += 1;

    if data.current_column == data.columns
    {
      data.current_column = 0;
      data.current_row += 1;

      if data.current_row & 1 == data.shift_type as i32
      {
        data.offset += 1;
      }
    }

    Some( coord )
  }

  fn next_flat( data : &mut ShiftedRectangleIterData ) -> Option< Coordinate< Axial, PointyTopped, OddParity > >
  {
    if data.current_column >= data.columns
    {
      return None;
    }

    let coord = Coordinate::new( data.current_column, data.current_row - data.offset );

    data.current_row += 1;

    if data.current_row == data.rows
    {
      data.current_row = 0;
      data.current_column += 1;

      if data.current_column & 1 == data.shift_type as i32
      {
        data.offset += 1;
      }
    }

    Some( coord )
  }
}

impl Iterator for ShiftedRectangleIter
{
  type Item = Coordinate< Axial, PointyTopped, OddParity >;

  fn next( &mut self ) -> Option< Self::Item >
  {
    match self.layout.orientation
    {
      Orientation::Pointy => Self::next_pointy( &mut self.data ),
      Orientation::Flat => Self::next_flat( &mut self.data ),
    }
  }
}