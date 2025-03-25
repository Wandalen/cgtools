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
