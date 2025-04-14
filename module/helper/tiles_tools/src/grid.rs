// use crate::*;
// use coordinates::*;
// use rustc_hash::FxHashMap;
// use layout::HexLayout;
use ndarray_cg::Array2;
use crate::coordinates::hexagonal::Coordinate;

pub struct HexArray< System, Orientation, T >
{
  data : Array2< Option< T > >,
  offset : Coordinate< System, Orientation >,
}

impl< System, Orientation, T > HexArray< System, Orientation, T >
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
  pub fn new( rows : i32, columns : i32, offset : Coordinate< System, Orientation > ) -> Self
  {
    let rows : usize = ( rows ).try_into().unwrap();
    let columns : usize = ( columns ).try_into().unwrap();
    Self { data : Array2::from_shape_fn( ( rows , columns ), | _ | None ), offset }
  }

  /// Insets a value at the given coordinates.
  /// Returns the previous value at the coordinates if there was one.
  ///
  /// # Panics
  /// Panics if the coordinates are out of bounds.
  pub fn insert( &mut self, coord : Coordinate< System, Orientation >, value : T ) -> Option< T >
  {
    let coord = Coordinate::< System, Orientation >::new( self.offset.q + coord.q, self.offset.r + coord.r );
    let i : usize = coord.r.try_into().unwrap();
    let j : usize = coord.q.try_into().unwrap();
    std::mem::replace( &mut self.data[ ( i, j ) ], Some( value ) )
  }

  /// Removes a value at the given coordinates.
  /// Returns the value if there was one.
  ///
  /// # Panics
  /// Panics if the coordinates are out of bounds.
  pub fn remove( &mut self, coord : Coordinate< System, Orientation, Parity > ) -> Option< T >
  {
    // todo!();
    let coord = Coordinate::< System, Orientation, Parity >::new( self.offset.q + coord.q, self.offset.r + coord.r );
    let i : usize = coord.r.try_into().unwrap();
    let j : usize = coord.q.try_into().unwrap();
    std::mem::take( &mut self.data[ ( i, j ) ] )
  }

  /// Returns a reference to the value at the given coordinates.
  pub fn get( &self, coord : Coordinate< System, Orientation, Parity > ) -> Option< &T >
  {
    // todo!();
    let coord = Coordinate::< System, Orientation, Parity >::new( self.offset.q + coord.q, self.offset.r + coord.r );
    let i : usize = coord.r.try_into().ok()?;
    let j : usize = coord.q.try_into().ok()?;
    self.data.get( ( i, j ) ).and_then( | x | x.as_ref() )
  }

  /// Returns a mutable reference to the value at the given coordinates.
  pub fn get_mut( &mut self, coord : Coordinate< System, Orientation, Parity > ) -> Option< &mut T >
  {
    let coord = Coordinate::< System, Orientation, Parity >::new( self.offset.q + coord.q, self.offset.r + coord.r );
    let i : usize = coord.r.try_into().ok()?;
    let j : usize = coord.q.try_into().ok()?;
    self.data.get_mut( ( i, j ) ).and_then( | x | x.as_mut() )
  }

  pub fn layout( &self ) -> HexLayout
  {
    self.layout
  }
}
