// use std::marker::PhantomData;
use rustc_hash::FxHashMap;
use crate::{ coordinates::Axial, layout::HexLayout };

/// A type alias for a hash map that associates axial coordinates with values.
/// This is commonly used to store data for hexagonal grids.
///
/// # Type Parameters
/// - `T`: The type of the values stored in the map.
pub type HexMap< T > = FxHashMap< Axial, T >;

pub struct HexGrid< T, Layout >
{
  map : HexMap< T >,
  layout : Layout,
  size : f32,
}

impl< T, Layout > HexGrid< T, Layout >
{
  pub fn new( map : HexMap< T >, size : f32, layout : Layout ) -> Self
  {
    Self { map, size, layout }
  }

  pub fn layout( &self ) -> &Layout
  {
    &self.layout
  }

  pub fn size( &self ) -> f32
  {
    self.size
  }

  pub fn map( &self ) -> &HexMap< T >
  {
    &self.map
  }

  pub fn map_mut( &mut self ) -> &mut HexMap< T >
  {
    &mut self.map
  }
}
