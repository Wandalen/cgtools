/// Trait for calculating distance between coordinates.
pub trait Distance
{
  /// Calculate the distance between two coordinate points.
  fn distance( &self, other: &Self ) -> u32;
}

/// Trait for finding neighboring coordinates.
pub trait Neighbors : Sized
{
  /// Get all neighboring coordinates for this position.
  fn neighbors( &self ) -> Vec< Self >;
}

pub mod hexagonal;
pub mod pixel;
