use crate::coordinates::Axial;

/// A trait that defines the hexagonal grid layout.
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
  fn hex_coordinates( &self, x : f32, y : f32, hex_size : f32 ) -> Axial;

  /// Calculates the 2d position of a hexagon center based on its axial coordinates.
  ///
  /// # Parameters
  /// - `coord`: The axial coordinates of the hexagon.
  /// - `hex_size`: The size of the hexagons in the grid.
  ///
  /// # Returns
  /// A tuple containing the x and y coordinates of the hexagon center.
  fn hex_2d_position( &self, coord : Axial, hex_size : f32 ) -> ( f32, f32 );

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
  fn horizontal_spacing( &self, size : f32 ) -> f32;

  /// Calculates the vertical distance between neighbor hexagons in the grid.
  ///
  /// # Parameters
  /// - `size`: The size of the hexagon.
  ///
  /// # Returns
  /// The vertical spacing between hexagons.
  fn vertical_spacing( &self, size : f32 ) -> f32;
}

/// A layout where the hexagons have pointy tops.
#[ derive( Copy, Clone ) ]
pub struct Pointy;

impl HexLayout for Pointy
{
  fn hex_coordinates( &self, x : f32, y : f32, hex_size : f32 ) -> Axial
  {
    Axial::from_2d_to_pointy( x, y, hex_size )
  }

  fn hex_2d_position( &self, coord : Axial, hex_size : f32 ) -> ( f32, f32 )
  {
    let ( x, y ) = coord.pointy_to_2d( hex_size );
    ( x, y )
  }

  fn orientation_angle( &self ) -> f32
  {
    30.0f32.to_radians()
  }

  fn horizontal_spacing( &self, size : f32 ) -> f32
  {
    pointy_layout_spacings( size ).0
  }

  fn vertical_spacing( &self, size : f32 ) -> f32
  {
    pointy_layout_spacings( size ).1
  }
}

/// A layout where the hexagons have flat tops.
#[ derive( Copy, Clone ) ]
pub struct Flat;

impl HexLayout for Flat
{
  fn hex_coordinates( &self, x : f32, y : f32, hex_size : f32 ) -> Axial
  {
    Axial::from_2d_to_flat( x, y, hex_size )
  }

  fn hex_2d_position( &self, coord : Axial, hex_size : f32 ) -> ( f32, f32 )
  {
    let ( x, y ) = coord.flat_to_2d( hex_size );
    ( x, y )
  }

  fn orientation_angle( &self ) -> f32
  {
    0.0f32.to_radians()
  }

  fn horizontal_spacing( &self, size : f32 ) -> f32
  {
    flat_layout_spacings( size ).0
  }

  fn vertical_spacing( &self, size : f32 ) -> f32
  {
    flat_layout_spacings( size ).1
  }
}

fn pointy_layout_spacings( size : f32 ) -> ( f32, f32 )
{
  ( 3.0f32.sqrt() * size , 1.5 * size )
}

fn flat_layout_spacings( size : f32 ) -> ( f32, f32 )
{
  ( 1.5 * size, 3.0f32.sqrt() * size )
}

#[ derive( Debug, Copy, Clone, PartialEq, Eq ) ]
pub enum ShiftType
{
  Odd = 0,
  Even = 1,
}

#[ derive( Debug ) ]
pub struct Shifted
{
  rows : i32,
  columns : i32,
  current_row : i32,
  current_column : i32,
  offset : i32,
  shift_type : ShiftType,
}

impl Shifted
{
  pub fn new( rows : i32, columns : i32, shift_type : ShiftType ) -> Self
  {
    Self
    {
      rows,
      columns,
      current_row : 0,
      current_column : 0,
      offset : 0,
      shift_type
    }
  }
}

impl Iterator for Shifted
{
  type Item = Axial;

  fn next( &mut self ) -> Option< Self::Item >
  {
    if self.current_row >= self.rows
    {
      return None;
    }

    let coord = Axial::new( self.current_column - self.offset, self.current_row );

    self.current_column += 1;

    if self.current_column == self.columns
    {
      self.current_column = 0;
      self.current_row += 1;

      if self.current_row & 1 == self.shift_type as i32
      {
        self.offset += 1;
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
pub fn grid_center< C, L >( coords : C, layout : &L, hex_size : f32 ) -> ( f32, f32 )
where
  C : Iterator< Item = Axial >,
  L : HexLayout
{
  let mut min_x = f32::INFINITY;
  let mut max_x = f32::NEG_INFINITY;
  let mut min_y = f32::INFINITY;
  let mut max_y = f32::NEG_INFINITY;

  for coor in coords
  {
    let ( x, y ) = layout.hex_2d_position( coor, hex_size );
    min_x = min_x.min( x );
    max_x = max_x.max( x );
    min_y = min_y.min( y );
    max_y = max_y.max( y );
  }

  ( min_x + ( max_x - min_x ) / 2.0, min_y + ( max_y - min_y ) / 2.0 )
}
