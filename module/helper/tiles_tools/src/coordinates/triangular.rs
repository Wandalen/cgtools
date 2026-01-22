use core::marker::PhantomData;
use std::{ fmt::Debug, hash::Hash };
use crate::coordinates;
use coordinates::{ ToDual, Neighbors, Distance, hexagonal, pixel::Pixel };

const SQRT_3 : f32 = 1.732_050_8;

#[ derive( Debug ) ]
pub struct FlatTopped;

#[ derive( Debug ) ]
pub struct FlatSided;

/// Represents a coordinate in a tri-axial grid system, often used for triangular tiling.
/// Each coordinate defines a unique triangle on the grid.
#[ derive( serde::Serialize ) ]
pub struct Coordinate< Orientation >
{
  /// The 'a' component of the tri-axial coordinate.
  pub a : i32,
  /// The 'b' component of the tri-axial coordinate.
  pub b : i32,
  /// The 'c' component of the tri-axial coordinate.
  pub c : i32,
  #[ serde( skip ) ]
  _marker : PhantomData< Orientation >,
}

// Helper type just for deserialization
#[ derive( serde::Deserialize ) ]
struct CoordinateHelper
{
  a : i32,
  b : i32,
  c : i32,
}

impl< 'de, Orientation > serde::Deserialize< 'de > for Coordinate< Orientation >
{
  fn deserialize< D >( deserializer : D ) -> Result< Self, D::Error >
  where
    D : serde::Deserializer< 'de >,
  {
    let helper = CoordinateHelper::deserialize( deserializer )?;

    let sum = helper.a + helper.b + helper.c;
    if sum != 1 && sum != 2
    {
      return Err
      (
        serde::de::Error::custom
        (
          format!( "Invalid coordinate: a + b + c must equal 1 or 2 (got {})", sum )
        )
      );
    }

    Ok( Coordinate::new_unchecked( helper.a, helper.b, helper.c ) )
  }
}

impl< Orientation > Debug for Coordinate< Orientation >
{
  fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
  {
    f
    .debug_struct( "Triangular" )
    .field( "a", &self.a )
    .field( "b", &self.b )
    .field( "c", &self.c )
    .field( "_marker", &self._marker )
    .finish()
  }
}

impl< Orientation > Default for Coordinate< Orientation >
{
  fn default() -> Self
  {
    Self { a : Default::default(), b : Default::default(), c : Default::default(), _marker : Default::default() }
  }
}

impl< Orientation > Clone for Coordinate< Orientation >
{
  fn clone( &self ) -> Self
  {
    Self::new_unchecked( self.a, self.b, self.c )
  }
}

impl< Orientation > Copy for Coordinate< Orientation > {}

impl< Orientation > Hash for Coordinate< Orientation >
{
  fn hash< H : std::hash::Hasher >( &self, state : &mut H )
  {
    self.a.hash( state );
    self.b.hash( state );
    self.c.hash( state );
    self._marker.hash( state );
  }
}

impl< Orientation > PartialEq for Coordinate< Orientation >
{
  fn eq( &self, other : &Self ) -> bool
  {
    self.a == other.a && self.b == other.b && self.c == other.c
  }
}

impl< Orientation > Eq for Coordinate< Orientation > {}

impl< Orientation > Coordinate< Orientation >
{
  /// Creates a new `TriAxial` coordinate from its three components.
  /// The sum of `a` `b` and `c` must equal `1` or `2`
  #[ inline ]
  #[ must_use ]
  pub fn new( a : i32, b : i32, c : i32 ) -> Option< Self >
  {
    let sum = a as i64 + b as i64 + c as i64;
    if ( 1..=2 ).contains( &sum )
    {
      Some( Self { a, b, c, _marker : PhantomData } )
    }
    else
    {
      None
    }
  }

  /// Creates a new `TriAxial` coordinate from its three components.
  #[ inline ]
  #[ must_use ]
  pub( crate ) const fn new_unchecked( a : i32, b : i32, c : i32 ) -> Self
  {
    Self { a, b, c, _marker : PhantomData }
  }

  /// Calculates the hexagonal coordinates of the three vertices of this triangle.
  #[ inline ]
  #[ must_use ]
  pub const fn corners( &self ) -> [ [ i32; 3 ]; 3 ]
  {
    let Self { a, b, c, .. } = *self;
    let is_right = self.is_up_or_right() as i32;

    let offset = -1;
    let additional_offset = -is_right;

    [
      [ offset + a,            b + additional_offset,  c                     ],
      [ a + additional_offset, b,                      offset + c            ],
      [ a,                     offset + b,             c + additional_offset ],
    ]
  }

  #[ inline ]
  pub const fn is_up_or_right( &self ) -> bool { self.a as i64 + self.b as i64 + self.c as i64 == 2 }

  #[ inline ]
  pub const fn is_down_or_left( &self ) -> bool { self.a as i64 + self.b as i64 + self.c as i64 == 1 }
}

impl< HOrientation, TOrientation > ToDual< hexagonal::Coordinate< hexagonal::Axial, HOrientation > > for Coordinate< TOrientation >
{
  #[ inline ]
  fn dual( &self ) -> Vec< hexagonal::Coordinate< hexagonal::Axial, HOrientation > >
  {
    let corners = self.corners();
    corners.map
    (
      | [ a, b, .. ] |
      {
        let q = a;
        let r = b;
        hexagonal::Coordinate::new_uncheked( q, r )
      }
    ).to_vec()
  }
}

impl< Orientation > Neighbors for Coordinate< Orientation >
{
  /// Returns the three immediate neighbors of the current triangle.
  #[ inline ]
  fn neighbors( &self ) -> Vec< Self >
  {
    let Self { a, b, c, .. } = *self;

    let is_right = self.is_up_or_right() as i32;
    let is_left = self.is_down_or_left() as i32;
    let offset = -is_right + is_left;

    [
      Self::new_unchecked( a + offset, b, c ),
      Self::new_unchecked( a, b + offset, c ),
      Self::new_unchecked( a, b, c + offset ),
    ].to_vec()
  }
}

impl< Orientation > Distance for Coordinate< Orientation >
{
  #[ inline ]
  fn distance( &self, other : &Self ) -> u32
  {
    (
      ( self.a as i64 - other.a as i64 ).abs()
      + ( self.b as i64 - other.b as i64 ).abs()
      + ( self.c as i64 - other.c as i64 ).abs()
    ) as u32
  }
}

impl Coordinate< FlatSided >
{
  /// Converts a 2D point (e.g., from a mouse click) to the nearest `TriAxial` coordinate.
  #[ inline ]
  #[ must_use ]
  #[ allow( clippy::cast_possible_truncation ) ]
  pub fn from_pixel_with_edge_len( Pixel { data : [ x, y ]  } : Pixel, edge_length : f32 ) -> Self
  {
    let cell_size : [ f32; 2 ] = [ edge_length * SQRT_3 / 2.0, edge_length ];

    let x = x / cell_size[ 0 ];
    let y = y / cell_size[ 1 ];

    Self::new_unchecked
    (
      x.floor() as i32 + 1,
      ( y - 0.5 * x ).ceil() as i32,
      ( -y - 0.5 * x ).ceil() as i32,
    )
  }

  /// Converts a `TriAxial` coordinate to its 2D point representation in space.
  #[ inline ]
  #[ must_use ]
  pub fn to_pixel_with_edge_len( &self, edge_length : f32 ) -> Pixel
  {
    let cell_size : [ f32; 2 ] = [ edge_length * SQRT_3 / 2.0, edge_length ];

    let Self { a, b, c, .. } = *self;

    [
      ( -1.0 / 3.0 * b as f32 + 2.0 / 3.0 * a as f32 - 1.0 / 3.0 * c as f32 ) * cell_size[ 0 ],
      ( 0.5 * b as f32 - 0.5 * c as f32 ) * cell_size[ 1 ],
    ].into()
  }
}

impl Coordinate< FlatTopped >
{
  /// Converts a 2D point (e.g., from a mouse click) to the nearest `TriAxial` coordinate.
  #[ inline ]
  #[ must_use ]
  #[ allow( clippy::cast_possible_truncation ) ]
  pub fn from_pixel_with_edge_len( Pixel { data : [ x, y ]  } : Pixel, edge_length : f32 ) -> Self
  {
    Self::new_unchecked
    (
      ( (   x - SQRT_3 / 3.0 * y ) / edge_length ).ceil()  as i32,
      ( (       SQRT_3 * 2.0 / 3.0 * y ) / edge_length ).floor() as i32 + 1,
      ( ( - x - SQRT_3 / 3.0 * y ) / edge_length ).ceil()  as i32,
    )
  }

  /// Converts a `TriAxial` coordinate to its 2D point representation in space.
  #[ inline ]
  #[ must_use ]
  pub fn to_pixel_with_edge_len( &self, edge_length : f32 ) -> Pixel
  {
    let Self { a, b, c, .. } = *self;

    [
      (           0.5 * a as f32 +                                   -0.5 * c as f32 ) * edge_length,
      ( -SQRT_3 / 6.0 * a as f32 + SQRT_3 / 3.0 * b as f32 - SQRT_3 / 6.0 * c as f32 ) * edge_length
    ].into()
  }
}

impl< Orientation > TryFrom< [ i32; 3 ] > for Coordinate< Orientation >
{
  type Error = String;

  fn try_from( [ a, b, c ] : [ i32; 3 ] ) -> Result< Self, Self::Error >
  {
    Self::new( a, b, c ).ok_or( "Sum of coordinates must equal 1 or 2".into() )
  }
}

impl< Orientation > TryFrom< ( i32, i32, i32 ) > for Coordinate< Orientation >
{
  type Error = String;

  fn try_from( ( a, b, c ) : ( i32, i32, i32 ) ) -> Result< Self, Self::Error >
  {
    Self::new( a, b, c ).ok_or( "Sum of coordinates must equal 1 or 2".into() )
  }
}

impl< Orientation > Into< [ i32; 3 ] > for Coordinate< Orientation >
{
  fn into( self ) -> [ i32; 3 ]
  {
    [ self.a, self.b, self.c ]
  }
}

impl< Orientation > Into< ( i32, i32, i32 ) > for Coordinate< Orientation >
{
  fn into( self ) -> ( i32, i32, i32 )
  {
    ( self.a, self.b, self.c )
  }
}
