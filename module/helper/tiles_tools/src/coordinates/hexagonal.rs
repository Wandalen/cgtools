use std::{ fmt::Debug, hash::Hash, marker::PhantomData };
use super::pixel::Pixel;

pub trait PixelConversion
{
  fn from_pixel( pixel : Pixel, hex_size : f32 ) -> Self;

  fn to_pixel( self, hex_size : f32 ) -> Pixel;
}

pub struct Axial;

pub struct Offset< Parity >( PhantomData< Parity > );

pub struct Pointy;

pub struct Flat;

pub struct Odd;

pub struct Even;

pub struct Coordinate< System, Orientation >
{
  /// Column index
  pub q : i32,
  /// Row index
  pub r : i32,
  system : PhantomData< System >,
  orientation : PhantomData< Orientation >,
}

impl< System, Orientation > Debug for Coordinate< System, Orientation >
{
  fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
  {
    f.debug_struct( "Coordinate" )
    .field( "q", &self.q )
    .field( "r", &self.r )
    .field( "system", &self.system )
    .field( "orientation", &self.orientation )
    .finish()
  }
}

impl< System, Orientation > Clone for Coordinate< System, Orientation >
{
  fn clone( &self ) -> Self
  {
    Self::new_uncheked( self.q, self.r )
  }
}

impl< System, Orientation > Copy for Coordinate< System, Orientation > {}

impl< System, Orientation > Eq for Coordinate< System, Orientation > {}

impl< System, Orientation > PartialEq for Coordinate< System, Orientation >
{
  fn eq( &self, other : &Self ) -> bool
  {
    self.q == other.q && self.r == other.r
  }
}

impl< System, Orientation > Hash for Coordinate< System, Orientation >
{
  fn hash< H : std::hash::Hasher >( &self, state : &mut H )
  {
    self.q.hash( state );
    self.r.hash( state );
    self.system.hash( state );
    self.orientation.hash( state );
  }
}

impl< System, Orientation > Coordinate< System, Orientation >
{
  /// Create a new coordinate
  const fn new_uncheked( q : i32, r : i32 ) -> Self
  {
    Self
    {
      q,
      r,
      system : PhantomData,
      orientation : PhantomData,
    }
  }
}

impl< Orientation, Parity > Coordinate< Offset< Parity >, Orientation >
{
  /// Create a new coordinate
  pub const fn new( q : i32, r : i32 ) -> Self
  {
    Self::new_uncheked( q, r )
  }
}

// impl< Orientation > Coordinate< Offset< Even >, Orientation >
// {
//   /// Create a new coordinate
//   pub const fn new( q : i32, r : i32 ) -> Self
//   {
//     Self::new_uncheked( q, r )
//   }
// }

impl< Orientation > Coordinate< Axial, Orientation >
{
  /// Create a new coordinate
  pub const fn new( q : i32, r : i32 ) -> Self
  {
    Self::new_uncheked( q, r )
  }
}

impl From< Coordinate< Axial, Pointy > > for Coordinate< Offset< Odd >, Pointy >
{
  fn from( value : Coordinate< Axial, Pointy > ) -> Self
  {
    let col = value.q + ( value.r - ( value.r & 1 ) ) / 2;
    let row = value.r;
    Self::new( col, row )
  }
}

impl From< Coordinate< Axial, Pointy > > for Coordinate< Offset< Even >, Pointy >
{
  fn from( value : Coordinate< Axial, Pointy > ) -> Self
  {
    let col = value.q + ( value.r + ( value.r & 1 ) ) / 2;
    let row = value.r;
    Self::new( col, row )
  }
}

impl From< Coordinate< Axial, Flat > > for Coordinate< Offset< Odd >, Flat >
{
  fn from( value : Coordinate< Axial, Flat > ) -> Self
  {
    let col = value.q;
    let row = value.r + ( value.q - ( value.q & 1 ) ) / 2;
    Self::new( col, row )
  }
}

impl From< Coordinate< Axial, Flat > > for Coordinate< Offset< Even >, Flat >
{
  fn from( value : Coordinate< Axial, Flat > ) -> Self
  {
    let col = value.q;
    let row = value.r + ( value.q + ( value.q & 1 ) ) / 2;
    Self::new( col, row )
  }
}

impl From< Coordinate< Offset< Odd >, Pointy > > for Coordinate< Axial, Pointy >
{
  fn from( value : Coordinate< Offset< Odd >, Pointy > ) -> Self
  {
    let q = value.q - ( value.r - ( value.r & 1 ) ) / 2;
    let r = value.r;
    Self::new( q, r )
  }
}

impl From< Coordinate< Offset< Even >, Pointy > > for Coordinate< Axial, Pointy >
{
  fn from( value : Coordinate< Offset< Even >, Pointy > ) -> Self
  {
    let q = value.q - ( value.r + ( value.r & 1 ) ) / 2;
    let r = value.r;
    Self::new( q, r )
  }
}

impl From< Coordinate< Offset< Odd >, Flat > > for Coordinate< Axial, Flat >
{
  fn from( value : Coordinate< Offset< Odd >, Flat > ) -> Self
  {
    let q = value.q;
    let r = value.r - ( value.q - ( value.q & 1 ) ) / 2;
    Self::new( q, r )
  }
}

impl From< Coordinate< Offset< Even >, Flat > > for Coordinate< Axial, Flat >
{
  fn from( value : Coordinate< Offset< Even >, Flat > ) -> Self
  {
    let q = value.q;
    let r = value.r - ( value.q + ( value.q & 1 ) ) / 2;
    Self::new( q, r )
  }
}

impl< Orientation > std::ops::Add for Coordinate< Axial, Orientation >
{
  type Output = Self;

  fn add( self, rhs : Self ) -> Self::Output
  {
    Self::new( self.q + rhs.q, self.r + rhs.r )
  }
}

impl< Orientation > std::ops::Sub for Coordinate< Axial, Orientation >
{
  type Output = Self;

  fn sub( self, rhs : Self ) -> Self::Output
  {
    Self::new( self.q - rhs.q, self.r - rhs.r )
  }
}

impl< Orientation > std::ops::Mul< i32 > for Coordinate< Axial, Orientation >
{
  type Output = Self;

  fn mul( self, rhs : i32 ) -> Self::Output
  {
    Self::new( self.q * rhs, self.r * rhs )
  }
}

impl< Orientation > std::ops::Div< i32 > for Coordinate< Axial, Orientation >
{
  type Output = Self;

  fn div( self, rhs : i32 ) -> Self::Output
  {
    Self::new( self.q / rhs, self.r / rhs )
  }
}

impl PixelConversion for Coordinate< Axial, Pointy >
{
  fn from_pixel( Pixel { data : [ x, y ] } : Pixel, hex_size : f32 ) -> Self
  {
    // implementation is taken from https://www.redblobgames.com/grids/hexagons/#pixel-to-hex
    let q = ( 3.0f32.sqrt() / 3.0 * x - 1.0 / 3.0 * y ) / hex_size;
    let r = (                           2.0 / 3.0 * y ) / hex_size;
    let ( q, r ) = axial_round( q, r );
    Self::new( q, r )
  }

  fn to_pixel( self, hex_size : f32 ) -> Pixel
  {
    // implementation is taken from https://www.redblobgames.com/grids/hexagons/#hex-to-pixel
    let q = self.q as f32;
    let r = self.r as f32;
    let x = hex_size * ( 3.0f32.sqrt() * q + 3.0f32.sqrt() / 2.0 * r );
    let y = hex_size * (                               3.0 / 2.0 * r );
    ( x, y ).into()
  }
}

impl PixelConversion for Coordinate< Axial, Flat >
{
  fn from_pixel( Pixel { data : [ x, y ] } : Pixel, hex_size : f32 ) -> Self
  {
    // implementation is taken from https://www.redblobgames.com/grids/hexagons/#pixel-to-hex
    let q = (  2.0 / 3.0 * x                           ) / hex_size;
    let r = ( -1.0 / 3.0 * x + 3.0f32.sqrt() / 3.0 * y ) / hex_size;
    let ( q, r ) = axial_round( q, r );
    Self::new( q, r )
  }

  fn to_pixel( self, hex_size : f32 ) -> Pixel
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
