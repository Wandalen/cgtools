use crate::*;
use coordinates::Axial;

/// A trait that defines geometric properties of the hexagonal grid layout.
pub trait HexLayout
{
  /// Converts 2d coordinates to closest hex center axial coordinates in a hexagonal grid.
  ///
  /// # Parameters
  /// - `x`: The x-coordinate in cartesian space.
  /// - `y`: The y-coordinate in cartesian space.
  /// - `hex_size`: The size of the hexagons in the grid.
  ///
  /// # Returns
  /// An `Axial` coordinate representing the hexagon at the given pixel position.
  fn hex_coordinates( &self, x : f32, y : f32 ) -> Axial;

  /// Calculates the 2d position of a hexagon center based on its axial coordinates.
  ///
  /// # Parameters
  /// - `coord`: The axial coordinates of the hexagon.
  /// - `hex_size`: The size of the hexagons in the grid.
  ///
  /// # Returns
  /// A tuple containing the x and y coordinates of the hexagon center.
  fn hex_2d_position( &self, coord : Axial ) -> ( f32, f32 );

  /// Determines the orientation of the hexagons (e.g., "pointy-topped" or "flat-topped").
  ///
  /// # Returns
  /// The rotation angle in radians.
  fn orientation_angle( &self ) -> f32;

  /// Calculates the horizontal distance between neighbor hexagons in the grid.
  ///
  /// # Parameters
  /// - `size`: The size of the hexagon.
  ///
  /// # Returns
  /// The horizontal spacing between hexagons.
  fn horizontal_spacing( &self ) -> f32;

  /// Calculates the vertical distance between neighbor hexagons in the grid.
  ///
  /// # Parameters
  /// - `size`: The size of the hexagon.
  ///
  /// # Returns
  /// The vertical spacing between hexagons.
  fn vertical_spacing( &self ) -> f32;

  /// Returns the size of the hexagons in the grid.
  /// Size is considered as the distance from the center to a vertex.
  fn size( &self ) -> f32;
}

/// A layout where the hexagons have pointy tops.
#[ derive( Copy, Clone ) ]
pub struct Pointy( pub f32 );

impl HexLayout for Pointy
{
  fn hex_coordinates( &self, x : f32, y : f32 ) -> Axial
  {
    Axial::from_2d_to_pointy( x, y, self.0 )
  }

  fn hex_2d_position( &self, coord : Axial ) -> ( f32, f32 )
  {
    let ( x, y ) = coord.pointy_to_2d( self.0 );
    ( x, y )
  }

  fn orientation_angle( &self ) -> f32
  {
    30.0f32.to_radians()
  }

  fn horizontal_spacing( &self ) -> f32
  {
    pointy_layout_spacings( self.0 ).0
  }

  fn vertical_spacing( &self ) -> f32
  {
    pointy_layout_spacings( self.0 ).1
  }

  fn size( &self ) -> f32
  {
    self.0
  }
}

/// A layout where the hexagons have flat tops.
#[ derive( Copy, Clone ) ]
pub struct Flat( pub f32 );

impl HexLayout for Flat
{
  fn hex_coordinates( &self, x : f32, y : f32 ) -> Axial
  {
    Axial::from_2d_to_flat( x, y, self.0 )
  }

  fn hex_2d_position( &self, coord : Axial ) -> ( f32, f32 )
  {
    let ( x, y ) = coord.flat_to_2d( self.0 );
    ( x, y )
  }

  fn orientation_angle( &self ) -> f32
  {
    0.0f32.to_radians()
  }

  fn horizontal_spacing( &self ) -> f32
  {
    flat_layout_spacings( self.0 ).0
  }

  fn vertical_spacing( &self ) -> f32
  {
    flat_layout_spacings( self.0 ).1
  }

  fn size( &self ) -> f32
  {
    self.0
  }
}

/// Calculates the horizontal and vertical spacings between neighbor hexagons in a pointy layout.
fn pointy_layout_spacings( size : f32 ) -> ( f32, f32 )
{
  ( 3.0f32.sqrt() * size , 1.5 * size )
}

/// Calculates the horizontal and vertical spacings between neighbor hexagons in a flat layout.
fn flat_layout_spacings( size : f32 ) -> ( f32, f32 )
{
  ( 1.5 * size, 3.0f32.sqrt() * size )
}

/// An enum that represents the type of shift in a shifted rectangle.
/// The shift can be either odd or even and determines which column or row will be shifted.
#[ derive( Debug, Copy, Clone, PartialEq, Eq ) ]
pub enum ShiftType
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
  shift_type : ShiftType,
}

impl ShiftedRectangleIterData
{
  fn new( rows : i32, columns : i32, shift_type : ShiftType ) -> Self
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

/// An iterator that generates axial coordinates for a shifted rectangle.
#[ derive( Debug ) ]
pub struct ShiftedRectangleIter< Layout >
{
  layout : Layout,
  data : ShiftedRectangleIterData,
}

impl< Layout > ShiftedRectangleIter< Layout >
{
  /// Creates a new `ShiftedRectangleIter`.
  ///
  /// # Parameters
  /// - `rows`: The number of rows in the rectangle.
  /// - `columns`: The number of columns in the rectangle.
  /// - `shift_type`: The type of shift in the rectangle.
  /// - `layout`: The layout of the hexagons.
  ///
  /// # Returns
  /// A new `ShiftedRectangleIter`.
  pub fn new( rows : i32, columns : i32, shift_type : ShiftType, layout : Layout ) -> Self
  {
    Self
    {
      layout,
      data : ShiftedRectangleIterData::new( rows, columns, shift_type ),
    }
  }
}

impl< Layout : ShiftedRectangle > Iterator for ShiftedRectangleIter< Layout >
{
  type Item = Axial;

  fn next( &mut self ) -> Option< Self::Item >
  {
    self.layout.next( &mut self.data )
  }
}

trait ShiftedRectangle
{
  /// Calculates the next axial coordinate in a shifted rectangle.
  fn next( &self, shifted : &mut ShiftedRectangleIterData ) -> Option< Axial >;
}

impl ShiftedRectangle for Pointy
{
  fn next( &self, shifted : &mut ShiftedRectangleIterData ) -> Option< Axial >
  {
    if shifted.current_row >= shifted.rows
    {
      return None;
    }

    let coord = Axial::new( shifted.current_column - shifted.offset, shifted.current_row );

    shifted.current_column += 1;

    if shifted.current_column == shifted.columns
    {
      shifted.current_column = 0;
      shifted.current_row += 1;

      if shifted.current_row & 1 == shifted.shift_type as i32
      {
        shifted.offset += 1;
      }
    }

    Some( coord )
  }
}

impl ShiftedRectangle for Flat
{
  fn next( &self, shifted : &mut ShiftedRectangleIterData ) -> Option< Axial >
  {
     if shifted.current_column >= shifted.columns
    {
      return None;
    }

    let coord = Axial::new( shifted.current_column, shifted.current_row - shifted.offset );

    shifted.current_row += 1;

    if shifted.current_row == shifted.rows
    {
      shifted.current_row = 0;
      shifted.current_column += 1;

      if shifted.current_column & 1 == shifted.shift_type as i32
      {
        shifted.offset += 1;
      }
    }

    Some( coord )
  }
}

/// Calculates a point that lies right in the center of a grid of hexagons.
///
/// # Parameters
/// - `coords`: An iterator over the axial coordinates of the hexagons.
/// - `layout`: The layout of the hexagons.
/// - `hex_size`: The size of the hexagons in the grid.
///
/// # Returns
/// A tuple containing the x and y coordinates of the center of the grid.
pub fn grid_center< C, L >( coords : C, layout : &L ) -> ( f32, f32 )
where
  C : Iterator< Item = Axial >,
  L : HexLayout
{
  let mut min_x = f32::INFINITY;
  let mut max_x = f32::NEG_INFINITY;
  let mut min_y = f32::INFINITY;
  let mut max_y = f32::NEG_INFINITY;

  for coord in coords
  {
    let ( x, y ) = layout.hex_2d_position( coord );
    min_x = min_x.min( x );
    max_x = max_x.max( x );
    min_y = min_y.min( y );
    max_y = max_y.max( y );
  }

  ( min_x + ( max_x - min_x ) / 2.0, min_y + ( max_y - min_y ) / 2.0 )
}
