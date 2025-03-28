use std::{ hash::Hash, marker::PhantomData };

/// Axial coordinates use two axes (`q` and `r`) to uniquely identify
/// hexes in a grid.
/// more info: https://www.redblobgames.com/grids/hexagons/#coordinates-axial
pub struct Axial;

pub struct Offset;

pub struct Doubled;

pub struct FlatTopped;

pub struct PointyTopped;

pub struct EvenParity;

pub struct OddParity;

/// Represents a coordinate in a hexagonal grid.
///
/// # Fields
/// - `q`: The "column" coordinate.
/// - `r`: The "row" coordinate.
#[ derive( Debug ) ]
pub struct Coordinate< System, Orientation, Parity >
{
  /// The "column" coordinate in the coordinate system.
  pub q : i32,
  /// The "row" coordinate in the coordinate system.
  pub r : i32,
  system : PhantomData< System >,
  orientation : PhantomData< Orientation >,
  parity : PhantomData< Parity >,
}

impl< System, Orientation, Parity > Hash for Coordinate< System, Orientation, Parity >
{
  fn hash< H : std::hash::Hasher >( &self, state : &mut H )
  {
    self.q.hash( state );
    self.r.hash( state );
    self.system.hash( state );
    self.orientation.hash( state );
    self.parity.hash( state );
  }
}

impl< System, Orientation, Parity > PartialEq for Coordinate< System, Orientation, Parity >
{
  fn eq( &self, other : &Self ) -> bool
  {
    self.q == other.q && self.r == other.r
  }
}

impl< System, Orientation, Parity > Eq for Coordinate< System, Orientation, Parity > {}

impl< System, Orientation, Parity > Clone for Coordinate< System, Orientation, Parity >
{
  fn clone( &self ) -> Self
  {
    Self { q : self.q, r : self.r, system : PhantomData, orientation : PhantomData, parity : PhantomData }
  }
}

impl< System, Orientation, Parity > Copy for Coordinate< System, Orientation, Parity > {}

impl< System, Orientation, Parity > Coordinate< System, Orientation, Parity >
{
  pub fn new( q : i32, r : i32 ) -> Self
  {
    Self { q, r, system : PhantomData, orientation : PhantomData, parity : PhantomData }
  }
}

impl From< Coordinate< Axial, PointyTopped, OddParity > > for Coordinate< Offset, PointyTopped, OddParity >
{
  fn from( value : Coordinate< Axial, PointyTopped, OddParity > ) -> Self
  {
    let col = value.q + ( value.r - ( value.r & 1 ) ) / 2;
    let row = value.r;
    Self::new( col, row )
  }
}

impl From< Coordinate< Axial, PointyTopped, EvenParity > > for Coordinate< Offset, PointyTopped, EvenParity >
{
  fn from( value : Coordinate< Axial, PointyTopped, EvenParity > ) -> Self
  {
    let col = value.q + ( value.r + ( value.r & 1 ) ) / 2;
    let row = value.r;
    Self::new( col, row )
  }
}

impl From< Coordinate< Axial, FlatTopped, OddParity > > for Coordinate< Offset, FlatTopped, OddParity >
{
  fn from( value : Coordinate< Axial, FlatTopped, OddParity > ) -> Self
  {
    let col = value.q;
    let row = value.r + ( value.q - ( value.q & 1 ) ) / 2;
    Self::new( col, row )
  }
}

impl From< Coordinate< Axial, FlatTopped, EvenParity > > for Coordinate< Offset, FlatTopped, EvenParity >
{
  fn from( value : Coordinate< Axial, FlatTopped, EvenParity > ) -> Self
  {
    let col = value.q;
    let row = value.r + ( value.q + ( value.q & 1 ) ) / 2;
    Self::new( col, row )
  }
}

impl From< Coordinate< Offset, PointyTopped, OddParity > > for Coordinate< Axial, PointyTopped, OddParity >
{
  fn from( value : Coordinate< Offset, PointyTopped, OddParity > ) -> Self
  {
    let q = value.q - ( value.r - ( value.r & 1 ) ) / 2;
    let r = value.r;
    Self::new( q, r )
  }
}

impl From< Coordinate< Offset, PointyTopped, EvenParity > > for Coordinate< Axial, PointyTopped, EvenParity >
{
  fn from( value : Coordinate< Offset, PointyTopped, EvenParity > ) -> Self
  {
    let q = value.q - ( value.r + ( value.r & 1 ) ) / 2;
    let r = value.r;
    Self::new( q, r )
  }
}

impl From< Coordinate< Offset, FlatTopped, OddParity > > for Coordinate< Axial, FlatTopped, OddParity >
{
  fn from( value : Coordinate< Offset, FlatTopped, OddParity > ) -> Self
  {
    let q = value.q;
    let r = value.r - ( value.q - ( value.q & 1 ) ) / 2;
    Self::new( q, r )
  }
}

impl From< Coordinate< Offset, FlatTopped, EvenParity > > for Coordinate< Axial, FlatTopped, EvenParity >
{
  fn from( value : Coordinate< Offset, FlatTopped, EvenParity > ) -> Self
  {
    let q = value.q;
    let r = value.r - ( value.q + ( value.q & 1 ) ) / 2;
    Self::new( q, r )
  }
}

impl< Orientation, Parity > Coordinate< Axial, Orientation, Parity >
{
  /// Converts pixel coordinates to axial coordinates in a pointy-topped hexagonal grid.
  ///
  /// # Parameters
  /// - `x`: The x-coordinate in pixels.
  /// - `y`: The y-coordinate in pixels.
  /// - `hex_size`: The size of the hexagons in the grid.
  ///
  /// # Returns
  /// An `Axial` coordinate representing the hexagon at the given pixel position.
  pub fn from_pixel_to_pointy( Pixel { x, y } : Pixel, hex_size : f32 ) -> Self
  {
    // implementation is taken from https://www.redblobgames.com/grids/hexagons/#pixel-to-hex
    let q = ( 3.0f32.sqrt() / 3.0 * x - 1.0 / 3.0 * y ) / hex_size;
    let r = (                           2.0 / 3.0 * y ) / hex_size;
    let ( q, r ) = axial_round( q, r );
    Self::new( q, r )
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
  pub fn from_pixel_to_flat( Pixel { x, y } : Pixel, hex_size : f32 ) -> Self
  {
    // implementation is taken from https://www.redblobgames.com/grids/hexagons/#pixel-to-hex
    let q = ( 2.0 / 3.0 * x                            ) / hex_size;
    let r = ( -1.0 / 3.0 * x + 3.0f32.sqrt() / 3.0 * y ) / hex_size;
    let ( q, r ) = axial_round( q, r );
    Self::new( q, r )
  }

  /// Converts axial coordinates to pixel coordinates in a pointy-topped hexagonal grid.
  ///
  /// # Parameters
  /// - `hex_size`: The size of the hexagons in the grid.
  ///
  /// # Returns
  /// A tuple containing the x and y pixel coordinates of the hexagon.
  pub fn pointy_to_pixel( &self, hex_size : f32 ) -> Pixel
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
  pub fn flat_to_pixel( &self, hex_size : f32 ) -> Pixel
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

impl< Orientation, Parity > std::ops::Add for Coordinate< Axial, Orientation, Parity >
{
  type Output = Self;

  fn add( self, rhs : Self ) -> Self::Output
  {
    Self::new( self.q + rhs.q, self.r + rhs.r )
  }
}

impl< Orientation, Parity > std::ops::Sub for Coordinate< Axial, Orientation, Parity >
{
  type Output = Self;

  fn sub( self, rhs : Self ) -> Self::Output
  {
    Self::new( self.q - rhs.q, self.r - rhs.r )
  }
}

impl< Orientation, Parity > std::ops::Mul< i32 > for Coordinate< Axial, Orientation, Parity >
{
  type Output = Self;

  fn mul( self, rhs : i32 ) -> Self::Output
  {
    Self::new( self.q * rhs, self.r * rhs )
  }
}

impl< Orientation, Parity > std::ops::Div< i32 > for Coordinate< Axial, Orientation, Parity >
{
  type Output = Self;

  fn div( self, rhs : i32 ) -> Self::Output
  {
    Self::new( self.q / rhs, self.r / rhs )
  }
}

impl< F : Into< i32 >, System, Orientation, Parity > From< ( F, F ) > for Coordinate< System, Orientation, Parity >
{
  fn from( ( q, r ) : ( F, F ) ) -> Self
  {
    Self::new( q.into(), r.into() )
  }
}

impl< F : Into< i32 >, System, Orientation, Parity > From< [ F; 2 ] > for Coordinate< System, Orientation, Parity >
{
  fn from( [ q, r ] : [ F; 2 ] ) -> Self
  {
    Self::new( q.into(), r.into() )
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

impl< F : Into< f32 > > From< ( F, F ) > for Pixel
{
  fn from( ( x, y ) : ( F, F ) ) -> Self
  {
    Self { x : x.into(), y : y.into() }
  }
}

impl< F : Into< f32 > > From< [ F; 2 ] > for Pixel
{
  fn from( [ x, y ] : [ F; 2 ] ) -> Self
  {
    Self { x : x.into(), y : y.into() }
  }
}
