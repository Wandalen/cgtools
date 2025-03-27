use crate::*;
use coordinates::Axial;

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
  /// Calculates axial coordinates of a closest hexagon in a grid
  ///
  /// # Parameters
  /// - `x`: The x-coordinate in cartesian space.
  /// - `y`: The y-coordinate in cartesian space.
  ///
  /// # Returns
  /// An `Axial` coordinate representing the hexagon at the given pixel position.
  pub fn hex_coordinates( &self, x : f32, y : f32 ) -> Axial
  {
    match self.orientation
    {
      Orientation::Pointy => Axial::from_2d_to_pointy( x, y, self.size ),
      Orientation::Flat => Axial::from_2d_to_flat( x, y, self.size ),
    }
  }

  /// Calculates the 2d position of a hexagon center based on its axial coordinates.
  ///
  /// # Parameters
  /// - `coord`: The axial coordinates of the hexagon.
  ///
  /// # Returns
  /// A tuple containing the x and y coordinates of the hexagon center.
  pub fn hex_2d_position( &self, coord : Axial ) -> ( f32, f32 )
  {
    match self.orientation
    {
      Orientation::Pointy => coord.pointy_to_2d( self.size ),
      Orientation::Flat => coord.flat_to_2d( self.size ),
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

/// Calculates a point that lies right in the center of a grid of hexagons.
///
/// # Parameters
/// - `coords`: An iterator over the axial coordinates of the hexagons.
/// - `layout`: The layout of the hexagons.
///
/// # Returns
/// A tuple containing the x and y coordinates of the center of the grid.
pub fn grid_center< C >( coords : C, layout : &HexLayout ) -> ( f32, f32 )
where
  C : Iterator< Item = Axial >
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
