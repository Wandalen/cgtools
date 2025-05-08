use super::hexagonal::*;

// aaa : move into a separate file
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

impl< F > From< ( F, F ) > for Pixel
where
  F : Into< f32 >
{
  fn from( ( x, y ) : ( F, F ) ) -> Self
  {
    Self { data : [ x.into(), y.into() ] }
  }
}

impl< F > From< [ F; 2 ] > for Pixel
where
  F : Into< f32 >
{
  fn from( [ x, y ] : [ F; 2 ] ) -> Self
  {

    Self { data : [ x.into(), y.into() ] }
  }
}

impl From< Coordinate< Axial, Pointy > > for Pixel
{
  fn from( value : Coordinate< Axial, Pointy > ) -> Self
  {
    let q = value.q as f32;
    let r = value.r as f32;
    let x = 3.0f32.sqrt() * q + 3.0f32.sqrt() / 2.0 * r;
    let y =                               3.0 / 2.0 * r;
    ( x, y ).into()
  }
}

impl From< Coordinate< Axial, Flat > > for Pixel
{
  fn from( value : Coordinate< Axial, Flat > ) -> Self
  {
    let q = value.q as f32;
    let r = value.r as f32;
    let x =           3.0 / 2.0 * q                    ;
    let y = 3.0f32.sqrt() / 2.0 * q + 3.0f32.sqrt() * r;
    ( x, y ).into()
  }
}

impl< E > From< ndarray_cg::Vector< E, 2 > > for Pixel
where
  E : ndarray_cg::MatEl + Into< f32 >
{
  fn from( value : ndarray_cg::Vector< E, 2 >) -> Self
  {
    Self
    {
      data : [ value[ 0 ].into(), value[ 1 ].into() ]
    }
  }
}

impl ndarray_cg::Collection for Pixel
{
  type Scalar = f32;
}

impl ndarray_cg::Add for Pixel
{
  type Output = Self;

  fn add( self, rhs : Self ) -> Self::Output
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

  fn sub( self, rhs : Self ) -> Self::Output
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

impl std::ops::Index< usize > for Pixel
{
  type Output = f32;

  fn index( &self, index : usize ) -> &Self::Output
  {
    &self.data[ index ]
  }
}

impl std::ops::IndexMut< usize > for Pixel
{
  fn index_mut( &mut self, index : usize ) -> &mut Self::Output
  {
    &mut self.data[ index ]
  }
}
