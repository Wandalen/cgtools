use std::{ marker::PhantomData, ops::{ Index, IndexMut } };
use ndarray_cg::{ nd::iter::Iter, Array2, I64x2 };
use crate::coordinates::hexagonal::Coordinate;

pub struct HexArray< System, Orientation, T >
{
  data : Array2< T >,
  min : I64x2,
  _marker : PhantomData< Coordinate< System, Orientation > >,
}

impl< System, Orientation, T > HexArray< System, Orientation, T >
{
  pub fn with_size_and_fn< F >
  (
    min_inclusive : Coordinate< System, Orientation >,
    max_exclusive : Coordinate< System, Orientation >,
    f : F,
  ) -> Self
  where
    F : Fn() -> T,
  {
    let Coordinate { q, r, .. } = min_inclusive;
    let min_q = q as i64;
    let min_r = r as i64;
    let Coordinate { q, r, .. } = max_exclusive;
    let max_q = q as i64;
    let max_r = r as i64;

    let columns : usize = ( max_q - min_q ).try_into().expect( "Invalid size" );
    let rows : usize = ( max_r - min_r ).try_into().expect( "Invalid size" );

    Self
    {
      data : Array2::from_shape_simple_fn( ( rows , columns ), f ),
      min : I64x2::from_array( [ min_q, min_r ] ),
      _marker: PhantomData
    }
  }

  pub fn iter( &self ) -> Iter< '_, T , ndarray_cg::Dim< [ usize; 2 ] > >
  {
    self.data.iter()
  }

  pub fn indexed_iter( &self ) -> impl Iterator< Item = ( Coordinate< System, Orientation >, &T ) >
  {
    self.data.indexed_iter().map
    (
      | ( ( i, j ), value ) |
      {
        let i = ( i as i64 + self.min[ 1 ] ) as i32;
        let j = ( j as i64 + self.min[ 0 ] ) as i32;
        let coord = Coordinate::< System, Orientation >::new_uncheked( j, i );
        ( coord, value )
      }
    )
  }
}

impl< System, Orientation, T > HexArray< System, Orientation, T >
where
  T : Default
{
  pub fn with_size_and_default
  (
    min_inclusive : Coordinate< System, Orientation >,
    max_exclusive : Coordinate< System, Orientation >
  ) -> Self
  {
    let Coordinate { q, r, .. } = min_inclusive;
    let min_q = q as i64;
    let min_r = r as i64;
    let Coordinate { q, r, .. } = max_exclusive;
    let max_q = q as i64;
    let max_r = r as i64;

    let columns : usize = ( max_q - min_q ).try_into().expect( "Invalid size" );
    let rows : usize = ( max_r - min_r ).try_into().expect( "Invalid size" );

    Self
    {
      data : Array2::from_shape_simple_fn( ( rows , columns ), T::default ),
      min : I64x2::from_array( [ min_q, min_r ] ),
      _marker: PhantomData
    }
  }
}

impl< System, Orientation, T > HexArray< System, Orientation, Option< T > >
{
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
    let i : usize = ( coord.r as i64 - self.min[ 1 ] ).try_into().expect( "Coordinate out of bound" );
    let j : usize = ( coord.q as i64 - self.min[ 0 ] ).try_into().expect( "Coordinate out of bound" );
    std::mem::replace( &mut self.data[ ( i, j ) ], Some( value ) )
  }

  /// Removes a value at the given coordinates.
  /// Returns the value if there was one.
  ///
  /// # Panics
  ///
  /// Panics if the coordinates are out of bounds.
  pub fn remove< C >( &mut self, coord : C ) -> Option< T >
  where
    C : Into< Coordinate< System, Orientation > >,
  {
    let coord : Coordinate::< System, Orientation > = coord.into();
    let i : usize = ( coord.r as i64 - self.min[ 1 ] ).try_into().expect( "Coordinate out of bound" );
    let j : usize = ( coord.q as i64 - self.min[ 0 ] ).try_into().expect( "Coordinate out of bound" );
    std::mem::take( &mut self.data[ ( i, j ) ] )
  }

  pub fn get< C >( &self, coord : C ) -> Option< &T >
  where
    C : Into< Coordinate< System, Orientation > >
  {
    let coord : Coordinate::< System, Orientation > = coord.into();
    let i : usize = ( coord.r as i64 - self.min[ 1 ] ).try_into().expect( "Coordinate out of bound" );
    let j : usize = ( coord.q as i64 - self.min[ 0 ] ).try_into().expect( "Coordinate out of bound" );
    self.data.get( ( i, j ) ).and_then( | o | o.as_ref() )
  }

  pub fn get_mut< C >( &mut self, coord : C ) -> Option< &mut T >
  where
    C : Into< Coordinate< System, Orientation > >
  {
    let coord : Coordinate::< System, Orientation > = coord.into();
    let i : usize = ( coord.r as i64 - self.min[ 1 ] ).try_into().expect( "Coordinate out of bound" );
    let j : usize = ( coord.q as i64 - self.min[ 0 ] ).try_into().expect( "Coordinate out of bound" );
    self.data.get_mut( ( i, j ) ).and_then( | o | o.as_mut() )
  }
}

impl< C, System, Orientation, T > Index< C > for HexArray< System, Orientation, T >
where
  C : Into< Coordinate< System, Orientation > >,
{
  type Output = T;

  fn index( &self, index : C ) -> &Self::Output
  {
    let coord : Coordinate::< System, Orientation > = index.into();
    let i : usize = ( coord.r as i64 - self.min[ 1 ] ).try_into().expect( "Coordinate out of bound" );
    let j : usize = ( coord.q as i64 - self.min[ 0 ] ).try_into().expect( "Coordinate out of bound" );
    self.data.index( ( i, j ) )
  }
}

impl< C, System, Orientation, T > IndexMut< C > for HexArray< System, Orientation, T >
where
  C : Into< Coordinate< System, Orientation > >,
{
  fn index_mut( &mut self, index : C ) -> &mut Self::Output
  {
    let coord : Coordinate::< System, Orientation > = index.into();
    let i : usize = ( coord.r as i64 - self.min[ 1 ] ).try_into().expect( "Coordinate out of bound" );
    let j : usize = ( coord.q as i64 - self.min[ 0 ] ).try_into().expect( "Coordinate out of bound" );
    self.data.index_mut( ( i, j ) )
  }
}
