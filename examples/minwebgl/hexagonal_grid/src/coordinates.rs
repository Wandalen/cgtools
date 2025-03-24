use std::marker::PhantomData;
use crate::layout::*;

/// Represents an offset coordinate in a hexagonal grid.
/// The `Offset` structure is parameterized by a layout type, which determines
/// the specific hexagonal grid layout (e.g., `HorizontalOddShifted`).
///
/// # Fields
/// - `row`: The row index of the hex.
/// - `column`: The column index of the hex.
/// - `layout`: A marker for the layout type.
#[ derive( Debug, Clone, Copy, Hash, PartialEq, Eq ) ]
pub struct Offset< Layout >
{
  /// The row index of the hexagon in the grid.
  pub row : i32,
  /// The column index of the hexagon in the grid.
  pub column : i32,
  /// A marker for the layout type of the hexagonal grid.
  pub layout : PhantomData< Layout >,
}

impl< Layout > Offset< Layout >
{
  /// Creates a new `Offset` coordinate with the specified row and column.
  ///
  /// # Parameters
  /// - `row`: The row index of the hexagon.
  /// - `column`: The column index of the hexagon.
  ///
  /// # Returns
  /// A new `Offset` instance.
  pub fn new( row : i32, column : i32 ) -> Self
  {
    Self
    {
      row,
      column,
      layout : PhantomData,
    }
  }
}

impl From< Axial > for Offset< PointyOddShifted >
{
  fn from( value : Axial ) -> Self
  {
    let col = value.q + ( value.r - value.r & 1 ) / 2;
    let row = value.r;
    Self::new( row, col )
  }
}

impl From< Axial > for Offset< PointyEvenShifted >
{
  fn from( value : Axial ) -> Self
  {
    let col = value.q + ( value.r + value.r & 1 ) / 2;
    let row = value.r;
    Self::new( row, col )
  }
}

impl From< Axial > for Offset< FlatOddShifted >
{
  fn from( value : Axial ) -> Self
  {
    let col = value.q;
    let row = value.r + ( value.q - value.q & 1 ) / 2;
    Self::new( row, col )
  }
}

impl From< Axial > for Offset< FlatEvenShifted >
{
  fn from( value : Axial ) -> Self
  {
    let col = value.q;
    let row = value.r + ( value.q + value.q & 1 ) / 2;
    Self::new( row, col )
  }
}

/// Represents an axial coordinate in a hexagonal grid.
/// Axial coordinates use two axes (`q` and `r`) to uniquely identify
/// hexes in a grid.
///
/// # Fields
/// - `q`: The "column" coordinate in the axial system.
/// - `r`: The "row" coordinate in the axial system.
#[ derive( Debug, Clone, Copy, Hash, PartialEq, Eq ) ]
pub struct Axial
{
  /// The "column" coordinate in the axial coordinate system.
  pub q: i32,
  /// The "row" coordinate in the axial coordinate system.
  pub r: i32,
}

impl Axial
{
  /// Creates a new `Axial` coordinate with the specified `q` and `r` values.
  ///
  /// # Parameters
  /// - `q`: The "column" coordinate in the axial system.
  /// - `r`: The "row" coordinate in the axial system.
  ///
  /// # Returns
  /// A new `Axial` instance.
  pub fn new( q : i32, r : i32 ) -> Self
  {
    Self { q, r }
  }
}

impl From< Offset< PointyOddShifted > > for Axial
{
  fn from( value : Offset< PointyOddShifted > ) -> Self
  {
    let q = value.column - ( value.row - value.row & 1 ) / 2;
    let r = value.row;
    Self { q, r }
  }
}

impl From< Offset< PointyEvenShifted > > for Axial
{
  fn from( value : Offset< PointyEvenShifted > ) -> Self
  {
    let q = value.column - ( value.row + value.row & 1 ) / 2;
    let r = value.row;
    Self { q, r }
  }
}

impl From< Offset< FlatOddShifted > > for Axial
{
  fn from( value : Offset< FlatOddShifted > ) -> Self
  {
    let q = value.column;
    let r = value.row - ( value.column - value.column & 1 ) / 2;
    Self { q, r }
  }
}

impl From< Offset< FlatEvenShifted > > for Axial
{
  fn from( value : Offset< FlatEvenShifted > ) -> Self
  {
    let q = value.column;
    let r = value.row - ( value.column + value.column & 1 ) / 2;
    Self { q, r }
  }
}
