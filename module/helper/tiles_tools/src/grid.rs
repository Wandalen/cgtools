use ndarray_cg::{ Array2, I32x2 };
use crate::coordinates::hexagonal::Coordinate;

pub struct HexArray< System, Orientation, T >
{
  data : Array2< Option< T > >,
  offset : I32x2,
  _marker : std::marker::PhantomData< Coordinate< System, Orientation > >,
}

impl< System, Orientation, T > HexArray< System, Orientation, T >
{
  pub fn new( size : I32x2, offset : I32x2 ) -> Self
  {
    let rows : usize = ( size[ 1 ] ).try_into().unwrap();
    let columns : usize = ( size [ 0 ] ).try_into().unwrap();
    Self { data : Array2::from_shape_fn( ( rows , columns ), | _ | None ), offset, _marker: std::marker::PhantomData }
  }

  /// Insets a value at the given coordinates.
  /// Returns the previous value at the coordinates if there was one.
  ///
  /// # Panics
  ///
  /// Panics if the coordinates are out of bounds.
  pub fn insert< C >( &mut self, coord : C, value : T ) -> Option< T >
  where
    C : Into< Coordinate< System, Orientation > >,
  {
    let coord : Coordinate::< System, Orientation > = coord.into();
    let i : usize = ( coord.r + self.offset[ 1 ] ).try_into().unwrap();
    let j : usize = ( coord.q + self.offset[ 0 ] ).try_into().unwrap();
    std::mem::replace( &mut self.data[ ( i, j ) ], Some( value ) )
  }

  /// Removes a value at the given coordinates.
  /// Returns the value if there was one.
  ///
  /// # Panics
  ///
  /// Panics if the coordinates are out of bounds.
  pub fn remove( &mut self, coord : Coordinate< System, Orientation > ) -> Option< T >
  {
    let coord : Coordinate::< System, Orientation > = coord.into();
    let i : usize = ( coord.r + self.offset[ 1 ] ).try_into().unwrap();
    let j : usize = ( coord.q + self.offset[ 0 ] ).try_into().unwrap();
    std::mem::take( &mut self.data[ ( i, j ) ] )
  }

  /// Returns a reference to the value at the given coordinates.
  ///
  /// # Panics
  /// Panics if the coordinates are out of bounds.
  pub fn get( &self, coord : Coordinate< System, Orientation > ) -> Option< &T >
  {
    let coord : Coordinate::< System, Orientation > = coord.into();
    let i : usize = ( coord.r + self.offset[ 1 ] ).try_into().unwrap();
    let j : usize = ( coord.q + self.offset[ 0 ] ).try_into().unwrap();
    self.data.get( ( i, j ) ).and_then( | x | x.as_ref() )
  }

  /// Returns a mutable reference to the value at the given coordinates.
  ///
  /// # Panics
  ///
  /// Panics if the coordinates are out of bounds.
  pub fn get_mut( &mut self, coord : Coordinate< System, Orientation > ) -> Option< &mut T >
  {
    let coord : Coordinate::< System, Orientation > = coord.into();
    let i : usize = ( coord.r + self.offset[ 1 ] ).try_into().unwrap();
    let j : usize = ( coord.q + self.offset[ 0 ] ).try_into().unwrap();
    self.data.get_mut( ( i, j ) ).and_then( | x | x.as_mut() )
  }
}
