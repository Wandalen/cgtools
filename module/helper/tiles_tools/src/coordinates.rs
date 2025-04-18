pub mod hexagonal;
pub mod pixel;

pub trait Distance
{
  fn distance( &self, other: &Self ) -> i32;
}

pub trait Neigbors : Sized
{
  fn neighbors( &self ) -> Vec< Self >;
}
