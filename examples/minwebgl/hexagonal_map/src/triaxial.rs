use serde::{ Serialize, Deserialize };

#[ derive( Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize ) ]
pub struct TriAxial
{
  pub a : i32,
  pub b : i32,
  pub c : i32,
}

impl TriAxial
{
  const SQRT_3 : f32 = 1.73205080757;
  // Distance between neighbor unit hexagonals equals to length of a triangle side
  const SIDE_LENGHT : f32 = Self::SQRT_3;
  const CELL_SIZE : [ f32; 2 ] = [ Self::SIDE_LENGHT * Self::SQRT_3 / 2.0, Self::SIDE_LENGHT * 1.0 ];

  pub const fn new( a : i32, b : i32, c : i32 ) -> Self
  {
    Self { a, b, c }
  }

  pub const fn is_left( &self ) -> bool { self.a + self.b + self.c == 1 }

  pub const fn is_right( &self ) -> bool { self.a + self.b + self.c == 2 }

  pub fn from_point( x : f32, y : f32 ) -> Self
  {
    let x = x / Self::CELL_SIZE[ 0 ];
    let y = y / Self::CELL_SIZE[ 1 ];

    TriAxial
    {
      a : x.floor() as i32 + 1,
      b : ( y - 0.5 * x ).ceil() as i32,
      c : ( -y - 0.5 * x ).ceil() as i32,
    }
  }

  pub const fn to_point( &self ) -> [ f32; 2 ]
  {
    let Self { a, b, c } = *self;

    [
      ( -1.0 / 3.0 * b as f32 + 2.0 / 3.0 * a as f32 - 1.0 / 3.0 * c as f32 ) * Self::CELL_SIZE[ 0 ],
      ( 0.5 * b as f32 - 0.5 * c as f32 ) * Self::CELL_SIZE[ 1 ],
    ]
  }

  pub const fn neighbors( &self ) -> [ TriAxial; 3 ]
  {
    let Self { a, b, c } = *self;

    let is_right = self.is_right() as i32;
    let is_left = self.is_left() as i32;
    let offset = -1 * is_right + is_left;

    [
      Self::new( a + offset, b, c ),
      Self::new( a, b + offset, c ),
      Self::new( a, b, c + offset ),
    ]
  }
}
