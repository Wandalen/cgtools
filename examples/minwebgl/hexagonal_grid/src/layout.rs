use crate::*;
use coordinates::*;

/// An enum that represents the orientation of the hexagons (e.g., "pointy-topped" or "flat-topped").
#[ derive( Debug, Copy, Clone ) ]
pub enum Orientation
{
  Pointy,
  Flat,
}

impl Orientation
{
  /// Orientation angle of the hexagons in radians.
  pub fn orientation_angle( &self ) -> f32
  {
    match self
    {
      Self::Pointy => 30.0f32.to_radians(),
      Self::Flat => 0.0f32.to_radians(),
    }
  }
}

/// A struct that defines geometric properties of the hexagonal grid layout.
#[ derive( Debug, Copy, Clone ) ]
pub struct HexLayout
{
  /// The orientation of the hexagons in the grid.
  pub orientation : Orientation,
  /// Size of a hexagon, the distance from the center to a corner.
  pub size : f32,
}

impl HexLayout
{
  /// Calculates coordinates of a hexagon that contains the given pixel position.
  ///
  /// # Parameters
  /// - `pixel`: The pixel coordinates.
  ///
  /// # Returns
  /// A coordinate representing the hexagon.
  pub fn hex_coord< C, Orientation, Parity >( &self, pixel : Pixel ) -> C
  where
    C : From< Coordinate< Axial, Orientation, Parity > >
  {
    match self.orientation
    {
      self::Orientation::Pointy => Coordinate::< Axial, Orientation, Parity >::from_pixel_to_pointy( pixel, self.size ).into(),
      self::Orientation::Flat => Coordinate::< Axial, Orientation, Parity >::from_pixel_to_flat( pixel, self.size ).into(),
    }
  }

  /// Calculates the 2d pixel position of a hexagon center based on its coordinates.
  ///
  /// # Parameters
  /// - `coord`: The coordinates of the hexagon.
  ///
  /// # Returns
  /// A `Pixel` containing the x and y coordinates of the hexagon center.
  pub fn pixel_coord< C, Orientation, Parity >( &self, coord : C ) -> Pixel
  where
    C : Into< Coordinate< Axial, Orientation, Parity > >
  {
    match self.orientation
    {
      self::Orientation::Pointy => coord.into().pointy_to_pixel( self.size ),
      self::Orientation::Flat => coord.into().flat_to_pixel( self.size ),
    }
  }

  /// Calculates the horizontal distance between neighbor hexagons in the grid.
  pub fn horizontal_spacing( &self ) -> f32
  {
    match self.orientation
    {
      Orientation::Pointy => pointy_layout_spacings( self.size ).0,
      Orientation::Flat => flat_layout_spacings( self.size ).0,
    }
  }

  /// Calculates the vertical distance between neighbor hexagons in the grid.
  pub fn vertical_spacing( &self ) -> f32
  {
    match self.orientation
    {
      Orientation::Pointy => pointy_layout_spacings( self.size ).1,
      Orientation::Flat => flat_layout_spacings( self.size ).1,
    }
  }

  /// Calculates a point that lies right in the center of a grid of hexagons.
  ///
  /// # Parameters
  /// - `coords`: An iterator over the coordinates of the hexagons.
  /// - `layout`: The layout of the hexagons.
  ///
  /// # Returns
  /// A tuple containing the x and y coordinates of the center of the grid.
  pub fn grid_center< I, C, Orientation, Parity >( &self, coords : I ) -> ( f32, f32 )
  where
    I : Iterator< Item = C >,
    C : Into< Coordinate< Axial, Orientation, Parity > >
  {
    // TODO: split this function into bounds_calculation and center_calculation based on bounds
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for coord in coords
    {
      let Pixel { x, y } = self.pixel_coord( coord );
      min_x = min_x.min( x );
      max_x = max_x.max( x );
      min_y = min_y.min( y );
      max_y = max_y.max( y );
    }

    ( min_x + ( max_x - min_x ) / 2.0, min_y + ( max_y - min_y ) / 2.0 )
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
