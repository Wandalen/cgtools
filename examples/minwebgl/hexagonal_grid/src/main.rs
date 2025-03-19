use std::marker::PhantomData;
use minwebgl as gl;

fn main()
{
  gl::browser::setup( Default::default() );
}

pub struct PointyTop;

pub struct FlatTop;

pub struct Hex< T >
{
  r#type: PhantomData< T >,
  size: f32,
}

impl< T > Hex< T >
{
  pub fn new( size: f32 ) -> Self
  {
    Self { r#type : PhantomData, size }
  }

  pub fn size( &self ) -> f32
  {
    self.size
  }
}

impl Hex< PointyTop >
{
  pub fn width( &self ) -> f32
  {
    3.0f32.sqrt() * self.size
  }

  pub fn height( &self ) -> f32
  {
    2.0 * self.size
  }
}

impl Hex< FlatTop >
{
  pub fn width( &self ) -> f32
  {
    2.0 * self.size
  }

  pub fn height( &self ) -> f32
  {
    3.0f32.sqrt() * self.size
  }
}

struct AxialCoordinate
{
  s : i32,
  q : i32,
  r : i32,
}

struct HexGrid
{

}
