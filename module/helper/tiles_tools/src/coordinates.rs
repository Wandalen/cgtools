pub mod hexagonal;
pub mod pixel;

pub trait Distance
{
  fn distance( &self, other: &Self ) -> u32;
}

pub trait Neigbors : Sized
{
  fn neighbors( &self ) -> Vec< Self >;
}
