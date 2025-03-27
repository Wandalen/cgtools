use std::{hash::Hash, marker::PhantomData};

/// Axial coordinates use two axes (`q` and `r`) to uniquely identify
/// hexes in a grid.
/// more info: https://www.redblobgames.com/grids/hexagons/#coordinates-axial
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub struct Axial;

#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub struct OffsetOddR;

#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub struct OffsetEvenR;

#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub struct OffsetOddQ;

#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub struct OffsetEvenQ;

/// Represents a coordinate in a hexagonal grid.
///
/// # Fields
/// - `q`: The "column" coordinate.
/// - `r`: The "row" coordinate.
#[ derive( Debug ) ]
pub struct Coordinate< Type >
{
  /// The "column" coordinate in the axial coordinate system.
  pub q : i32,
  /// The "row" coordinate in the axial coordinate system.
  pub r : i32,
  r#type : PhantomData< Type >
}

impl< Type > Hash for Coordinate< Type >
{
  fn hash< H : std::hash::Hasher >( &self, state : &mut H )
  {
    self.q.hash( state );
    self.r.hash( state );
    self.r#type.hash( state );
  }
}

impl< Type > PartialEq for Coordinate< Type >
{
  fn eq( &self, other : &Self ) -> bool
  {
    self.q == other.q && self.r == other.r
  }
}

impl< Type > Eq for Coordinate< Type > {}

impl< Type > Clone for Coordinate< Type >
{
  fn clone( &self ) -> Self
  {
    Self { q : self.q, r : self.r, r#type : PhantomData }
  }
}

impl< Type > Copy for Coordinate< Type > {}

impl< Type > Coordinate< Type >
{
  pub fn new( q : i32, r : i32 ) -> Self
  {
    Self { q, r, r#type : PhantomData }
  }
}

impl From< Coordinate< Axial > > for Coordinate< OffsetOddR >
{
  fn from( value : Coordinate< Axial > ) -> Self
  {
    let col = value.q + ( value.r - ( value.r & 1 ) ) / 2;
    let row = value.r;
    Self::new( col, row )
  }
}

impl From< Coordinate< Axial > > for Coordinate< OffsetEvenR >
{
  fn from( value : Coordinate< Axial > ) -> Self
  {
    let col = value.q + ( value.r + ( value.r & 1 ) ) / 2;
    let row = value.r;
    Self::new( col, row )
  }
}

impl From< Coordinate< Axial > > for Coordinate< OffsetOddQ >
{
  fn from( value : Coordinate< Axial > ) -> Self
  {
    let col = value.q;
    let row = value.r + ( value.q - ( value.q & 1 ) ) / 2;
    Self::new( col, row )
  }
}

impl From< Coordinate< Axial > > for Coordinate< OffsetEvenQ >
{
  fn from( value : Coordinate< Axial > ) -> Self
  {
    let col = value.q;
    let row = value.r + ( value.q + ( value.q & 1 ) ) / 2;
    Self::new( col, row )
  }
}

impl From< Coordinate< OffsetOddR > > for Coordinate< Axial >
{
  fn from( value : Coordinate< OffsetOddR > ) -> Self
  {
    let q = value.q - ( value.r - ( value.r & 1 ) ) / 2;
    let r = value.r;
    Self::new( q, r )
  }
}

impl From< Coordinate< OffsetEvenR > > for Coordinate< Axial >
{
  fn from( value : Coordinate< OffsetEvenR > ) -> Self
  {
    let q = value.q - ( value.r + ( value.r & 1 ) ) / 2;
    let r = value.r;
    Self::new( q, r )
  }
}

impl From< Coordinate< OffsetOddQ > > for Coordinate< Axial >
{
  fn from( value : Coordinate< OffsetOddQ > ) -> Self
  {
    let q = value.q;
    let r = value.r - ( value.q - ( value.q & 1 ) ) / 2;
    Self::new( q, r )
  }
}

impl From< Coordinate< OffsetEvenQ > > for Coordinate< Axial >
{
  fn from( value : Coordinate< OffsetEvenQ > ) -> Self
  {
    let q = value.q;
    let r = value.r - ( value.q + ( value.q & 1 ) ) / 2;
    Self::new( q, r )
  }
}

impl Coordinate< Axial >
{
  // Creates a new `Axial` coordinate with the specified `q` and `r` values.
  //
  // # Parameters
  // - `q`: The "column" coordinate in the axial system.
  // - `r`: The "row" coordinate in the axial system.
  //
  // # Returns
  // A new `Axial` instance.


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

impl< T > std::ops::Add for Coordinate< T >
{
  type Output = Self;

  fn add( self, rhs : Self ) -> Self::Output
  {
    Self::new( self.q + rhs.q, self.r + rhs.r )
  }
}

impl< T > std::ops::Sub for Coordinate< T >
{
  type Output = Self;

  fn sub( self, rhs : Self ) -> Self::Output
  {
    Self::new( self.q - rhs.q, self.r - rhs.r )
  }
}

impl< T, F : Into< i32 > > From< ( F, F ) > for Coordinate< T >
{
  fn from( ( q, r ) : ( F, F ) ) -> Self
  {
    Self::new( q.into(), r.into() )
  }
}

impl< T, F : Into< i32 > > From< [ F; 2 ] > for Coordinate< T >
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
