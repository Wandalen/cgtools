/// Represents an axial coordinate in a hexagonal grid.
/// Axial coordinates use two axes (`q` and `r`) to uniquely identify
/// hexes in a grid.
/// more info: https://www.redblobgames.com/grids/hexagons/#coordinates-axial
///
/// # Fields
/// - `q`: The "column" coordinate in the axial system.
/// - `r`: The "row" coordinate in the axial system.
#[ derive( Debug, Clone, Copy, Hash, PartialEq, Eq ) ]
pub struct Axial
{
  /// The "column" coordinate in the axial coordinate system.
  pub q : i32,
  /// The "row" coordinate in the axial coordinate system.
  pub r : i32,
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

  /// Converts pixel coordinates to axial coordinates in a pointy-topped hexagonal grid.
  ///
  /// # Parameters
  /// - `x`: The x-coordinate in pixels.
  /// - `y`: The y-coordinate in pixels.
  /// - `hex_size`: The size of the hexagons in the grid.
  ///
  /// # Returns
  /// An `Axial` coordinate representing the hexagon at the given pixel position.
  // qqq : use new type for each coordinate system
  pub fn from_2d_to_pointy( Pixel { x, y } : Pixel, hex_size : f32 ) -> Self
  {
    // implementation is taken from https://www.redblobgames.com/grids/hexagons/#pixel-to-hex
    let q = ( 3.0f32.sqrt() / 3.0 * x - 1.0 / 3.0 * y ) / hex_size;
    let r = (                           2.0 / 3.0 * y ) / hex_size;
    let ( q, r ) = axial_round( q, r );
    Axial { q, r }
  }

  /// Converts pixel coordinates to axial coordinates in a flat-topped hexagonal grid.
  ///
  /// # Parameters
  /// - `x`: The x-coordinate in pixels.
  /// - `y`: The y-coordinate in pixels.
  /// - `hex_size`: The size of the hexagons in the grid (outer circle radius).
  ///
  /// # Returns
  /// An `Axial` coordinate representing the hexagon at the given pixel position.
  pub fn from_2d_to_flat( Pixel { x, y } : Pixel, hex_size : f32 ) -> Self
  {
    // implementation is taken from https://www.redblobgames.com/grids/hexagons/#pixel-to-hex
    let q = ( 2.0 / 3.0 * x                            ) / hex_size;
    let r = ( -1.0 / 3.0 * x + 3.0f32.sqrt() / 3.0 * y ) / hex_size;
    let ( q, r ) = axial_round( q, r );
    Axial { q, r }
  }

  /// Converts axial coordinates to pixel coordinates in a pointy-topped hexagonal grid.
  ///
  /// # Parameters
  /// - `hex_size`: The size of the hexagons in the grid.
  ///
  /// # Returns
  /// A tuple containing the x and y pixel coordinates of the hexagon.
  pub fn pointy_to_2d( &self, hex_size : f32 ) -> Pixel
  {
    // implementation is taken from https://www.redblobgames.com/grids/hexagons/#hex-to-pixel
    let q = self.q as f32;
    let r = self.r as f32;
    let x = hex_size * ( 3.0f32.sqrt() * q + 3.0f32.sqrt() / 2.0 * r );
    let y = hex_size * (                               3.0 / 2.0 * r );
    ( x, y ).into()
  }

  /// Converts axial coordinates to pixel coordinates in a flat-topped hexagonal grid.
  ///
  /// # Parameters
  /// - `hex_size`: The size of the hexagons in the grid.
  ///
  /// # Returns
  /// A tuple containing the x and y pixel coordinates of the hexagon.
  pub fn flat_to_2d( &self, hex_size : f32 ) -> Pixel
  {
    // implementation is taken from https://www.redblobgames.com/grids/hexagons/#hex-to-pixel
    let q = self.q as f32;
    let r = self.r as f32;
    let x = hex_size * (           3.0 / 2.0 * q                     );
    let y = hex_size * ( 3.0f32.sqrt() / 2.0 * q + 3.0f32.sqrt() * r );
    ( x, y ).into()
  }
}

/// Rounds the given floating-point axial coordinates to the nearest integer axial coordinates.
/// This function is used to convert floating-point axial coordinates to integer axial coordinates.
///
/// # Parameters
/// - `q`: The floating-point q-coordinate.
/// - `r`: The floating-point r-coordinate.
///
/// # Returns
/// A tuple containing the rounded integer q and r coordinates.
fn axial_round( q: f32, r: f32 ) -> ( i32, i32 )
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

impl std::ops::Add for Axial
{
  type Output = Self;

  fn add( self, rhs: Self ) -> Self::Output
  {
    Self::new( self.q + rhs.q, self.r + rhs.r )
  }
}

impl std::ops::Sub for Axial
{
  type Output = Self;

  fn sub( self, rhs: Self ) -> Self::Output
  {
    Self::new( self.q - rhs.q, self.r - rhs.r )
  }
}

impl< T : Into< i32 > > From< ( T, T ) > for Axial
{
  fn from( ( q, r ) : ( T, T ) ) -> Self
  {
    Self { q : q.into(), r : r.into() }
  }
}

impl< T : Into< i32 > > From< [ T; 2 ] > for Axial
{
  fn from( [ q, r ] : [ T; 2 ] ) -> Self
  {
    Self { q : q.into(), r : r.into() }
  }
}

/// Represents a pixel coordinate in a 2D space.
/// Assumes that Y-axis points down.
#[ derive( Debug, Clone, Copy, PartialEq ) ]
pub struct Pixel
{
  pub x : f32,
  pub y : f32,
}

impl Pixel
{
  /// Creates a new `Pixel` coordinate with the specified `x` and `y` values.
  pub fn new( x : f32, y : f32 ) -> Self
  {
    Self { x, y }
  }
}

impl< T : Into< f32 > > From< ( T, T ) > for Pixel
{
  fn from( ( x, y ) : ( T, T ) ) -> Self
  {
    Self { x : x.into(), y : y.into() }
  }
}

impl< T : Into< f32 > > From< [ T; 2 ] > for Pixel
{
  fn from( [ x, y ] : [ T; 2 ] ) -> Self
  {
    Self { x : x.into(), y : y.into() }
  }
}
