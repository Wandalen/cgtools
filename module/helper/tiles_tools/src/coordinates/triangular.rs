use core::marker::PhantomData;
use std::{ fmt::Debug, hash::Hash };
use crate::coordinates;
use coordinates::{ ToDual, Neighbors, hexagonal::*, pixel::Pixel };

#[ derive( Debug ) ]
pub struct FlatTopped;

#[ derive( Debug ) ]
pub struct FlatSided;

/// Represents a coordinate in a tri-axial grid system, often used for triangular tiling.
/// Each coordinate defines a unique triangle on the grid.
#[ derive( serde::Serialize, serde::Deserialize ) ]
pub struct Triangular< Orientation >
{
  /// The 'a' component of the tri-axial coordinate.
  pub a : i32,
  /// The 'b' component of the tri-axial coordinate.
  pub b : i32,
  /// The 'c' component of the tri-axial coordinate.
  pub c : i32,
  #[ serde( skip ) ]
  _marker : PhantomData< Orientation >
}

impl< Orientation > Debug for Triangular< Orientation >
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

impl< Orientation > Clone for Triangular< Orientation >
{
  fn clone( &self ) -> Self
  {
    Self::new( self.a, self.b, self.c )
  }
}

impl< Orientation > Copy for Triangular< Orientation > {}

impl< Orientation > Hash for Triangular< Orientation >
{
  fn hash< H : std::hash::Hasher >( &self, state : &mut H )
  {
    self.a.hash( state );
    self.b.hash( state );
    self.c.hash( state );
    self._marker.hash( state );
  }
}

impl< Orientation > PartialEq for Triangular< Orientation >
{
  fn eq(&self, other: &Self) -> bool
  {
    self.a == other.a && self.b == other.b && self.c == other.c
  }
}

impl< Orientation > Eq for Triangular< Orientation > {}

impl< Orientation > Triangular< Orientation >
{
  /// A pre-calculated constant for the square root of 3.
  const SQRT_3 : f32 = 1.732_050_8;
  /// Distance between neighbor unit hexagonals equals to length of a triangle side
  const SIDE_LENGHT : f32 = Self::SQRT_3;
  /// The size of a cell used for converting between pixel and grid coordinates.
  const CELL_SIZE : [ f32; 2 ] = [ Self::SIDE_LENGHT * Self::SQRT_3 / 2.0, Self::SIDE_LENGHT ];

  /// Creates a new `TriAxial` coordinate from its three components.
  #[ inline ]
  #[ must_use ]
  pub const fn new( a : i32, b : i32, c : i32 ) -> Self
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
  pub const fn is_up_or_right( &self ) -> bool { self.a + self.b + self.c == 2 }

  #[ inline ]
  pub const fn is_down_or_left( &self ) -> bool { self.a + self.b + self.c == 1 }

  /// Finds the six triangles that surround a given hexagonal coordinate.
  #[ inline ]
  #[ must_use ]
  pub const fn from_hex_coord( HexCoord { q, r, .. } : HexCoord ) -> [ Self; 6 ]
  {
    let s = -q - r;

    let a = q;
    let b = r;
    let c = s;

    [
      Triangular::new( a + 1, b,     c     ),
      Triangular::new( a,     b + 1, c     ),
      Triangular::new( a,     b,     c + 1 ),
      Triangular::new( a + 1, b + 1, c     ),
      Triangular::new( a,     b + 1, c + 1 ),
      Triangular::new( a + 1, b,     c + 1 ),
    ]
  }
}

impl< HOrientation, TOrientation > ToDual< Coordinate< Axial, HOrientation > > for Triangular< TOrientation >
{
  #[ inline ]
  fn dual( &self ) -> Vec< Coordinate< Axial, HOrientation > >
  {
    let corners = self.corners();
    corners.map
    (
      | [ a, b, .. ] |
      {
        let q = a;
        let r = b;
        Coordinate::new_uncheked( q, r )
      }
    ).to_vec()
  }
}

impl< Orientation > Neighbors for Triangular< Orientation >
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
      Self::new( a + offset, b, c ),
      Self::new( a, b + offset, c ),
      Self::new( a, b, c + offset ),
    ].to_vec()
  }
}

impl Triangular< FlatSided >
{
   /// Converts a 2D point (e.g., from a mouse click) to the nearest `TriAxial` coordinate.
  #[ inline ]
  #[ must_use ]
  #[ allow( clippy::cast_possible_truncation ) ]
  pub fn from_pixel_with_size( Pixel { data : [ x, y ]  } : Pixel, side_size : f32 ) -> Self
  {
    let x = x / Self::CELL_SIZE[ 0 ];
    let y = y / Self::CELL_SIZE[ 1 ];

    Self::new
    (
      x.floor() as i32 + 1,
      ( y - 0.5 * x ).ceil() as i32,
      ( -y - 0.5 * x ).ceil() as i32,
    )
  }

  /// Converts a `TriAxial` coordinate to its 2D point representation in space.
  #[ inline ]
  #[ must_use ]
  pub const fn to_pixel_with_size( &self, side_size : f32 ) -> Pixel
  {
    let Self { a, b, c, .. } = *self;
    [
      ( -1.0 / 3.0 * b as f32 + 2.0 / 3.0 * a as f32 - 1.0 / 3.0 * c as f32 ) * Self::CELL_SIZE[ 0 ],
      ( 0.5 * b as f32 - 0.5 * c as f32 ) * Self::CELL_SIZE[ 1 ],
    ].into()
  }
}
