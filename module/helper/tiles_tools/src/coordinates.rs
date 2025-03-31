use std::{ hash::Hash, marker::PhantomData, ops::{ Index, IndexMut } };
use ndarray_cg::{ Collection, MatEl, NdFloat, Vector };

pub trait CoordinateSystem {}

pub trait OrientationType {}

pub trait ParityType {}

/// Axial coordinates use two axes (`q` and `r`) to uniquely identify
/// hexes in a grid.
/// more info: https://www.redblobgames.com/grids/hexagons/#coordinates-axial
pub struct Axial;

impl CoordinateSystem for Axial {}

/// Offset coordinate system comes in 4 forms:
/// - Pointy-topped odd parity
/// - Pointy-topped even parity
/// - Flat-topped odd parity
/// - Flat-topped even parity
/// more info: https://www.redblobgames.com/grids/hexagons/#coordinates-offset
pub struct Offset;

impl CoordinateSystem for Offset {}

/// Doubled variant of Offset coordinates.
/// Instead of alternation, the doubled coordinates double either the horizontal or vertical step size.
/// https://www.redblobgames.com/grids/hexagons/#coordinates-doubled
pub struct Doubled;

impl CoordinateSystem for Doubled {}

/// Orientation of the hexagons when the top is flat.
pub struct FlatTopped;

impl OrientationType for FlatTopped {}

/// Orientation of the hexagons when the top is pointed.
pub struct PointyTopped;

impl OrientationType for PointyTopped {}

/// Parity of the hexagons where odd rows/columns are shoved
pub struct OddParity;

impl ParityType for OddParity {}

/// Parity of the hexagons where even rows/columns are shoved
pub struct EvenParity;

impl ParityType for EvenParity {}

/// Represents a coordinate in a hexagonal grid.
///
/// # Fields
/// - `q`: The "column" coordinate.
/// - `r`: The "row" coordinate.
#[ derive( Debug ) ]
pub struct Coordinate< System : CoordinateSystem, Orientation : OrientationType, Parity : ParityType >
{
  /// The "column" coordinate in the coordinate system.
  pub q : i32,
  /// The "row" coordinate in the coordinate system.
  pub r : i32,
  system : PhantomData< System >,
  orientation : PhantomData< Orientation >,
  parity : PhantomData< Parity >,
}

impl< System : CoordinateSystem, Orientation : OrientationType, Parity : ParityType > Hash for Coordinate< System, Orientation, Parity >
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

impl< System : CoordinateSystem, Orientation : OrientationType, Parity : ParityType > PartialEq for Coordinate< System, Orientation, Parity >
{
  fn eq( &self, other : &Self ) -> bool
  {
    self.q == other.q && self.r == other.r
  }
}

impl< System : CoordinateSystem, Orientation : OrientationType, Parity : ParityType > Eq for Coordinate< System, Orientation, Parity > {}

impl< System : CoordinateSystem, Orientation : OrientationType, Parity : ParityType > Clone for Coordinate< System, Orientation, Parity >
{
  fn clone( &self ) -> Self
  {
    Self { q : self.q, r : self.r, system : PhantomData, orientation : PhantomData, parity : PhantomData }
  }
}

impl< System : CoordinateSystem, Orientation : OrientationType, Parity : ParityType > Copy for Coordinate< System, Orientation, Parity > {}

impl< System : CoordinateSystem, Orientation : OrientationType, Parity : ParityType > Coordinate< System, Orientation, Parity >
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

impl< Parity : ParityType > From< Coordinate< Doubled, FlatTopped, Parity > > for Coordinate< Axial, FlatTopped, Parity >
{
  fn from( value : Coordinate< Doubled, FlatTopped, Parity > ) -> Self
  {
    let q = value.q;
    let r = ( value.r - value.q ) / 2;
    Self::new( q, r )
  }
}

impl< Parity : ParityType > From< Coordinate< Doubled, PointyTopped, Parity > > for Coordinate< Axial, PointyTopped, Parity >
{
  fn from( value : Coordinate< Doubled, PointyTopped, Parity > ) -> Self
  {
    let q = ( value.q - value.r ) / 2;
    let r = value.r;
    Self::new( q, r )
  }
}

impl< Parity : ParityType > From< Coordinate< Axial, FlatTopped, Parity > > for Coordinate< Doubled, FlatTopped, Parity >
{
  fn from( value : Coordinate< Axial, FlatTopped, Parity > ) -> Self
  {
    let q = value.q;
    let r = 2 * value.r + value.q;
    Self::new( q, r )
  }
}

impl< Parity : ParityType > From< Coordinate< Axial, PointyTopped, Parity > > for Coordinate< Doubled, PointyTopped, Parity >
{
  fn from( value : Coordinate< Axial, PointyTopped, Parity > ) -> Self
  {
    let q = 2 * value.q + value.r;
    let r = value.r;
    Self::new( q, r )
  }
}

// qqq : xxx : all 4 methods depends on Orientation! so remove and keep 2 and implement better trait for 2 specified structs

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

pub trait CoordinateConversion
{
  fn from_pixel( pixel : Pixel, hex_size : f32 ) -> Self;

  fn to_pixel( self, hex_size : f32 ) -> Pixel;
}

impl< Parity : ParityType > CoordinateConversion for Coordinate< Axial, PointyTopped, Parity >
{
  fn from_pixel( Pixel { data : [ x, y ] } : Pixel, hex_size : f32 ) -> Self
  {
    // implementation is taken from https://www.redblobgames.com/grids/hexagons/#pixel-to-hex
    let q = ( 3.0f32.sqrt() / 3.0 * x - 1.0 / 3.0 * y ) / hex_size;
    let r = (                           2.0 / 3.0 * y ) / hex_size;
    let ( q, r ) = axial_round( q, r );
    Self::new( q, r )
    // todo!()
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

impl< Parity : ParityType > CoordinateConversion for Coordinate< Axial, FlatTopped, Parity >
{
  fn from_pixel( Pixel { data : [ x, y ] } : Pixel, hex_size : f32 ) -> Self
  {
    // implementation is taken from https://www.redblobgames.com/grids/hexagons/#pixel-to-hex
    let q = ( 2.0 / 3.0 * x                            ) / hex_size;
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

impl< Orientation : OrientationType, Parity : ParityType > std::ops::Add for Coordinate< Axial, Orientation, Parity >
{
  type Output = Self;

  fn add( self, rhs : Self ) -> Self::Output
  {
    Self::new( self.q + rhs.q, self.r + rhs.r )
  }
}

impl< Orientation : OrientationType, Parity : ParityType > std::ops::Sub for Coordinate< Axial, Orientation, Parity >
{
  type Output = Self;

  fn sub( self, rhs : Self ) -> Self::Output
  {
    Self::new( self.q - rhs.q, self.r - rhs.r )
  }
}

impl< Orientation : OrientationType, Parity : ParityType > std::ops::Mul< i32 > for Coordinate< Axial, Orientation, Parity >
{
  type Output = Self;

  fn mul( self, rhs : i32 ) -> Self::Output
  {
    Self::new( self.q * rhs, self.r * rhs )
  }
}

impl< Orientation : OrientationType, Parity : ParityType > std::ops::Div< i32 > for Coordinate< Axial, Orientation, Parity >
{
  type Output = Self;

  fn div( self, rhs : i32 ) -> Self::Output
  {
    Self::new( self.q / rhs, self.r / rhs )
  }
}

impl< F : Into< i32 >, System : CoordinateSystem, Orientation : OrientationType, Parity : ParityType > From< ( F, F ) > for Coordinate< System, Orientation, Parity >
{
  fn from( ( q, r ) : ( F, F ) ) -> Self
  {
    Self::new( q.into(), r.into() )
  }
}

impl< F : Into< i32 >, System : CoordinateSystem, Orientation : OrientationType, Parity : ParityType > From< [ F; 2 ] > for Coordinate< System, Orientation, Parity >
{
  fn from( [ q, r ] : [ F; 2 ] ) -> Self
  {
    Self::new( q.into(), r.into() )
  }
}

// qqq : implement all applicable math_core traits for the newtype
/// Represents a pixel coordinate in a 2D space.
/// Assumes that Y-axis points down.
#[ derive( Debug, Clone, Copy, PartialEq ) ]
pub struct Pixel
{
  pub data : [ f32; 2 ],
}

impl Pixel
{
  /// Creates a new `Pixel` coordinate with the specified `x` and `y` values.
  pub fn new( x : f32, y : f32 ) -> Self
  {
    Self { data : [ x.into(), y.into() ] }
  }
}

impl< F : Into< f32 > > From< ( F, F ) > for Pixel
{
  fn from( ( x, y ) : ( F, F ) ) -> Self
  {
    Self { data : [ x.into(), y.into() ] }
  }
}

impl< F : Into< f32 > > From< [ F; 2 ] > for Pixel
{
  fn from( [ x, y ] : [ F; 2 ] ) -> Self
  {

    Self { data : [ x.into(), y.into() ] }
  }
}

impl< E : MatEl + Into< f32 > > From< Vector< E, 2 > > for Pixel
{
  fn from( value : Vector< E, 2 >) -> Self
  {
    Self
    {
      data : [ value.0[ 0 ].into(), value.0[ 1 ].into() ]
    }
  }
}


impl Collection for Pixel
{
  type Scalar = f32;
}

impl ndarray_cg::Add for Pixel
{
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output
  {
    Self
    {
      data :
      [
        self.data[ 0 ] + rhs.data[ 0 ], 
        self.data[ 1 ] + rhs.data[ 1 ]
      ]
    }
  }
}

impl ndarray_cg::Sub for Pixel
{
  type Output = Self;

  fn sub(self, rhs: Self) -> Self::Output
  {
    Self
    {
      data :
      [
        self.data[ 0 ] - rhs.data[ 0 ], 
        self.data[ 1 ] - rhs.data[ 1 ]
      ]
    }
  }
}

impl< E : Into< f32 > + MatEl + NdFloat > From< Vector< E, 2 > > for Pixel
{
  fn from( value: Vector< E, 2 > ) -> Self
  {
    Self { data : [ value.x().into(), value.y().into() ] }
  }
}

impl Index< usize > for Pixel
{
  type Output = f32;

  fn index( &self, index: usize ) -> &Self::Output
  {
    &self.data[ index ]
  }
}

impl IndexMut< usize > for Pixel
{
  fn index_mut(&mut self, index: usize) -> &mut Self::Output
  {
    &mut self.data[ index ]    
  }
}

// impl Indexable for Pixel
// {
//   type Index = Ix1;

//   fn dim( &self ) -> Self::Index
//   {
//     Ix1( 0 )
//   }
// }