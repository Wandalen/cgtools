use minwebgl::math::Array2;
use rustc_hash::FxHashMap;
use crate::{ coordinates::Axial, layout::HexLayout };

/// A type alias for a hash map that associates axial coordinates with values.
/// This is commonly used to store data for hexagonal grids.
///
/// # Type Parameters
/// - `T`: The type of the values stored in the map.
pub type HexMap< T > = FxHashMap< Axial, T >;

pub struct HexGrid< T >
{
  map : HexMap< T >,
  layout : HexLayout,
}

impl< T > HexGrid< T >
{
  pub fn new( map : HexMap< T >, size : f32, layout : HexLayout ) -> Self
  {
    Self { map, layout }
  }

  pub fn layout( &self ) -> HexLayout
  {
    self.layout
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

pub struct HexArray< T, Layout >
{
  data : Array2< Option< T > >,
  layout : Layout,
  /// The offset is added to the coordinates when indexing the array,
  /// it is needed to make negative coordinates usable.
  offset : Axial,
}

impl< T, Layout > HexArray< T, Layout >
{
  /// Creates a new hexagonal grid with the given number of rows and columns.
  ///
  /// # Parameters
  /// - `rows`: The number of rows in the grid.
  /// - `columns`: The number of columns in the grid.
  /// - `offset`: The offset is added to the coordinates when indexing the array,
  /// it is needed to make negative coordinates usable.
  /// For example if you want Axial( -1, -4 ) to be valid, set offset to Axial( 1, 4 ).
  /// All negative coordinates up to ( -1, -4 ) will be valid.
  /// - `layout`: The layout of the hexagons in the grid.
  pub fn new( rows : i32, columns : i32, offset: Axial, layout : Layout ) -> Self
  {
    let rows : usize = ( rows + offset.r ).try_into().unwrap();
    let columns : usize = ( columns + offset.q ).try_into().unwrap();
    Self { data : Array2::from_shape_fn( ( rows , columns ), | _ | None ), layout, offset }
  }

  /// Insets a value at the given axial coordinates.
  /// Returns the previous value at the coordinates if there was one.
  pub fn insert( &mut self, coord : Axial, value : T ) -> Option< T >
  {
    let coord = Axial::new( self.offset.q + coord.q, self.offset.r + coord.r );
    let i : usize = coord.r.try_into().unwrap();
    let j : usize = coord.q.try_into().unwrap();
    std::mem::replace( &mut self.data[ ( i, j ) ], Some( value ) )
  }

  pub fn get( &self, coord : Axial ) -> Option< &T >
  {
    let coord = Axial::new( self.offset.q + coord.q, self.offset.r + coord.r );
    let i : usize = coord.r.try_into().ok()?;
    let j : usize = coord.q.try_into().ok()?;
    self.data.get( ( i, j ) ).and_then( | x | x.as_ref() )
  }
}
